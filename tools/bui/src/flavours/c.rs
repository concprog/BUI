//! C: CMake fetching BUI at a release tag, clang-format and clang-tidy configured.

use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{File, Flavour, Step, Vars, read, render, rewrite, step};
use crate::check::{Check, Fix, Work};
use crate::registry;
use crate::sys::{Cmd, semver};
use crate::tools::Tool;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CApp {}

fn cmake(dir: &Path, args: &[&str]) -> Cmd {
    Cmd::new("cmake", args.iter().copied()).in_dir(dir)
}

/// The release tag `CMakeLists.txt` fetches (`GIT_TAG v0.5.1`).
fn pinned_tag(dir: &Path) -> Option<String> {
    let text = read(dir, "CMakeLists.txt")?;
    let rest = text.split("GIT_TAG").nth(1)?;
    rest.split_whitespace().next().map(str::to_owned)
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
        Ok(vec![cmake(dir, &["--preset", "default"]), cmake(dir, &["--build", "--preset", "release"]).with_args(args)])
    }

    fn checks(&self, dir: &Path) -> Vec<Check> {
        let Some(have) = pinned_tag(dir) else {
            return vec![Check::fail("BUI version", "CMakeLists.txt has no GIT_TAG", None)];
        };
        let latest = registry::bui_version().map(|v| format!("v{v}"));
        match latest {
            Some(l) if semver(&l) > semver(&have) => {
                let fix = Fix::edit(format!("moves GIT_TAG to {l}"), vec![Work::Rewrite { path: dir.join("CMakeLists.txt"), from: format!("GIT_TAG {have}"), to: format!("GIT_TAG {l}") }]);
                vec![Check::warn("BUI release", format!("{have}, newest is {l}"), Some(fix))]
            }
            Some(_) => vec![Check::ok("BUI release", format!("{have} (newest)"))],
            None => vec![Check::ok("BUI release", format!("{have} (could not look up the newest)"))],
        }
    }

    fn update(&self, dir: &Path) -> Vec<Step> {
        let (Some(have), Some(latest)) = (pinned_tag(dir), registry::bui_version().map(|v| format!("v{v}"))) else {
            return Vec::new();
        };
        vec![rewrite("Moving BUI to the newest release", dir.join("CMakeLists.txt"), format!("GIT_TAG {have}"), format!("GIT_TAG {latest}"))]
    }
}
