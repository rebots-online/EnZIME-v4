//! Audit-event row, mirroring the `audit_events` SQLite table from §6.2:
//!
//! ```sql
//! CREATE TABLE audit_events (
//!   audit_event_id TEXT PRIMARY KEY,
//!   event_type TEXT NOT NULL,
//!   entity_kind TEXT NOT NULL,
//!   entity_id TEXT NOT NULL,
//!   detail_json TEXT NOT NULL CHECK(json_valid(detail_json)),
//!   severity TEXT NOT NULL CHECK(severity IN ('info','warning','error','security')),
//!   created_at_ms INTEGER NOT NULL
//! ) STRICT;
//! ```
//!
//! This module owns the typed Rust analogue. It does not write to SQLite —
//! the storage-sqlite crate owns that — but it produces values that match
//! the schema row-for-row.
//!
//! The `detail_json` column is populated from a [`AuditEventBuilder`] that
//! accepts only redacted typed values, so an event raised from inside a
//! secret- or user-text-bearing code path cannot accidentally leak bytes via
//! the JSON column.

use crate::redactor::{
    ByteCount, DurationMillis, EntityId, ErrorCode, RedactedPath, RedactedText, RedactedUrl,
    StateTransition, Version,
};
use serde::{Deserialize, Serialize};

/// Severity values for the `severity` column. The serde rename matches the
/// CHECK constraint in §6.2 character-for-character.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
    Security,
}

impl AuditSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            AuditSeverity::Info => "info",
            AuditSeverity::Warning => "warning",
            AuditSeverity::Error => "error",
            AuditSeverity::Security => "security",
        }
    }
}

/// Typed analogue of one `audit_events` row.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub audit_event_id: String,
    pub event_type: String,
    pub entity_kind: String,
    pub entity_id: String,
    pub detail_json: String,
    pub severity: AuditSeverity,
    pub created_at_ms: i64,
}

impl AuditEvent {
    /// Construct an audit event with an already-built JSON detail string.
    /// The detail string must be valid JSON; prefer [`AuditEventBuilder`]
    /// when raising events from typed values so that redaction is enforced
    /// at construction time.
    pub fn new(
        audit_event_id: impl Into<String>,
        event_type: impl Into<String>,
        entity_kind: impl Into<String>,
        entity_id: impl Into<String>,
        detail_json: impl Into<String>,
        severity: AuditSeverity,
        created_at_ms: i64,
    ) -> Result<Self, AuditEventError> {
        let detail_json = detail_json.into();
        // serde_json rejects non-object/document inputs; verify here so the
        // storage layer can trust the value matches `json_valid(detail_json)`.
        serde_json::from_str::<serde_json::Value>(&detail_json)
            .map_err(|_| AuditEventError::InvalidDetailJson)?;
        Ok(Self {
            audit_event_id: audit_event_id.into(),
            event_type: event_type.into(),
            entity_kind: entity_kind.into(),
            entity_id: entity_id.into(),
            detail_json,
            severity,
            created_at_ms,
        })
    }

    /// Begin a builder for the JSON detail column. The builder only accepts
    /// redacted typed values, so events raised from inside a secret- or
    /// user-text-bearing code path cannot leak bytes.
    pub fn build_detail() -> AuditEventBuilder {
        AuditEventBuilder::new()
    }
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum AuditEventError {
    #[error("detail_json is not valid JSON")]
    InvalidDetailJson,
    #[error("key is empty or already used: {0}")]
    DuplicateKey(String),
}

/// Builder for the `detail_json` column that accepts only redacted typed
/// values. Inserting the same key twice is an error.
#[derive(Clone, Debug, Default)]
pub struct AuditEventBuilder {
    object: serde_json::Map<String, serde_json::Value>,
    seen: std::collections::HashSet<String>,
}

impl AuditEventBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    fn place(&mut self, key: &str, value: serde_json::Value) -> Result<(), AuditEventError> {
        if key.is_empty() {
            return Err(AuditEventError::DuplicateKey(key.to_string()));
        }
        if !self.seen.insert(key.to_string()) {
            return Err(AuditEventError::DuplicateKey(key.to_string()));
        }
        self.object.insert(key.to_string(), value);
        Ok(())
    }

    /// Crate-internal string placement. Used by modules inside
    /// `diagnostics` that have already redacted or scrubbed the value
    /// (digest hex, scrubbed verifier error, fixed directory labels).
    /// Exposed only to `pub(crate)` so external callers cannot bypass the
    /// typed redacting helpers.
    pub(crate) fn place_string(
        &mut self,
        key: &str,
        value: String,
    ) -> Result<(), AuditEventError> {
        self.place(key, serde_json::Value::String(value))
    }

