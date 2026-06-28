//! Git Status Watcher — a read-only terminal UI that continuously monitors a
//! local git repository's branch, working-tree status, and recent commits.
//!
//! The crate is split so the repository-reading logic (`git`) produces plain
//! data types (`model`) that the UI (`ui`) renders, keeping all git logic
//! testable without driving a real terminal.

pub mod app;
pub mod cli;
pub mod git;
pub mod model;
pub mod ui;
