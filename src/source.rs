//! Fetches and displays syntax-highlighted source code for `-s/--source`,
//! following the `source` link rustdoc embeds on every item page.

use crate::error::{RsHelpError, Result};
use crate::highlight::highlight_line;
use crate::http::{FetchOutcome, HttpCtx};
use crate::ui::{palette, Theme};
use scraper::{Html, Selector};
use url::Url;

pub struct SourceView {
    pub file_hint: String,
    pub lines: Vec<String>,
    /// 1-indexed, inclusive line range rustdoc pointed us at (from the
    /// page's `#123-456` / `#123` URL fragment), if any.
    pub highlight_range: Option<(usize, usize)>,
    pub from_cache: bool,
}

/// Fetch and parse a rustdoc source-view page (the target of an item
/// page's "source" link) into raw source lines plus the highlighted range,
/// if the link pointed at one.
pub fn fetch_source(source_url: &str, http: &HttpCtx) -> Result<SourceView> {
    let outcome = http.get(source_url)?;
    let body = match outcome {
        FetchOutcome::Found { body, from_cache } => (body, from_cache),
        FetchOutcome::NotFound => {
            return Err(RsHelpError::NotFound(format!(
                "source view at {source_url}"
            )))
        }
    };
    let (html, from_cache) = body;

    let doc = Html::parse_document(&html);
    let selectors = ["pre.rust code", "main pre code", ".rust code", "pre code", "pre.rust"];

    let mut raw_text = String::new();
    for sel_str in selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            if let Some(el) = doc.select(&sel).next() {
                raw_text = el.text().collect::<Vec<_>>().join("");
                if !raw_text.trim().is_empty() {
                    break;
                }
            }
        }
    }

    if raw_text.trim().is_empty() {
        return Err(RsHelpError::Parse(format!(
            "could not locate source code block on {source_url}"
        )));
    }

    let lines: Vec<String> = raw_text.lines().map(str::to_string).collect();

    let highlight_range = Url::parse(source_url)
        .ok()
        .and_then(|u| u.fragment().map(str::to_string))
        .and_then(|frag| parse_fragment_range(&frag));

    let file_hint = Url::parse(source_url)
        .ok()
        .map(|u| u.path().to_string())
        .unwrap_or_else(|| source_url.to_string());

    Ok(SourceView {
        file_hint,
        lines,
        highlight_range,
        from_cache,
    })
}

fn parse_fragment_range(frag: &str) -> Option<(usize, usize)> {
    if let Some((a, b)) = frag.split_once('-') {
        let start: usize = a.parse().ok()?;
        let end: usize = b.parse().ok()?;
        Some((start.min(end), start.max(end)))
    } else {
        let n: usize = frag.parse().ok()?;
        Some((n, n))
    }
}

/// Render a [`SourceView`] into a printable, line-numbered, syntax
/// highlighted string. Long files are windowed around the highlighted
/// range (or truncated from the top) unless `show_all` is set.
pub fn render(theme: &Theme, view: &SourceView, show_all: bool) -> String {
    let total = view.lines.len();
    let gutter_width = total.to_string().len().max(3);

    let (from, to): (usize, usize) = if show_all {
        (1, total)
    } else if let Some((start, end)) = view.highlight_range {
        let window = 12usize;
        (start.saturating_sub(window).max(1), (end + window).min(total))
    } else {
        (1, total.min(200))
    };

    let mut out = String::new();
    for (idx, line) in view.lines.iter().enumerate() {
        let lineno = idx + 1;
        if lineno < from || lineno > to {
            continue;
        }

        let in_range = view
            .highlight_range
            .map(|(s, e)| lineno >= s && lineno <= e)
            .unwrap_or(false);

        let gutter = format!("{lineno:>gutter_width$} │ ");

        if in_range {
            let full = format!("{gutter}{line}");
            out.push_str(&theme.cb(&full, palette::WARNING));
        } else {
            out.push_str(&theme.c(&gutter, palette::DIM));
            out.push_str(&highlight_line(theme, line));
        }
        out.push('\n');
    }

    if !show_all && to < total {
        out.push('\n');
        out.push_str(&theme.c(
            &format!("… {} more line(s) not shown, use -a/--show-all to see the full file …", total - to),
            palette::DIM,
        ));
        out.push('\n');
    }

    out
}
