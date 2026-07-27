//! Inspectable, redacted diagnostic bundle.
//!
//! A [`DiagnosticBundle`] is the unit a user reads before sharing. It owns a
//! [`Redactor`] (so every value is classified at insertion time) and an
//! ordered list of [`Entry`] rows. It exposes three renderers
//! ([`RenderFormat::Markdown`], [`RenderFormat::Text`], [`RenderFormat::Json`])
//! so the user can pick the form most readable to them.
//!
//! Entries are accepted only in their typed form. There is no `push_string`
//! or `push_raw`: redaction is not an option the caller can forget to pass.

use crate::audit::AuditEvent;
use crate::redactor::{
    ByteCount, DurationMillis, EntityId, ErrorCode, RedactedPath, RedactedText, RedactedUrl,
    Redactor, RedactionConfig, StateTransition, Version,
};
use serde::{Deserialize, Serialize};

/// Render format for [`DiagnosticBundle::render`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderFormat {
    Markdown,
    Text,
    Json,
}

/// One row in a [`DiagnosticBundle`]. The variant set is exactly the §18.6
/// "may include" list (pass-through) plus the three §18.6 "excludes"
/// redacted categories. There is no free-form string variant.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Entry {
    EntityId {
        key: String,
        value: EntityId,
    },
    ErrorCode {
        code: ErrorCode,
        detail: Option<String>,
    },
    ByteCount {
        key: String,
        value: ByteCount,
    },
    DurationMillis {
        key: String,
        value: DurationMillis,
    },
    StateTransition {
        value: StateTransition,
    },
    Version {
        value: Version,
    },
    RedactedText {
        key: String,
        value: RedactedText,
    },
    RedactedPath {
        key: String,
        value: RedactedPath,
    },
    RedactedUrl {
        key: String,
        value: RedactedUrl,
    },
    Audit {
        value: AuditEvent,
    },
}

impl DiagnosticBundle {
    /// Append an opaque entity identifier (§18.6 "may include").
    pub fn push_entity_id(&mut self, key: impl Into<String>, value: EntityId) {
        self.entries.push(Entry::EntityId {
            key: key.into(),
            value,
        });
    }

    /// Append an error code (§18.6 "may include"). Optional detail MUST be
    /// an error code, status enum, or short fixed vocabulary — never user
    /// text or a path. The caller is the only place that can know whether
    /// the string is vocabulary, so it is their responsibility to pass only
    /// fixed-vocabulary strings here.
    pub fn push_error_code(&mut self, code: ErrorCode, detail: Option<String>) {
        self.entries.push(Entry::ErrorCode { code, detail });
    }

    /// Append a byte count (§18.6 "may include").
    pub fn push_byte_count(&mut self, key: impl Into<String>, value: ByteCount) {
        self.entries.push(Entry::ByteCount {
            key: key.into(),
            value,
        });
    }

    /// Append a duration in milliseconds (§18.6 "may include").
    pub fn push_duration_millis(&mut self, key: impl Into<String>, value: DurationMillis) {
        self.entries.push(Entry::DurationMillis {
            key: key.into(),
            value,
        });
    }

    /// Append a state transition (§18.6 "may include").
    pub fn push_state_transition(&mut self, value: StateTransition) {
        self.entries.push(Entry::StateTransition { value });
    }

    /// Append an app/platform version (§18.6 "may include").
    pub fn push_version(&mut self, value: Version) {
        self.entries.push(Entry::Version { value });
    }

    /// Append a redacted user-text entry (§18.6 "excludes"). The caller
    /// supplies the already-redacted [`RedactedText`]; the original bytes
    /// never enter the bundle.
    pub fn push_redacted_text(&mut self, key: impl Into<String>, value: RedactedText) {
        self.entries.push(Entry::RedactedText {
            key: key.into(),
            value,
        });
    }

    /// Append a redacted filesystem path (§18.6 "excludes").
    pub fn push_redacted_path(&mut self, key: impl Into<String>, value: RedactedPath) {
        self.entries.push(Entry::RedactedPath {
            key: key.into(),
            value,
        });
    }

