# Phase 1 Data Model: Git Status Watcher

**Feature**: `001-git-status-watch` | **Date**: 2026-06-28

These are in-memory, read-only value types produced by the git layer (`src/git/repo.rs`) and consumed by the UI layer (`src/ui/render.rs`). Nothing is persisted. A single `RepoSnapshot` is the immutable result of reading the repository at one instant; the app replaces it wholesale on each refresh.

---

## Entity: `RepoSnapshot`

The complete, point-in-time view of the watched repository. Rendered as a whole; recomputed each refresh tick.

| Field | Type | Description | Source / Rule |
|-------|------|-------------|---------------|
| `path` | `PathBuf` | Absolute path of the watched repository (working dir root) | Resolved from CLI arg or cwd (FR-002/FR-003) |
| `branch` | `BranchInfo` | Current branch or detached-HEAD state | FR-004; edge case: detached HEAD |
| `status` | `StatusGroups` | Working-tree changes grouped by stage | FR-005 |
| `commits` | `Vec<CommitInfo>` | Up to 10 most recent commits, newest first | FR-006 |
| `is_clean` | `bool` | True when all status groups are empty | FR-012; derived from `status` |

**Rules**:
- Produced only by reads; constructing a `RepoSnapshot` never mutates the repository (FR-009/SC-006).
- `commits` length is `min(10, total reachable from HEAD)`; empty when the repo has no commits yet (edge case).
- `is_clean == status.untracked.is_empty() && status.staged.is_empty() && status.unstaged.is_empty()`.

---

## Entity: `BranchInfo`

The current position of HEAD.

| Field | Type | Description |
|-------|------|-------------|
| `name` | `Option<String>` | Branch short name (e.g., `main`); `None` when detached |
| `detached` | `bool` | True when HEAD points directly at a commit, not a branch |
| `head_short_id` | `Option<String>` | Short hash of HEAD commit; shown when detached or repo has commits |

**State / display rule**: when `detached == true`, the UI shows e.g. `HEAD detached at <head_short_id>` instead of a branch name (edge case). When the repo has no commits, `name` may be the unborn branch (e.g., `main`) with `head_short_id == None`.

---

## Entity: `StatusGroups`

Working-tree file changes partitioned into the three groups shown to the user. Mirrors how `git status` presents state.

| Field | Type | Description | Maps to |
|-------|------|-------------|---------|
| `staged` | `Vec<FileChange>` | Changes staged for commit ("changes to be committed") | git2 index-vs-HEAD status flags |
| `unstaged` | `Vec<FileChange>` | Tracked changes not staged ("changes not staged for commit") | git2 worktree-vs-index status flags |
| `untracked` | `Vec<FileChange>` | New files not tracked by git | git2 `WT_NEW` |

**Rules**:
- A single path may legitimately appear in more than one group (e.g., partially staged), matching `git status` semantics.
- Each list preserves a stable order (path-sorted) so the display does not jitter between refreshes.

---

## Entity: `FileChange`

One file's change within a status group.

| Field | Type | Description |
|-------|------|-------------|
| `path` | `String` | Repository-relative path of the file |
| `kind` | `ChangeKind` | Nature of the change |

### Enum: `ChangeKind`

`Added` · `Modified` · `Deleted` · `Renamed` · `Untracked` · `TypeChange` · `Conflicted`

- Derived from `git2::Status` bit flags. Used to pick a short status indicator/label in the UI (e.g., `M`, `A`, `D`, `??`).

---

## Entity: `CommitInfo`

One entry in the recent-commits list (bottom pane).

| Field | Type | Description | Rule |
|-------|------|-------------|------|
| `short_hash` | `String` | Abbreviated commit id (e.g., 7 chars) | FR-006 |
| `summary` | `String` | First line of the commit message | FR-006; later lines ignored |
| `relative_date` | `String` | Human relative time, e.g., "2 hours ago" | FR-006; formatted from commit time |
| `author` | `String` | Author name | FR-006 |

**Ordering**: newest first (revwalk from HEAD), capped at 10 (Clarifications).

---

## Relationships

```text
RepoSnapshot
├── branch  : BranchInfo        (1)
├── status  : StatusGroups      (1)
│   ├── staged    : [FileChange]
│   ├── unstaged  : [FileChange]
│   └── untracked : [FileChange]   (FileChange.kind : ChangeKind)
└── commits : [CommitInfo]      (0..=10, newest first)
```

## Lifecycle

1. **Startup**: validate `path` is a git repository → on failure, error + exit (no snapshot built).
2. **Refresh tick (~500 ms)**: build a fresh `RepoSnapshot` from current git state.
3. **Compare & render**: if changed from the previous snapshot, redraw the two panes; otherwise skip redraw.
4. **Shutdown**: on quit key or fatal read error, tear down the terminal; no snapshot is persisted.
