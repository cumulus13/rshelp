//! Optional TOML config file support (`--config`, `--init-config`).
//!
//! Nothing here is required -- `rshelp` runs perfectly well with zero
//! config, using the same built-in defaults it always has. The config file
//! only ever *overrides* individual values; anything left unset (or the
//! absence of a file entirely) falls back to the hardcoded defaults.

use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub defaults: DefaultsConfig,
}

/// 24-bit hex color overrides (`"#RRGGBB"`), plus an optional named preset
/// applied before the individual fields so a preset can be picked and then
/// fine-tuned with just one or two overrides on top.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ThemeConfig {
    pub preset: Option<String>,
    pub primary: Option<String>,
    pub accent: Option<String>,
    pub success: Option<String>,
    pub warning: Option<String>,
    pub error: Option<String>,
    pub info: Option<String>,
    pub dim: Option<String>,
    pub keyword: Option<String>,
    pub type_name: Option<String>,
    pub string: Option<String>,
    pub comment: Option<String>,
    #[serde(rename = "macro")]
    pub macro_color: Option<String>,
    pub attribute: Option<String>,
    pub number: Option<String>,
}

/// Fallback values for CLI flags when the flag isn't explicitly passed.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DefaultsConfig {
    pub no_emoji: Option<bool>,
    pub plain: Option<bool>,
    pub quiet: Option<bool>,
    pub cache_ttl: Option<u64>,
    pub timeout: Option<u64>,
    pub crate_version: Option<String>,
}

/// `$XDG_CONFIG_HOME/rshelp/config.toml` (or the platform equivalent).
pub fn default_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("rshelp")
        .join("config.toml")
}

/// Load config from `explicit_path` if given, otherwise the default
/// location. A missing file is not an error (empty/default config);
/// a malformed file is reported to the caller so it can warn the user
/// without aborting the whole lookup over a typo.
pub fn load(explicit_path: Option<&Path>) -> (Config, Option<String>) {
    let path = explicit_path
        .map(Path::to_path_buf)
        .unwrap_or_else(default_config_path);

    match std::fs::read_to_string(&path) {
        Ok(contents) => match toml::from_str::<Config>(&contents) {
            Ok(cfg) => (cfg, None),
            Err(e) => (
                Config::default(),
                Some(format!("could not parse config at {}: {e}", path.display())),
            ),
        },
        Err(_) => (Config::default(), None),
    }
}

/// Write the annotated default config template to `path`, creating parent
/// directories as needed. Used by `--init-config`.
pub fn write_default(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, DEFAULT_CONFIG_TOML)
}

pub const DEFAULT_CONFIG_TOML: &str = r##"# rshelp configuration file
#
# Every field is optional. Anything left commented out (or the whole file
# being absent) uses rshelp's built-in defaults. Override --config to point
# at a different file, or run `rshelp --init-config` again to regenerate
# this template at the default location.

[theme]
# Start from a named preset, then optionally fine-tune individual colors
# below. Built-in presets: "default", "dracula", "nord", "monokai",
# "gruvbox".
# preset = "dracula"

# Individual 24-bit hex overrides (applied after the preset, if any).
# primary    = "#00FFFF"  # identity / panel borders / titles
# accent     = "#FF55FF"  # secondary emphasis (methods panel)
# success    = "#55FF55"
# warning    = "#FFAA00"
# error      = "#FF5555"
# info       = "#00AAFF"  # documentation panel
# dim        = "#888888"
# keyword    = "#569CD6"  # fn, struct, pub, ...
# type_name  = "#4EC9B0"  # UpperCamelCase types/traits
# string     = "#CE9178"
# comment    = "#6A9955"
# macro      = "#DCDCAA"
# attribute  = "#C586C0"  # #[derive(...)]
# number     = "#B5CEA8"

[defaults]
# Fallback values used only when the matching CLI flag isn't passed.
# no_emoji      = false
# plain         = false
# quiet         = false
# cache_ttl     = 86400
# timeout       = 15
# crate_version = "1.0.0"
"##;

/// A single preset: `(default_hex_constant, override_hex)` pairs, matched
/// against `crate::ui::palette` constants the same way per-field config
/// overrides are.
pub fn preset_overrides(name: &str) -> Option<Vec<(&'static str, &'static str)>> {
    use crate::ui::palette::*;

    let pairs: Vec<(&'static str, &'static str)> = match name.to_ascii_lowercase().as_str() {
        "default" => vec![],
        "dracula" => vec![
            (PRIMARY, "#8BE9FD"),
            (ACCENT, "#FF79C6"),
            (SUCCESS, "#50FA7B"),
            (WARNING, "#FFB86C"),
            (ERROR, "#FF5555"),
            (INFO, "#BD93F9"),
            (DIM, "#6272A4"),
            (KEYWORD, "#FF79C6"),
            (TYPE_NAME, "#8BE9FD"),
            (STRING, "#F1FA8C"),
            (COMMENT, "#6272A4"),
            (MACRO, "#50FA7B"),
            (ATTRIBUTE, "#FFB86C"),
            (NUMBER, "#BD93F9"),
        ],
        "nord" => vec![
            (PRIMARY, "#88C0D0"),
            (ACCENT, "#B48EAD"),
            (SUCCESS, "#A3BE8C"),
            (WARNING, "#EBCB8B"),
            (ERROR, "#BF616A"),
            (INFO, "#81A1C1"),
            (DIM, "#4C566A"),
            (KEYWORD, "#81A1C1"),
            (TYPE_NAME, "#8FBCBB"),
            (STRING, "#A3BE8C"),
            (COMMENT, "#616E88"),
            (MACRO, "#EBCB8B"),
            (ATTRIBUTE, "#B48EAD"),
            (NUMBER, "#B48EAD"),
        ],
        "monokai" => vec![
            (PRIMARY, "#66D9EF"),
            (ACCENT, "#FD971F"),
            (SUCCESS, "#A6E22E"),
            (WARNING, "#FD971F"),
            (ERROR, "#F92672"),
            (INFO, "#66D9EF"),
            (DIM, "#75715E"),
            (KEYWORD, "#F92672"),
            (TYPE_NAME, "#66D9EF"),
            (STRING, "#E6DB74"),
            (COMMENT, "#75715E"),
            (MACRO, "#A6E22E"),
            (ATTRIBUTE, "#FD971F"),
            (NUMBER, "#AE81FF"),
        ],
        "gruvbox" => vec![
            (PRIMARY, "#83A598"),
            (ACCENT, "#D3869B"),
            (SUCCESS, "#B8BB26"),
            (WARNING, "#FABD2F"),
            (ERROR, "#FB4934"),
            (INFO, "#83A598"),
            (DIM, "#928374"),
            (KEYWORD, "#FB4934"),
            (TYPE_NAME, "#FABD2F"),
            (STRING, "#B8BB26"),
            (COMMENT, "#928374"),
            (MACRO, "#8EC07C"),
            (ATTRIBUTE, "#D3869B"),
            (NUMBER, "#D3869B"),
        ],
        _ => return None,
    };
    Some(pairs)
}

/// Loosely validate a user-supplied hex color so a typo in the config file
/// degrades to "ignored, default used" rather than a garbled escape code.
pub fn is_valid_hex(s: &str) -> bool {
    s.len() == 7
        && s.starts_with('#')
        && s[1..].chars().all(|c| c.is_ascii_hexdigit())
}
