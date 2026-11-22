//! Rust: `cargo add blenderui`, clippy's pedantic lints on, rustfmt.

use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{File, Flavour, Step, Vars, read, render, step};
use crate::check::{self, Check, Fix};
use crate::registry;
use crate::sys::Cmd;
use crate::tools::Tool;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RustApp {}

/// `cargo add blenderui`: adds the newest blenderui.
fn add(dir: &Path) -> Cmd {
    Cmd::new("cargo", ["add", "blenderui"]).in_dir(dir)
}

/// `cargo add blenderui@newest`: moves the requirement in `Cargo.toml` to `newest`, as the C project moves its
/// tag. `cargo update` stays inside the requirement (`^0.5` never reaches 0.6), and `cargo add` without a
/// version keeps the one a dependency has.
fn move_to(dir: &Path, newest: &str) -> Cmd {
    Cmd::new("cargo", ["add".to_owned(), format!("blenderui@{newest}")]).in_dir(dir)
}

impl Flavour for RustApp {
    fn label(&self) -> String {
        "Rust".into()
    }

    fn tools(&self) -> Vec<Tool> {
        // blenderui-sys builds BUI's C kit with CMake while it compiles
        vec![Tool::Rust, Tool::Cc, Tool::Cmake]
    }

    fn files(&self, vars: &Vars) -> Vec<File> {
        render(&["rust"], vars)
    }

    fn deps(&self, dir: &Path) -> Vec<Step> {
        vec![
            step("Adding blenderui (newest)", add(dir)),
            step(
                "Compiling BUI once (a few minutes now, seconds after)",
                Cmd::new("cargo", ["build"]).in_dir(dir),
            ),
        ]
    }

    fn dev(&self, dir: &Path, args: &[String]) -> Result<Vec<Cmd>> {
        // `bui dev -- --release` is `cargo run --release`; the app's own arguments need a second `--`
        Ok(vec![Cmd::new("cargo", ["run"]).in_dir(dir).with_args(args)])
    }

    fn build(&self, dir: &Path, args: &[String]) -> Result<Vec<Cmd>> {
        Ok(vec![Cmd::new("cargo", ["build", "--release"]).in_dir(dir).with_args(args)])
    }

    fn checks(&self, dir: &Path) -> Vec<Check> {
        let Some(manifest) = read(dir, "Cargo.toml") else {
            return vec![Check::fail("Cargo.toml", "missing", None)];
        };
        let Some(have) = locked_version(dir, "blenderui") else {
            let fix = Fix::run("adds blenderui", vec![add(dir)]);
            let detail = if manifest.contains("blenderui") { "not fetched yet" } else { "not a dependency" };
            return vec![Check::fail("blenderui crate", detail, Some(fix))];
        };
        let newest = registry::bui_version();
        let fix = Fix::run("moves blenderui to the newest", newest.iter().map(|v| move_to(dir, v)).collect());
        vec![check::newest("blenderui crate", &have, newest.as_deref(), fix)]
    }

    fn update(&self, dir: &Path) -> Vec<Step> {
        registry::bui_version().map(|v| step("Moving blenderui to the newest", move_to(dir, &v))).into_iter().collect()
    }
}

/// The version of `package` in `Cargo.lock`.
fn locked_version(dir: &Path, package: &str) -> Option<String> {
    let lock = read(dir, "Cargo.lock")?;
    let mut lines = lock.lines();
    while let Some(l) = lines.next() {
        if l == format!("name = \"{package}\"") {
            return lines.next()?.strip_prefix("version = \"")?.strip_suffix('"').map(str::to_owned);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moving_names_the_version() {
        assert_eq!(move_to(Path::new("."), "0.6.0").display(), "cargo add blenderui@0.6.0");
    }
}
