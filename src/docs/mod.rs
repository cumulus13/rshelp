//! Resolves a user-supplied item path (`std::vec::Vec`, `serde::Deserialize`,
//! `tokio::spawn`, ...) to a documentation page, without requiring a local
//! Rust project or compiler introspection.
//!
//! Rustdoc itself doesn't publish a stable, easy-to-consume "what kind of
//! item is this" index, so rather than depending on the fragile minified
//! `search-index*.js` rustdoc emits, `rshelp` does what a human browsing
//! docs.rs would do: try the module-index URL, then try every plausible
//! "kind.Name.html" file (struct, trait, enum, fn, macro, ...) in an order
//! biased by Rust naming conventions, and take the first one that resolves.

pub mod render;

use crate::error::{Result, RsHelpError};
use crate::http::{FetchOutcome, HttpCtx};
use render::RenderedDoc;

const STD_ROOTS: &[&str] = &["std", "core", "alloc", "proc_macro", "test"];

/// Bare single-word shortcuts, the Rust analogue of `pyhelp print`/`pyhelp
/// list` resolving directly to Python builtins.
const BUILTIN_SHORTCUTS: &[(&str, &str)] = &[
    ("vec", "std::vec::Vec"),
    ("string", "std::string::String"),
    ("str", "std::primitive::str"),
    ("hashmap", "std::collections::HashMap"),
    ("hashset", "std::collections::HashSet"),
    ("btreemap", "std::collections::BTreeMap"),
    ("btreeset", "std::collections::BTreeSet"),
    ("vecdeque", "std::collections::VecDeque"),
    ("option", "std::option::Option"),
    ("result", "std::result::Result"),
    ("box", "std::boxed::Box"),
    ("rc", "std::rc::Rc"),
    ("arc", "std::sync::Arc"),
    ("mutex", "std::sync::Mutex"),
    ("rwlock", "std::sync::RwLock"),
    ("cell", "std::cell::Cell"),
    ("refcell", "std::cell::RefCell"),
    ("path", "std::path::Path"),
    ("pathbuf", "std::path::PathBuf"),
    ("duration", "std::time::Duration"),
    ("instant", "std::time::Instant"),
];

/// Item-page filename prefixes tried in priority order once we've decided
/// the final path segment looks like a type/trait/value rather than a
/// module. Order matters: it's the "most likely first" guess.
const TYPE_LIKE_KINDS: &[&str] = &["struct", "trait", "enum", "type", "union", "derive"];
const VALUE_LIKE_KINDS: &[&str] = &["fn", "macro", "constant", "static"];

pub struct Resolved {
    pub doc: RenderedDoc,
    pub page_url: String,
    pub from_cache: bool,
}

/// Normalize user input: `.` and whitespace are accepted as `::` separators
/// so `std.vec.Vec` and `std vec Vec` both work, matching the forgiving
/// input style `pyhelp` uses for Python dotted paths.
pub fn normalize_path(raw: &str) -> String {
    let replaced = raw.replace('.', "::").replace(char::is_whitespace, "::");
    let mut segments: Vec<&str> = replaced.split("::").filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        segments.push("");
    }
    segments.join("::")
}

fn expand_shortcut(path: &str) -> String {
    if !path.contains("::") {
        let lower = path.to_ascii_lowercase();
        for (short, full) in BUILTIN_SHORTCUTS {
            if *short == lower {
                return (*full).to_string();
            }
        }
    }
    path.to_string()
}

struct Target {
    base: String,
    module_path_segments: Vec<String>,
    display_path: String,
}

fn build_target(path: &str, crate_version: Option<&str>) -> Result<Target> {
    let segments: Vec<String> = path.split("::").map(str::to_string).collect();
    let root = segments
        .first()
        .ok_or_else(|| RsHelpError::InvalidPath(path.to_string()))?
        .clone();

    if STD_ROOTS.contains(&root.as_str()) {
        return Ok(Target {
            base: "https://doc.rust-lang.org/stable".to_string(),
            module_path_segments: segments,
            display_path: path.to_string(),
        });
    }

    let version = crate_version.unwrap_or("latest");
    let module_root = root.replace('-', "_");
    let mut module_path_segments = vec![module_root];
    module_path_segments.extend(segments.iter().skip(1).cloned());

    Ok(Target {
        base: format!("https://docs.rs/{root}/{version}"),
        module_path_segments,
        display_path: path.to_string(),
    })
}