    pub fn entity_id(&mut self, key: &str, v: &EntityId) -> Result<&mut Self, AuditEventError> {
        self.place(key, serde_json::to_value(&v.0).unwrap())?;
        Ok(self)
    }

    pub fn error_code(&mut self, key: &str, v: &ErrorCode) -> Result<&mut Self, AuditEventError> {
        self.place(key, serde_json::to_value(&v.0).unwrap())?;
        Ok(self)
    }

    pub fn byte_count(&mut self, key: &str, v: ByteCount) -> Result<&mut Self, AuditEventError> {
        self.place(key, serde_json::to_value(v.0).unwrap())?;
        Ok(self)
    }

    pub fn duration_millis(
        &mut self,
        key: &str,
        v: DurationMillis,
    ) -> Result<&mut Self, AuditEventError> {
        self.place(key, serde_json::to_value(v.0).unwrap())?;
        Ok(self)
    }

    pub fn state_transition(
        &mut self,
        key: &str,
        v: &StateTransition,
    ) -> Result<&mut Self, AuditEventError> {
        self.place(key, serde_json::to_value(v).unwrap())?;
        Ok(self)
    }

    pub fn version(&mut self, key: &str, v: &Version) -> Result<&mut Self, AuditEventError> {
        self.place(key, serde_json::to_value(v).unwrap())?;
        Ok(self)
    }

    pub fn redacted_text(
        &mut self,
        key: &str,
        v: &RedactedText,
    ) -> Result<&mut Self, AuditEventError> {
        self.place(key, serde_json::to_value(v).unwrap())?;
        Ok(self)
    }

    pub fn redacted_path(
        &mut self,
        key: &str,
        v: &RedactedPath,
    ) -> Result<&mut Self, AuditEventError> {
        self.place(key, serde_json::to_value(v).unwrap())?;
        Ok(self)
    }

    pub fn redacted_url(
        &mut self,
        key: &str,
        v: &RedactedUrl,
    ) -> Result<&mut Self, AuditEventError> {
        self.place(key, serde_json::to_value(v).unwrap())?;
        Ok(self)
    }

    /// Finalise the detail column as a JSON string suitable for the
    /// `detail_json` CHECK constraint.
    pub fn to_json_string(&self) -> String {
        serde_json::Value::Object(self.object.clone()).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redactor::UserTextKind;

    #[test]
    fn builder_produces_valid_json() {
        let r = crate::redactor::Redactor::default();
        let mut b = AuditEvent::build_detail();
        b.entity_id("pack", &r.entity_id("pack:wikipedia")).unwrap();
        b.error_code("code", &r.error_code("E_VERIFY")).unwrap();
        b.byte_count("size", r.byte_count(2048)).unwrap();
        b.redacted_text(
            "snippet",
            &r.redact_text(UserTextKind::Article, b"hello"),
        )
        .unwrap();
        let s = b.to_json_string();
        let parsed: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert!(parsed.is_object());
        assert_eq!(parsed["pack"], "pack:wikipedia");
        assert_eq!(parsed["code"], "E_VERIFY");
        assert_eq!(parsed["size"], 2048);
        assert_eq!(parsed["snippet"]["kind"], "article");
    }

    #[test]
    fn builder_rejects_duplicate_keys() {
        let r = crate::redactor::Redactor::default();
        let mut b = AuditEvent::build_detail();
        b.byte_count("n", r.byte_count(1)).unwrap();
        let err = b.byte_count("n", r.byte_count(2)).unwrap_err();
        assert!(matches!(err, AuditEventError::DuplicateKey(_)));
    }

    #[test]
    fn audit_event_rejects_invalid_json() {
        let err = AuditEvent::new(
            "id",
            "type",
            "kind",
            "eid",
            "not json",
            AuditSeverity::Info,
            0,
        )
        .unwrap_err();
        assert!(matches!(err, AuditEventError::InvalidDetailJson));
    }

    #[test]
    fn severity_serialises_to_sql_check_values() {
        for (sev, expect) in [
            (AuditSeverity::Info, "info"),
            (AuditSeverity::Warning, "warning"),
            (AuditSeverity::Error, "error"),
            (AuditSeverity::Security, "security"),
        ] {
            let s = serde_json::to_string(&sev).unwrap();
            let trimmed = s.trim_matches('"');
            assert_eq!(trimmed, expect);
        }
    }
}
