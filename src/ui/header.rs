//! The application banner shown once at startup, the Rust equivalent of
//! `EnhancedPyHelp.print_header()`.

use super::{palette, panel, Theme};

pub fn print(theme: &Theme, version: &str) {
    if theme.quiet {
        return;
    }
    let title = format!(
        "{}rshelp {}v{version} {}",
        theme.e("🦀"),
        theme.e("🚀"),
        theme.e("📚")
    );
    let title = theme.cb(title.trim_end(), palette::PRIMARY);
    let subtitle = theme.c("Beautiful terminal help for the Rust ecosystem", palette::DIM);
    let lines = vec![title, subtitle];
    println!("{}", panel::panel_lines(theme, "", &lines, palette::PRIMARY));
}
