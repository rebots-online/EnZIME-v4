//! HTML projection: text extraction and internal-link classification for
//! `ArticleDocument`. Performs NO sanitizer rewriting of `raw_html` — the
//! rendering boundary pipeline (ARCHITECTURE.md §7.3) owns that step
//! upstream of the privileged React DOM. The functions in this module are
//! read-only projections over the article's already-validated bytes.

use crate::identity::{ArticleKey, EntryKey};
use crate::model::{InternalLink, InternalLinkTarget};
use bytes::Bytes;
use ego_tree::NodeRef;
use scraper::node::Node;
use scraper::Html;

pub struct ArticleProjection {
    pub normalized_text: String,
    pub internal_links: Vec<InternalLink>,
}

/// Parses `raw_html` and produces two artifacts:
///
/// 1. `normalized_text` — concatenation of all text nodes outside of
///    `script`, `style`, `noscript` and `template` subtrees, with runs of
///    ASCII whitespace collapsed to a single space and leading/trailing
///    whitespace trimmed.
/// 2. `internal_links` — every `<a href="...">` in document order with the
///    href classified into `Article`, `Resource` or `Unsupported`.
pub fn project(raw_html: &Bytes) -> ArticleProjection {
    let html_str = std::str::from_utf8(raw_html).unwrap_or("");
    let doc = Html::parse_document(html_str);
    let normalized_text = normalize_whitespace(&extract_text(&doc));
    let internal_links = extract_links(&doc);
    ArticleProjection {
        normalized_text,
        internal_links,
    }
}

fn extract_text(doc: &Html) -> String {
    let mut out = String::new();
    walk_text(&doc.tree.root(), &mut out, false);
    out
}

fn walk_text(node: &NodeRef<Node>, out: &mut String, skip: bool) {
    if skip {
        return;
    }
    let mut child_skip = false;
    match node.value() {
        Node::Text(t) => out.push_str(&t.text),
        Node::Element(e) => {
            let name = e.name.local.as_ref();
            if matches!(name, "script" | "style" | "noscript" | "template") {
                child_skip = true;
            }
        }
        _ => {}
    }
    for child in node.children() {
        walk_text(&child, out, child_skip);
    }
}

fn extract_links(doc: &Html) -> Vec<InternalLink> {
    use scraper::Selector;
    let mut links = Vec::new();
    let Ok(sel) = Selector::parse("a[href]") else {
        return links;
    };
    for el in doc.select(&sel) {
        if let Some(href) = el.value().attr("href") {
            links.push(InternalLink {
                href: href.to_string(),
                target: classify_target(href),
            });
        }
    }
    links
}

fn classify_target(href: &str) -> InternalLinkTarget {
    // ZIM URL conventions:
    //   * "A/<url>"          — internal article
    //   * "<ns>/<url>"       — internal resource (image, css, font, ...)
    //   * "<absolute URL>"   — external, unsupported inside the sandbox
    //   * "#<fragment>"      — same-article fragment, unsupported as a
    //                          cross-article target
    let href = href.trim();
    if href.is_empty() {
        return InternalLinkTarget::Unsupported {
            reason: "empty href".into(),
        };
    }
    if href.starts_with('#') {
        return InternalLinkTarget::Unsupported {
            reason: "same-article fragment".into(),
        };
    }
    if href.contains("://") || href.starts_with("//") {
        return InternalLinkTarget::Unsupported {
            reason: "external URL".into(),
        };
    }
    if href.starts_with("A/") {
        return InternalLinkTarget::Article {
            article_key: ArticleKey(href.to_string()),
        };
    }
    // Anything else with a namespace prefix that survives is treated as an
    // internal resource reference: "<ns>/<url>".
    if href.len() >= 2 && href.as_bytes()[0].is_ascii_uppercase() && href.as_bytes()[1] == b'/' {
        return InternalLinkTarget::Resource {
            entry_key: EntryKey(href.to_string()),
        };
    }
    // Bare path without namespace — common inside articles where the
    // renderer implicitly routes to namespace A.
    InternalLinkTarget::Article {
        article_key: ArticleKey(format!("A/{}", href)),
    }
}

fn normalize_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = true; // collapses leading whitespace
    for c in s.chars() {
        if c.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    if out.ends_with(' ') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_script_style_and_collapses_whitespace() {
        let raw = Bytes::from_static(
            b"<html><body><h1>Hello</h1><script>evil()</script>\
             <style>.x{}</style>\n<p>World   of\n  ZIM</p></body></html>",
        );
        let proj = project(&raw);
        assert_eq!(proj.normalized_text, "Hello World of ZIM");
    }

    #[test]
    fn classifies_internal_and_external_links() {
        let raw = Bytes::from_static(
            b"<a href='A/Python'>py</a>\
             <a href='I/img.png'>img</a>\
             <a href='https://example.com/'>ex</a>\
             <a href='#frag'>frag</a>\
             <a href='bare'>bare</a>",
        );
        let proj = project(&raw);
        let kinds: Vec<&str> = proj
            .internal_links
            .iter()
            .map(|l| match &l.target {
                InternalLinkTarget::Article { .. } => "A",
                InternalLinkTarget::Resource { .. } => "R",
                InternalLinkTarget::Unsupported { .. } => "U",
            })
            .collect();
        // 5 links in document order: article, resource, unsupported, unsupported, article.
        assert_eq!(kinds, vec!["A", "R", "U", "U", "A"]);
    }
}
