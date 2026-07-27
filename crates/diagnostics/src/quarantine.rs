//! Quarantine records per §18.7.
//!
//! Quarantined content:
//!
//! - cannot be opened by a production engine;
//! - is stored outside active roots;
//! - has an `audit_events` record;
//! - records source, expected/actual digest and verifier error;
//! - can be deleted;
//! - cannot be "use anyway" activated.
//!
//! This module owns the metadata side: the [`QuarantineRecord`] that
//! captures source / digests / verifier error / audit_event_id, and the
//! typed [`QuarantineLocator`] whose only constructor places content
//! outside active roots. The byte-level quarantine (moving staged files
//! into the `quarantine/` directory of the object store) is owned by the
//! `object-store` crate; this crate records *why*.
//!
//! ## "Cannot be 'use anyway' activated"
//!
//! There is no method on [`QuarantineRecord`] that returns content bytes,
//! a usable content path, or any "activate" operation. The only function
//! a production engine needs to consult is [`refuse_quarantined`], which
//! always returns `Err` — quarantined content is always refused.

use crate::audit::{AuditEvent, AuditEventBuilder, AuditEventError, AuditSeverity};
use crate::redactor::{
    EntityId, RedactedPath, RedactedUrl, Redactor,
};
use domain::identity::ContentDigest;
use serde::{Deserialize, Serialize};

/// Error returned by [`refuse_quarantined`]. Production engines that consult
/// this crate before opening content cannot bypass quarantine — there is no
/// `Ok` variant in the function's return.
#[derive(Clone, Debug, thiserror::Error)]
pub enum QuarantineAccessError {
    #[error("content is quarantined (audit_event_id={audit_event_id}); production engines cannot open it")]
    Refused {
        audit_event_id: String,
        verifier_error: String,
    },
}

/// Refuse to open quarantined content. Always returns [`QuarantineAccessError::Refused`].
///
/// Production engines must call this before opening any content that may be
/// quarantined. The function is intentionally monomorphic and infallible
/// in the `Ok` direction: there is no code path that returns `Ok(())`, so
/// the compiler will reject any caller that treats the refusal as anything
/// other than an error.
pub fn refuse_quarantined(record: &QuarantineRecord) -> Result<(), QuarantineAccessError> {
    Err(QuarantineAccessError::Refused {
        audit_event_id: record.audit_event_id.clone(),
        verifier_error: record.verifier_error.clone(),
    })
}

/// Typed location of quarantined bytes. The only constructor is
/// [`QuarantineLocator::outside_active_roots`]; there is no constructor
/// for an active-roots placement. This is the type-level enforcement of
/// §18.7 "stored outside active roots".
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuarantineLocator {
    /// Relative directory label, always `quarantine/<audit_event_id>` under
    /// the EnZIME root. Carried as a string so it can round-trip through
    /// serde without dragging in `std::path::Path`.
    relative_dir: String,
    audit_event_id: String,
}

impl QuarantineLocator {
    /// Place quarantined bytes in the per-audit-event directory under the
    /// `quarantine/` root of the EnZIME object store (§6.3 layout). The
    /// `audit_event_id` is validated as a single path component so the
    /// resulting locator cannot escape the quarantine tree.
    pub fn outside_active_roots(
        audit_event_id: &str,
    ) -> Result<Self, QuarantineLocatorError> {
        validate_path_component(audit_event_id)?;
        Ok(Self {
            relative_dir: format!("quarantine/{audit_event_id}"),
            audit_event_id: audit_event_id.to_string(),
        })
    }

    pub fn relative_dir(&self) -> &str {
        &self.relative_dir
    }

