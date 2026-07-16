//! Turns a raw rustdoc HTML page into structured, terminal-friendly content.
//!
//! Rustdoc's generated markup has changed shape multiple times over the
//! years (and docs.rs serves whatever rustdoc version a crate happened to
//! be built with), so every extraction step here tries several selectors
//! and degrades gracefully: if fine-grained parsing turns up nothing, the
//! whole page is converted to readable text instead of failing outright.

use crate::error::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use url::Url;

#[derive(Debug, Clone)]
pub struct RenderedDoc {
    pub title: String,
    pub signature: Option<String>,
    pub description: Vec<DocBlock>,
    /// Method / trait-impl signatures, used by `-a/--show-all` and the
    /// truncated default view.
    pub items: Vec<String>,
    pub source_link: Option<String>,
    /// Set when fine-grained selectors found nothing and we fell back to a
    /// whole-page text dump, so the caller can label the panel accordingly.
    pub fallback: bool,
}

/// A segment of a documentation body: either ordinary prose (which may
/// still contain short inline `` `code` `` spans) or a whole fenced code
/// example, rendered and highlighted differently by the caller.
#[derive(Debug, Clone)]
pub enum DocBlock {
    Text(String),
    Code(String),
}

fn select_first_text(doc: &Html, selectors: &[&str]) -> Option<String> {
    for sel in selectors {
        if let Ok(parsed) = Selector::parse(sel) {
            if let Some(el) = doc.select(&parsed).next() {
                let text = collapse_whitespace(&el.text().collect::<String>());
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
    }
    None
}

fn select_first_element<'a>(doc: &'a Html, selectors: &[&str]) -> Option<ElementRef<'a>> {
    for sel in selectors {
        if let Ok(parsed) = Selector::parse(sel) {
            if let Some(el) = doc.select(&parsed).next() {
                return Some(el);
            }
        }
    }
    None
}

fn collapse_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Convert an element's inner HTML to wrapped, readable plain text
/// (preserving paragraph/code-block structure reasonably well) via
/// `html2text`.
fn element_to_text(el: &ElementRef, width: usize) -> String {
    let html = el.inner_html();
    let text = html2text::from_read(html.as_bytes(), width.max(40)).unwrap_or_default();
    let cleaned = clean_markdown(&text);
    cleaned
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn find_source_link(doc: &Html, page_url: &str) -> Option<String> {
    let sel = Selector::parse("a[href]").ok()?;
    let base = Url::parse(page_url).ok();

    for a in doc.select(&sel) {
        let href = a.value().attr("href").unwrap_or("");
        if !href.contains("/src/") {
            continue;
        }
        let text = collapse_whitespace(&a.text().collect::<String>()).to_lowercase();
        let title = a
            .value()
            .attr("title")
            .map(str::to_lowercase)
            .unwrap_or_default();
        let class = a.value().attr("class").unwrap_or("");

        let looks_like_source_link =
            text == "source" || title.contains("source") || class.contains("srclink") || class.contains("src");

        if looks_like_source_link {
            return match &base {
                Some(b) => b.join(href).ok().map(|u| u.to_string()),
                None => Some(href.to_string()),
            };
        }
    }
    None
}

fn collect_items(doc: &Html, cap: Option<usize>) -> Vec<String> {
    let selectors = [
        "#implementations ~ details summary .code-header",
        "#trait-implementations ~ details summary .code-header",
        ".impl-items .method .code-header",
        "h3.code-header",
        "h4.code-header",
        ".method .code-header",
        "section.method .code-header",
    ];

    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();

    'sel: for sel in selectors {
        let Ok(parsed) = Selector::parse(sel) else { continue };
        for el in doc.select(&parsed) {
            let text = collapse_whitespace(&el.text().collect::<String>());
            if text.is_empty() || !seen.insert(text.clone()) {
                continue;
            }
            out.push(text);
            if let Some(limit) = cap {
                if out.len() >= limit {
                    break 'sel;
                }
            }
        }
    }
    out
}

/// `html2text` renders rustdoc's `<a>` links as reference-style markdown:
/// `[label][3]` inline, plus a `[3]: https://...` definition dumped at the
/// bottom of the whole block. That's exactly right for a document you'll
/// keep re-reading, but in a terminal panel it's just noise -- so this
/// strips the definition list and collapses `[label][N]` down to `label`
/// (or `§` for rustdoc's anchor markers, which have an empty label).
fn clean_markdown(text: &str) -> String {
    static FOOTNOTE_DEF: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^\s*\[\d+\]:\s").expect("static footnote-def regex must compile")
    });
    static REF_LINK: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"\[([^\]\[]*)\]\[\d+\]").expect("static ref-link regex must compile")
    });
    static BLANK_RUN: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\n{3,}").expect("static blank-run regex must compile"));

    let kept: Vec<&str> = text
        .lines()
        .filter(|line| !FOOTNOTE_DEF.is_match(line))
        .collect();
    let joined = kept.join("\n");

    let simplified = REF_LINK.replace_all(&joined, |caps: &regex::Captures| {
        let label = caps.get(1).map(|g| g.as_str()).unwrap_or_default();
        if label.trim().is_empty() {
            "§".to_string()
        } else {
            label.to_string()
        }
    });

    BLANK_RUN.replace_all(&simplified, "\n\n").trim().to_string()
}

