//! C: CMake fetching BUI at a release tag, clang-format and clang-tidy configured.

use std::ops::Range;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{File, Flavour, Step, Vars, read, render, step, work_step};
use crate::check::{Check, Fix, Work};
use crate::registry;
use crate::sys::{Cmd, semver};
use crate::tools::Tool;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CApp {}

fn cmake(dir: &Path, args: &[&str]) -> Cmd {
    Cmd::new("cmake", args.iter().copied()).in_dir(dir)
}

/// What a CMake token is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    /// `v0.5.1`, `GIT_TAG`, a command's name
    Unquoted,
    /// `"..."`, with `\` escapes
    Quoted,
    /// `[[...]]`, `[=[...]=]`
    Bracket,
    /// `# ...` and `#[[ ... ]]`
    Comment,
    Open,
    Close,
}

/// CMake's tokens of a text in order, whitespace dropped, each with its byte range. A `#` or `)` inside a quoted or
/// bracket argument belongs to the argument; an unterminated one runs to the end of the text.
struct Tokens<'a> {
    src: &'a [u8],
    at: usize,
}

impl<'a> Tokens<'a> {
    const fn new(text: &'a str) -> Self {
        Self { src: text.as_bytes(), at: 0 }
    }

    /// The length of a bracket opener `[==[` at `i` (its `=` count + 2), if one starts there.
    fn bracket(&self, i: usize) -> Option<usize> {
        let rest = self.src.get(i..)?;
        let eq = rest.iter().skip(1).take_while(|&&b| b == b'=').count();
        (rest.first() == Some(&b'[') && rest.get(1 + eq) == Some(&b'[')).then_some(eq + 2)
    }

    /// The end of the bracket body whose `n`-byte opener is at `i`: past its `]==]`, or the end of the text.
    fn close(&self, i: usize, n: usize) -> usize {
        let body = i + n;
        let shut = |w: &[u8]| w[0] == b']' && w[n - 1] == b']' && w[1..n - 1].iter().all(|&b| b == b'=');
        self.src[body..].windows(n).position(shut).map_or(self.src.len(), |at| body + at + n)
    }

    /// The end of a run from `i` that stops before a byte `stop` accepts, stepping over `\` escapes.
    fn run(&self, mut i: usize, stop: impl Fn(u8) -> bool) -> usize {
        while i < self.src.len() && !stop(self.src[i]) {
            i += if self.src[i] == b'\\' { 2 } else { 1 };
        }
        i.min(self.src.len())
    }
}

impl Iterator for Tokens<'_> {
    type Item = (Kind, Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        let src = self.src;
        let start = self.at + src.get(self.at..)?.iter().take_while(|b| b.is_ascii_whitespace()).count();
        let (kind, end) = match *src.get(start)? {
            b'(' => (Kind::Open, start + 1),
            b')' => (Kind::Close, start + 1),
            b'"' => (Kind::Quoted, (self.run(start + 1, |b| b == b'"') + 1).min(src.len())),
            b'#' => match self.bracket(start + 1) {
                Some(n) => (Kind::Comment, self.close(start + 1, n)),
                None => (Kind::Comment, src[start..].iter().position(|&b| b == b'\n').map_or(src.len(), |at| start + at)),
            },
            _ => match self.bracket(start) {
                Some(n) => (Kind::Bracket, self.close(start, n)),
                None => (Kind::Unquoted, self.run(start, |b| b.is_ascii_whitespace() || matches!(b, b'(' | b')' | b'#' | b'"'))),
            },
        };
        self.at = end;
        Some((kind, start..end))
    }
}