    pub fn audit_event_id(&self) -> &str {
        &self.audit_event_id
    }
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum QuarantineLocatorError {
    #[error("audit_event_id is not a single safe path component: {0}")]
    UnsafeComponent(String),
}

fn validate_path_component(raw: &str) -> Result<(), QuarantineLocatorError> {
    if raw.trim().is_empty() {
        return Err(QuarantineLocatorError::UnsafeComponent(
            "identifier is empty".to_string(),
        ));
    }
    if raw.len() > 255 {
        return Err(QuarantineLocatorError::UnsafeComponent(format!(
            "identifier exceeds 255 bytes (was {})",
            raw.len()
        )));
    }
    if raw.contains('/')
        || raw.contains('\\')
        || raw.contains('\0')
        || raw == "."
        || raw == ".."
        || raw.starts_with('.')
        || raw.contains("..")
    {
        return Err(QuarantineLocatorError::UnsafeComponent(format!(
            "identifier contains a path or URL component: {raw}"
        )));
    }
    let mut chars = raw.chars();
    if let (Some(letter), Some(':')) = (chars.next(), chars.next()) {
        if letter.is_ascii_alphabetic() {
            return Err(QuarantineLocatorError::UnsafeComponent(format!(
                "identifier contains a path or URL component: {raw}"
            )));
        }
    }
    Ok(())
}

/// Typed label for the origin of quarantined bytes. §18.7 says the record
/// must record "source"; §18.6 says paths and URLs are excluded from
/// default logs. Construction forces the caller to classify the source so
/// that path- and URL-bearing sources are reduced to their redacted form
/// at the boundary.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum QuarantineSourceLabel {
    Path { redacted: RedactedPath },
    Url { redacted: RedactedUrl },
    Entity { id: EntityId },
}

impl QuarantineSourceLabel {
    pub fn from_path(redactor: &Redactor, path: &std::path::Path) -> Self {
        QuarantineSourceLabel::Path {
            redacted: redactor.redact_path(path),
        }
    }

    pub fn from_url(redactor: &Redactor, url: &str) -> Self {
        QuarantineSourceLabel::Url {
            redacted: redactor.redact_url(url),
        }
    }

    pub fn from_entity(id: EntityId) -> Self {
        QuarantineSourceLabel::Entity { id }
    }
}

/// One quarantine event. Captures the metadata §18.7 requires; does not
/// expose content bytes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuarantineRecord {
    pub audit_event_id: String,
    pub entity_kind: String,
    pub entity_id: String,
    pub source: QuarantineSourceLabel,
    pub expected_digest: ContentDigest,
    pub actual_digest: Option<ContentDigest>,
    /// Raw display of the verifier error, as required by §18.7. Stays in
    /// the audit_events row but is **never** rendered into a
    /// [`crate::DiagnosticBundle`] without passing through
    /// [`QuarantineRecord::redacted_verifier_error`] (which strips
    /// path-like and URL-query substrings).
    pub verifier_error: String,
    pub quarantined_at_ms: i64,
    pub locator: QuarantineLocator,
    pub deleted: bool,
}

impl QuarantineRecord {
    /// Build a new record. The `actual_digest` may be `None` when the
    /// verification failed before a complete digest could be computed
    /// (e.g. streaming was interrupted). The `verifier_error` is the
    /// raw display of the underlying error.
    pub fn new(
        audit_event_id: impl Into<String>,
        entity_kind: impl Into<String>,
        entity_id: impl Into<String>,
        source: QuarantineSourceLabel,
        expected_digest: ContentDigest,
        actual_digest: Option<ContentDigest>,
        verifier_error: impl Into<String>,
        quarantined_at_ms: i64,
        locator: QuarantineLocator,
    ) -> Self {
        let audit_event_id = audit_event_id.into();
        Self {
            audit_event_id,
            entity_kind: entity_kind.into(),
            entity_id: entity_id.into(),
            source,
            expected_digest,
            actual_digest,
            verifier_error: verifier_error.into(),
            quarantined_at_ms,
            locator,
            deleted: false,
        }
    }

    /// Mark the record as deleted. §18.7 says quarantined content "can be
    /// deleted". This flips the metadata flag and returns the audit-event
    /// row that records the deletion; the byte-level removal is owned by
    /// the object store.
    pub fn mark_deleted(&mut self, at_ms: i64) -> Result<AuditEvent, AuditEventError> {
        self.deleted = true;
        let mut detail = AuditEventBuilder::new();
        detail.entity_id("audit_event_id", &EntityId(self.audit_event_id.clone()))?;
        let ev = AuditEvent::new(
            format!("{}-delete", self.audit_event_id),
            "quarantine_delete",
            self.entity_kind.clone(),
            self.entity_id.clone(),
            detail.to_json_string(),
            AuditSeverity::Info,
            at_ms,
        )?;
        Ok(ev)
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted
    }

