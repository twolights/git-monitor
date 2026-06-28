# Quickstart: Git Status Watcher

**Feature**: `001-git-status-watch` | **Date**: 2026-06-28

How to build, run, and verify the tool against the spec's acceptance scenarios.

## Prerequisites

- Rust toolchain (stable, 1.75+) via `rustup`.
- A C toolchain/CMake available for building libgit2 (the `git2` crate vendors libgit2 by default, so usually no system git lib is required).

## Build & run

```bash
# from the repository root
cargo build            # debug build
cargo run              # watch the current directory's repository
cargo run -- ../some-other-repo   # watch a different repository
cargo run -- --help    # usage

# release binary
cargo build --release
./target/release/git-monitor [DIRECTORY]
```

Press **`q`** (or `Esc` / `Ctrl-C`) to quit; the terminal is restored on exit.

## Verify the acceptance scenarios

Run the tool in one terminal and make changes from a second terminal.

### User Story 1 — live status (P1)
1. `cargo run` inside a clean repo → top pane shows the branch and a "working tree clean" indicator (FR-012, SC-001).
2. In another shell: `touch notes.txt` → within ~2 s, `notes.txt` appears under **Untracked** (SC-002).
3. `git add notes.txt` → it moves to **Staged**. Edit a tracked file → it appears under **Changes not staged**.
4. `git restore --staged notes.txt` / commit → entries update/disappear automatically.

### User Story 2 — recent commits (P2)
1. In a repo with history, the bottom pane lists up to 10 commits: short hash · summary · relative date · author (FR-006).
2. `git commit --allow-empty -m "test"` in another shell → the new commit appears at the top within ~2 s.

### User Story 3 — directory selection (P3)
1. `cargo run -- /path/to/another/repo` → panes reflect *that* repo, not the cwd.
2. `cargo run -- /tmp` (not a git repo) → clear error on **stderr**, exit code `1`, no TUI flash (FR-010, SC-005).

### Edge checks
- Detached HEAD (`git checkout <hash>`) → header shows `HEAD detached at <hash>`.
- Resize the terminal while running → layout adapts without corruption.
- Repo with many changed files → list truncates/indicates overflow without breaking layout (FR-013).

## Test

```bash
cargo test             # unit + integration (fixture repos via tempfile + git2)
cargo fmt --check
cargo clippy -- -D warnings
```

Integration tests build throwaway repositories in temp dirs, exercise the git-reading layer, and assert the resulting `RepoSnapshot` and the non-git-path error/exit behavior — no live terminal required.

## Confirm read-only guarantee (SC-006)

Record the repo state, run a session, and confirm nothing changed:

```bash
git -C <repo> rev-parse HEAD && git -C <repo> status --porcelain   # before
# ... run git-monitor, interact, quit ...
git -C <repo> rev-parse HEAD && git -C <repo> status --porcelain   # after — identical
```