/// The byte range of the tag (`v0.5.1`) in BUI's own `FetchContent_Declare(...)` in a CMakeLists, inside its
/// quotes if it has them. Other dependencies' blocks have tags of their own, which are not ours to read or move.
/// `GIT_TAG` counts only as a whole unquoted argument and its value is the next argument, so a commented-out,
/// quoted or bracket `GIT_TAG` is not the tag.
fn bui_pin(text: &str) -> Option<Range<usize>> {
    let mut tokens = Tokens::new(text).filter(|&(kind, _)| kind != Kind::Comment);
    let mut name: Option<Range<usize>> = None; // the unquoted token just before: a command's name if `(` follows
    while let Some((kind, span)) = tokens.next() {
        let declare = name.take().is_some_and(|n| {
            kind == Kind::Open
                && text[n.clone()].eq_ignore_ascii_case("FetchContent_Declare")
                && text[n.end..span.start].bytes().all(|b| b == b' ' || b == b'\t')
        });
        if !declare {
            name = (kind == Kind::Unquoted).then_some(span);
            continue;
        }
        let (mut depth, mut first, mut ours, mut key, mut tag) = (0u32, true, false, false, None);
        for (kind, span) in tokens.by_ref() {
            match kind {
                Kind::Close if depth == 0 => break,
                Kind::Close => depth -= 1,
                Kind::Open => depth += 1,
                _ => {}
            }
            let arg = &text[span.clone()];
            ours |= first && ["bui", "blender-ui", "blenderui"].iter().any(|n| arg.eq_ignore_ascii_case(n));
            ours |= arg.as_bytes().windows(15).any(|w| w.eq_ignore_ascii_case(b"ppmpreetham/bui"));
            if key {
                tag = match kind {
                    Kind::Unquoted => Some(span),
                    Kind::Quoted => Some(span.start + 1..span.end.saturating_sub(1).max(span.start + 1)),
                    _ => None,
                };
            }
            key = tag.is_none() && kind == Kind::Unquoted && arg.eq_ignore_ascii_case("GIT_TAG");
            first = false;
        }
        if ours {
            return tag;
        }
    }
    None
}

/// The release tag `CMakeLists.txt` fetches BUI at.
fn pinned_tag(dir: &Path) -> Option<String> {
    let mut text = read(dir, "CMakeLists.txt")?;
    let tag = bui_pin(&text)?;
    text.truncate(tag.end);
    text.drain(..tag.start);
    Some(text)
}

/// The edit that moves BUI's tag to `to`: exactly the bytes `bui_pin` read, so it cannot land on another
/// dependency's tag or on a commented-out copy of the declaration.
fn move_tag(dir: &Path, to: &str) -> Option<Work> {
    let text = read(dir, "CMakeLists.txt")?;
    let at = bui_pin(&text)?;
    let was = text[at.clone()].to_owned();
    Some(Work::Rewrite { path: dir.join("CMakeLists.txt"), at, was, to: to.to_owned() })
}

impl Flavour for CApp {
    fn label(&self) -> String {
        "C".into()
    }

    fn tools(&self) -> Vec<Tool> {
        vec![Tool::Cc, Tool::Cmake, Tool::Git]
    }

    fn files(&self, vars: &Vars) -> Vec<File> {
        render(&["c"], vars)
    }

    fn deps(&self, dir: &Path) -> Vec<Step> {
        vec![step(
            "Fetching and building BUI once (a few minutes now, seconds after)",
            cmake(dir, &["--workflow", "--preset", "default"]),
        )]
    }

    fn dev(&self, dir: &Path, args: &[String]) -> Result<Vec<Cmd>> {
        let exe = dir.join("build/bin").join(if cfg!(windows) { "app.exe" } else { "app" });
        Ok(vec![
            cmake(dir, &["--preset", "default"]),
            cmake(dir, &["--build", "--preset", "default"]),
            Cmd::new(&exe.to_string_lossy(), Vec::<String>::new()).in_dir(dir).with_args(args),
        ])
    }

    fn build(&self, dir: &Path, args: &[String]) -> Result<Vec<Cmd>> {
        // its own build folder: a single-config generator (Ninja, Make) builds one CMAKE_BUILD_TYPE per folder
        Ok(vec![cmake(dir, &["--preset", "release"]), cmake(dir, &["--build", "--preset", "release"]).with_args(args)])
    }

    fn checks(&self, dir: &Path) -> Vec<Check> {
        let Some(have) = pinned_tag(dir) else {
            return vec![Check::fail("BUI version", "CMakeLists.txt has no BUI FetchContent_Declare with a GIT_TAG", None)];
        };
        let latest = registry::bui_version().map(|v| format!("v{v}"));
        match latest {
            Some(l) if semver(&l) > semver(&have) => {
                let fix = move_tag(dir, &l).map(|w| Fix::edit(format!("moves BUI's GIT_TAG to {l}"), vec![w]));
                vec![Check::warn("BUI release", format!("{have}, newest is {l}"), fix)]
            }
            Some(_) => vec![Check::ok("BUI release", format!("{have} (newest)"))],
            None => vec![Check::ok("BUI release", format!("{have} (could not look up the newest)"))],
        }
    }