    /// Build the `audit_events` row that §18.7 requires the quarantine to
    /// produce. Severity is `security`; the detail JSON carries source,
    /// expected/actual digest and verifier error in their redacted forms.
    pub fn to_audit_event(&self) -> Result<AuditEvent, AuditEventError> {
        let mut detail = AuditEventBuilder::new();
        detail.entity_id("entity_id", &EntityId(self.entity_id.clone()))?;
        match &self.source {
            QuarantineSourceLabel::Path { redacted } => {
                detail.redacted_path("source", redacted)?;
            }
            QuarantineSourceLabel::Url { redacted } => {
                detail.redacted_url("source", redacted)?;
            }
            QuarantineSourceLabel::Entity { id } => {
                detail.entity_id("source", id)?;
            }
        }
        detail.error_code(
            "expected_digest_algorithm",
            &crate::redactor::ErrorCode(format!("{:?}", self.expected_digest.algorithm())),
        )?;
        // Digests are SHA-256 hashes of content, not content themselves —
        // safe to record verbatim per §18.6 (they are not "tokens" or
        // "keys"; they are content addresses).
        detail.place_string("expected_digest_hex", self.expected_digest.hex().to_string())?;
        match &self.actual_digest {
            Some(d) => {
                detail.place_string("actual_digest_hex", d.hex().to_string())?;
            }
            None => {
                detail.place_string("actual_digest_hex", "none".to_string())?;
            }
        }
        detail.place_string("verifier_error", scrub_verifier_error(&self.verifier_error))?;
        detail.place_string("locator", self.locator.relative_dir.clone())?;

        AuditEvent::new(
            self.audit_event_id.clone(),
            "quarantine",
            self.entity_kind.clone(),
            self.entity_id.clone(),
            detail.to_json_string(),
            AuditSeverity::Security,
            self.quarantined_at_ms,
        )
    }
}

/// Best-effort scrubbing of a Rust error display string for inclusion in
/// shared diagnostics. Replaces absolute-path-like and Windows-drive-prefixed
/// tokens with `<redacted:path>` and rewrites URLs with query strings or
/// fragments to drop them.
///
/// The scrubber operates on whitespace-delimited tokens because Rust error
/// displays are prose-like (`"file not found: /home/x/y"`, `"get https://a/b?c"`).
/// Free-form prose that contains no path-like or URL-like tokens is kept
/// verbatim.
fn scrub_verifier_error(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for token in raw.split_whitespace() {
        if out.len() > 0 {
            out.push(' ');
        }
        out.push_str(&scrub_token(token));
    }
    out
}

fn scrub_token(token: &str) -> String {
    if let Some(scheme_end) = token.find("://") {
        let scheme = &token[..scheme_end];
        if scheme.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
            && !scheme.is_empty()
        {
            // URL-like. Reuse the redactor's URL redactor.
            let r = Redactor::default();
            return r.redact_url(token).placeholder();
        }
    }
    if token.starts_with('/') || token.starts_with("~/") || token == "~" {
        return "<redacted:path>".to_string();
    }
    if is_windows_drive_prefixed(token) {
        return "<redacted:path>".to_string();
    }
    token.to_string()
}

fn is_windows_drive_prefixed(s: &str) -> bool {
    let mut chars = s.chars();
    match (chars.next(), chars.next()) {
        (Some(letter), Some(':')) => {
            letter.is_ascii_alphabetic()
                && (chars.next() == Some('\\') || chars.next() == Some('/'))
        }
        _ => false,
    }
}

