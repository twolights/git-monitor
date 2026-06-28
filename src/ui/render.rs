//! Rendering of the split view: a header, a status pane (top), a recent-commits
//! pane (bottom) separated by a horizontal divider, and a footer keybind hint.
//!
//! Rendering is a pure function of the [`RepoSnapshot`]; the layout is
//! recomputed every draw, so terminal resizes are handled automatically and
//! overflowing content is clipped rather than corrupting the display.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::model::{BranchInfo, FileChange, RepoSnapshot};

/// Height (in rows) reserved for the bottom commits pane: 10 commits + borders.
const COMMITS_PANE_HEIGHT: u16 = 12;

/// Render the whole view for the given snapshot.
pub fn render(frame: &mut Frame, snapshot: &RepoSnapshot) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),                   // header
            Constraint::Min(3),                      // status pane (top)
            Constraint::Length(COMMITS_PANE_HEIGHT), // commits pane (bottom)
            Constraint::Length(1),                   // footer
        ])
        .split(frame.area());

    render_header(frame, chunks[0], snapshot);
    render_status(frame, chunks[1], snapshot);
    render_commits(frame, chunks[2], snapshot);
    render_footer(frame, chunks[3]);
}

/// Human-readable branch label, accounting for detached HEAD and unknown states.
fn branch_label(branch: &BranchInfo) -> String {
    if branch.detached {
        match &branch.head_short_id {
            Some(id) => format!("HEAD detached at {id}"),
            None => "HEAD (detached)".to_string(),
        }
    } else {
        match &branch.name {
            Some(name) => name.clone(),
            None => "(unknown)".to_string(),
        }
    }
}

fn render_header(frame: &mut Frame, area: Rect, snapshot: &RepoSnapshot) {
    let line = Line::from(vec![
        Span::styled("Repo: ", Style::default().fg(Color::DarkGray)),
        Span::raw(snapshot.path.display().to_string()),
        Span::raw("   "),
        Span::styled("Branch: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            branch_label(&snapshot.branch),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn change_line(fc: &FileChange, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {:<2} ", fc.kind.indicator()),
            Style::default().fg(color),
        ),
        Span::raw(fc.path.clone()),
    ])
}

fn render_status(frame: &mut Frame, area: Rect, snapshot: &RepoSnapshot) {
    let status = &snapshot.status;
    let mut lines: Vec<Line> = Vec::new();

    if snapshot.is_clean() {
        lines.push(Line::styled(
            "\u{2713} working tree clean",
            Style::default().fg(Color::Green),
        ));
    } else {
        if !status.staged.is_empty() {
            lines.push(Line::styled(
                "Changes to be committed:",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ));
            lines.extend(status.staged.iter().map(|fc| change_line(fc, Color::Green)));
        }
        if !status.unstaged.is_empty() {
            lines.push(Line::styled(
                "Changes not staged:",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ));
            lines.extend(status.unstaged.iter().map(|fc| change_line(fc, Color::Red)));
        }
        if !status.untracked.is_empty() {
            lines.push(Line::styled(
                "Untracked files:",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
            lines.extend(
                status
                    .untracked
                    .iter()
                    .map(|fc| change_line(fc, Color::Cyan)),
            );
        }
    }

    let block = Block::default().borders(Borders::ALL).title(" Status ");
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_commits(frame: &mut Frame, area: Rect, snapshot: &RepoSnapshot) {
    let mut lines: Vec<Line> = Vec::new();

    if snapshot.commits.is_empty() {
        lines.push(Line::styled(
            "(no commits yet)",
            Style::default().fg(Color::DarkGray),
        ));
    } else {
        for c in &snapshot.commits {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:<8}", c.short_hash),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(" "),
                Span::raw(truncate(&c.summary, 50)),
                Span::raw("  "),
                Span::styled(
                    c.relative_date.clone(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw("  "),
                Span::styled(c.author.clone(), Style::default().fg(Color::Blue)),
            ]));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Recent Commits ");
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled(" [q] ", Style::default().fg(Color::Black).bg(Color::Gray)),
        Span::raw(" quit"),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

/// Truncate `s` to at most `max` characters, appending an ellipsis when cut.
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let head: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{head}\u{2026}")
    }
}
