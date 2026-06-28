//! Entry point: parse arguments, validate the repository before taking over the
//! screen, then run the watch loop. Errors go to stderr with a non-zero exit.

use std::process::ExitCode;

use clap::Parser;

use git_monitor::app;
use git_monitor::cli::Cli;
use git_monitor::git::repo::open_repo;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let target = cli.target_dir();

    // Validate the repository before entering the TUI so a non-git path reports
    // a clear error and exits, rather than flashing an empty view.
    let repo = match open_repo(&target) {
        Ok(repo) => repo,
        Err(err) => {
            eprintln!("git-monitor: {err:#}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(err) = app::run(repo) {
        eprintln!("git-monitor: {err:#}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
