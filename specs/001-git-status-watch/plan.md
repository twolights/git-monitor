# Implementation Plan: Git Status Watcher

**Branch**: `001-git-status-watch` | **Date**: 2026-06-28 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-git-status-watch/spec.md`

## Summary

A read-only command-line tool that opens a full-screen terminal UI to continuously monitor a single local git repository. It shows the current branch and live working-tree status (untracked, staged, unstaged) in a top pane and the 10 most recent commits (short hash, summary, relative date, author) in a bottom pane, split by a horizontal divider. The view refreshes automatically as the repository changes. Implemented in Rust using `ratatui` + `crossterm` for the TUI and `git2` (libgit2 bindings) for repository reads, with `clap` for argument parsing. Real-time updates are achieved with a single-threaded event loop that re-reads git state on a short poll tick (default ~500 ms), comfortably meeting the ≤2 s freshness target without mutating the repository.

## Technical Context

**Language/Version**: Rust (stable, edition 2021, MSRV 1.75+)  
**Primary Dependencies**: `ratatui` (terminal UI widgets/layout), `crossterm` (cross-platform terminal backend + event polling), `git2` (libgit2 bindings for status/branch/commit reads), `clap` (CLI argument parsing, derive)  
**Storage**: N/A — read-only access to an existing git repository; no persistence  
**Testing**: `cargo test`; unit tests inline (`#[cfg(test)]`), integration tests in `tests/` using `tempfile` + `git2` to build throwaway fixture repositories  
**Target Platform**: Interactive terminal on macOS and Linux (Windows supported via `crossterm`, best-effort)  
**Project Type**: Single-binary CLI / terminal application  
**Performance Goals**: Initial render ≤2 s (SC-001); reflect repository changes ≤2 s with no user interaction (SC-002); UI redraw stays responsive for repositories with 20+ changed files (SC-003)  
**Constraints**: Strictly read-only — repository contents, index, and history byte-for-byte unchanged after a session (SC-006, FR-009); low memory footprint; graceful degradation on small/resized terminals and oversized content (FR-013); clear error + exit on non-git path (FR-010, SC-005)  
**Scale/Scope**: Single local repository, single user, one terminal session; fixed 10 recent commits; no configuration or persistence in v1

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

The project constitution (`.specify/memory/constitution.md`) is an unratified template containing only placeholder principles — it defines no concrete, binding gates. There are therefore no constitutional rules to violate.

The implementation nonetheless adheres to the spirit of the common Spec Kit principles:
- **CLI interface**: the deliverable *is* a CLI; errors go to stderr with a non-zero exit, the TUI to the terminal.
- **Simplicity / YAGNI**: single binary, no persistence, polling refresh over an event-watch abstraction, no interactive git operations (matches the spec's "simple, read-only" scope).
- **Test-First / Integration testing**: git-reading logic is isolated behind a pure data model so it can be unit/integration tested against fixture repos without a live TUI.

**Result**: PASS (no gates defined; no justified or unjustified violations). Re-evaluated post-design — still PASS; the design introduces no complexity requiring a Complexity Tracking entry.

## Project Structure

### Documentation (this feature)

```text
specs/001-git-status-watch/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/
│   └── cli-interface.md # Phase 1 output — CLI contract (args, exit codes, output)
├── checklists/
│   └── requirements.md  # Spec quality checklist (/speckit.specify output)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
Cargo.toml               # crate manifest + dependencies
src/
├── main.rs              # entry point: parse args, run app, map errors to exit codes
├── cli.rs               # clap argument definitions (optional directory)
├── app.rs               # application state + event/refresh loop orchestration
├── model.rs             # domain types: RepoSnapshot, FileChange, StatusGroups, CommitInfo, BranchInfo
├── git/
│   ├── mod.rs           # module surface
│   ├── repo.rs          # open repository; read branch, status groups, recent commits via git2
│   └── format.rs        # relative-date formatting helper
└── ui/
    ├── mod.rs           # module surface
    └── render.rs        # ratatui layout (horizontal split) + widget rendering from RepoSnapshot

tests/
├── status_reads.rs      # integration (US1): fixture repos → branch + status groups
├── commit_reads.rs      # integration (US2): fixture repos → recent commits
└── cli.rs               # integration: arg handling, non-git path error + exit code
```

**Structure Decision**: Single-binary Rust application (the default single-project layout). Repository-reading logic lives in `src/git/` and produces plain data types in `src/model.rs`; the TUI in `src/ui/` is a pure function of those types. This separation keeps all git logic testable headlessly (the bulk of the risk) while the rendering layer stays thin. No workspace/multi-crate split is warranted for a tool of this size (YAGNI).

## Complexity Tracking

> No Constitution Check violations — this section is intentionally empty.
