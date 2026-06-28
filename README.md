# git-monitor

A read-only terminal UI that continuously watches a local git repository and
shows, in a split view:

- the current **branch** (or detached-HEAD state),
- the live **working-tree status** — staged, unstaged, and untracked files
  (or a "clean" indicator), and
- the **10 most recent commits** (short hash · summary · relative date · author).

The view refreshes automatically as the repository changes. It never modifies
the repository.

## Install / build

Requires a Rust toolchain (1.75+).

```bash
cargo build --release
# binary at target/release/git-monitor
```

## Usage

```bash
git-monitor                 # watch the repository in the current directory
git-monitor ../other-repo   # watch a repository elsewhere
git-monitor --help          # usage
git-monitor --version       # version
```

Press **`q`** (or `Esc` / `Ctrl-C`) to quit; the terminal is restored on exit.

If the target path is not inside a git repository, the tool prints an error to
stderr and exits with code `1`.

## Layout

```text
Repo: /path/to/repo            Branch: main
┌ Status ──────────────────────────────────────────────────┐
│ Changes to be committed:                                  │
│   M  src/app.rs                                           │
│ Untracked files:                                          │
│   ?? notes.txt                                            │
└──────────────────────────────────────────────────────────┘
┌ Recent Commits ──────────────────────────────────────────┐
│ a1b2c3d  Fix render bug         2 hours ago    E. Chen    │
│ 9f8e7d6  Add commit pane        1 day ago      E. Chen    │
└──────────────────────────────────────────────────────────┘
 [q]  quit
```

## Development

```bash
cargo test                       # unit + integration tests (fixture repos)
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
```

Architecture: the git-reading layer (`src/git/`) produces plain data types
(`src/model.rs`) that the UI (`src/ui/`) renders as a pure function of a
snapshot, so all repository logic is testable without a live terminal.
