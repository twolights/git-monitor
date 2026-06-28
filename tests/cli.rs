//! US3 integration tests: CLI behavior for the binary.
//!
//! Note: the tool runs an interactive full-screen loop on a valid repository,
//! so these tests exercise the paths that exit immediately (errors and
//! `--help`/`--version`). Targeting an explicit valid repo is covered at the
//! library level in `status_reads.rs` / `commit_reads.rs` via `open_repo`.

use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_git-monitor"))
}

#[test]
fn non_git_path_errors_with_exit_code_1() {
    let tmp = tempfile::tempdir().unwrap();
    let output = bin().arg(tmp.path()).output().expect("run binary");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not inside a git repository"),
        "stderr was: {stderr}"
    );
}

#[test]
fn help_and_version_succeed() {
    assert!(bin().arg("--help").output().unwrap().status.success());
    assert!(bin().arg("--version").output().unwrap().status.success());
}
