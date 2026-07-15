//! Orchestrates a single lookup: resolve -> fetch -> render -> print.
//! Shared by both the one-shot CLI path and the interactive REPL.

use crate::docs;
use crate::error::RsHelpError;
use crate::highlight::{highlight_line, highlight_prose_line};
use crate::http::HttpCtx;
use crate::source;
use crate::ui::{self, palette, panel, spinner::Spinner, Theme};

/// Runs one lookup for `path` and prints the result. Returns `true` if the
/// lookup succeeded (used by the REPL to color its own status line).
pub fn run_lookup(
    theme: &Theme,
    http: &HttpCtx,
    path: &str,
    crate_version: Option<&str>,
    show_source: bool,
    show_all: bool,
) -> bool {
    if !theme.quiet {
        let msg = format!("{}Target: {path}", theme.e("🎯"));
        println!("{}", panel::status(theme, &msg, palette::INFO));
    }

    let spinner = Spinner::start(theme, &format!("{}Looking up {path}...", theme.e("🔍")));
    let result = docs::lookup(path, crate_version, http);
    spinner.finish();

    let resolved = match result {
        Ok(r) => r,
        Err(err) => {
            print_error(theme, &err);
            return false;
        }
    };

    if !theme.quiet {
        let cache_note = if resolved.from_cache { " (from cache)" } else { "" };
        let msg = format!(
            "{}Found: {}{cache_note}\n{}",
            theme.e("✅"),
            resolved.doc.title,
            theme.decorate(&resolved.page_url)
        );
        println!("{}", panel::status(theme, &msg, palette::SUCCESS));
    }

    if show_source {
        return show_source_view(theme, http, &resolved, show_all);
    }

    if resolved.doc.fallback && !theme.quiet {
        let note = format!(
            "{}This crate's docs use a page layout rshelp doesn't fully understand yet; showing a raw text fallback.",
            theme.e("ℹ️")
        );
        println!("{}", panel::panel(theme, "", &note, palette::WARNING));
    }

    if let Some(sig) = &resolved.doc.signature {
        print_signature(theme, sig);
    }

    if !resolved.doc.description.is_empty() {
        print_description(theme, &resolved.doc.description);
    }

    if !resolved.doc.items.is_empty() {
        print_items(theme, &resolved.doc.items, show_all);
    }

    true
}

/// Signature panel: word-wrap the plain signature text first, *then*
/// syntax-highlight each wrapped line, so wrapping never has to reason
/// about invisible ANSI bytes.
fn print_signature(theme: &Theme, sig: &str) {
    let inner = panel::inner_width(theme);
    let lines: Vec<String> = ui::wrap_paragraphs(sig, inner)
        .into_iter()
        .map(|line| highlight_line(theme, &line))
        .collect();

    let title = format!("{}Signature", theme.e("📄"));
    println!("{}", panel::panel_lines(theme, &title, &lines, palette::PRIMARY));
}

/// Documentation panel: same wrap-then-highlight ordering, but using the
/// prose highlighter so only backtick-delimited inline code gets colored,
/// leaving surrounding sentences as normal text.
fn print_description(theme: &Theme, description: &str) {
    let inner = panel::inner_width(theme);
    let decorated = theme.decorate(description);
    let lines: Vec<String> = ui::wrap_paragraphs(&decorated, inner)
        .into_iter()
        .map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                theme.cb(trimmed.trim_start_matches('#').trim_start(), palette::ACCENT)
            } else {
                highlight_prose_line(theme, &line)
            }
        })
        .collect();

    let title = format!("{}Documentation", theme.e("📖"));
    println!("{}", panel::panel_lines(theme, &title, &lines, palette::INFO));
}

fn show_source_view(theme: &Theme, http: &HttpCtx, resolved: &docs::Resolved, show_all: bool) -> bool {
    let Some(src_url) = &resolved.doc.source_link else {
        let msg = format!("{}No source link found for this item.", theme.e("❌"));
        println!("{}", panel::status(theme, &msg, palette::ERROR));
        return false;
    };

    let spinner = Spinner::start(theme, &format!("{}Fetching source...", theme.e("📄")));
    let view = source::fetch_source(src_url, http);
    spinner.finish();

    match view {
        Ok(view) => {
            let cache_note = if view.from_cache { " (from cache)" } else { "" };
            let title = format!("{}Source Code - {}{cache_note}", theme.e("📄"), view.file_hint);
            // `source::render` already produces fully wrapped, padded,
            // syntax-highlighted lines -- hand them straight to
            // `panel_lines` rather than `panel`, which would re-wrap
            // (and mis-measure) text that already contains ANSI codes.
            let lines = source::render(theme, &view, show_all);
            println!("{}", panel::panel_lines(theme, &title, &lines, palette::SUCCESS));
            true
        }
        Err(err) => {
            print_error(theme, &err);
            false
        }
    }
}

/// Methods/trait-impls panel: each signature is wrapped and highlighted
/// independently (continuation lines get a small indent so multi-line
/// signatures read as one entry rather than a jagged table row), with a
/// blank separator line between entries.
fn print_items(theme: &Theme, items: &[String], show_all: bool) {
    let cap = 20usize;
    let shown = if show_all { items } else { &items[..items.len().min(cap)] };

    let inner = panel::inner_width(theme);
    let mut lines: Vec<String> = Vec::new();
    for (idx, item) in shown.iter().enumerate() {
        let wrapped = ui::wrap_paragraphs(item, inner.saturating_sub(2));
        for (i, w) in wrapped.iter().enumerate() {
            let indented = if i == 0 { w.clone() } else { format!("  {w}") };
            lines.push(highlight_line(theme, &indented));
        }
        if idx + 1 < shown.len() {
            lines.push(String::new());
        }
    }

    let title = format!("{}Methods & Trait Implementations", theme.e("🔧"));
    println!("{}", panel::panel_lines(theme, &title, &lines, palette::ACCENT));

    if !show_all && items.len() > cap {
        let note = format!(
            "{}Total: {} item(s), showing first {cap} -- use -a/--show-all to see them all",
            theme.e("📊"),
            items.len()
        );
        println!("{}", panel::status(theme, &note, palette::DIM));
    }
}

fn print_error(theme: &Theme, err: &RsHelpError) {
    let msg = format!("{}{err}", theme.e("❌"));
    println!("{}", panel::status(theme, &msg, palette::ERROR));
    if !theme.quiet {
        let hint = format!(
            "{}Check the path/spelling, or try --crate-version, --offline, or a longer --timeout.",
            theme.e("💡")
        );
        println!("{}", panel::status(theme, &hint, palette::DIM));
    }
}
