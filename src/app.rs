//! Application orchestration: terminal lifecycle and the watch loop.

use std::io::{self, Stdout};
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use git2::Repository;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::git::repo::read_snapshot;
use crate::model::RepoSnapshot;
use crate::ui::render::render;

/// How often the loop wakes to re-read git state. Comfortably under the 2s
/// freshness target while keeping CPU negligible.
const TICK: Duration = Duration::from_millis(500);

/// RAII guard that takes over the terminal on creation and always restores it
/// on drop, so the user's terminal is never left in raw/alternate-screen mode.
struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    fn new() -> Result<Self> {
        enable_raw_mode().context("failed to enable raw mode")?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).context("failed to enter alternate screen")?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout))
            .context("failed to initialize terminal")?;
        // Force a full clear so leftover content on the alternate screen does
        // not show through cells we render as blank (ratatui diffs against an
        // empty buffer and would otherwise skip emitting unchanged blanks).
        terminal.clear().context("failed to clear terminal")?;
        Ok(Self { terminal })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

/// Run the watch loop until the user quits. Read-only throughout.
pub fn run(repo: Repository) -> Result<()> {
    let mut guard = TerminalGuard::new()?;

    let mut last: Option<RepoSnapshot> = None;
    let mut current = read_snapshot(&repo).ok();

    loop {
        // Redraw only when the snapshot changed (or on the first frame / after a
        // resize), to avoid flicker.
        if current != last {
            if let Some(snapshot) = &current {
                guard.terminal.draw(|frame| render(frame, snapshot))?;
            }
            last = current.clone();
        }

        if event::poll(TICK).context("failed to poll terminal events")? {
            match event::read().context("failed to read terminal event")? {
                Event::Key(key) if is_quit(&key) => break,
                // Force a redraw on the next iteration.
                Event::Resize(_, _) => last = None,
                _ => {}
            }
        }

        // Refresh after the tick / event. A transient read error keeps the last
        // good frame rather than crashing.
        current = read_snapshot(&repo).ok();
    }

    Ok(())
}

/// Whether a key event should quit: `q`, `Esc`, or `Ctrl-C`.
fn is_quit(key: &KeyEvent) -> bool {
    if key.kind == KeyEventKind::Release {
        return false;
    }
    matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
        || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
}
