#![forbid(unsafe_code)]

//! Ring-2 entitlement engine: signed offline artifact codec, coupon ledger,
//! and offline capability policy. Implements ARCHITECTURE.md §13.
//!
//! # Roles
//!
//! - The **public contract surface** — capability keys, `CapabilityDecision`,
//!   `CapabilityPolicy` trait, `EntitlementSource` trait, `EntitlementError`,
//!   provider event structs — is owned by `domain::entitlement` and
//!   `ports::entitlement`. This crate does not redefine them; it imports and
//!   implements them.
//! - This crate owns the **signed offline artifact** (§13.3): CBOR codec,
//!   canonical signing/verifying bytes, sign-and-verify flow over the
//!   `SignatureProvider` port.
//! - This crate owns the **coupon ledger** (§13.4): single-redemption signed
//!   coupon, deterministic SHA-256 receipt derivation, in-memory ledger with
//!   persistence hooks.
//! - This crate owns the **offline capability policy** (§13.5): a real
//!   `CapabilityPolicy` implementation that decides from the verified artifact
//!   only, with A-09 (commerce cannot revoke ownership) as the binding rule.
//!
//! # Provider boundary
//!
//! This crate imports no RevenueCat type and never queries the network. The
//! release `EntitlementSource` lives in a separate adapter crate. Everything
//! here verifies offline from bytes plus the `SignatureProvider` port.

pub mod artifact;
pub mod coupon;
pub mod policy;

pub use artifact::{
    canonical_bytes_for_artifact, decode_artifact, encode_artifact, generate_nonce,
    sign_artifact, verify_artifact, ArtifactSignature, EntitlementArtifact, Grant, GrantKind,
    ProviderReceipt, Subject, SubjectKind, ARTIFACT_SCHEMA, ENTITLEMENT_SIGNING_DOMAIN,
    MEDIA_TYPE, NONCE_LEN,
};
pub use coupon::{
    canonical_bytes_for_coupon, decode_coupon, derive_coupon_receipt, encode_coupon,
    generate_coupon_secret, sign_coupon, verify_coupon, Coupon, CouponLedger, CouponRedemption,
    COUPON_RECEIPT_DOMAIN, COUPON_SECRET_LEN, COUPON_SIGNING_DOMAIN,
};
pub use policy::{OfflineCapabilityPolicy, DEFAULT_GRACE_WINDOW_MS, KNOWN_CAPABILITY_KEYS};
