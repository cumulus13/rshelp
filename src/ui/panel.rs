//! Bordered panel rendering, the Rust equivalent of Python `rich.panel.Panel`.

use super::Theme;

/// Render `body` inside a full-width box-drawn panel with an optional title
/// embedded in the top border, colored with `hex`. Body text itself is left
/// in the terminal's default foreground so it stays readable regardless of
/// the accent color chosen for the frame.
pub fn panel(theme: &Theme, title: &str, body: &str, hex: &str) -> String {
    let width = theme.width.max(24);
    let inner = width.saturating_sub(4).max(10); // "│ " + content + " │"

    let mut lines: Vec<String> = Vec::new();
    for paragraph in body.split('\n') {
        if paragraph.trim().is_empty() {
            lines.push(String::new());
        } else {
            for wrapped in textwrap::wrap(paragraph, inner) {
                lines.push(wrapped.into_owned());
            }
        }
    }
    if lines.is_empty() {
        lines.push(String::new());
    }

    let title_segment = if title.is_empty() {
        "─".repeat(width.saturating_sub(2))
    } else {
        let t = format!(" {title} ");
        let dashes = width.saturating_sub(2).saturating_sub(t.chars().count());
        format!("{t}{}", "─".repeat(dashes))
    };

    let mut out = String::new();
    out.push_str(&theme.c(&format!("┌{title_segment}┐"), hex));
    out.push('\n');

    for line in &lines {
        let pad = inner.saturating_sub(line.chars().count());
        out.push_str(&theme.c("│ ", hex));
        out.push_str(line);
        out.push_str(&" ".repeat(pad));
        out.push_str(&theme.c(" │", hex));
        out.push('\n');
    }

    out.push_str(&theme.c(&format!("└{}┘", "─".repeat(width.saturating_sub(2))), hex));
    out
}

/// Convenience wrapper: a single-line status panel, e.g. success/error
/// banners, matching the little colored boxes `pyhelp` prints after each
/// operation.
pub fn status(theme: &Theme, text: &str, hex: &str) -> String {
    panel(theme, "", text, hex)
}
