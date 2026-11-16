//! `bui dev` and `bui build`: the project's own commands, with the toolchain checked first.

use std::path::Path;
use std::process::ExitCode;

use anyhow::Result;

use crate::project::{self, Project};
use crate::sys::Cmd;
use crate::{tools, ui};

pub fn dev() -> Result<ExitCode> {
    go("Running", |p| p.meta.stack.app().dev(&p.dir))
}

pub fn build() -> Result<ExitCode> {
    go("Building", |p| p.meta.stack.app().build(&p.dir))
}

fn go(verb: &str, commands: impl Fn(&Project) -> Result<Vec<Cmd>>) -> Result<ExitCode> {
    let project = project::require(Path::new("."))?;
    let app = project.meta.stack.app();
    let missing = tools::ensure(&app.tools())?;
    if !missing.is_empty() {
        ui::error("Cannot continue without those. `bui doctor` shows what to do.");
        return Ok(ExitCode::FAILURE);
    }
    let cmds = commands(&project)?;
    ui::info(format!("{verb} {} ({})", project.meta.name, app.label()));
    for cmd in &cmds {
        ui::flush();
        let status = cmd.run_inherit()?;
        if !status.success() {
            ui::error(format!("`{}` failed", cmd.display()));
            return Ok(ExitCode::from(u8::try_from(status.code().unwrap_or(1)).unwrap_or(1)));
        }
    }
    Ok(ExitCode::SUCCESS)
}
