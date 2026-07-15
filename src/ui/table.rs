//! Minimal fixed-width table renderer, the Rust equivalent of Python
//! `rich.table.Table`.

use super::Theme;

/// Render a simple two-or-more column table. Column widths are computed
/// from the widest cell (header included) in each column; `hex` colors the
/// header row and the separator rule.
pub fn table(theme: &Theme, headers: &[&str], rows: &[Vec<String>], hex: &str) -> String {
    let cols = headers.len();
    let mut widths: Vec<usize> = headers.iter().map(|h| h.chars().count()).collect();

    for row in rows {
        for (i, cell) in row.iter().enumerate().take(cols) {
            let w = cell.chars().count();
            if w > widths[i] {
                widths[i] = w;
            }
        }
    }

    let pad_cell = |s: &str, w: usize| format!("{s}{}", " ".repeat(w.saturating_sub(s.chars().count())));

    let header_line = headers
        .iter()
        .enumerate()
        .map(|(i, h)| pad_cell(h, widths[i]))
        .collect::<Vec<_>>()
        .join("  ");

    let separator = widths
        .iter()
        .map(|w| "─".repeat(*w))
        .collect::<Vec<_>>()
        .join("  ");

    let mut out = String::new();
    out.push_str(&theme.cb(&header_line, hex));
    out.push('\n');
    out.push_str(&theme.c(&separator, hex));
    out.push('\n');

    for row in rows {
        let line = (0..cols)
            .map(|i| pad_cell(row.get(i).map(String::as_str).unwrap_or(""), widths[i]))
            .collect::<Vec<_>>()
            .join("  ");
        out.push_str(&line);
        out.push('\n');
    }

    // Drop the trailing newline so callers can decide their own spacing.
    out.pop();
    out
}
