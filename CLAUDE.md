# git-monitor Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-06-28

## Active Technologies

- Rust (stable, edition 2021, MSRV 1.75+) + `ratatui` (terminal UI widgets/layout), `crossterm` (cross-platform terminal backend + event polling), `git2` (libgit2 bindings for status/branch/commit reads), `clap` (CLI argument parsing, derive) (001-git-status-watch)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust (stable, edition 2021, MSRV 1.75+): Follow standard conventions

## Recent Changes

- 001-git-status-watch: Added Rust (stable, edition 2021, MSRV 1.75+) + `ratatui` (terminal UI widgets/layout), `crossterm` (cross-platform terminal backend + event polling), `git2` (libgit2 bindings for status/branch/commit reads), `clap` (CLI argument parsing, derive)

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
