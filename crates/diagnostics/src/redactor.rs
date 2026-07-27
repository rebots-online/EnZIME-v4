//! Typed redaction of values that may appear in diagnostics.
//!
//! See the crate-level docs for the threat model and the §18.6 / §18.5
//! binding. The shape of this module is dictated by those two lists:
//!
//! * §18.6 "may include" categories are exposed as pass-through constructors
//!   ([`Redactor::entity_id`], [`Redactor::error_code`],
//!   [`Redactor::byte_count`], [`Redactor::duration_millis`],
//!   [`Redactor::state_transition`], [`Redactor::version`]).
//! * §18.6 "excludes" categories are exposed only as redacting constructors
//!   ([`Redactor::redact_text`], [`Redactor::redact_path`],
//!   [`Redactor::redact_url`]). The original bytes never reach the
//!   returned value.
//! * §18.5 secrets have no constructor at all.
//!
//! There is intentionally no `Redactor::raw` or `push_string`. Redaction is
//! not an option the caller can forget to pass.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Kinds of user-authored text that §18.6 excludes from default logs. Each
/// is reduced to a short placeholder that carries only the kind, a stable
/// SHA-256 prefix of the original bytes, and the original byte length — never
/// the text itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserTextKind {
    Article,
    Annotation,
    Message,
    Lore,
}

impl UserTextKind {
    pub fn label(self) -> &'static str {
        match self {
            UserTextKind::Article => "article_text",
            UserTextKind::Annotation => "annotation_text",
            UserTextKind::Message => "message_text",
            UserTextKind::Lore => "lore_text",
        }
    }
}

/// Configuration for a [`Redactor`]. The default matches the §18.6 "may
/// include" / "excludes" lists verbatim: the excludes-list categories are
/// always redacted regardless of config, because the architecture does not
/// allow them to be relaxed.
#[derive(Clone, Debug)]
pub struct RedactionConfig {
    correlation_hex_len: usize,
}

impl Default for RedactionConfig {
    fn default() -> Self {
        Self {
            correlation_hex_len: 8,
        }
    }
}

impl RedactionConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of hex characters of the SHA-256 of the original bytes that
    /// redacted placeholders expose. Defaults to 8 — enough for correlation
    /// across entries inside one bundle, not enough to be useful as a
    /// fingerprint outside it.
    pub fn correlation_hex_len(&self) -> usize {
        self.correlation_hex_len
    }

    /// Set the number of hex characters of SHA-256 prefix to embed in
    /// redacted placeholders. Clamped to `0..=64`.
    pub fn with_correlation_hex_len(mut self, len: usize) -> Self {
        self.correlation_hex_len = len.min(64);
        self
    }
}

/// Stateful redactor. Cheap to construct; clone per task or share by
/// reference. Holds only the [`RedactionConfig`].
#[derive(Clone, Debug)]
pub struct Redactor {
    config: RedactionConfig,
}

impl Default for Redactor {
    fn default() -> Self {
        Self::new(RedactionConfig::default())
    }
}