    /// Append a redacted URL (§18.6 "excludes").
    pub fn push_redacted_url(&mut self, key: impl Into<String>, value: RedactedUrl) {
        self.entries.push(Entry::RedactedUrl {
            key: key.into(),
            value,
        });
    }

    /// Append an audit-event row. The detail_json inside the event is the
    /// only path that carries a free-form JSON column; the audit module
    /// provides a builder that only accepts redacted typed values.
    pub fn push_audit(&mut self, value: AuditEvent) {
        self.entries.push(Entry::Audit { value });
    }
}

/// Inspectable, redacted diagnostic bundle.
#[derive(Clone, Debug)]
pub struct DiagnosticBundle {
    redactor: Redactor,
    entries: Vec<Entry>,
    created_at_ms: i64,
}

impl DiagnosticBundle {
    /// Build a new bundle from a config and an authoritative "now" value
    /// (epoch milliseconds). The config is fixed for the lifetime of the
    /// bundle; all entries redacted through [`Self::redactor`] use it.
    pub fn new(config: RedactionConfig, created_at_ms: i64) -> Self {
        Self {
            redactor: Redactor::new(config),
            entries: Vec::new(),
            created_at_ms,
        }
    }

    /// Borrow the redactor so the caller can pre-redact paths, URLs and
    /// text before pushing them.
    pub fn redactor(&self) -> &Redactor {
        &self.redactor
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn created_at_ms(&self) -> i64 {
        self.created_at_ms
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Render the bundle in the requested format. The output is what the
    /// user reads before consenting to share.
    pub fn render(&self, format: RenderFormat) -> String {
        match format {
            RenderFormat::Markdown => self.to_markdown(),
            RenderFormat::Text => self.to_text(),
            RenderFormat::Json => self.to_json(),
        }
    }

    /// Markdown rendering — the most human-readable form.
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str("# EnZIME diagnostic bundle\n\n");
        out.push_str(&format!("- created_at_ms: {}\n", self.created_at_ms));
        out.push_str(&format!("- entry_count: {}\n\n", self.entries.len()));
        for entry in &self.entries {
            out.push_str(&render_entry_markdown(entry));
            out.push('\n');
        }
        out
    }

    /// Plain-text rendering — line-oriented, no markup.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("EnZIME diagnostic bundle\n"));
        out.push_str(&format!("created_at_ms={}\n", self.created_at_ms));
        out.push_str(&format!("entry_count={}\n", self.entries.len()));
        for entry in &self.entries {
            out.push_str(&render_entry_text(entry));
            out.push('\n');
        }
        out
    }

    /// JSON rendering — the machine-readable form. Carries every redacted
    /// value, never the original.
    pub fn to_json(&self) -> String {
        let payload = serde_json::json!({
            "created_at_ms": self.created_at_ms,
            "entries": self.entries,
        });
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
    }
}

fn render_entry_markdown(e: &Entry) -> String {
    match e {
        Entry::EntityId { key, value } => format!("- **{}** = `{}`", key, value.0),
        Entry::ErrorCode { code, detail } => match detail {
            Some(d) => format!("- **error** = `{}` — {}", code.0, d),
            None => format!("- **error** = `{}`", code.0),
        },
        Entry::ByteCount { key, value } => format!("- **{}** = {} bytes", key, value.0),
        Entry::DurationMillis { key, value } => format!("- **{}** = {} ms", key, value.0),
        Entry::StateTransition { value } => {
            format!("- **{}**: {} → {}", value.entity, value.from, value.to)
        }
        Entry::Version { value } => format!("- **{}** = {}", value.component, value.version),
        Entry::RedactedText { key, value } => format!("- **{}** = {}", key, value.placeholder()),
        Entry::RedactedPath { key, value } => format!("- **{}** = {}", key, value.placeholder()),
        Entry::RedactedUrl { key, value } => format!("- **{}** = {}", key, value.placeholder()),
        Entry::Audit { value } => format!(
            "- **audit** `{}` ({}/{}/{}) at {} — {}",
            value.audit_event_id,
            value.event_type,
            value.entity_kind,
            value.severity.as_str(),
            value.created_at_ms,
            value.detail_json
        ),
    }
}

