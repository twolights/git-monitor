# Phase 0 Research: Git Status Watcher

**Feature**: `001-git-status-watch` | **Date**: 2026-06-28

All Technical Context items were resolvable from the spec, the source instructions (Rust + libraries for git and TUI), and established ecosystem practice. No `NEEDS CLARIFICATION` markers remain. Decisions below.

---

## R1. TUI framework

- **Decision**: `ratatui` with the `crossterm` backend.
- **Rationale**: `ratatui` is the actively maintained successor to `tui-rs` and the de-facto standard for Rust terminal UIs. It provides exactly what this feature needs: a `Layout` engine for splitting the screen (a `Constraint`-based vertical layout gives the required horizontal-divider split), `List`/`Paragraph`/`Block` widgets for the status groups and commit list, automatic handling of terminal resize, and an immediate-mode redraw model that pairs naturally with a poll-and-redraw loop. `crossterm` is cross-platform (macOS/Linux/Windows), supports raw mode, the alternate screen, and event polling with a timeout — the timeout doubles as our refresh tick.
- **Alternatives considered**:
  - `cursive` — higher-level/retained-mode; heavier and less idiomatic for a simple read-only dashboard.
  - Hand-rolled ANSI escapes — rejected; reinvents layout, resize handling, and raw-mode management (violates simplicity/quality goals).
  - `termion` backend — Unix-only; `crossterm` is preferred for cross-platform support.

## R2. Git access

- **Decision**: `git2` (Rust bindings to libgit2) for all repository reads.
- **Rationale**: In-process library calls are faster and more robust than shelling out to `git` and parsing text, and they avoid spawning processes on every refresh tick. `git2` exposes everything required: `Repository::discover`/`open` (locate repo, satisfies default-cwd and explicit-directory cases), `Repository::head` + `Reference` (current branch name / detached-HEAD detection), `Repository::statuses` with `StatusOptions` (untracked / staged / unstaged grouping, FR-005), and `revwalk` over `HEAD` for the 10 most recent commits with `Commit` accessors for short id, summary, author, and time (FR-006). All operations are reads — nothing in this path mutates the repo, satisfying FR-009/SC-006.
- **Alternatives considered**:
  - Shelling out to `git status --porcelain=v2` / `git log` — workable but adds parsing surface, process-spawn latency per tick, and dependency on a `git` binary in PATH; rejected for the steady-state refresh loop.
  - `gitoxide` (`gix`) — pure-Rust and promising, but `git2`/libgit2 is more mature and complete for the status+log read set needed here; revisit later if a pure-Rust build is desired.

## R3. Real-time update strategy

- **Decision**: Single-threaded event loop using `crossterm`'s `event::poll(timeout)` with a default timeout of ~500 ms. Each loop iteration: drain any key events (handle quit), then on timeout (or after a relevant event) re-read the full `RepoSnapshot` from git and redraw.
- **Rationale**: Polling git state on a sub-second tick is the simplest mechanism that detects *all* relevant changes uniformly — working-tree edits, index/staging changes, and new commits — because every one of them is reflected by re-running `statuses` + branch + revwalk. A 500 ms tick comfortably meets the ≤2 s freshness requirement (SC-002) while keeping CPU negligible for a single repo. Single-threaded keeps the design trivial (no channels, no shared-state locking).
- **Refinement (optional, not required for v1)**: a filesystem watcher (`notify` crate) could trigger immediate refreshes instead of waiting for the next tick. Deferred — it adds threads/event plumbing for marginal benefit given the 2 s target. Recorded here so `/speckit.tasks` can treat it as an optional enhancement, not core scope.
- **Alternatives considered**:
  - Pure `notify`-driven refresh (no polling) — risks missing changes not surfaced as fs events on every platform and complicates detached-HEAD/branch-switch detection; rejected as sole mechanism.
  - Fixed full-screen redraw every tick regardless of change — acceptable but we can cheaply skip redraw when the snapshot is unchanged to reduce flicker.

## R4. CLI argument parsing

- **Decision**: `clap` (derive API). Single optional positional/`--dir` argument for the target directory; defaults to the current working directory when omitted.
- **Rationale**: `clap` derive gives a typed args struct, auto-generated `--help`/`--version`, and clear usage errors with standard exit codes — aligning with the "AI-friendly / scriptable CLI" guidance (predictable, documented interface). Matches FR-002 (default cwd) and FR-003 (specify directory).
- **Alternatives considered**: hand-parsed `std::env::args` — rejected; loses `--help`/validation for no benefit. `argh`/`pico-args` — lighter but `clap` is the ecosystem default and the size cost is irrelevant here.

## R5. Error handling & exit behavior

- **Decision**: Use `anyhow::Result` in app/main for ergonomic error propagation; on a non-git or inaccessible path, print a clear message to **stderr** and exit with a **non-zero** status *before* entering the alternate screen. Terminal teardown (leave alternate screen, disable raw mode) is guaranteed on all exit paths so the user's terminal is never left corrupted.
- **Rationale**: Satisfies FR-010/SC-005 (clear error, no misleading empty TUI) and FR-011 (clean exit). Validating the repository before taking over the screen avoids flashing an empty UI then erroring.
- **Alternatives considered**: panicking with a backtrace — rejected (ugly, leaves terminal in raw mode). Entering the TUI then showing an error pane — rejected for the "not a repo" case per the spec's explicit "report error and exit" wording.

## R6. Testing approach

- **Decision**: Keep git-reading logic as pure functions returning the `model.rs` data types; test them with integration tests that build fixture repositories in a `tempfile::TempDir` using `git2` (init repo, create/stage/commit files, produce detached HEAD, etc.) and assert on the resulting `RepoSnapshot`. Add a CLI-level test asserting the non-git-path error and exit code. Render logic is exercised lightly (snapshot of formatting helpers); full TUI rendering is left to manual/quickstart verification.
- **Rationale**: The repository-reading layer carries essentially all correctness risk and is fully testable headlessly. Driving a real terminal in CI is brittle, so the thin render layer is intentionally kept logic-light.
- **Alternatives considered**: `ratatui`'s `TestBackend` for buffer-level UI assertions — viable as a nice-to-have for the layout test, but not required for v1 correctness; may be added during `/speckit.tasks`.

---

## Resolved unknowns summary

| Technical Context item | Resolution |
|------------------------|------------|
| Language/Version | Rust stable, edition 2021, MSRV 1.75+ |
| TUI dependency | `ratatui` + `crossterm` (R1) |
| Git dependency | `git2` / libgit2 (R2) |
| Update mechanism | Poll + redraw loop, ~500 ms tick (R3); `notify` deferred |
| CLI parsing | `clap` derive (R4) |
| Error/exit | `anyhow`, stderr + non-zero exit, validate before TUI (R5) |
| Testing | Fixture repos via `tempfile`+`git2` (R6) |
