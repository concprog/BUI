//! Python: uv (the Python, the virtual environment and the lock file), Ruff for lint and format.

use std::path::Path;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use super::{File, Flavour, Step, Vars, read, render, step};
use crate::check::{self, Check, Fix};
use crate::project::{PyApi, depends_on_blenderui};
use crate::registry;
use crate::sys::Cmd;
use crate::tools::Tool;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PythonApp {
    pub api: PyApi,
}

fn uv(dir: &Path, args: &[&str]) -> Cmd {
    Cmd::new("uv", args.iter().copied()).in_dir(dir)
}

/// The commands that add blenderui and Ruff at their newest versions: what `new` installs, and what `doctor`
/// runs for a project made with `--no-install` (`uv sync` alone would install nothing).
fn adds(dir: &Path) -> [Cmd; 2] {
    [uv(dir, &["add", "blenderui"]), uv(dir, &["add", "--dev", "ruff"])]
}

impl Flavour for PythonApp {
    fn label(&self) -> String {
        let api = match self.api {
            PyApi::Components => "Blender components",
            PyApi::Qt => "Qt syntax",
            PyApi::Tk => "tkinter syntax",
        };
        format!("Python ({api})")
    }

    fn tools(&self) -> Vec<Tool> {
        vec![Tool::Uv]
    }

    fn files(&self, vars: &Vars) -> Vec<File> {
        let api = match self.api {
            PyApi::Components => "python/components",
            PyApi::Qt => "python/qt",
            PyApi::Tk => "python/tk",
        };
        render(&["python/common", api], vars)
    }

    fn deps(&self, dir: &Path) -> Vec<Step> {
        let [bui, ruff] = adds(dir);
        vec![step("Adding blenderui (newest)", bui), step("Adding Ruff (newest)", ruff)]
    }

    fn dev(&self, dir: &Path, args: &[String]) -> Result<Vec<Cmd>> {
        Ok(vec![uv(dir, &["run", "main.py"]).with_args(args)])
    }

    fn build(&self, _dir: &Path, _args: &[String]) -> Result<Vec<Cmd>> {
        bail!("a Python app is not built: run it with `bui dev`. To hand it out as one file, use PyInstaller (`uv run --with pyinstaller pyinstaller --onefile main.py`).")
    }

    fn checks(&self, dir: &Path) -> Vec<Check> {
        if !depends_on_blenderui(&read(dir, "pyproject.toml").unwrap_or_default()) {
            let add = Fix::run("adds blenderui and Ruff at their newest", adds(dir).into());
            return vec![Check::fail("blenderui", "not a dependency in pyproject.toml", Some(add))];
        }
        let sync = Fix::run("creates the environment and installs the packages", vec![uv(dir, &["sync"])]);
        let Some(lock) = read(dir, "uv.lock") else {
            return vec![Check::fail("environment", "no uv.lock: nothing installed yet", Some(sync))];
        };
        let mut lines = lock.lines();
        let mut have = None;
        while let Some(l) = lines.next() {
            if l == "name = \"blenderui\"" {
                have = lines.next().and_then(|v| v.strip_prefix("version = \"")).and_then(|v| v.strip_suffix('"')).map(str::to_owned);
                break;
            }
        }
        let Some(have) = have else {
            return vec![Check::fail("blenderui", "not in uv.lock yet", Some(sync))];
        };
        let mut checks = Vec::new();
        if !dir.join(".venv").is_dir() {
            checks.push(Check::warn("environment", ".venv is missing", Some(sync)));
        }
        let fix = Fix::run("updates blenderui to the newest", vec![uv(dir, &["lock", "--upgrade-package", "blenderui"]), uv(dir, &["sync"])]);
        checks.push(check::newest("blenderui (PyPI)", &have, registry::pypi_version("blenderui").as_deref(), fix));
        checks
    }

    fn update(&self, dir: &Path) -> Vec<Step> {
        vec![
            step("Updating blenderui", uv(dir, &["lock", "--upgrade-package", "blenderui"])),
            step("Installing", uv(dir, &["sync"])),
        ]
    }
}
