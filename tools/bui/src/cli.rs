//! The command line: what `bui --help` shows.

use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

use crate::project::{Framework, Language, PackageManager, PyApi};

#[derive(Parser)]
#[command(
    name = "bui",
    version,
    about = "Create, run, check and update BUI apps: Blender's UI from Rust, JavaScript, Python, Lua and C",
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// No animation and no colors (also when there is no terminal, in CI, or with NO_COLOR)
    #[arg(long, global = true)]
    pub plain: bool,

    /// Answer every question with its default
    #[arg(short, long, global = true)]
    pub yes: bool,

    /// Install missing tools (Rust, Bun, uv, ...) without asking
    #[arg(long, global = true)]
    pub install_tools: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Make a new project in a new folder
    New(NewArgs),
    /// Make a project in this folder
    Init(Choices),
    /// Run the project (restarting on every save where the language can)
    Dev,
    /// Make a release build
    Build,
    /// Find what is wrong with the setup and the project, and fix it
    Doctor(DoctorArgs),
    /// Update bui, and the project's BUI packages
    Update(UpdateArgs),
    /// Print a shell's completions (`bui completions bash > /etc/bash_completion.d/bui`)
    Completions {
        shell: Shell,
    },
}

#[derive(Args)]
pub struct NewArgs {
    /// The project's name (also its folder)
    pub name: Option<String>,
    #[command(flatten)]
    pub choices: Choices,
}

/// What to make; anything left out is asked.
#[derive(Args, Default)]
pub struct Choices {
    /// The language
    #[arg(long, value_enum)]
    pub flavor: Option<Flavor>,
    /// JavaScript: the framework
    #[arg(long, value_enum)]
    pub framework: Option<Framework>,
    /// JavaScript: TypeScript or JavaScript
    #[arg(long, value_enum)]
    pub language: Option<Language>,
    /// JavaScript: the package manager
    #[arg(long, value_enum)]
    pub pm: Option<PackageManager>,
    /// Python: which API
    #[arg(long, value_enum)]
    pub api: Option<PyApi>,
    /// Do not install dependencies
    #[arg(long)]
    pub no_install: bool,
    /// Do not run `git init`
    #[arg(long)]
    pub no_git: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Flavor {
    Rust,
    Js,
    Python,
    Lua,
    C,
}

#[derive(Args)]
pub struct DoctorArgs {
    /// Check every language's toolchain, not only this project's
    #[arg(long)]
    pub all: bool,
    /// Fix what can be fixed without asking
    #[arg(long)]
    pub fix: bool,
    /// Print the results as JSON and change nothing
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct UpdateArgs {
    /// Only say what is out of date
    #[arg(long)]
    pub check: bool,
}
