//! Interactive REPL mode (`-i/--interactive`): keep querying items until
//! the user types `q`/`quit`/`x`/`exit`. Prefixing or suffixing a query
//! with a bare `c` clears the screen first, mirroring `pyhelp`'s behavior.

use crate::app;
use crate::http::HttpCtx;
use crate::ui::{palette, Theme};
use std::io::{self, Write};

const EXIT_WORDS: &[&str] = &["q", "quit", "x", "exit"];

/// Splits a leading/trailing bare `c` (case-insensitive) off a query,
/// signalling "clear the screen, then run the rest as the query".
fn strip_clear_prefix(input: &str) -> (bool, String) {
    let trimmed = input.trim();
    let mut clear = false;
    let mut rest = trimmed.to_string();

    let words: Vec<&str> = trimmed.split_whitespace().collect();
    if words.len() > 1 {
        if words[0].eq_ignore_ascii_case("c") {
            clear = true;
            rest = words[1..].join(" ");
        } else if words[words.len() - 1].eq_ignore_ascii_case("c") {
            clear = true;
            rest = words[..words.len() - 1].join(" ");
        }
    }

    (clear, rest)
}

fn clear_screen() {
    if cfg!(target_os = "windows") {
        let _ = std::process::Command::new("cmd").args(["/C", "cls"]).status();
    } else {
        print!("\x1B[2J\x1B[1;1H");
        let _ = io::stdout().flush();
    }
}

pub fn run(theme: &Theme, http: &HttpCtx, first_path: &str, crate_version: Option<&str>, show_source: bool, show_all: bool) {
    let mut current = first_path.to_string();

    loop {
        app::run_lookup(theme, http, &current, crate_version, show_source, show_all);

        let prompt = theme.cb("q/quit/x/exit = leave · c <query> = clear + query > ", palette::PRIMARY);
        print!("\n{prompt}");
        let _ = io::stdout().flush();

        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            break;
        }
        let raw = line.trim();
        if raw.is_empty() {
            continue; // repeat the same query
        }
        if EXIT_WORDS.contains(&raw.to_ascii_lowercase().as_str()) {
            break;
        }

        let (clear, rest) = strip_clear_prefix(raw);
        if clear {
            clear_screen();
        }
        if !rest.trim().is_empty() {
            current = rest;
        }
        // `show_source`/`show_all` stay fixed for the whole interactive
        // session (matching pyhelp: once started with `-s`, every
        // subsequent query keeps showing source until the user exits).
    }
}
