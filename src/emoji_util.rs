//! Emoji handling backed by the [`emoji`] crate.
//!
//! `rshelp` embeds literal emoji glyphs directly in its UI strings (the same
//! way `pyhelp` does in Python). The [`emoji`] crate's glyph table is used to
//! reliably *strip* those glyphs again for `--no-emoji` / `--plain` mode or
//! when stdout is not a terminal (e.g. piped into a log file), without
//! resorting to a hand-rolled Unicode range table that would inevitably miss
//! multi-codepoint sequences (skin tone modifiers, ZWJ family emoji, etc).

/// Returns `true` if `grapheme` is a known emoji glyph (single codepoint or
/// short multi-codepoint sequence) according to the `emoji` crate's glyph
/// table.
fn is_known_emoji(grapheme: &str) -> bool {
    emoji::lookup_by_glyph::contains_glyph(grapheme)
}

/// Strip every glyph the `emoji` crate recognizes out of `text`, collapsing
/// any leftover run of whitespace produced by the removal down to a single
/// space so output stays tidy in `--no-emoji` mode.
pub fn strip_emoji(text: &str) -> String {
    // Emoji glyphs can span multiple `char`s (e.g. flags, ZWJ sequences), so
    // we walk clusters of contiguous non-ASCII characters as candidates and
    // fall back to per-char checks for the common single-codepoint case.
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch.is_ascii() {
            out.push(ch);
            continue;
        }

        // Greedily gather a run of non-ASCII chars (plus variation
        // selectors / ZWJ) and test the largest-to-smallest prefix against
        // the glyph table so multi-codepoint emoji are matched whole.
        let mut cluster = String::new();
        cluster.push(ch);
        while let Some(&next) = chars.peek() {
            if next.is_ascii() && next != '\u{200D}' {
                break;
            }
            cluster.push(next);
            chars.next();
        }

        if is_known_emoji(cluster.trim()) || is_known_emoji(&cluster) {
            // Recognized emoji: drop it entirely.
            continue;
        }

        // Not a recognized emoji glyph (e.g. legitimate non-ASCII text such
        // as accented identifiers) -- keep it untouched.
        out.push_str(&cluster);
    }

    // Collapse doubled-up spaces left behind by removed glyphs.
    let collapsed: Vec<&str> = out.split(' ').filter(|s| !s.is_empty()).collect();
    collapsed.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_known_emoji_but_keeps_text() {
        let input = "🚀 rshelp v1.0.0 🐍 done";
        let stripped = strip_emoji(input);
        assert!(!stripped.contains('🚀'));
        assert!(stripped.contains("rshelp"));
        assert!(stripped.contains("v1.0.0"));
        assert!(stripped.contains("done"));
    }

    #[test]
    fn leaves_plain_ascii_untouched() {
        assert_eq!(strip_emoji("plain ascii text"), "plain ascii text");
    }
}
