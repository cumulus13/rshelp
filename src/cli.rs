//! Command line interface definition for `rshelp`.
//!
//! Styling is delegated to [`clap_color_help`] (24-bit hex themed help output)
//! and version printing to [`clap_version_flag`] (24-bit hex themed version
//! banner). Both respect the standard `CLAP_COLOR_*` / `NO_COLOR` conventions.

use clap::Parser;
use clap_color_help::default_styles;

/// Enhanced Rust documentation helper with beautiful terminal output.
///
/// `rshelp` looks up documentation for items in the Rust standard library
/// (`std`, `core`, `alloc`), for any crate published on crates.io (via
/// docs.rs), and can display syntax-highlighted source code -- all without
/// leaving your terminal.
#[derive(Parser, Debug)]
#[command(
    name = "rshelp",
    styles = default_styles(),
    disable_version_flag = true,
    after_help = AFTER_HELP
)]
pub struct Cli {
    /// Item, module, or crate path to look up.
    ///
    /// Examples: `std::vec::Vec`, `std.collections.HashMap`, `serde::Deserialize`,
    /// `tokio::spawn`, `clap`. Dots and double-colons are interchangeable, and
    /// multiple words are joined, so `rshelp std vec Vec` also works.
    #[arg(value_name = "PATH")]
    pub path: Vec<String>,

    /// Show syntax-highlighted source code instead of documentation.
    #[arg(short = 's', long = "source")]
    pub source: bool,

    /// Show every associated item / method / trait impl instead of a
    /// truncated summary.
    #[arg(short = 'a', long = "show-all")]
    pub show_all: bool,

    /// Stay in an interactive REPL after showing a result. Type `q`, `quit`,
    /// `x`, or `exit` to leave. Prefix or suffix a query with `c` (e.g. `c
    /// serde::Serialize`) to clear the screen first.
    #[arg(short = 'i', long = "interactive")]
    pub interactive: bool,

    /// Pin a specific crate version instead of docs.rs `latest` (e.g. `1.0.4`).
    #[arg(long = "crate-version", value_name = "VERSION")]
    pub crate_version: Option<String>,

    /// Only use the local cache; never touch the network.
    #[arg(long = "offline")]
    pub offline: bool,

    /// Bypass the cache for this lookup (still writes a fresh copy back).
    #[arg(long = "no-cache")]
    pub no_cache: bool,

    /// Delete all locally cached documentation pages and exit.
    #[arg(long = "clear-cache")]
    pub clear_cache: bool,

    /// How long cached pages stay fresh, in seconds [default: 86400, or the
    /// config file's `[defaults] cache_ttl`].
    #[arg(long = "cache-ttl", value_name = "SECS")]
    pub cache_ttl: Option<u64>,

    /// Network request timeout, in seconds [default: 15, or the config
    /// file's `[defaults] timeout`].
    #[arg(long = "timeout", value_name = "SECS")]
    pub timeout: Option<u64>,

    /// Disable emoji in output (useful for logs / CI).
    #[arg(long = "no-emoji")]
    pub no_emoji: bool,

    /// Disable colors and emoji, and use plain ASCII panels (implies
    /// `--no-emoji`). Automatically enabled when stdout is not a terminal.
    #[arg(long = "plain")]
    pub plain: bool,

    /// Suppress the banner/status panels; print only the requested content.
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Path to a config file (default: the platform config directory, e.g.
    /// `~/.config/rshelp/config.toml` on Linux).
    #[arg(long = "config", value_name = "PATH")]
    pub config: Option<std::path::PathBuf>,

    /// Write an annotated default config file to the config path (or
    /// `--config <PATH>` if given) and exit.
    #[arg(long = "init-config")]
    pub init_config: bool,

    /// Use a built-in color preset (default, dracula, nord, monokai,
    /// gruvbox), overriding the config file's `[theme] preset`.
    #[arg(long = "preset", value_name = "NAME")]
    pub preset: Option<String>,
}

// Note: `-V` / `--version` is intentionally *not* declared as a field here.
// `disable_version_flag = true` above turns off clap's built-in plain-text
// version flag, and `main.rs` uses `clap_version_flag::parse_with_version`
// to intercept `-V`/`--version` and print a colorful banner before clap
// ever sees the rest of the arguments.

const AFTER_HELP: &str = "\
Examples:
  rshelp std::vec::Vec                 Show help for std::vec::Vec
  rshelp std.collections.HashMap       Dots work just like ::
  rshelp -s serde::Deserialize         Show source code for a trait
  rshelp -a tokio::process::Command    Show every method, unabridged
  rshelp -i clap                       Interactive mode; keep querying
  rshelp --crate-version 1.0.4 anyhow  Look up docs for a pinned version
  rshelp --clear-cache                 Wipe the local documentation cache
  rshelp --init-config                 Write an editable config file
  rshelp --preset dracula std::vec::Vec  Try a built-in color preset
";
