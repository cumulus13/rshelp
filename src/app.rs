//! Orchestrates a single lookup: resolve -> fetch -> render -> print.
//! Shared by both the one-shot CLI path and the interactive REPL.

use crate::docs;
use crate::error::RsHelpError;
use crate::http::HttpCtx;
use crate::source;
use crate::ui::{palette, panel, spinner::Spinner, table, Theme};

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
        println!(
            "{}",
            panel::panel(theme, &format!("{}Signature", theme.e("📄")), sig, palette::PRIMARY)
        );
    }

    if !resolved.doc.description.is_empty() {
        println!(
            "{}",
            panel::panel(
                theme,
                &format!("{}Documentation", theme.e("📖")),
                &theme.decorate(&resolved.doc.description),
                palette::INFO
            )
        );
    }

    if !resolved.doc.items.is_empty() {
        print_items(theme, &resolved.doc.items, show_all);
    }

    true
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
            let rendered = source::render(theme, &view, show_all);
            println!("{}", panel::panel(theme, &title, &rendered, palette::SUCCESS));
            true
        }
        Err(err) => {
            print_error(theme, &err);
            false
        }
    }
}

fn print_items(theme: &Theme, items: &[String], show_all: bool) {
    let cap = 20usize;
    let shown: Vec<Vec<String>> = if show_all {
        items.iter().map(|i| vec![i.clone()]).collect()
    } else {
        items.iter().take(cap).map(|i| vec![i.clone()]).collect()
    };

    let title = format!("{}Methods & Trait Implementations", theme.e("🔧"));
    let body = table::table(theme, &["Signature"], &shown, palette::ACCENT);
    println!("{}", panel::panel(theme, &title, &body, palette::ACCENT));

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