    fn update(&self, dir: &Path) -> Vec<Step> {
        let Some(latest) = registry::bui_version().map(|v| format!("v{v}")) else {
            return Vec::new();
        };
        move_tag(dir, &latest).map(|work| work_step("Moving BUI to the newest release", work)).into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::splice;

    /// BUI's tag in `text`, as `pinned_tag` reads it.
    fn tag(text: &str) -> Option<&str> {
        bui_pin(text).map(|at| &text[at])
    }

    /// `text` up to BUI's tag, and the tag.
    fn split(text: &str) -> (&str, &str) {
        let at = bui_pin(text).unwrap();
        (&text[..at.start], &text[at])
    }

    const TWO: &str = "include(FetchContent)
FetchContent_Declare(
  fmt
  GIT_REPOSITORY https://github.com/fmtlib/fmt.git
  GIT_TAG v0.5.1)
fetchcontent_declare(
  bui
  GIT_REPOSITORY https://github.com/ppmpreetham/BUI.git
  GIT_TAG v0.5.1
  GIT_SHALLOW TRUE)
";

    #[test]
    fn the_tag_is_read_from_buis_own_declaration() {
        assert_eq!(tag(TWO), Some("v0.5.1"));
        let other = TWO.replace("GIT_TAG v0.5.1\n  GIT_SHALLOW", "GIT_TAG v0.6.0\n  GIT_SHALLOW");
        assert_eq!(tag(&other).unwrap(), "v0.6.0", "not the first dependency's");
        assert!(bui_pin("FetchContent_Declare(fmt GIT_TAG 1)").is_none());
    }

    #[test]
    fn moving_the_tag_leaves_other_dependencies_alone() {
        let dir = std::env::temp_dir().join(format!("bui-c-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("CMakeLists.txt"), TWO).unwrap();
        let work = move_tag(&dir, "v0.9.0").unwrap();
        assert!(work.perform("test").unwrap());
        let text = std::fs::read_to_string(dir.join("CMakeLists.txt")).unwrap();
        assert_eq!(text, TWO.replacen("GIT_TAG v0.5.1\n  GIT_SHALLOW", "GIT_TAG v0.9.0\n  GIT_SHALLOW", 1));
        assert_eq!(pinned_tag(&dir).as_deref(), Some("v0.9.0"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn spaces_before_the_parenthesis_and_comments_are_cmake_syntax() {
        let text = "FetchContent_Declare (bui # the \u{e9}tag below\n  GIT_REPOSITORY https://github.com/ppmpreetham/BUI.git\n  # GIT_TAG v0.4.0\n  GIT_TAG v0.5.1)\n";
        let (head, tag) = split(text);
        assert_eq!(tag, "v0.5.1");
        assert!(head.ends_with("# GIT_TAG v0.4.0\n  GIT_TAG "));
    }

    #[test]
    fn a_hash_in_a_quoted_or_bracket_argument_is_not_a_comment() {
        let text = "FetchContent_Declare(bui SOURCE_DIR \"bui#src \\\" #x\" GIT_TAG v0.5.1)\n";
        assert_eq!(tag(text).unwrap(), "v0.5.1");
        let text = "FetchContent_Declare(bui #[==[ GIT_TAG v0.1 ]] ]==] GIT_TAG v0.5.1 #[[ x\n GIT_TAG v0.2 ]])\n";
        assert_eq!(tag(text).unwrap(), "v0.5.1");
        let text = "FetchContent_Declare(bui SOURCE_DIR [[a#b]] GIT_TAG v0.5.1)\n";
        assert_eq!(tag(text).unwrap(), "v0.5.1");
    }

    #[test]
    fn a_comment_between_the_key_and_the_value_is_skipped() {
        let text = "FetchContent_Declare (bui\n  GIT_TAG # pinned \u{e9}\n    v0.5.1)\n";
        let (head, tag) = split(text);
        assert_eq!(tag, "v0.5.1");
        assert!(head.ends_with("pinned \u{e9}\n    "));
    }

    /// `text` with BUI's tag moved to `to` the way `move_tag` rewrites it, checked to read back.
    fn moved(text: &str, to: &str) -> String {
        let at = bui_pin(text).unwrap();
        let mut out = text.to_owned();
        assert!(splice(&mut out, at.clone(), &text[at], to));
        assert_eq!(tag(&out).unwrap(), to, "the rewrite still parses: {out}");
        out
    }

    #[test]
    fn git_tag_counts_only_as_a_whole_unquoted_argument() {
        let text = "FetchContent_Declare(bui SOURCE_DIR [[bui#src GIT_TAG old]] GIT_TAG v0.5.1)\n";
        assert_eq!(tag(text).unwrap(), "v0.5.1");
        assert_eq!(moved(text, "v0.9.0"), text.replace("v0.5.1", "v0.9.0"));
        let text = "FetchContent_Declare(bui SOURCE_DIR \"GIT_TAG old\" MY_GIT_TAG x GIT_TAG \"v0.5.1\")\n";
        assert_eq!(tag(text).unwrap(), "v0.5.1");
        assert_eq!(moved(text, "v0.9.0"), text.replace("v0.5.1", "v0.9.0"));
        assert_eq!(moved(TWO, "v0.9.0"), TWO.replacen("v0.5.1\n  GIT_SHALLOW", "v0.9.0\n  GIT_SHALLOW", 1));
    }

    #[test]
    fn a_parenthesis_in_a_quoted_or_bracket_argument_does_not_end_the_block() {
        let text = "FetchContent_Declare(bui SOURCE_DIR \"a)b\" PATCH_COMMAND [=[x)]=] GIT_TAG v0.5.1)\n";
        assert_eq!(tag(text).unwrap(), "v0.5.1");
        assert_eq!(moved(text, "v0.9.0"), text.replace("v0.5.1", "v0.9.0"));
    }

    /// A commented-out copy of the declaration before the real one is the same text up to the tag: the edit
    /// moves the tag `bui_pin` read, not the first match of that text.
    #[test]
    fn a_commented_out_declaration_before_the_real_one_is_left_alone() {
        const DECL: &str = "FetchContent_Declare(bui\n  GIT_REPOSITORY https://github.com/ppmpreetham/BUI.git\n  GIT_TAG v0.5.1)\n";
        let line = format!("# {}\n", DECL.trim_end().replace('\n', "\n# "));
        for old in [format!("#[[\n{DECL}]]\n"), format!("#[==[ {DECL} ]==]\n"), line] {
            let text = format!("{old}{DECL}");
            assert_eq!(moved(&text, "v0.9.0"), format!("{old}{}", DECL.replace("v0.5.1", "v0.9.0")), "{text}");
        }
        let dir = std::env::temp_dir().join(format!("bui-c-comment-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("CMakeLists.txt"), format!("#[[\n{DECL}]]\n{DECL}")).unwrap();
        let work = move_tag(&dir, "v0.9.0").unwrap();
        assert!(work.perform("test").unwrap());
        let out = std::fs::read_to_string(dir.join("CMakeLists.txt")).unwrap();
        assert_eq!(out, format!("#[[\n{DECL}]]\n{}", DECL.replace("v0.5.1", "v0.9.0")));
        std::fs::write(dir.join("CMakeLists.txt"), "project(x)\n").unwrap();
        assert!(!work.perform("test").unwrap(), "a file that changed since is not edited");
        assert_eq!(std::fs::read_to_string(dir.join("CMakeLists.txt")).unwrap(), "project(x)\n");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// Each build preset builds the configuration its configure preset was made for: Ninja and Make take it
    /// from `CMAKE_BUILD_TYPE`, Visual Studio and Xcode from `--config`.
    #[test]
    fn every_preset_names_its_build_type() {
        let file = render(&["c"], &Vars::new("demo", "v1.0.0")).into_iter().find(|f| f.path == "CMakePresets.json").unwrap();
        let doc: serde_json::Value = serde_json::from_str(&file.body).unwrap();
        let configure = doc["configurePresets"].as_array().unwrap();
        let builds = doc["buildPresets"].as_array().unwrap();
        for (preset, config) in [("default", "Debug"), ("release", "Release")] {
            let build = builds.iter().find(|b| b["name"] == preset).unwrap();
            assert_eq!(build["configuration"], config);
            let made = configure.iter().find(|c| c["name"] == build["configurePreset"]).unwrap();
            assert_eq!(made["cacheVariables"]["CMAKE_BUILD_TYPE"], config, "{preset}");
        }
        let dirs: Vec<&str> = configure.iter().filter_map(|c| c["binaryDir"].as_str()).collect();
        assert_eq!(dirs.len(), 2);
        assert_ne!(dirs[0], dirs[1], "one folder per build type");
    }

    #[test]
    fn unterminated_text_at_the_end_does_not_panic() {
        for text in ["project(x)\n#", "#", "#[", "#[=", "x [=", "[[", "\"\\", "\\", "FetchContent_Declare(bui GIT_TAG"] {
            assert!(bui_pin(text).is_none(), "{text:?}");
        }
    }
}
