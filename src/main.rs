//! `rshelp` -- Enhanced Rust documentation helper with beautiful terminal
//! output. Entry point: parses arguments, loads optional config, builds the
//! shared theme/HTTP context, and dispatches to a one-shot lookup or the
//! interactive REPL.

mod app;
mod cache;
mod cli;
mod config;
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

    let (mut file_config, config_warning) = config::load(cli.config.as_deref());
    if let Some(preset) = &cli.preset {
        file_config.theme.preset = Some(preset.clone());
    }

    let theme = Theme::new(
        cli.no_emoji || file_config.defaults.no_emoji.unwrap_or(false),
        cli.plain || file_config.defaults.plain.unwrap_or(false),
        cli.quiet || file_config.defaults.quiet.unwrap_or(false),
        &file_config.theme,
    );

    if let Some(warning) = &config_warning {
        let msg = format!("{}{warning}", theme.e("⚠️"));
        println!("{}", panel::status(&theme, &msg, palette::WARNING));
    }

    if cli.init_config {
        let path = cli.config.clone().unwrap_or_else(config::default_config_path);
        match config::write_default(&path) {
            Ok(()) => {
                let msg = format!("{}Wrote default config to {}", theme.e("📝"), path.display());
                println!("{}", panel::status(&theme, &msg, palette::SUCCESS));
            }
            Err(e) => {
                let msg = format!("{}Failed to write config: {e}", theme.e("❌"));
                println!("{}", panel::status(&theme, &msg, palette::ERROR));
                std::process::exit(1);
            }
        }
        std::process::exit(0);
    }

    if cli.clear_cache {
        let cache_ttl = cli.cache_ttl.or(file_config.defaults.cache_ttl).unwrap_or(86_400);
        let cache = cache::Cache::new(cache_ttl);
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

    let cache_ttl = cli.cache_ttl.or(file_config.defaults.cache_ttl).unwrap_or(86_400);
    let timeout = cli.timeout.or(file_config.defaults.timeout).unwrap_or(15);

    let http = match http::HttpCtx::new(timeout, cache_ttl, cli.offline, cli.no_cache) {
        Ok(h) => h,
        Err(e) => {
            let msg = format!("{}Could not initialize HTTP client: {e}", theme.e("❌"));
            println!("{}", panel::status(&theme, &msg, palette::ERROR));
            std::process::exit(1);
        }
    };

    let path = cli.path.join(" ");
    let crate_version = cli.crate_version.as_deref().or(file_config.defaults.crate_version.as_deref());

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