impl Redactor {
    pub fn new(config: RedactionConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &RedactionConfig {
        &self.config
    }

    fn correlation(&self, bytes: &[u8]) -> String {
        let mut h = Sha256::new();
        h.update(bytes);
        let full = hex::encode(h.finalize());
        let len = self.config.correlation_hex_len.min(full.len());
        full[..len].to_string()
    }

    /// Record an opaque entity identifier (§18.6 "may include: opaque entity
    /// IDs"). Passed through unchanged. Use only for values the architecture
    /// treats as opaque (PackId, ArticleKey, etc.) — never for free-form
    /// text.
    pub fn entity_id(&self, raw: impl Into<String>) -> EntityId {
        EntityId(raw.into())
    }

    /// Record an error code (§18.6 "may include: error codes"). Passed
    /// through unchanged.
    pub fn error_code(&self, code: impl Into<String>) -> ErrorCode {
        ErrorCode(code.into())
    }

    /// Record a byte count (§18.6 "may include: byte counts").
    pub fn byte_count(&self, bytes: u64) -> ByteCount {
        ByteCount(bytes)
    }

    /// Record a duration in milliseconds (§18.6 "may include: durations").
    pub fn duration_millis(&self, ms: u64) -> DurationMillis {
        DurationMillis(ms)
    }

    /// Record a state transition (§18.6 "may include: state transitions").
    pub fn state_transition(
        &self,
        entity: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
    ) -> StateTransition {
        StateTransition {
            entity: entity.into(),
            from: from.into(),
            to: to.into(),
        }
    }

    /// Record an app/platform version string (§18.6 "may include:
    /// app/platform versions"). Passed through unchanged.
    pub fn version(&self, component: impl Into<String>, version: impl Into<String>) -> Version {
        Version {
            component: component.into(),
            version: version.into(),
        }
    }

    /// Redact user-authored prose (article/annotation/message/lore text). The
    /// returned [`RedactedText`] carries only the kind, a SHA-256 prefix and
    /// the original byte length — never the text itself.
    pub fn redact_text(&self, kind: UserTextKind, raw: &[u8]) -> RedactedText {
        RedactedText {
            kind,
            correlation: self.correlation(raw),
            byte_len: raw.len(),
        }
    }

    /// Redact a filesystem path (§18.6 "excludes: filesystem paths"). The
    /// returned [`RedactedPath`] carries only a SHA-256 prefix and a byte
    /// length — never any path component.
    pub fn redact_path(&self, raw: &Path) -> RedactedPath {
        let bytes = path_bytes(raw);
        RedactedPath {
            correlation: self.correlation(&bytes),
            byte_len: bytes.len(),
        }
    }

    /// Redact a URL (§18.6 "excludes: URLs with query strings"). If the URL
    /// parses, the returned [`RedactedUrl`] carries scheme, host, port and
    /// path with the query, fragment and userinfo removed. If the URL does
    /// not parse, the entire value is reduced to an opaque placeholder —
    /// never the original bytes.
    pub fn redact_url(&self, raw: &str) -> RedactedUrl {
        match url::Url::parse(raw) {
            Ok(u) => {
                let mut s = String::new();
                s.push_str(u.scheme());
                s.push_str("://");
                if let Some(host) = u.host_str() {
                    s.push_str(host);
                }
                if let Some(port) = u.port() {
                    s.push(':');
                    s.push_str(&port.to_string());
                }
                s.push_str(u.path());
                let had_query = u.query().is_some() || u.fragment().is_some();
                RedactedUrl {
                    sanitized: Some(s),
                    correlation: self.correlation(raw.as_bytes()),
                    had_query,
                }
            }
            Err(_) => RedactedUrl {
                sanitized: None,
                correlation: self.correlation(raw.as_bytes()),
                had_query: false,
            },
        }
    }
}

/// Best-effort byte view of a path. On Unix-family targets (android, linux)
/// this uses the native OS-byte view of the path; on Windows it falls back
/// to the UTF-8 lossy representation.
fn path_bytes(p: &Path) -> Vec<u8> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        p.as_os_str().as_bytes().to_vec()
    }
    #[cfg(not(unix))]
    {
        p.to_string_lossy().as_bytes().to_vec()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ErrorCode(pub String);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ByteCount(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DurationMillis(pub u64);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StateTransition {
    pub entity: String,
    pub from: String,
    pub to: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Version {
    pub component: String,
    pub version: String,
}

/// Redacted form of user-authored text. Carries the kind, a SHA-256 prefix
/// of the original bytes and the original byte length. The original text is
/// not stored.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RedactedText {
    pub kind: UserTextKind,
    pub correlation: String,
    pub byte_len: usize,
}

impl RedactedText {
    pub fn placeholder(&self) -> String {
        format!(
            "<redacted:{} sha={} len={}>",
            self.kind.label(),
            self.correlation,
            self.byte_len
        )
    }
}

/// Redacted form of a filesystem path. Carries a SHA-256 prefix and a byte
/// length. The original path is not stored.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RedactedPath {
    pub correlation: String,
    pub byte_len: usize,
}

impl RedactedPath {
    pub fn placeholder(&self) -> String {
        format!("<redacted:path sha={} len={}>", self.correlation, self.byte_len)
    }
}

/// Redacted form of a URL. Carries scheme/host/port/path with query,
/// fragment and userinfo removed. If the original did not parse, carries
/// only an opaque placeholder correlation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RedactedUrl {
    pub sanitized: Option<String>,
    pub correlation: String,
    pub had_query: bool,
}

