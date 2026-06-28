---

description: "Task list for Git Status Watcher implementation"
---

# Tasks: Git Status Watcher

**Input**: Design documents from `/specs/001-git-status-watch/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/cli-interface.md, quickstart.md

**Tests**: Test tasks ARE included — the plan's research decision R6 defines a headless testing strategy (fixture repos via `tempfile` + `git2`) and the project structure includes `tests/`. Tests are written before the implementation they cover within each story.

**Organization**: Tasks are grouped by user story (P1 → P2 → P3) so each story is an independently testable increment.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- All paths are relative to the repository root (`/Users/ykchen/Projects/ykchen/git-monitor/`)

## Path Conventions

Single Rust binary crate: `src/` and `tests/` at repository root, `Cargo.toml` at root (per plan.md Structure Decision).

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [X] T001 Initialize Cargo binary crate named `git-monitor` (set `package.name = "git-monitor"` and a `[[bin]]` named `git-monitor`) and declare dependencies in `Cargo.toml` (deps: `ratatui`, `crossterm`, `git2`, `clap` with `derive`, `anyhow`; dev-deps: `tempfile`)
- [X] T002 [P] Create module skeleton files with empty stubs: `src/main.rs`, `src/cli.rs`, `src/app.rs`, `src/model.rs`, `src/git/mod.rs`, `src/git/repo.rs`, `src/git/format.rs`, `src/ui/mod.rs`, `src/ui/render.rs`
- [X] T003 [P] Add `rustfmt.toml`, a clippy lint config, and `.gitignore` entry for `target/` at repository root

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 [P] Define domain types in `src/model.rs`: `RepoSnapshot`, `BranchInfo`, `StatusGroups`, `FileChange`, `ChangeKind` (enum), `CommitInfo`, plus the `is_clean` helper — per data-model.md
- [X] T005 Implement repository open/discover with a not-a-repo error path in `src/git/repo.rs` (`open_repo(path) -> anyhow::Result<Repository>` using `Repository::discover`)
- [X] T006 [P] Implement terminal lifecycle guard in `src/app.rs`: enable raw mode + enter alternate screen on start, and guarantee restore (leave alt screen, disable raw mode) on every exit path (RAII drop guard)
- [X] T007 Implement the base CLI argument struct in `src/cli.rs` (clap derive) with NO directory argument yet, plus a `target_dir()` accessor that returns the current working directory (sole path-resolution point)
- [X] T008 Implement the app event/refresh loop skeleton in `src/app.rs`: `crossterm::event::poll(~500ms)` tick, quit on `q`/`Esc`/`Ctrl-C`, and a refresh→render cycle that rebuilds a `RepoSnapshot` (depends on T004, T005, T006)
- [X] T009 [P] Implement the UI shell in `src/ui/render.rs`: a `Layout` with a horizontal divider (status pane on top, commits pane on bottom) and a `render(frame, &RepoSnapshot)` dispatch that draws empty panes for now (depends on T004)
- [X] T010 Wire `src/main.rs`: parse args → resolve target path → `open_repo` (on error: message to stderr, exit code 1) → run the app loop (depends on T005, T007, T008)

**Checkpoint**: App launches in a repo, shows an empty split layout, refreshes on a tick, and quits cleanly restoring the terminal.

---

## Phase 3: User Story 1 - Monitor repository status in real time (Priority: P1) 🎯 MVP

**Goal**: Show the current branch and live working-tree status (staged, unstaged, untracked, or clean) in the top pane, updating automatically as the repository changes.

**Independent Test**: Launch in a repo; create an untracked file, stage a file, modify a tracked file from another shell — each change appears in the correct group within ~2 s, and the branch name is shown; a clean repo shows a "working tree clean" indicator.

### Tests for User Story 1

- [X] T011 [P] [US1] Integration test in `tests/status_reads.rs`: build a fixture repo (`tempfile` + `git2`), assert `RepoSnapshot` reports correct branch name, the three status groups for untracked/staged/unstaged files, and `is_clean == true` for a clean tree

### Implementation for User Story 1

- [X] T012 [P] [US1] Implement current-branch read into `BranchInfo` (incl. detached-HEAD and unborn-branch detection) in `src/git/repo.rs`
- [X] T013 [US1] Implement working-tree status read into `StatusGroups` / `FileChange` / `ChangeKind` using `git2` `statuses` with `StatusOptions` (include untracked) in `src/git/repo.rs` (same file as T012)
- [X] T014 [US1] Render the top status pane in `src/ui/render.rs`: branch header, three labeled groups with per-file change indicators, and the clean-tree indicator (FR-005, FR-012)
- [X] T015 [US1] Populate `RepoSnapshot.branch` and `RepoSnapshot.status` on each refresh tick in `src/app.rs` (depends on T012, T013)

**Checkpoint**: MVP — branch + live status fully functional and independently testable; commits pane may remain empty.

---

## Phase 4: User Story 2 - View recent commit history alongside status (Priority: P2)

**Goal**: Show the 10 most recent commits (short hash · summary · relative date · author) in the bottom pane, updating when new commits appear.

**Independent Test**: In a repo with history, the bottom pane lists up to 10 commits newest-first with the four fields; an empty commit made from another shell appears at the top within ~2 s.

### Tests for User Story 2

- [X] T016 [P] [US2] Integration test in `tests/commit_reads.rs`: fixture repo with >10 commits → `RepoSnapshot.commits` has exactly 10, newest-first, each with short hash, summary, relative date, and author
- [X] T017 [P] [US2] Unit test for the relative-date formatting helper (seconds/hours/days ago) as an inline `#[cfg(test)]` module in `src/git/format.rs`

