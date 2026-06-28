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
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

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
    // Available text width inside the bordered block (minus left/right borders).
    let inner_width = area.width.saturating_sub(2) as usize;
    let mut lines: Vec<Line> = Vec::new();

    if snapshot.commits.is_empty() {
        lines.push(Line::styled(
            "(no commits yet)",
            Style::default().fg(Color::DarkGray),
        ));
    } else {
        // Fixed-width hash column.
        const HASH_W: usize = 8;
        for c in &snapshot.commits {
            let hash = format!("{:<width$}", c.short_hash, width = HASH_W);
            // Columns: hash + ' ' + summary + '  ' + date + '  ' + author.
            // The summary takes whatever width is left so the line fills the
            // pane exactly without wrapping or overflowing at any terminal size.
            let used = hash.width() + 1 + 2 + c.relative_date.width() + 2 + c.author.width();
            let summary_width = inner_width.saturating_sub(used);
            let summary = fit(&c.summary, summary_width);

            lines.push(Line::from(vec![
                Span::styled(hash, Style::default().fg(Color::Yellow)),
                Span::raw(" "),
                Span::raw(summary),
                Span::raw("  "),
                Span::styled(
                    c.relative_date.clone(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw("  "),
                Span::styled(c.author.clone(), Style::default().fg(Color::LightCyan)),
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

/// Fit `s` into exactly `width` *display columns*: pad with spaces when
/// narrower, truncate with a trailing ellipsis when wider. Width is measured by
/// Unicode display width (so wide glyphs like CJK and emoji count as 2 columns),
/// matching how ratatui renders the span — this keeps the trailing date/author
/// columns aligned even when a commit summary contains emoji.
fn fit(s: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let s_width = s.width();
    if s_width <= width {
        // Pad by the column deficit, not the character deficit.
        return format!("{s}{}", " ".repeat(width - s_width));
    }
    if width == 1 {
        return "\u{2026}".to_string();
    }
    // Accumulate whole grapheme clusters until adding the next would exceed
    // `width - 1`, leaving one column for the ellipsis. Iterating *graphemes*
    // (not chars) mirrors how ratatui renders, so a multi-codepoint emoji like
    // a 🧑‍💻 ZWJ sequence is measured and kept/dropped as a single unit rather
    // than split or miscounted. A wide cluster that won't fit is dropped and
    // the gap padded, so the result is exactly `width` columns wide.
    let budget = width - 1;
    let mut head = String::new();
    let mut used = 0usize;
    for g in s.graphemes(true) {
        let w = g.width();
        if used + w > budget {
            break;
        }
        head.push_str(g);
        used += w;
    }
    let pad = budget - used;
    format!("{head}{}\u{2026}", " ".repeat(pad))
}

#[cfg(test)]
mod tests {
    use super::fit;
    use unicode_width::UnicodeWidthStr;

    #[test]
    fn pads_ascii_to_exact_width() {
        let out = fit("hi", 5);
        assert_eq!(out, "hi   ");
        assert_eq!(out.width(), 5);
    }

    #[test]
    fn truncates_ascii_with_ellipsis() {
        let out = fit("hello world", 5);
        assert_eq!(out, "hell\u{2026}");
        assert_eq!(out.width(), 5);
    }

    #[test]
    fn emoji_summary_occupies_exact_columns_when_padded() {
        // "🚀 ci" is 5 display columns (emoji=2, space=1, "ci"=2).
        let s = "🚀 ci";
        assert_eq!(s.width(), 5);
        let out = fit(s, 10);
        // Must be padded to exactly 10 columns, not 10 chars.
        assert_eq!(out.width(), 10);
    }

    #[test]
    fn does_not_split_wide_char_on_truncation() {
        // Budget leaves one column before the ellipsis, but the emoji needs two,
        // so it is dropped rather than split; the gap is padded.
        let out = fit("🚀 x", 2);
        assert_eq!(out.width(), 2);
        assert!(!out.contains('🚀'));
        assert!(out.ends_with('\u{2026}'));
    }

    #[test]
    fn truncation_keeps_exact_width_with_emoji() {
        let out = fit("🚀🚀🚀🚀 release workflow", 9);
        assert_eq!(out.width(), 9);
    }

    #[test]
    fn truncation_keeps_zwj_emoji_cluster_whole() {
        // A 🧑‍💻 ZWJ sequence is 2 display columns but 3 codepoints; it must be
        // kept or dropped as one unit and never split mid-cluster.
        let out = fit("🧑‍💻 chore: long summary here", 6);
        assert_eq!(out.width(), 6);
        // Either the whole cluster survives or none of it does — never a lone
        // person/laptop fragment.
        let has_person = out.contains('\u{1F9D1}');
        let has_laptop = out.contains('\u{1F4BB}');
        assert_eq!(has_person, has_laptop);
    }

    #[test]
    fn zero_width_is_empty() {
        assert_eq!(fit("anything", 0), "");
    }
}
