//! `bui new` and `bui init`: ask what to make, make sure its toolchain is there, write the project,
//! install the newest BUI and tooling with the language's own package manager.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use crate::cli::{Choices, Flavor, NewArgs};
use crate::flavours::Vars;
use crate::flavours::{c::CApp, js::JsApp, lua::LuaApp, python::PythonApp, rust::RustApp};
use crate::project::{FILE, Framework, Language, Meta, PackageManager, PyApi, Stack};
use crate::sys::{Cmd, which};
use crate::ui::{self, choice};
use crate::{registry, tools};

pub fn new(args: NewArgs) -> Result<()> {
    ui::banner(true);
    ui::intro("Create a BUI app");
    let name = match args.name {
        Some(n) => n,
        None => ui::input("What is the project called?", "my-app", |s| bad_name(s.as_str()))?,
    };
    if let Some(problem) = bad_name(&name) {
        bail!("{problem}");
    }
    let dir = std::env::current_dir()?.join(&name);
    if dir.exists() && fs::read_dir(&dir)?.next().is_some() {
        bail!("{} already exists and is not empty (to make a project in a folder that has files, `cd` into it and run `bui init`)", dir.display());
    }
    make(&name, &dir, &args.choices, true)
}

pub fn init(choices: &Choices) -> Result<()> {
    ui::banner(true);
    ui::intro("Add BUI to this folder");
    let dir = std::env::current_dir()?;
    if dir.join(FILE).exists() {
        bail!("this folder is already a BUI project ({FILE} exists). `bui doctor` checks it.");
    }
    let raw = dir.file_name().map_or_else(|| "app".into(), |n| n.to_string_lossy().into_owned());
    let name = sanitize(&raw);
    make(&name, &dir, choices, false)
}

/// A project name as a package name: lowercase letters, digits, `-` and `_`.
fn sanitize(raw: &str) -> String {
    let s: String = raw.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' { c.to_ascii_lowercase() } else { '-' }).collect();
    let s = s.trim_matches('-').to_owned();
    if s.is_empty() || s.starts_with(|c: char| c.is_ascii_digit()) { format!("app-{s}") } else { s }
}

fn bad_name(name: &str) -> Option<String> {
    let ok = !name.is_empty()
        && name.starts_with(|c: char| c.is_ascii_lowercase())
        && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_');
    (!ok).then(|| "Use lowercase letters, digits, - and _, starting with a letter (it becomes the package name).".to_owned())
}

fn make(name: &str, dir: &Path, choices: &Choices, new_folder: bool) -> Result<()> {
    let stack = ask(choices)?;
    let app = stack.app();
    ui::info(format!("Making {}", ui::accent(&app.label())));

    // the language's programs, offered for install when missing
    let missing = tools::ensure(&app.tools())?;

    let tag = registry::bui_version().map_or_else(|| format!("v{}", env!("CARGO_PKG_VERSION")), |v| format!("v{v}"));
    let vars = Vars::new(name, &tag);
    let task = ui::task("Writing the project");
    fs::create_dir_all(dir)?;
    let mut skipped = Vec::new();
    let mut files = app.files(&vars);
    files.push(crate::flavours::File { path: FILE.into(), body: Meta { name: name.to_owned(), stack: stack.clone() }.to_toml()? });
    let total = files.len();
    for f in files {
        let path: PathBuf = dir.join(&f.path);
        if path.exists() {
            skipped.push(f.path);
            continue;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, f.body)?;
    }
    task.done(&format!("Wrote {} files", total - skipped.len()));
    if !skipped.is_empty() {
        ui::warn(format!("Left your existing {} alone", skipped.join(", ")));
    }

    git_init(dir, choices.no_git)?;

    let finished = if choices.no_install {
        false
    } else if missing.is_empty() {
        install(&app.deps(dir))?
    } else {
        ui::warn("Skipped installing packages: the tools above are missing. Run `bui doctor` after installing them.");
        false
    };

    let mut steps = Vec::new();
    if new_folder {
        steps.push(format!("cd {name}"));
    }
    if !finished {
        steps.push("bui doctor".to_owned());
    }
    steps.extend(app.next().into_iter().map(str::to_owned));
    let shown: Vec<String> = steps.iter().map(|s| format!("$ {}", ui::code(s))).collect();
    ui::note("Next", &shown.join("\n"));
    ui::outro(&format!("{} {} is ready", ui::accent("✦"), name));
    Ok(())
}