### Implementation for User Story 2

- [X] T018 [US2] Implement recent-commits read in `src/git/repo.rs`: revwalk from HEAD, cap at 10, map each to `CommitInfo` (short id, summary, author, time)
- [X] T019 [US2] Implement the relative-date formatting helper in `src/git/format.rs` (same file as T017; write after the test)
- [X] T020 [US2] Render the bottom commits pane in `src/ui/render.rs`: one row per commit showing hash · summary · relative date · author (FR-006)
- [X] T021 [US2] Populate `RepoSnapshot.commits` on each refresh tick in `src/app.rs` (depends on T018)

**Checkpoint**: Both panes populated and live; US1 and US2 work independently.

---

## Phase 5: User Story 3 - Watch a repository other than the current directory (Priority: P3)

**Goal**: Accept a target directory argument and watch that repository; default to cwd; report a clear error and exit for a non-git path.

**Independent Test**: Run with a path to another repo → panes reflect that repo; run with no arg → watches cwd; run with a non-git path → clear stderr message and exit code 1, with no TUI flash.

### Tests for User Story 3

- [X] T022 [P] [US3] Integration test in `tests/cli.rs`: invoking with an explicit fixture-repo path targets that repo; invoking with a non-git path produces a stderr message and exit code 1

### Implementation for User Story 3

- [X] T023 [US3] Add the optional positional `DIRECTORY` argument to the CLI struct and extend `target_dir()` to return it when present (else cwd) in `src/cli.rs` (FR-003)
- [X] T024 [US3] Validate the target is a git repository before entering the TUI; on failure print a clear message to stderr and exit with code 1 in `src/main.rs` (FR-010, SC-005)

**Checkpoint**: All three user stories independently functional.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Robustness and edge cases that span stories

- [X] T025 [P] Display detached-HEAD (`HEAD detached at <short-hash>`) and unborn-branch states in `src/ui/render.rs` (edge cases)
- [X] T026 [P] Handle terminal resize and content overflow (truncate or indicate overflow without corrupting layout) in `src/ui/render.rs` (FR-013, edge cases)
- [X] T027 [P] Skip redraw when the new `RepoSnapshot` equals the previous one, to reduce flicker, in `src/app.rs`
- [X] T028 [P] Add header (repo path) and footer keybind hint (`[q] quit`) per the display contract in `src/ui/render.rs`
- [X] T029 [P] Write `README.md` usage section and ensure `cargo fmt --check` and `cargo clippy -- -D warnings` pass
- [X] T030 Run `quickstart.md` validation: all user-story scenarios, edge checks, and the read-only guarantee (SC-006)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories
- **User Stories (Phase 3–5)**: All depend on Foundational; can then proceed in parallel or in priority order (P1 → P2 → P3)
- **Polish (Phase 6)**: Depends on the desired user stories being complete

### User Story Dependencies

- **US1 (P1)**: Depends only on Foundational. No dependency on other stories.
- **US2 (P2)**: Depends only on Foundational. Independent of US1 (different `RepoSnapshot` field + different render pane).
- **US3 (P3)**: Depends only on Foundational. Independent of US1/US2 (touches `cli.rs` + `main.rs` only).

### Within Each User Story

- The story's test task is written first and should fail before its implementation tasks.
- In `git/repo.rs`, branch read (T012) precedes status read (T013) as they share the file.
- App-loop wiring (T015 / T021) depends on the corresponding git-read tasks.

### Parallel Opportunities

- Setup: T002 and T003 in parallel.
- Foundational: T004, T006, T009 in parallel (distinct files); T005/T007 also independent; T008 and T010 are integration points and come after their deps.
- After Foundational, US1 / US2 / US3 can be developed in parallel by different people (US1↔US2 share `app.rs` and `ui/render.rs` for separate panes — coordinate or sequence those two edits; US3 is fully isolated).
- Test tasks marked [P] (T011, T016, T017, T022) can run in parallel.
- Most Polish tasks (T025–T029) are [P].

---

## Parallel Example: Foundational Phase

```bash
# Distinct files, no interdependencies — launch together:
Task: "Define domain types in src/model.rs"            # T004
Task: "Implement terminal lifecycle guard in src/app.rs"  # T006
Task: "Implement UI shell layout in src/ui/render.rs"  # T009
```

## Parallel Example: User Story tests

```bash
Task: "US1 status/branch integration test in tests/status_reads.rs"  # T011
Task: "US2 recent-commits integration test in tests/commit_reads.rs" # T016
Task: "US3 CLI/non-git-path test in tests/cli.rs"                  # T022
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 (Setup) and Phase 2 (Foundational).
2. Complete Phase 3 (US1).
3. **STOP and VALIDATE**: launch in a repo, verify branch + live status updates and the clean indicator.
4. This is a usable, demoable MVP — a live `git status` dashboard.

### Incremental Delivery

1. Setup + Foundational → app shell runs.
2. + US1 → live status (MVP).
3. + US2 → recent commits pane.
4. + US3 → arbitrary directory + robust error handling.
5. + Polish → detached HEAD, resize/overflow, flicker-free redraw, docs, quickstart validation.

---

## Notes

- [P] = different files, no incomplete-task dependencies.
- US1 and US2 both edit `src/app.rs` and `src/ui/render.rs` but in separate panes/fields; if worked in parallel, coordinate those two files or sequence the edits.
- Optional future enhancement (out of scope, from research R3): replace/augment polling with a `notify` filesystem watcher for instant refresh — not a task here.
- Commit after each task or logical group; verify each story at its checkpoint.
