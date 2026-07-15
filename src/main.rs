//! `rshelp` -- Enhanced Rust documentation helper with beautiful terminal
//! output. Entry point: parses arguments, builds the shared theme/HTTP
//! context, and dispatches to a one-shot lookup or the interactive REPL.

mod app;
mod cache;
mod cli;
mod docs;
mod emoji_util;
mod error;
mod highlight;
mod http;
mod interactive;
mod source;
mod ui;

use clap::CommandFactory;
use clap_version_flag::{colorful_version, parse_with_version};
use cli::Cli;
use ui::{palette, panel, Theme};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Mirror pyhelp: running with no arguments at all prints help and
    // exits 0 (not clap's usual exit-2-as-a-usage-error), so `rshelp` on
    // its own is a friendly discovery command rather than an error.
    if std::env::args().len() <= 1 {
        let mut cmd = Cli::command();
        let _ = cmd.print_help();
        println!();
        std::process::exit(0);
    }

    let version = colorful_version!();
    let cli: Cli = parse_with_version(Cli::command(), &version)?;

    let theme = Theme::new(cli.no_emoji, cli.plain, cli.quiet);

    if cli.clear_cache {
        let cache = cache::Cache::new(cli.cache_ttl);
        match cache.clear() {
            Ok(n) => {
                let msg = format!(
                    "{}Removed {n} cached page(s) from {}",
                    theme.e("🧹"),
                    cache.dir().display()
                );
                println!("{}", panel::status(&theme, &msg, palette::SUCCESS));
            }
            Err(e) => {
                let msg = format!("{}Failed to clear cache: {e}", theme.e("❌"));
                println!("{}", panel::status(&theme, &msg, palette::ERROR));
                std::process::exit(1);
            }
        }
        if cli.path.is_empty() {
            std::process::exit(0);
        }
    }

    if cli.path.is_empty() {
        let mut cmd = Cli::command();
        let _ = cmd.print_help();
        println!();
        std::process::exit(0);
    }

    ui::header::print(&theme, env!("CARGO_PKG_VERSION"));

    let http = match http::HttpCtx::new(cli.timeout, cli.cache_ttl, cli.offline, cli.no_cache) {
        Ok(h) => h,
        Err(e) => {
            let msg = format!("{}Could not initialize HTTP client: {e}", theme.e("❌"));
            println!("{}", panel::status(&theme, &msg, palette::ERROR));
            std::process::exit(1);
        }
    };

    let path = cli.path.join(" ");
    let crate_version = cli.crate_version.as_deref();

    if cli.interactive {
        interactive::run(&theme, &http, &path, crate_version, cli.source, cli.show_all);
        return Ok(());
    }

    let ok = app::run_lookup(&theme, &http, &path, crate_version, cli.source, cli.show_all);
    if !ok {
        std::process::exit(1);
    }

    Ok(())
}
