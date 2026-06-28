//! Shared helpers for building throwaway git repositories in integration tests.

use std::fs;
use std::path::Path;

use git2::{IndexAddOption, Repository, Signature};

/// Initialize a fresh repository in `dir` with a test identity configured.
pub fn init_repo(dir: &Path) -> Repository {
    let repo = Repository::init(dir).expect("init repo");
    {
        let mut cfg = repo.config().expect("config");
        cfg.set_str("user.name", "Test").expect("set name");
        cfg.set_str("user.email", "test@example.com")
            .expect("set email");
    }
    repo
}

/// Stage every change in the working tree and create a commit with `message`.
pub fn commit_all(repo: &Repository, message: &str) {
    let mut index = repo.index().expect("index");
    index
        .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .expect("add_all");
    index.write().expect("write index");
    let tree_id = index.write_tree().expect("write_tree");
    let tree = repo.find_tree(tree_id).expect("find_tree");
    let sig = Signature::now("Test", "test@example.com").expect("signature");

    let parent = repo
        .head()
        .ok()
        .and_then(|h| h.target())
        .and_then(|oid| repo.find_commit(oid).ok());
    let parents: Vec<&git2::Commit> = parent.iter().collect();

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
        .expect("commit");
}

/// Write `contents` to a file named `name` inside `dir`.
pub fn write_file(dir: &Path, name: &str, contents: &str) {
    fs::write(dir.join(name), contents).expect("write file");
}
