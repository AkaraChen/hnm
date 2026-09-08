use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::stack::Stack;

#[derive(Debug, Parser)]
#[command(
    name = "hnm",
    version,
    about = "Install the agent documentation harness into a project",
    long_about = "hnm writes AGENTS.md, docs/{prd,adr,spec}, feature-dev and git-commit skills, and commit-time doc review hooks so AI agents follow a consistent documentation harness."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Write the full harness into a target directory
    Init(InitArgs),
}

#[derive(Debug, Parser)]
pub struct InitArgs {
    /// Target project directory (created if missing)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Project name embedded in AGENTS.md and docs/spec.md
    #[arg(long, short = 'n')]
    pub name: Option<String>,

    /// Commands-section preset for AGENTS.md
    #[arg(long, short = 's', value_enum, default_value_t = Stack::Generic)]
    pub stack: Stack,

    /// Overwrite existing files and replace incorrect symlinks
    #[arg(long, short = 'f')]
    pub force: bool,

    /// Print the plan without writing files
    #[arg(long)]
    pub dry_run: bool,
}
