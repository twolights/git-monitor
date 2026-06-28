//! In-memory, read-only value types describing a point-in-time view of the
//! watched repository. Nothing here is persisted; a fresh [`RepoSnapshot`] is
//! produced on each refresh and rendered as a whole.

use std::path::PathBuf;

/// The complete, point-in-time view of the watched repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoSnapshot {
    /// Absolute path of the repository working directory.
    pub path: PathBuf,
    /// Current branch or detached-HEAD state.
    pub branch: BranchInfo,
    /// Working-tree changes grouped by stage.
    pub status: StatusGroups,
    /// Up to 10 most recent commits, newest first.
    pub commits: Vec<CommitInfo>,
}

impl RepoSnapshot {
    /// True when there are no untracked, staged, or unstaged changes.
    pub fn is_clean(&self) -> bool {
        self.status.is_clean()
    }
}

/// The current position of HEAD.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BranchInfo {
    /// Branch short name (e.g., `main`); `None` when detached or unresolved.
    pub name: Option<String>,
    /// True when HEAD points directly at a commit rather than a branch.
    pub detached: bool,
    /// Short hash of the HEAD commit, when the repository has commits.
    pub head_short_id: Option<String>,
}

/// Working-tree file changes partitioned into the three displayed groups.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StatusGroups {
    /// Changes staged for commit ("changes to be committed").
    pub staged: Vec<FileChange>,
    /// Tracked changes not staged ("changes not staged for commit").
    pub unstaged: Vec<FileChange>,
    /// New files not tracked by git.
    pub untracked: Vec<FileChange>,
}

impl StatusGroups {
    /// True when all three groups are empty.
    pub fn is_clean(&self) -> bool {
        self.staged.is_empty() && self.unstaged.is_empty() && self.untracked.is_empty()
    }
}

/// One file's change within a status group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    /// Repository-relative path of the file.
    pub path: String,
    /// Nature of the change.
    pub kind: ChangeKind,
}

/// The nature of a file change, derived from git status flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Untracked,
    TypeChange,
    Conflicted,
}

impl ChangeKind {
    /// Short indicator shown next to a file in the UI.
    pub fn indicator(self) -> &'static str {
        match self {
            ChangeKind::Added => "A",
            ChangeKind::Modified => "M",
            ChangeKind::Deleted => "D",
            ChangeKind::Renamed => "R",
            ChangeKind::Untracked => "??",
            ChangeKind::TypeChange => "T",
            ChangeKind::Conflicted => "U",
        }
    }
}

/// One entry in the recent-commits list (bottom pane).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitInfo {
    /// Abbreviated commit id (e.g., 7 chars).
    pub short_hash: String,
    /// First line of the commit message.
    pub summary: String,
    /// Human relative time, e.g., "2 hours ago".
    pub relative_date: String,
    /// Author name.
    pub author: String,
}
