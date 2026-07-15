//! Small terminal UI toolkit built on top of [`make_colors`] hex colors.
//!
//! There is no dependency on a full TUI framework -- `rshelp` only ever
//! prints framed panels and a spinner, so a couple hundred lines of
//! hand-rolled box-drawing is enough, and keeps the binary small and the
//! behavior easy to reason about across every terminal emulator.

pub mod header;
pub mod panel;
pub mod spinner;

use std::io::IsTerminal;

/// 24-bit hex color palette, shared across every UI element so the whole
/// tool reads as one coherent theme.
pub mod palette {
    pub const PRIMARY: &str = "#00FFFF"; // cyan   - identity / titles
    pub const ACCENT: &str = "#FF55FF"; // magenta - secondary emphasis
    pub const SUCCESS: &str = "#55FF55"; // green
    pub const WARNING: &str = "#FFAA00"; // orange
    pub const ERROR: &str = "#FF5555"; // red
    pub const INFO: &str = "#00AAFF"; // blue
    pub const DIM: &str = "#888888"; // gray
    pub const KEYWORD: &str = "#569CD6"; // rust keywords
    pub const TYPE_NAME: &str = "#4EC9B0"; // struct/enum/trait names
    pub const STRING: &str = "#CE9178"; // string literals
    pub const COMMENT: &str = "#6A9955"; // comments
    pub const MACRO: &str = "#DCDCAA"; // macro!/fn names
    pub const ATTRIBUTE: &str = "#C586C0"; // #[attr]
    pub const NUMBER: &str = "#B5CEA8"; // numeric literals
}

/// Word-wrap `text` to `width` columns, preserving explicit blank lines as
/// empty output lines. Always wraps *plain* text -- callers that need
/// colored output should wrap first, then highlight each returned line, so
/// wrapping decisions are never made on top of invisible ANSI bytes.
pub fn wrap_paragraphs(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.trim().is_empty() {
            lines.push(String::new());
        } else {
            for wrapped in textwrap::wrap(paragraph, width.max(10)) {
                lines.push(wrapped.into_owned());
            }
        }
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Resolved rendering preferences for the current run: whether to emit ANSI
/// color, whether to keep emoji, how wide the terminal is, and whether
/// decorative panels should be suppressed (`--quiet`).
#[derive(Debug, Clone)]
pub struct Theme {
    pub color: bool,
    pub emoji: bool,
    pub quiet: bool,
    pub width: usize,
    /// Maps a *default* `palette::*` hex constant to a user-configured
    /// replacement. Keyed by the default value (not a field name) so every
    /// existing call site -- `theme.c(text, palette::PRIMARY)` and so on --
    /// keeps working unchanged; the substitution happens transparently
    /// inside [`Theme::c`]/[`Theme::cb`].
    overrides: std::collections::HashMap<&'static str, String>,
}

impl Theme {
    /// Build a theme from CLI flags and a loaded [`crate::config::ThemeConfig`],
    /// auto-detecting non-TTY output (piped into a file or another program)
    /// and disabling color/emoji in that case even if the flags weren't
    /// explicitly passed.
    pub fn new(no_emoji: bool, plain: bool, quiet: bool, theme_config: &crate::config::ThemeConfig) -> Self {
        let is_tty = std::io::stdout().is_terminal();
        let no_color_env = std::env::var_os("NO_COLOR").is_some();

        let width = terminal_size::terminal_size()
            .map(|(w, _)| w.0 as usize)
            .unwrap_or(100)
            .clamp(48, 120);

        Theme {
            color: !plain && !no_color_env && is_tty,
            emoji: !plain && !no_emoji,
            quiet,
            width,
            overrides: build_overrides(theme_config),
        }
    }

    /// Colorize `text` with a `#RRGGBB` hex foreground, respecting the
    /// current color preference and any configured override for `hex`.
    /// Falls back to plain text if color is disabled or if the effective
    /// hex fails to parse for any reason.
    pub fn c(&self, text: &str, hex: &str) -> String {
        if !self.color {
            return text.to_string();
        }
        let effective = self.effective_hex(hex);
        make_colors::make_colors_hex(text, effective, None).unwrap_or_else(|_| text.to_string())
    }

    /// Same as [`Theme::c`] but bold.
    pub fn cb(&self, text: &str, hex: &str) -> String {
        if !self.color {
            return text.to_string();
        }
        let effective = self.effective_hex(hex);
        make_colors::make_colors_hex_with_attrs(text, effective, None, &["bold"])
            .unwrap_or_else(|_| text.to_string())
    }

    fn effective_hex<'a>(&'a self, default_hex: &'a str) -> &'a str {
        self.overrides
            .get(default_hex)
            .map(String::as_str)
            .unwrap_or(default_hex)
    }

    /// Prefix an emoji glyph (with a trailing space) if emoji are enabled,
    /// otherwise return an empty string.
    pub fn e(&self, glyph: &str) -> String {
        if self.emoji {
            format!("{glyph} ")
        } else {
            String::new()
        }
    }

    /// Strip emoji from arbitrary text (e.g. fetched documentation) when
    /// emoji are disabled, so `--no-emoji`/`--plain` output stays clean.
    pub fn decorate(&self, text: &str) -> String {
        if self.emoji {
            text.to_string()
        } else {
            crate::emoji_util::strip_emoji(text)
        }
    }
}

/// Build the default-hex -> override-hex map from a preset (if any) plus
/// individual field overrides layered on top. Invalid hex values are
/// silently ignored (falls back to the default) rather than producing
/// broken escape codes.
fn build_overrides(cfg: &crate::config::ThemeConfig) -> std::collections::HashMap<&'static str, String> {
    use crate::config::{is_valid_hex, preset_overrides};
    use palette::*;

    let mut map = std::collections::HashMap::new();

    if let Some(preset_name) = &cfg.preset {
        if let Some(pairs) = preset_overrides(preset_name) {
            for (default_hex, override_hex) in pairs {
                map.insert(default_hex, override_hex.to_string());
            }
        } else {
            eprintln!("rshelp: unknown theme preset '{preset_name}', ignoring");
        }
    }

    let mut set = |default_hex: &'static str, value: &Option<String>| {
        if let Some(v) = value {
            if is_valid_hex(v) {
                map.insert(default_hex, v.clone());
            } else {
                eprintln!("rshelp: invalid color '{v}' in config, expected '#RRGGBB', ignoring");
            }
        }
    };

    set(PRIMARY, &cfg.primary);
    set(ACCENT, &cfg.accent);
    set(SUCCESS, &cfg.success);
    set(WARNING, &cfg.warning);
    set(ERROR, &cfg.error);
    set(INFO, &cfg.info);
    set(DIM, &cfg.dim);
    set(KEYWORD, &cfg.keyword);
    set(TYPE_NAME, &cfg.type_name);
    set(STRING, &cfg.string);
    set(COMMENT, &cfg.comment);
    set(MACRO, &cfg.macro_color);
    set(ATTRIBUTE, &cfg.attribute);
    set(NUMBER, &cfg.number);

    map
}

/// The actual terminal column width of `s`: ANSI color escapes are
/// invisible (zero columns) and must never be counted, and wide characters
/// (most emoji, CJK) occupy two columns, not one -- `.chars().count()` gets
/// both of these wrong, which is what causes panel borders to drift out of
/// alignment whenever a line contains color codes or emoji.
pub fn visible_width(s: &str) -> usize {
    use unicode_width::UnicodeWidthStr;
    strip_ansi(s).width()
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1B}' && chars.peek() == Some(&'[') {
            chars.next(); // consume '['
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}