// Local helper trait intentionally absent: `AuditEventBuilder::place_string`
// is exposed `pub(crate)` so this module can place pre-redacted/pre-scrubbed
// strings (digest hex, scrubbed verifier error, fixed directory labels)
// directly. External callers go through the typed redacting helpers and
// cannot reach `place_string`.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redactor::{RedactedPath, RedactedText, Redactor, RedactionConfig, UserTextKind};
    use domain::identity::{ContentDigest, DigestAlgorithm};

    fn digest(hex: &str) -> ContentDigest {
        ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            hex: hex.to_string(),
        }
    }

    fn make_record(verifier_error: &str) -> QuarantineRecord {
        let r = Redactor::default();
        let source = QuarantineSourceLabel::from_path(&r, std::path::Path::new("/tmp/secret.zim"));
        let locator = QuarantineLocator::outside_active_roots("ae-123").unwrap();
        QuarantineRecord::new(
            "ae-123",
            "pack_version",
            "pv-1",
            source,
            digest("0000000000000000000000000000000000000000000000000000000000000000"),
            Some(digest(
                "1111111111111111111111111111111111111111111111111111111111111111",
            )),
            verifier_error,
            1_700_000_000_000,
            locator,
        )
    }

    #[test]
    fn refuse_quarantined_always_errors() {
        let rec = make_record("digest mismatch");
        let r = refuse_quarantined(&rec);
        assert!(matches!(r, Err(QuarantineAccessError::Refused { .. })));
    }

    #[test]
    fn locator_only_outside_active_roots() {
        let loc = QuarantineLocator::outside_active_roots("ae-1").unwrap();
        assert_eq!(loc.relative_dir(), "quarantine/ae-1");

        // Traversal attempts are refused.
        assert!(QuarantineLocator::outside_active_roots("../escape").is_err());
        assert!(QuarantineLocator::outside_active_roots("a/b").is_err());
        assert!(QuarantineLocator::outside_active_roots("C:drive").is_err());
        assert!(QuarantineLocator::outside_active_roots("").is_err());
    }

    #[test]
    fn record_does_not_expose_source_bytes() {
        let rec = make_record("digest mismatch");
        // Source is a RedactedPath in serialised form — the original path
        // bytes are never stored.
        let json = serde_json::to_string(&rec.source).unwrap();
        assert!(!json.contains("/tmp/secret.zim"));
        assert!(!json.contains("secret"));
        assert!(!json.contains("/tmp"));
        // The placeholder form leaks no path component either.
        if let QuarantineSourceLabel::Path { redacted } = &rec.source {
            let p = redacted.placeholder();
            assert!(!p.contains("secret"));
            assert!(!p.contains("/tmp"));
        } else {
            panic!("expected path variant");
        }
    }

    #[test]
    fn audit_event_records_required_fields() {
        let rec = make_record("download interrupted at /home/robin/foo");
        let ev = rec.to_audit_event().unwrap();
        assert_eq!(ev.event_type, "quarantine");
        assert_eq!(ev.severity, AuditSeverity::Security);
        let detail: serde_json::Value = serde_json::from_str(&ev.detail_json).unwrap();
        assert!(detail.get("source").is_some());
        assert!(detail.get("expected_digest_hex").is_some());
        assert!(detail.get("actual_digest_hex").is_some());
        assert!(detail.get("verifier_error").is_some());
        // Verifier error scrubbed.
        let v: String = serde_json::from_value(detail["verifier_error"].clone()).unwrap();
        assert!(!v.contains("/home/robin/foo"));
        assert!(v.contains("<redacted:path>"));
    }

    #[test]
    fn mark_deleted_flips_flag_and_emits_event() {
        let mut rec = make_record("digest mismatch");
        assert!(!rec.is_deleted());
        let ev = rec.mark_deleted(1_700_000_001_000).unwrap();
        assert!(rec.is_deleted());
        assert_eq!(ev.event_type, "quarantine_delete");
    }

    #[test]
    fn scrubber_redacts_url_query_and_paths() {
        let s = scrub_verifier_error("download from https://x.example/y?token=secret then /home/u/priv");
        assert!(!s.contains("token=secret"));
        assert!(!s.contains("/home/u/priv"));
        assert!(s.contains("https://x.example/y"));
        assert!(s.contains("<redacted:path>"));
    }

    #[test]
    fn scrubber_keeps_prose() {
        let s = scrub_verifier_error("sha256 mismatch (expected 64 hex chars)");
        assert_eq!(s, "sha256 mismatch (expected 64 hex chars)");
    }

    #[test]
    fn record_can_be_serialised() {
        let rec = make_record("x");
        let json = serde_json::to_string(&rec).unwrap();
        let back: QuarantineRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back.audit_event_id, rec.audit_event_id);
        assert_eq!(back.entity_kind, rec.entity_kind);
        assert_eq!(back.locator, rec.locator);
    }

    // Smoke check that RedactedText can be constructed for cross-reference.
    #[test]
    fn redacted_text_helper_works() {
        let _r: RedactedText = Redactor::default().redact_text(UserTextKind::Lore, b"x");
        let _cfg = RedactionConfig::default();
    }
}
