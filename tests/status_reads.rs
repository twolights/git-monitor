//! US1 integration tests: branch name and working-tree status groups.

mod common;

use std::path::Path;

use git_monitor::git::repo::{open_repo, read_snapshot};

#[test]
fn clean_repo_reports_clean_and_branch() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = common::init_repo(tmp.path());
    common::write_file(tmp.path(), "a.txt", "hello");
    common::commit_all(&repo, "init");

    let opened = open_repo(tmp.path()).unwrap();
    let snap = read_snapshot(&opened).unwrap();

    assert!(
        snap.is_clean(),
        "expected clean tree, got {:?}",
        snap.status
    );
    assert!(snap.branch.name.is_some(), "expected a branch name");
    assert!(!snap.branch.detached);
}

#[test]
fn detects_untracked_staged_and_unstaged() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = common::init_repo(tmp.path());
    common::write_file(tmp.path(), "tracked.txt", "v1");
    common::commit_all(&repo, "init");

    // Modify a tracked file (unstaged), add an untracked file, and stage a new file.
    common::write_file(tmp.path(), "tracked.txt", "v2");
    common::write_file(tmp.path(), "untracked.txt", "new");
    common::write_file(tmp.path(), "staged.txt", "stage");
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("staged.txt")).unwrap();
    index.write().unwrap();

    let opened = open_repo(tmp.path()).unwrap();
    let snap = read_snapshot(&opened).unwrap();

    assert!(!snap.is_clean());
    assert!(
        snap.status.staged.iter().any(|f| f.path == "staged.txt"),
        "staged: {:?}",
        snap.status.staged
    );
    assert!(
        snap.status.unstaged.iter().any(|f| f.path == "tracked.txt"),
        "unstaged: {:?}",
        snap.status.unstaged
    );
    assert!(
        snap.status
            .untracked
            .iter()
            .any(|f| f.path == "untracked.txt"),
        "untracked: {:?}",
        snap.status.untracked
    );
}

#[test]
fn non_git_path_is_an_error() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(open_repo(tmp.path()).is_err());
}