/// The language, and that language's options: each question can be answered by a flag.
fn ask(c: &Choices) -> Result<Stack> {
    let flavor = ui::select(
        "What do you want to write it in?",
        &[
            choice(Flavor::Rust, "Rust".into(), "Builders like GPUI, or HTML and Tailwind"),
            choice(Flavor::Js, "JavaScript / TypeScript".into(), "React, Svelte or plain DOM"),
            choice(Flavor::Python, "Python".into(), "Blender components, or Qt or tkinter code"),
            choice(Flavor::Lua, "Lua".into(), "Small and fast to start"),
            choice(Flavor::C, "C".into(), "The kit's own API, with CMake"),
        ],
        c.flavor.as_ref(),
    )?;
    Ok(match flavor {
        Flavor::Rust => Stack::Rust(RustApp {}),
        Flavor::Lua => Stack::Lua(LuaApp {}),
        Flavor::C => Stack::C(CApp {}),
        Flavor::Python => {
            let api = ui::select(
                "Which API?",
                &[
                    choice(PyApi::Components, "Blender components".into(), "Panels, sliders, color swatches: Blender's look"),
                    choice(PyApi::Qt, "Qt syntax".into(), "Qt code; only the import changes, no PySide"),
                    choice(PyApi::Tk, "tkinter syntax".into(), "tkinter code; only the import changes"),
                ],
                c.api.as_ref(),
            )?;
            Stack::Python(PythonApp { api })
        }
        Flavor::Js => {
            let framework = ui::select(
                "Which framework?",
                &[
                    choice(Framework::React, "React".into(), "React 19, hooks backed by signals"),
                    choice(Framework::Svelte, "Svelte".into(), "Svelte 5, runes"),
                    choice(Framework::Vanilla, "No framework".into(), "BUI's DOM-like API"),
                ],
                c.framework.as_ref(),
            )?;
            // Svelte files are compiled by BUI's loader: they are written in plain JavaScript
            let language = if framework == Framework::Svelte {
                Language::JavaScript
            } else {
                ui::select(
                    "TypeScript or JavaScript?",
                    &[
                        choice(Language::TypeScript, "TypeScript".into(), "strict, recommended"),
                        choice(Language::JavaScript, "JavaScript".into(), ""),
                    ],
                    c.language.as_ref(),
                )?
            };
            let have = |bin: &str| if which(bin).is_some() { "installed" } else { "not installed: bui offers to install it" };
            if framework == Framework::Svelte && c.pm == Some(PackageManager::Bun) {
                bail!("Svelte runs on Node.js (its loader fails under Bun): use --pm npm or --pm pnpm");
            }
            let mut runtimes = vec![
                choice(PackageManager::Npm, "Node.js + npm".into(), have("node").to_owned()),
                choice(PackageManager::Pnpm, "Node.js + pnpm".into(), have("pnpm").to_owned()),
            ];
            // Svelte's loader runs on Node only (it fails under Bun)
            if framework != Framework::Svelte {
                runtimes.insert(0, choice(PackageManager::Bun, "Bun".into(), format!("fastest, runs TypeScript directly, {}", have("bun"))));
            }
            let pm = ui::select("Which runtime and package manager?", &runtimes, c.pm.as_ref())?;
            Stack::Js(JsApp { framework, language, pm })
        }
    })
}

fn git_init(dir: &Path, skip: bool) -> Result<()> {
    if skip || which("git").is_none() || Cmd::new("git", ["rev-parse", "--git-dir"]).in_dir(dir).capture().is_some() {
        return Ok(());
    }
    if ui::confirm("Start a git repository?", true)? {
        ui::run("Starting a git repository", &Cmd::new("git", ["init", "--quiet"]).in_dir(dir))?;
    }
    Ok(())
}

/// Runs the install steps; stops at the first that fails and says how to finish by hand.
fn install(steps: &[crate::flavours::Step]) -> Result<bool> {
    for (i, s) in steps.iter().enumerate() {
        if !s.work.perform(&s.title)? {
            if s.optional {
                ui::info("That one is optional: the project works without it.");
                continue;
            }
            let rest: Vec<String> = steps[i..].iter().map(|s| ui::code(&s.work.display())).collect();
            ui::warn(format!("Finish the install later with:\n  {}", rest.join("\n  ")));
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_checked() {
        assert!(bad_name("my-app").is_none());
        assert!(bad_name("app_2").is_none());
        for bad in ["", "My-App", "2fast", "-x", "a b", "a/b"] {
            assert!(bad_name(bad).is_some(), "{bad}");
        }
    }

    #[test]
    fn a_folder_name_becomes_a_package_name() {
        assert_eq!(sanitize("My App"), "my-app");
        assert_eq!(sanitize("2048"), "app-2048");
        assert_eq!(sanitize("__x__"), "__x__");
    }
}