/// Build the ordered list of candidate documentation page URLs for a
/// resolved target, biased by whether the final segment looks like a
/// type/trait (`UpperCamelCase`) or a module/fn/const (lowercase-leading).
fn candidate_urls(target: &Target) -> Vec<String> {
    let mut candidates = Vec::new();
    let segs = &target.module_path_segments;

    if segs.len() <= 1 {
        // Crate root or std/core/alloc root: only a module index makes sense.
        candidates.push(format!("{}/{}/index.html", target.base, segs.join("/")));
        return candidates;
    }

    let (parent, last_raw) = segs.split_at(segs.len() - 1);
    let last = last_raw[0].trim_end_matches('!');
    let parent_path = parent.join("/");
    let module_index = format!("{}/{parent_path}/{last}/index.html", target.base);

    let starts_upper = last.chars().next().map(char::is_uppercase).unwrap_or(false);

    let item_urls = |kinds: &[&str]| -> Vec<String> {
        kinds
            .iter()
            .map(|kind| format!("{}/{parent_path}/{kind}.{last}.html", target.base))
            .collect()
    };

    if starts_upper {
        candidates.extend(item_urls(TYPE_LIKE_KINDS));
        candidates.extend(item_urls(VALUE_LIKE_KINDS));
        candidates.push(module_index);
    } else {
        candidates.push(module_index);
        candidates.extend(item_urls(VALUE_LIKE_KINDS));
        candidates.extend(item_urls(TYPE_LIKE_KINDS));
    }

    candidates
}

/// Resolve `raw_path` to a documentation page, trying each candidate URL in
/// turn and returning the first one that exists.
pub fn lookup(raw_path: &str, crate_version: Option<&str>, http: &HttpCtx) -> Result<Resolved> {
    let normalized = normalize_path(raw_path);
    let expanded = expand_shortcut(&normalized);
    let target = build_target(&expanded, crate_version)?;
    let candidates = candidate_urls(&target);

    let mut last_error: Option<RsHelpError> = None;

    for url in &candidates {
        match http.get(url) {
            Ok(FetchOutcome::Found { body, from_cache }) => {
                let doc = render::parse(&body, url)?;
                return Ok(Resolved {
                    doc,
                    page_url: url.clone(),
                    from_cache,
                });
            }
            Ok(FetchOutcome::NotFound) => continue,
            Err(e) => {
                last_error = Some(e);
                continue;
            }
        }
    }

    if let Some(e) = last_error {
        // Every candidate failed outright (network down, DNS, etc.) rather
        // than cleanly 404ing -- surface the most informative error.
        return Err(e);
    }

    Err(RsHelpError::NotFound(target.display_path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_dots_and_spaces() {
        assert_eq!(normalize_path("std.vec.Vec"), "std::vec::Vec");
        assert_eq!(normalize_path("std vec Vec"), "std::vec::Vec");
        assert_eq!(normalize_path("std::vec::Vec"), "std::vec::Vec");
    }

    #[test]
    fn expands_known_shortcuts_case_insensitively() {
        assert_eq!(expand_shortcut("vec"), "std::vec::Vec");
        assert_eq!(expand_shortcut("HashMap"), "std::collections::HashMap");
    }

    #[test]
    fn leaves_unrecognized_bare_words_untouched() {
        assert_eq!(expand_shortcut("MyCustomType"), "MyCustomType");
    }

    #[test]
    fn candidate_order_prefers_types_for_uppercase() {
        let target = build_target("serde::Deserialize", None).unwrap();
        let urls = candidate_urls(&target);
        assert!(urls[0].contains("trait.Deserialize.html") || urls[0].contains("struct.Deserialize.html"));
    }

    #[test]
    fn candidate_order_prefers_module_for_lowercase() {
        let target = build_target("tokio::spawn", None).unwrap();
        let urls = candidate_urls(&target);
        assert!(urls[0].ends_with("spawn/index.html"));
    }
}
