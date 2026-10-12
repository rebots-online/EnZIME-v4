//! Read-only text/link projection, not an HTML security sanitizer.
pub struct ArticleProjection {
    pub normalized_text: String,
    pub internal_links: Vec<ports::zim::InternalLink>,
}
pub fn project(raw_html: &bytes::Bytes) -> ArticleProjection {
    crate::convert::projection(anzimmerman::sanitize::project(raw_html))
}
