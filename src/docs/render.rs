//! Turns a raw rustdoc HTML page into structured, terminal-friendly content.
//!
//! Rustdoc's generated markup has changed shape multiple times over the
//! years (and docs.rs serves whatever rustdoc version a crate happened to
//! be built with), so every extraction step here tries several selectors
//! and degrades gracefully: if fine-grained parsing turns up nothing, the
//! whole page is converted to readable text instead of failing outright.

use crate::error::Result;
use scraper::{ElementRef, Html, Selector};
use url::Url;

#[derive(Debug, Clone)]
pub struct RenderedDoc {
    pub title: String,
    pub signature: Option<String>,
    pub description: String,
    /// `(kind_or_empty, signature)` pairs for methods / trait impls, used by
    /// `-a/--show-all` and the truncated default view.
    pub items: Vec<String>,
    pub source_link: Option<String>,
    /// Set when fine-grained selectors found nothing and we fell back to a
    /// whole-page text dump, so the caller can label the panel accordingly.
    pub fallback: bool,
}

fn select_first_text(doc: &Html, selectors: &[&str]) -> Option<String> {
    for sel in selectors {
        if let Ok(parsed) = Selector::parse(sel) {
            if let Some(el) = doc.select(&parsed).next() {
                let text = collapse_whitespace(&el.text().collect::<Vec<_>>().join(" "));
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
    html2text::from_read(html.as_bytes(), width.max(40))
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
        let text = collapse_whitespace(&a.text().collect::<Vec<_>>().join(" ")).to_lowercase();
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
            let text = collapse_whitespace(&el.text().collect::<Vec<_>>().join(" "));
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

/// Parse a fetched rustdoc page. `page_url` is used to resolve relative
/// source links to absolute URLs.
pub fn parse(html: &str, page_url: &str) -> Result<RenderedDoc> {
    let doc = Html::parse_document(html);

    let title = select_first_text(&doc, &["h1.main-heading", "h1", "title"]).unwrap_or_else(|| "(untitled)".into());

    let signature = select_first_text(
        &doc,
        &[
            "pre.item-decl",
            ".item-decl pre",
            "pre.rust.item-decl",
            "pre.rust.fn",
            "pre.rust.struct",
            "pre.rust.trait",
            "pre.rust.enum",
            "pre.fn",
            "pre.struct",
            "pre.trait",
            "pre.enum",
            "pre.rust",
        ],
    );

    let description_el = select_first_element(&doc, &["details.top-doc .docblock", ".docblock", "#main-content .docblock"]);
    let description = description_el
        .as_ref()
        .map(|el| element_to_text(el, 100))
        .unwrap_or_default();

    let source_link = find_source_link(&doc, page_url);
    let items = collect_items(&doc, None);

    if signature.is_none() && description.is_empty() && items.is_empty() {
        // Nothing recognizable in the markup: fall back to a whole-page
        // text dump so the user still sees *something* useful.
        let body_sel = Selector::parse("#main-content, main, body").ok();
        let text = body_sel
            .and_then(|sel| doc.select(&sel).next())
            .map(|el| element_to_text(&el, 100))
            .unwrap_or_else(|| collapse_whitespace(&doc.root_element().text().collect::<Vec<_>>().join(" ")));

        return Ok(RenderedDoc {
            title,
            signature: None,
            description: text,
            items: Vec::new(),
            source_link,
            fallback: true,
        });
    }

    Ok(RenderedDoc {
        title,
        signature,
        description,
        items,
        source_link,
        fallback: false,
    })
}
