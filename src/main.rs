mod cli;
mod error;
mod init;
mod plan;
mod render;
mod stack;

use std::process::ExitCode;

use anyhow::Context;
use clap::Parser;

use cli::{Cli, Commands};
use init::{resolve_project_name, run_init, InitOptions};

fn main() -> ExitCode {
    if let Err(err) = try_main() {
        eprintln!("error: {err:#}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn try_main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init(args) => {
            let project_name = resolve_project_name(args.name, &args.path);
            let opts = InitOptions {
                target: args.path,
                project_name: project_name.clone(),
                stack: args.stack,
                force: args.force,
                dry_run: args.dry_run,
            };
            let report = run_init(&opts).context("init failed")?;

            if opts.dry_run {
                println!("dry-run harness for `{project_name}` (stack: {})", opts.stack);
            } else {
                println!(
                    "installed harness for `{project_name}` (stack: {}) into {}",
                    opts.stack,
                    opts.target.display()
                );
            }
            for line in report.summary_lines() {
                println!("  {line}");
            }
        }
    }
    Ok(())
}
