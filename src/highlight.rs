//! A small, dependency-light Rust syntax highlighter.
//!
//! This is intentionally not a full tokenizer (no `syn`/`syntect`): a single
//! regex pass with named alternatives covers comments, strings, chars,
//! attributes, macro calls, numbers, keywords, and `UpperCamelCase`
//! type/trait names -- which is what actually matters for skimming a
//! fetched source file in a terminal, at a fraction of the dependency
//! weight and compile time.

use crate::ui::{palette, Theme};
use once_cell::sync::Lazy;
use regex::Regex;

static TOKEN_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?x)
        (?P<comment>//[^\n]*)
        |(?P<string>"(?:[^"\\]|\\.)*")
        |(?P<char>'(?:[^'\\]|\\.)*')
        |(?P<attr>\#!?\[[^\]]*\])
        |(?P<macro_call>\b[A-Za-z_][A-Za-z0-9_]*!)
        |(?P<number>\b[0-9][0-9_]*(?:\.[0-9_]+)?(?:[eE][+-]?[0-9]+)?[A-Za-z0-9_]*\b)
        |(?P<keyword>\b(?:as|async|await|break|const|continue|crate|dyn|else|enum|extern|false|fn|for|if|impl|in|let|loop|match|mod|move|mut|pub|ref|return|self|Self|static|struct|super|trait|true|type|unsafe|use|where|while|union|yield)\b)
        |(?P<type_name>\b[A-Z][A-Za-z0-9_]*\b)
        "#,
    )
    .expect("static highlighter regex must compile")
});

/// Highlight a single line of Rust source. Safe to call on non-Rust text
/// too (it will simply find fewer/no matches).
pub fn highlight_line(theme: &Theme, line: &str) -> String {
    if !theme.color {
        return line.to_string();
    }

    let mut out = String::with_capacity(line.len() + 16);
    let mut last_end = 0;

    for caps in TOKEN_RE.captures_iter(line) {
        let m = caps.get(0).unwrap();
        out.push_str(&line[last_end..m.start()]);

        let colored = if caps.name("comment").is_some() {
            theme.c(m.as_str(), palette::COMMENT)
        } else if caps.name("string").is_some() || caps.name("char").is_some() {
            theme.c(m.as_str(), palette::STRING)
        } else if caps.name("attr").is_some() {
            theme.c(m.as_str(), palette::ATTRIBUTE)
        } else if caps.name("macro_call").is_some() {
            theme.c(m.as_str(), palette::MACRO)
        } else if caps.name("number").is_some() {
            theme.c(m.as_str(), palette::NUMBER)
        } else if caps.name("keyword").is_some() {
            theme.cb(m.as_str(), palette::KEYWORD)
        } else if caps.name("type_name").is_some() {
            theme.c(m.as_str(), palette::TYPE_NAME)
        } else {
            m.as_str().to_string()
        };

        out.push_str(&colored);
        last_end = m.end();
    }
    out.push_str(&line[last_end..]);
    out
}

static BACKTICK_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"`([^`\n]+)`").expect("static backtick regex must compile")
});

/// Highlight inline code spans in prose text -- rustdoc's docblocks come
/// through `html2text` as markdown-ish text with `` `code` `` spans for
/// inline code and fenced examples. This finds those spans and runs their
/// contents through [`highlight_line`], turning a flat wall of text into
/// something that reads like a real syntax-highlighted doc viewer
/// (Python's `rich.syntax.Syntax`, applied inline), while leaving
/// surrounding prose untouched.
pub fn highlight_prose_line(theme: &Theme, line: &str) -> String {
    if !theme.color {
        return line.to_string();
    }

    let mut out = String::with_capacity(line.len() + 16);
    let mut last_end = 0;

    for caps in BACKTICK_RE.captures_iter(line) {
        let m = caps.get(0).unwrap();
        let inner = caps.get(1).map(|g| g.as_str()).unwrap_or_default();
        out.push_str(&line[last_end..m.start()]);
        out.push_str(&highlight_line(theme, inner));
        last_end = m.end();
    }
    out.push_str(&line[last_end..]);
    out
}
