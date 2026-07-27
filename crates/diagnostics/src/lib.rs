#![forbid(unsafe_code)]

//! Ring-1 diagnostics: inspectable, redacted diagnostic bundles and
//! quarantine metadata, per ARCHITECTURE.md §18.6 (Logging) and §18.7
//! (Quarantine).
//!
//! # Redaction by default
//!
//! Every value placed in a [`DiagnosticBundle`] passes through a typed
//! [`Redactor`] entry point that classifies it against the §18.6 "may include"
//! and "excludes" lists. There is no `push_string` or `push_raw`: a caller
//! cannot forget to redact, because the only way to add a value is to name
//! its category.
//!
//! The §18.6 "excludes" categories (article/annotation/message/lore text,
//! filesystem paths, URLs with query strings, tokens, keys, coupon envelopes)
//! become short opaque placeholders that carry enough information for
//! cross-entry correlation (a SHA-256 prefix and a byte length) and nothing
//! else.
//!
//! §18.5 secrets (device Ed25519 private key, RevenueCat restore/session
//! token, encrypted backup key material, optional coupon secret before
//! redemption) have *no* representation in this crate. There is no method
//! that accepts a secret value, because secrets must never appear in
//! diagnostics at all — not even as redacted placeholders.
//!
//! # Inspection before sharing
//!
//! A bundle has three human-readable renderers
//! ([`DiagnosticBundle::to_markdown`], [`DiagnosticBundle::to_text`],
//! [`DiagnosticBundle::to_json`]) so the user can read exactly what would be
//! shared before consenting to share it.

pub mod audit;
pub mod bundle;
pub mod quarantine;
pub mod redactor;

pub use audit::{AuditEvent, AuditEventBuilder, AuditSeverity};
pub use bundle::{DiagnosticBundle, Entry, RenderFormat};
pub use quarantine::{
    QuarantineAccessError, QuarantineLocator, QuarantineRecord, QuarantineSourceLabel,
};
pub use redactor::{
    ByteCount, DurationMillis, EntityId, ErrorCode, RedactedPath, RedactedText, RedactedUrl,
    Redactor, RedactionConfig, StateTransition, UserTextKind, Version,
};
