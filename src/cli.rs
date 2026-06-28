//! Command-line interface definition.

use std::env;
use std::path::PathBuf;

use clap::Parser;

/// Watch a git repository's status in real time.
#[derive(Debug, Parser)]
#[command(name = "git-monitor", version, about)]
pub struct Cli {
    /// Directory of the git repository to watch (defaults to the current directory).
    #[arg(value_name = "DIRECTORY")]
    pub directory: Option<PathBuf>,
}

impl Cli {
    /// Resolve the directory to watch: the provided argument when present,
    /// otherwise the current working directory. This is the single point of
    /// path resolution.
    pub fn target_dir(&self) -> PathBuf {
        match &self.directory {
            Some(dir) => dir.clone(),
            None => env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }
}
