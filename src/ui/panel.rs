//! Bordered panel rendering, the Rust equivalent of Python `rich.panel.Panel`.

use super::{visible_width, Theme};

/// The usable content width inside a panel frame for this theme -- callers
/// that need to wrap and/or syntax-highlight text themselves before handing
/// lines to [`panel_lines`] should wrap to this width.
pub fn inner_width(theme: &Theme) -> usize {
    theme.width.max(24).saturating_sub(4).max(10)
}

/// Render `body` inside a full-width box-drawn panel with an optional title
/// embedded in the top border, colored with `hex`. Body text itself is left
/// in the terminal's default foreground so it stays readable regardless of
/// the accent color chosen for the frame.
///
/// `body` is assumed to be *plain* (uncolored) text: it gets word-wrapped
/// internally, and wrapping already-colored text would count invisible
/// ANSI escape bytes as visible characters and break alignment. For
/// pre-rendered / syntax-highlighted content, wrap and colorize it
/// yourself and use [`panel_lines`] instead.
pub fn panel(theme: &Theme, title: &str, body: &str, hex: &str) -> String {
    let inner = inner_width(theme);

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

    panel_lines(theme, title, &lines, hex)
}

/// Render already-wrapped lines (which may already contain ANSI color
/// codes, e.g. from [`crate::highlight::highlight_line`]) inside a framed
/// panel, without re-wrapping or re-flowing them. Padding is computed with
/// [`visible_width`] so ANSI escapes and double-width emoji don't throw off
/// alignment. Lines wider than the panel are left as-is (no padding, frame
/// simply widens visually for that line) rather than corrupted by naive
/// truncation.
pub fn panel_lines(theme: &Theme, title: &str, lines: &[String], hex: &str) -> String {
    let width = theme.width.max(24);
    let inner = inner_width(theme);

    let title_segment = if title.is_empty() {
        "─".repeat(width.saturating_sub(2))
    } else {
        let t = format!(" {title} ");
        let dashes = (width.saturating_sub(2)).saturating_sub(visible_width(&t));
        format!("{t}{}", "─".repeat(dashes))
    };

    let mut out = String::new();
    out.push_str(&theme.c(&format!("┌{title_segment}┐"), hex));
    out.push('\n');

    for line in lines {
        let w = visible_width(line);
        let pad = inner.saturating_sub(w);
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