impl RedactedUrl {
    pub fn placeholder(&self) -> String {
        match &self.sanitized {
            Some(s) if self.had_query => format!("{s} <redacted:query>"),
            Some(s) => s.clone(),
            None => format!("<redacted:url sha={}>", self.correlation),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacted_text_strips_original_bytes() {
        let r = Redactor::default();
        let original = "the candidate answer is 42";
        let red = r.redact_text(UserTextKind::Message, original.as_bytes());
        let placeholder = red.placeholder();
        assert!(!placeholder.contains(original));
        assert!(placeholder.contains("redacted:message_text"));
        assert!(placeholder.contains("len=26"));
        // Correlation is stable.
        let red2 = r.redact_text(UserTextKind::Message, original.as_bytes());
        assert_eq!(red, red2);
        // Different input → different correlation (8 hex chars).
        let red3 = r.redact_text(UserTextKind::Message, b"different content");
        assert_ne!(red.correlation, red3.correlation);
    }

    #[test]
    fn redacted_path_strips_original_bytes() {
        let r = Redactor::default();
        let p = Path::new("/home/robin/secret-diary.md");
        let red = r.redact_path(p);
        let placeholder = red.placeholder();
        assert!(!placeholder.contains("robin"));
        assert!(!placeholder.contains("secret-diary"));
        assert!(!placeholder.contains("home"));
        assert!(placeholder.starts_with("<redacted:path sha="));
    }

    #[test]
    fn redacted_url_drops_query_and_fragment() {
        let r = Redactor::default();
        let red = r.redact_url("https://example.com/path?token=abc#section");
        assert_eq!(
            red.sanitized.as_deref(),
            Some("https://example.com/path")
        );
        assert!(red.had_query);
        assert!(!red.placeholder().contains("token=abc"));
        assert!(!red.placeholder().contains("section"));
    }

    #[test]
    fn redacted_url_keeps_plain_url() {
        let r = Redactor::default();
        let red = r.redact_url("https://download.kiwix.org/catalog/v3/");
        assert_eq!(
            red.placeholder(),
            "https://download.kiwix.org/catalog/v3/"
        );
        assert!(!red.had_query);
    }

    #[test]
    fn redacted_url_unparseable_becomes_opaque() {
        let r = Redactor::default();
        let red = r.redact_url("not a url at all");
        assert!(red.sanitized.is_none());
        assert!(!red.placeholder().contains("not a url"));
    }

    #[test]
    fn correlation_length_is_bounded() {
        let r = Redactor::new(RedactionConfig::new().with_correlation_hex_len(64));
        let red = r.redact_text(UserTextKind::Article, b"abc");
        assert_eq!(red.correlation.len(), 64);

        let r2 = Redactor::new(RedactionConfig::new().with_correlation_hex_len(0));
        let red2 = r2.redact_text(UserTextKind::Article, b"abc");
        assert_eq!(red2.correlation.len(), 0);
    }

    #[test]
    fn safe_categories_pass_through() {
        let r = Redactor::default();
        assert_eq!(r.entity_id("pack:wikipedia").0, "pack:wikipedia");
        assert_eq!(r.error_code("E_VERIFY").0, "E_VERIFY");
        assert_eq!(r.byte_count(1024).0, 1024);
        assert_eq!(r.duration_millis(750).0, 750);
        let st = r.state_transition("download", "staged", "verified");
        assert_eq!(st.entity, "download");
        assert_eq!(st.from, "staged");
        assert_eq!(st.to, "verified");
        let v = r.version("reader", "1.0.52684");
        assert_eq!(v.component, "reader");
        assert_eq!(v.version, "1.0.52684");
    }
}