/// `html2text` renders a multi-line fenced code example (from rustdoc's
/// `<pre><code>...</code></pre>`) by prefixing a single backtick directly
/// onto the first line and appending a single backtick onto the last line,
/// with plain, unmarked code lines in between -- it does *not* wrap every
/// line individually. A per-line "does this line contain a `` `..` ``
/// pair" check (as used for short inline code) therefore never matches a
/// whole example, and it renders as a wall of unhighlighted text with two
/// stray backticks. This splits the cleaned text into alternating prose /
/// code segments so each can be highlighted appropriately.
fn split_code_blocks(text: &str) -> Vec<DocBlock> {
    let mut blocks = Vec::new();
    let mut in_code = false;
    let mut code_buf: Vec<String> = Vec::new();
    let mut text_buf: Vec<String> = Vec::new();

    let flush_text = |buf: &mut Vec<String>, blocks: &mut Vec<DocBlock>| {
        if !buf.is_empty() {
            blocks.push(DocBlock::Text(buf.join("\n")));
            buf.clear();
        }
    };
    let flush_code = |buf: &mut Vec<String>, blocks: &mut Vec<DocBlock>| {
        if !buf.is_empty() {
            blocks.push(DocBlock::Code(buf.join("\n")));
            buf.clear();
        }
    };

    for line in text.lines() {
        let backtick_count = line.matches('`').count();

        if !in_code {
            let opens_fence = backtick_count % 2 == 1 && line.trim_start().starts_with('`');
            if opens_fence {
                flush_text(&mut text_buf, &mut blocks);
                in_code = true;
                let rest = line.trim_start();
                code_buf.push(rest[1..].to_string());
            } else {
                text_buf.push(line.to_string());
            }
        } else {
            let closes_fence = backtick_count % 2 == 1 && line.trim_end().ends_with('`');
            if closes_fence {
                let rest = line.trim_end();
                code_buf.push(rest[..rest.len() - 1].to_string());
                flush_code(&mut code_buf, &mut blocks);
                in_code = false;
            } else {
                code_buf.push(line.to_string());
            }
        }
    }

    flush_code(&mut code_buf, &mut blocks);
    flush_text(&mut text_buf, &mut blocks);
    blocks
}

const SIGNATURE_SELECTORS: &[&str] = &[
    "pre.item-decl",
    ".item-decl pre",
    "pre.rust.item-decl",
    "pre.rust.fn",
    "pre.rust.struct",
    "pre.rust.trait",
    "pre.rust.enum",
    "pre.rust.macro",
    "pre.rust.union",
    "pre.rust.type",
    "pre.fn",
    "pre.struct",
    "pre.trait",
    "pre.enum",
    "pre.macro",
    "pre.union",
    "pre.type",
];

/// `true` if `el` is nested inside a `.docblock`/`.top-doc` element -- i.e.
/// it's a code example embedded in prose, not the page's actual item
/// declaration. Crate/module index pages have no item declaration at all,
/// so without this check the "Quick Start" example in the crate-level docs
/// gets mistaken for one.
fn is_inside_docblock(el: &ElementRef) -> bool {
    el.ancestors().any(|node| {
        node.value()
            .as_element()
            .and_then(|e| e.attr("class"))
            .map(|class| class.contains("docblock") || class.contains("top-doc"))
            .unwrap_or(false)
    })
}

/// Find the page's real item-declaration signature, if any, rejecting any
/// candidate that's actually just an example snippet inside the docblock.
fn find_signature_text(doc: &Html) -> Option<String> {
    for sel_str in SIGNATURE_SELECTORS {
        let Ok(sel) = Selector::parse(sel_str) else { continue };
        for el in doc.select(&sel) {
            if is_inside_docblock(&el) {
                continue;
            }
            let text = collapse_whitespace(&el.text().collect::<String>());
            if !text.is_empty() {
                return Some(text);
            }
        }
    }
    None
}

/// Parse a fetched rustdoc page. `page_url` is used to resolve relative
/// source links to absolute URLs.
pub fn parse(html: &str, page_url: &str) -> Result<RenderedDoc> {
    let doc = Html::parse_document(html);

    let title = select_first_text(&doc, &["h1.main-heading", "h1", "title"]).unwrap_or_else(|| "(untitled)".into());

    let signature = find_signature_text(&doc);

    let description_el = select_first_element(&doc, &["details.top-doc .docblock", ".docblock", "#main-content .docblock"]);
    let description_text = description_el
        .as_ref()
        .map(|el| element_to_text(el, 100))
        .unwrap_or_default();

    let source_link = find_source_link(&doc, page_url);
    let items = collect_items(&doc, None);

    if signature.is_none() && description_text.is_empty() && items.is_empty() {
        // Nothing recognizable in the markup: fall back to a whole-page
        // text dump so the user still sees *something* useful.
        let body_sel = Selector::parse("#main-content, main, body").ok();
        let text = body_sel
            .and_then(|sel| doc.select(&sel).next())
            .map(|el| element_to_text(&el, 100))
            .unwrap_or_else(|| collapse_whitespace(&doc.root_element().text().collect::<String>()));

        return Ok(RenderedDoc {
            title,
            signature: None,
            description: vec![DocBlock::Text(text)],
            items: Vec::new(),
            source_link,
            fallback: true,
        });
    }

    Ok(RenderedDoc {
        title,
        signature,
        description: split_code_blocks(&description_text),
        items,
        source_link,
        fallback: false,
    })
}