fn render_entry_text(e: &Entry) -> String {
    match e {
        Entry::EntityId { key, value } => format!("{}={}", key, value.0),
        Entry::ErrorCode { code, detail } => match detail {
            Some(d) => format!("error={} ({})", code.0, d),
            None => format!("error={}", code.0),
        },
        Entry::ByteCount { key, value } => format!("{}_bytes={}", key, value.0),
        Entry::DurationMillis { key, value } => format!("{}_ms={}", key, value.0),
        Entry::StateTransition { value } => {
            format!("{}: {}->{}", value.entity, value.from, value.to)
        }
        Entry::Version { value } => format!("{}={}", value.component, value.version),
        Entry::RedactedText { key, value } => format!("{}={}", key, value.placeholder()),
        Entry::RedactedPath { key, value } => format!("{}={}", key, value.placeholder()),
        Entry::RedactedUrl { key, value } => format!("{}={}", key, value.placeholder()),
        Entry::Audit { value } => format!(
            "audit={} type={} kind={} sev={} detail={}",
            value.audit_event_id,
            value.event_type,
            value.entity_kind,
            value.severity.as_str(),
            value.detail_json
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redactor::UserTextKind;

    #[test]
    fn bundle_redacts_sensitive_inputs() {
        let mut b = DiagnosticBundle::new(RedactionConfig::default(), 1_700_000_000_000);
        let r = b.redactor().clone();

        b.push_entity_id("pack", r.entity_id("pack:wikipedia"));
        b.push_error_code(r.error_code("E_VERIFY"), None);
        b.push_byte_count("chunk", r.byte_count(4096));
        b.push_duration_millis("verify", r.duration_millis(120));
        b.push_state_transition(r.state_transition("download", "staged", "verified"));
        b.push_version(r.version("reader", "1.0.52684"));
        b.push_redacted_text(
            "article_snippet",
            r.redact_text(UserTextKind::Article, b"The quick brown fox"),
        );
        b.push_redacted_path(
            "download_source",
            r.redact_path(std::path::Path::new("/home/robin/secret.md")),
        );
        b.push_redacted_url(
            "catalog",
            r.redact_url("https://example.com/x?token=abc123"),
        );

        for fmt in [RenderFormat::Markdown, RenderFormat::Text, RenderFormat::Json] {
            let out = b.render(fmt);
            assert!(!out.contains("The quick brown fox"), "fmt {:?} leaked text", fmt);
            assert!(!out.contains("robin"), "fmt {:?} leaked path", fmt);
            assert!(!out.contains("secret.md"), "fmt {:?} leaked filename", fmt);
            assert!(!out.contains("token=abc123"), "fmt {:?} leaked query", fmt);
            assert!(out.contains("pack:wikipedia"), "fmt {:?} dropped entity id", fmt);
            assert!(out.contains("E_VERIFY"), "fmt {:?} dropped error code", fmt);
        }
    }

    #[test]
    fn empty_bundle_renders_header() {
        let b = DiagnosticBundle::new(RedactionConfig::default(), 42);
        let md = b.to_markdown();
        assert!(md.contains("entry_count: 0"));
        assert!(md.contains("created_at_ms: 42"));
    }

    #[test]
    fn audit_event_appears_in_render() {
        let mut b = DiagnosticBundle::new(RedactionConfig::default(), 1);
        let mut detail = AuditEvent::build_detail();
        detail.byte_count("size", ByteCount(100)).unwrap();
        let ev = AuditEvent::new(
            "ae-1",
            "quarantine",
            "pack_version",
            "pv-1",
            detail.to_json_string(),
            crate::audit::AuditSeverity::Security,
            5,
        )
        .unwrap();
        b.push_audit(ev);
        let out = b.to_text();
        assert!(out.contains("ae-1"));
        assert!(out.contains("quarantine"));
        assert!(out.contains("security"));
    }
}
