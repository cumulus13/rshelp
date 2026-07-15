//! The application banner shown once at startup, the Rust equivalent of
//! `EnhancedPyHelp.print_header()`.

use super::{palette, panel, Theme};

pub fn print(theme: &Theme, version: &str) {
    if theme.quiet {
        return;
    }
    let title = format!(
        "{}rshelp{} v{version} {}",
        theme.e("🦀"),
        theme.e("🚀"),
        theme.e("📚")
    );
    let subtitle = "Beautiful terminal help for the Rust ecosystem";
    let body = format!("{title}\n{subtitle}");
    println!("{}", panel::panel(theme, "", &body, palette::PRIMARY));
}
