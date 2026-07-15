//! Small terminal UI toolkit built on top of [`make_colors`] hex colors.
//!
//! There is no dependency on a full TUI framework -- `rshelp` only ever
//! prints framed panels, simple tables, and a spinner, so a couple hundred
//! lines of hand-rolled box-drawing is enough, and keeps the binary small
//! and the behavior easy to reason about across every terminal emulator.

pub mod header;
pub mod panel;
pub mod spinner;
pub mod table;

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

/// Resolved rendering preferences for the current run: whether to emit ANSI
/// color, whether to keep emoji, how wide the terminal is, and whether
/// decorative panels should be suppressed (`--quiet`).
#[derive(Debug, Clone)]
pub struct Theme {
    pub color: bool,
    pub emoji: bool,
    pub quiet: bool,
    pub width: usize,
}

impl Theme {
    /// Build a theme from CLI flags, auto-detecting non-TTY output (piped
    /// into a file or another program) and disabling color/emoji in that
    /// case even if the flags weren't explicitly passed.
    pub fn new(no_emoji: bool, plain: bool, quiet: bool) -> Self {
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
        }
    }

    /// Colorize `text` with a `#RRGGBB` hex foreground, respecting the
    /// current color preference. Falls back to plain text if color is
    /// disabled or if `hex` fails to parse for any reason.
    pub fn c(&self, text: &str, hex: &str) -> String {
        if !self.color {
            return text.to_string();
        }
        make_colors::make_colors_hex(text, hex, None).unwrap_or_else(|_| text.to_string())
    }

    /// Same as [`Theme::c`] but bold.
    pub fn cb(&self, text: &str, hex: &str) -> String {
        if !self.color {
            return text.to_string();
        }
        make_colors::make_colors_hex_with_attrs(text, hex, None, &["bold"])
            .unwrap_or_else(|_| text.to_string())
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
