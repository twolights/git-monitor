# CLI Contract: Git Status Watcher

**Feature**: `001-git-status-watch` | **Date**: 2026-06-28

This tool's external interface is its command line and its terminal rendering. This contract defines the invocation surface, exit codes, and the displayed layout so the behavior is verifiable and stable. The binary name is `git-monitor` (see tasks.md T001).

---

## Invocation

```text
git-monitor [OPTIONS] [DIRECTORY]
```

### Arguments

| Argument | Required | Default | Description |
|----------|----------|---------|-------------|
| `DIRECTORY` | No | current working directory | Path to the git repository (or any subdirectory of it) to watch. Satisfies FR-002 (default cwd) and FR-003 (explicit directory). |

### Options

| Option | Description |
|--------|-------------|
| `-h`, `--help` | Print usage and exit 0. |
| `-V`, `--version` | Print version and exit 0. |

> Note: `DIRECTORY` is provided as a positional argument. An equivalent `-d/--dir` flag MAY be added; if so, the positional remains supported for ergonomics.

### Examples

```bash
git-monitor                 # watch the repository in the current directory
git-monitor ../other-repo   # watch a repository elsewhere
git-monitor --help          # usage
```

---

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | Normal termination — user quit the live view (FR-011), or `--help`/`--version`. |
| `1` | Runtime error — e.g., `DIRECTORY` is not inside a git repository (FR-010/SC-005), path inaccessible, or repository became unreadable. A human-readable message is written to **stderr**. |
| `2` | Usage error — invalid arguments/options (emitted by the arg parser). |

**Guarantee**: on every exit path the terminal is restored (alternate screen left, raw mode disabled). The watched repository is never modified (FR-009/SC-006).

---

## Standard streams

- **stdout**: the full-screen TUI (alternate screen) while running; nothing parseable is written for scripting in v1.
- **stderr**: error messages (non-git path, read failures), one clear line; no TUI noise.
- The tool requires an interactive TTY; behavior when stdout is not a TTY is out of scope for v1 (Assumptions).

---

## Display contract (TUI layout)

The screen is split by a **horizontal divider** into two stacked panes (Clarifications):

```text
┌────────────────────────────────────────────────────────────┐
│ Repo: /path/to/repo            Branch: main                 │  header line(s)
├────────────────────────────────────────────────────────────┤
│ STATUS                                                      │  ── top pane ──
│   Staged (changes to be committed):                        │
│     M  src/app.rs                                           │
│   Changes not staged:                                      │
│     M  README.md                                           │
│   Untracked:                                               │
│     ?? notes.txt                                           │
│   (working tree clean)            ← shown when is_clean    │
├────────────────────────────────────────────────────────────┤
│ RECENT COMMITS                                             │  ── bottom pane ──
│   a1b2c3d  Fix render bug          2 hours ago   E. Chen   │
│   9f8e7d6  Add commit pane         1 day ago     E. Chen   │
│   ...                              (up to 10 rows)         │
└────────────────────────────────────────────────────────────┘
 [q] quit                                                       footer / keybind hint
```

### Layout requirements

| Requirement | Contract |
|-------------|----------|
| Branch shown | Top/header shows current branch; detached HEAD shown as `HEAD detached at <short-hash>` (FR-004, edge case). |
| Status groups | Three labeled groups: staged, unstaged, untracked, each item prefixed with a short change indicator (FR-005). |
| Clean state | When no changes exist, an explicit "working tree clean" indicator is shown (FR-012). |
| Recent commits | Up to 10 rows, newest first, each: short hash · summary · relative date · author (FR-006). |
| Split view | Status pane on top, commits pane on bottom, both visible simultaneously (FR-007). |
| Overflow | Content exceeding a pane is truncated or indicated without corrupting layout (FR-013). |
| Resize | Layout adapts to terminal resize without corruption (edge case). |

### Keybindings

| Key | Action |
|-----|--------|
| `q` or `Esc` or `Ctrl-C` | Quit and restore terminal (FR-011). |

No other keys mutate anything — the tool is read-only (FR-009). Scrolling MAY be added later but is not required for v1.

---

## Refresh contract

- The view updates automatically (no manual refresh) within **2 seconds** of any repository change — file created/modified/deleted, staged/unstaged, or new commit (FR-008/SC-002).
- Initial content appears within **2 seconds** of launch (SC-001).
