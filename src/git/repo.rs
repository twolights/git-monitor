//! Open a repository and read its state into a [`RepoSnapshot`] (read-only).

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use git2::{ErrorCode, Repository, Sort, Status, StatusOptions};

use crate::git::format::relative_date;
use crate::model::{BranchInfo, ChangeKind, CommitInfo, FileChange, RepoSnapshot, StatusGroups};

/// Maximum number of recent commits to display.
const RECENT_COMMIT_LIMIT: usize = 10;

/// Open the git repository that contains `path`, searching upward from it.
///
/// Returns an error (used to print a message and exit) when `path` is not
/// inside a git repository.
pub fn open_repo(path: &Path) -> Result<Repository> {
    Repository::discover(path)
        .with_context(|| format!("'{}' is not inside a git repository", path.display()))
}

/// Build a complete point-in-time snapshot of the repository. Read-only: this
/// never mutates the repository, its index, or its history.
pub fn read_snapshot(repo: &Repository) -> Result<RepoSnapshot> {
    let workdir = repo.workdir().unwrap_or_else(|| repo.path());
    Ok(RepoSnapshot {
        path: workdir.to_path_buf(),
        branch: read_branch(repo)?,
        status: read_status(repo)?,
        commits: read_commits(repo)?,
    })
}

fn read_branch(repo: &Repository) -> Result<BranchInfo> {
    let head = match repo.head() {
        Ok(head) => head,
        // No commits yet: HEAD points at a ref that doesn't exist.
        Err(e) if e.code() == ErrorCode::UnbornBranch => {
            let name = repo.find_reference("HEAD").ok().and_then(|r| {
                r.symbolic_target()
                    .map(|s| s.trim_start_matches("refs/heads/").to_string())
            });
            return Ok(BranchInfo {
                name,
                detached: false,
                head_short_id: None,
            });
        }
        Err(e) => return Err(e).context("failed to read HEAD"),
    };

    let detached = repo.head_detached().unwrap_or(false);
    let head_short_id = head
        .target()
        .and_then(|oid| repo.find_object(oid, None).ok())
        .and_then(|obj| obj.short_id().ok())
        .and_then(|buf| buf.as_str().map(|s| s.to_string()));

    let name = if detached {
        None
    } else {
        head.shorthand().map(|s| s.to_string())
    };

    Ok(BranchInfo {
        name,
        detached,
        head_short_id,
    })
}

fn read_status(repo: &Repository) -> Result<StatusGroups> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .include_ignored(false)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true);

    let statuses = repo
        .statuses(Some(&mut opts))
        .context("failed to read repository status")?;

    let mut groups = StatusGroups::default();
    for entry in statuses.iter() {
        let s = entry.status();
        let path = entry.path().unwrap_or_default().to_string();

        if s.contains(Status::CONFLICTED) {
            groups.unstaged.push(FileChange {
                path,
                kind: ChangeKind::Conflicted,
            });
            continue;
        }
        if s.contains(Status::WT_NEW) {
            groups.untracked.push(FileChange {
                path: path.clone(),
                kind: ChangeKind::Untracked,
            });
        }
        if let Some(kind) = staged_kind(s) {
            groups.staged.push(FileChange {
                path: path.clone(),
                kind,
            });
        }
        if let Some(kind) = unstaged_kind(s) {
            groups.unstaged.push(FileChange { path, kind });
        }
    }

    // Stable, path-sorted order so the display does not jitter between refreshes.
    groups.staged.sort_by(|a, b| a.path.cmp(&b.path));
    groups.unstaged.sort_by(|a, b| a.path.cmp(&b.path));
    groups.untracked.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(groups)
}

fn staged_kind(s: Status) -> Option<ChangeKind> {
    if s.contains(Status::INDEX_NEW) {
        Some(ChangeKind::Added)
    } else if s.contains(Status::INDEX_MODIFIED) {
        Some(ChangeKind::Modified)
    } else if s.contains(Status::INDEX_DELETED) {
        Some(ChangeKind::Deleted)
    } else if s.contains(Status::INDEX_RENAMED) {
        Some(ChangeKind::Renamed)
    } else if s.contains(Status::INDEX_TYPECHANGE) {
        Some(ChangeKind::TypeChange)
    } else {
        None
    }
}

fn unstaged_kind(s: Status) -> Option<ChangeKind> {
    if s.contains(Status::WT_MODIFIED) {
        Some(ChangeKind::Modified)
    } else if s.contains(Status::WT_DELETED) {
        Some(ChangeKind::Deleted)
    } else if s.contains(Status::WT_RENAMED) {
        Some(ChangeKind::Renamed)
    } else if s.contains(Status::WT_TYPECHANGE) {
        Some(ChangeKind::TypeChange)
    } else {
        None
    }
}

fn read_commits(repo: &Repository) -> Result<Vec<CommitInfo>> {
    let mut commits = Vec::new();

    let mut revwalk = match repo.revwalk() {
        Ok(r) => r,
        Err(_) => return Ok(commits),
    };
    // No commits yet (unborn branch) → empty list, not an error.
    if revwalk.push_head().is_err() {
        return Ok(commits);
    }
    let _ = revwalk.set_sorting(Sort::TIME);

    let now = now_secs();
    for oid in revwalk.take(RECENT_COMMIT_LIMIT) {
        let oid = match oid {
            Ok(oid) => oid,
            Err(_) => continue,
        };
        let commit = match repo.find_commit(oid) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let short_hash = commit
            .as_object()
            .short_id()
            .ok()
            .and_then(|buf| buf.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| oid.to_string().chars().take(7).collect());
        let summary = commit.summary().unwrap_or("").to_string();
        let author = commit.author().name().unwrap_or("unknown").to_string();
        let relative_date = relative_date(commit.time().seconds(), now);

        commits.push(CommitInfo {
            short_hash,
            summary,
            relative_date,
            author,
        });
    }

    Ok(commits)
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
