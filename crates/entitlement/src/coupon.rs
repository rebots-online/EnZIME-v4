//! §13.4 Coupon ledger.
//!
//! An offline coupon is a signed single-redemption offer: a coupon id, one or
//! more capability grants, the issuer key id, a validity window, a random
//! 256-bit coupon secret, and a signature. The signature covers everything
//! except itself; the signing domain is `enzime.coupon.v1\0`.
//!
//! Redemption derives a deterministic per-device receipt:
//!
//! ```text
//! receipt_sha256 = SHA256("enzime.coupon.receipt.v1\0"
//!                         || coupon_id || device_id || coupon_secret)
//! ```
//!
//! The local ledger stores redeemed receipts and rejects a repeat redemption
//! on the same device with `EntitlementError::CouponAlreadyRedeemed`. The
//! ledger is in-memory here; persistence is the storage layer's job. Cross-
//! device reconciliation is out of scope and, per A-09, can never revoke an
//! unrelated perpetual grant or any user data.

use std::collections::HashSet;

use domain::trust::{SignatureEnvelope, SignatureVerificationState};
use ports::entitlement::EntitlementError;
use ports::signature::SignatureProvider;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::artifact::Grant;

/// Signing domain string for offline coupons.
pub const COUPON_SIGNING_DOMAIN: &str = "enzime.coupon.v1\0";

/// Domain string fed into the SHA-256 receipt derivation. The trailing NUL
/// is part of the domain.
pub const COUPON_RECEIPT_DOMAIN: &str = "enzime.coupon.receipt.v1\0";

/// Required length of the per-coupon random secret, in bytes (256 bits).
pub const COUPON_SECRET_LEN: usize = 32;

/// Signed offline coupon. Wire format is CBOR via `encode_coupon` /
/// `decode_coupon`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Coupon {
    pub coupon_id: String,
    pub grants: Vec<Grant>,
    pub issuer_key_id: String,
    pub not_before_ms: i64,
    pub expires_at_ms: Option<i64>,
    #[serde(with = "serde_bytes")]
    pub coupon_secret: Vec<u8>,
    pub signature: crate::artifact::ArtifactSignature,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct UnsignedCoupon {
    coupon_id: String,
    grants: Vec<Grant>,
    issuer_key_id: String,
    not_before_ms: i64,
    expires_at_ms: Option<i64>,
    #[serde(with = "serde_bytes")]
    coupon_secret: Vec<u8>,
}

impl Coupon {
    pub fn validate_shape(&self) -> Result<(), EntitlementError> {
        if self.coupon_id.is_empty() {
            return Err(EntitlementError::Policy(
                "coupon_id must not be empty".to_string(),
            ));
        }
        if self.grants.is_empty() {
            return Err(EntitlementError::Policy(
                "coupon must carry at least one grant".to_string(),
            ));
        }
        if self.issuer_key_id.is_empty() {
            return Err(EntitlementError::Policy(
                "issuer_key_id must not be empty".to_string(),
            ));
        }
        if self.coupon_secret.len() != COUPON_SECRET_LEN {
            return Err(EntitlementError::Policy(format!(
                "coupon_secret must be {COUPON_SECRET_LEN} bytes, was {}",
                self.coupon_secret.len()
            )));
        }
        Ok(())
    }

    /// True iff `at_ms` falls inside `[not_before_ms, expires_at_ms]`. A
    /// coupon without `expires_at_ms` is treated as unbounded (issuers
    /// SHOULD set `expires_at_ms` for coupons).
    pub fn is_in_window_at(&self, at_ms: i64) -> bool {
        at_ms >= self.not_before_ms && self.expires_at_ms.map_or(true, |e| at_ms <= e)
    }
}

/// Canonical bytes for signing — every coupon field except `signature`.
pub fn canonical_bytes_for_coupon(coupon: &Coupon) -> Result<Vec<u8>, EntitlementError> {
    let unsigned = UnsignedCoupon {
        coupon_id: coupon.coupon_id.clone(),
        grants: coupon.grants.clone(),
        issuer_key_id: coupon.issuer_key_id.clone(),
        not_before_ms: coupon.not_before_ms,
        expires_at_ms: coupon.expires_at_ms,
        coupon_secret: coupon.coupon_secret.clone(),
    };
    let mut buf = Vec::new();
    ciborium::into_writer(&unsigned, &mut buf)
        .map_err(|e| EntitlementError::Policy(format!("cbor encode failed: {e}")))?;
    Ok(buf)
}

pub fn encode_coupon(coupon: &Coupon) -> Result<Vec<u8>, EntitlementError> {
    let mut buf = Vec::new();
    ciborium::into_writer(coupon, &mut buf)
        .map_err(|e| EntitlementError::Policy(format!("cbor encode failed: {e}")))?;
    Ok(buf)
}

pub fn decode_coupon(bytes: &[u8]) -> Result<Coupon, EntitlementError> {
    let coupon: Coupon = ciborium::from_reader(bytes)
        .map_err(|e| EntitlementError::Policy(format!("cbor decode failed: {e}")))?;
    coupon.validate_shape()?;
    Ok(coupon)
}

/// Generate a fresh random 256-bit coupon secret.
pub fn generate_coupon_secret() -> Result<Vec<u8>, EntitlementError> {
    let mut buf = vec![0u8; COUPON_SECRET_LEN];
    getrandom::getrandom(&mut buf)
        .map_err(|_| EntitlementError::Policy("coupon secret generation failed".to_string()))?;
    Ok(buf)
}

/// Deterministic per-device receipt. Defined verbatim by §13.4:
///
/// `SHA256("enzime.coupon.receipt.v1\0" || coupon_id || device_id || coupon_secret)`
///
/// Returned as lowercase hex.
pub fn derive_coupon_receipt(coupon_id: &str, device_id: &str, coupon_secret: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(COUPON_RECEIPT_DOMAIN.as_bytes());
    h.update(coupon_id.as_bytes());
    h.update(device_id.as_bytes());
    h.update(coupon_secret);
    hex::encode(h.finalize())
}

/// Sign `coupon` in place using `signer`. The signature field is overwritten
/// with the freshly produced envelope. As with artifacts, the coupon's
/// `issuer_key_id` must equal the `key_id` reported by the signer.
pub async fn sign_coupon(
    coupon: &mut Coupon,
    signer: &(impl SignatureProvider + Send + Sync),
) -> Result<(), EntitlementError> {
    coupon.validate_shape()?;
    let canonical = canonical_bytes_for_coupon(coupon)?;
    let envelope = signer
        .sign(COUPON_SIGNING_DOMAIN, &canonical)
        .await
        .map_err(|e| EntitlementError::Policy(format!("signer error: {e}")))?;
    if envelope.signed_domain != COUPON_SIGNING_DOMAIN {
        return Err(EntitlementError::Policy(format!(
            "signer did not honour the coupon signing domain (got '{}')",
            envelope.signed_domain
        )));
    }
    if coupon.issuer_key_id != envelope.key_id {
        return Err(EntitlementError::Policy(format!(
            "issuer_key_id '{}' does not match signing key '{}'",
            coupon.issuer_key_id, envelope.key_id
        )));
    }
    coupon.signature = envelope.into();
    Ok(())
}

/// Verify `coupon` against `verifier`. Checks signature, signing domain,
/// issuer/key binding, and the revocation set. Does NOT consult the validity
/// window — `redeem` does that.
pub async fn verify_coupon(
    coupon: &Coupon,
    verifier: &(impl SignatureProvider + Send + Sync),
    revoked_issuers: &HashSet<String>,
) -> Result<(), EntitlementError> {
    coupon.validate_shape()?;
    if coupon.issuer_key_id != coupon.signature.key_id {
        return Err(EntitlementError::InvalidArtifactSignature);
    }
    if coupon.signature.signed_domain != COUPON_SIGNING_DOMAIN {
        return Err(EntitlementError::InvalidArtifactSignature);
    }
    if revoked_issuers.contains(&coupon.issuer_key_id) {
        return Err(EntitlementError::IssuerRevoked);
    }
    let canonical = canonical_bytes_for_coupon(coupon)?;
    let envelope: SignatureEnvelope = coupon.signature.clone().into();
    let v = verifier
        .verify(COUPON_SIGNING_DOMAIN, &canonical, &envelope)
        .await
        .map_err(|_| EntitlementError::InvalidArtifactSignature)?;
    match v.state {
        SignatureVerificationState::Verified => Ok(()),
        SignatureVerificationState::RevokedKey => Err(EntitlementError::IssuerRevoked),
        SignatureVerificationState::Unsigned
        | SignatureVerificationState::Invalid
        | SignatureVerificationState::UnknownKey => Err(EntitlementError::InvalidArtifactSignature),
    }
}

/// Successful redemption result. The `receipt_sha256` is the per-device
/// receipt that proves this coupon was spent on this device; the grants are
/// the capability grants the coupon carried, ready to fold into the active
/// entitlement artifact by the application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CouponRedemption {
    pub coupon_id: String,
    pub receipt_sha256: String,
    pub grants: Vec<Grant>,
}

/// In-memory single-device coupon ledger. Persistence is the storage layer's
/// responsibility; `known_receipts` / `import_receipts` round-trip the
/// redeemed-receipt set so the storage layer can hydrate the ledger on
/// startup and persist new redemptions when they happen.
#[derive(Default)]
pub struct CouponLedger {
    redeemed: HashSet<String>,
}

impl CouponLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of receipts currently in the ledger.
    pub fn len(&self) -> usize {
        self.redeemed.len()
    }

    /// True iff the ledger currently holds no redeemed receipts.
    pub fn is_empty(&self) -> bool {
        self.redeemed.is_empty()
    }

    /// True iff this device has already redeemed a coupon producing this
    /// receipt.
    pub fn is_redeemed(&self, receipt_hex: &str) -> bool {
        self.redeemed.contains(receipt_hex)
    }

    /// Sorted snapshot of every receipt this device has redeemed. Use to
    /// persist the ledger across restarts.
    pub fn known_receipts(&self) -> Vec<String> {
        let mut v: Vec<String> = self.redeemed.iter().cloned().collect();
        v.sort();
        v
    }

    /// Re-hydrate the ledger from a persisted receipt set. Idempotent.
    pub fn import_receipts<I: IntoIterator<Item = String>>(&mut self, receipts: I) {
        for r in receipts {
            self.redeemed.insert(r);
        }
    }

    /// Attempt to redeem `coupon` on this device (`device_id`) at wall-clock
    /// time `at_ms`. Verifies the coupon signature, enforces the validity
    /// window, derives the deterministic receipt, and refuses to record the
    /// same receipt twice on this device.
    pub async fn redeem(
        &mut self,
        coupon: &Coupon,
        device_id: &str,
        verifier: &(impl SignatureProvider + Send + Sync),
        revoked_issuers: &HashSet<String>,
        at_ms: i64,
    ) -> Result<CouponRedemption, EntitlementError> {
        // 1. Signature / domain / revocation.
        verify_coupon(coupon, verifier, revoked_issuers).await?;
        // 2. Validity window.
        if at_ms < coupon.not_before_ms {
            return Err(EntitlementError::Policy(format!(
                "coupon not yet valid: not_before_ms={}, now={}",
                coupon.not_before_ms, at_ms
            )));
        }
        if let Some(exp) = coupon.expires_at_ms {
            if at_ms > exp {
                return Err(EntitlementError::ArtifactExpired);
            }
        }
        // 3. Per-device receipt.
        let receipt = derive_coupon_receipt(&coupon.coupon_id, device_id, &coupon.coupon_secret);
        // 4. Single-device idempotency.
        if self.redeemed.contains(&receipt) {
            return Err(EntitlementError::CouponAlreadyRedeemed);
        }
        // 5. Record and return.
        self.redeemed.insert(receipt.clone());
        Ok(CouponRedemption {
            coupon_id: coupon.coupon_id.clone(),
            receipt_sha256: receipt,
            grants: coupon.grants.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::{ArtifactSignature, GrantKind};

    // Re-use the same HMAC test signer pattern as artifact::tests. Duplicated
    // here to keep the modules independently testable; the body is identical
    // to the one in artifact::tests.
    mod hmac_signer {
        use async_trait::async_trait;
        use domain::trust::{SignatureEnvelope, SignatureVerificationState};
        use ports::signature::{SignatureError, SignatureProvider};
        use sha2::{Digest, Sha256};
        use std::collections::HashSet;

        pub struct HmacTestSigner {
            pub key_id: String,
            pub key: Vec<u8>,
            pub trusted_key_ids: HashSet<String>,
        }

        impl HmacTestSigner {
            pub fn new(key_id: &str, key: &[u8]) -> Self {
                let mut trusted = HashSet::new();
                trusted.insert(key_id.to_string());
                Self {
                    key_id: key_id.to_string(),
                    key: key.to_vec(),
                    trusted_key_ids: trusted,
                }
            }
            fn mac(&self, domain: &str, payload: &[u8]) -> [u8; 32] {
                const BLOCK: usize = 64;
                let mut k = [0u8; BLOCK];
                if self.key.len() > BLOCK {
                    let mut h = Sha256::new();
                    h.update(&self.key);
                    let d = h.finalize();
                    k[..32].copy_from_slice(&d);
                } else {
                    k[..self.key.len()].copy_from_slice(&self.key);
                }
                let mut ipad = [0u8; BLOCK];
                let mut opad = [0u8; BLOCK];
                for i in 0..BLOCK {
                    ipad[i] = k[i] ^ 0x36;
                    opad[i] = k[i] ^ 0x5c;
                }
                let mut inner = Sha256::new();
                inner.update(ipad);
                inner.update(domain.as_bytes());
                inner.update(payload);
                let inner_h = inner.finalize();
                let mut outer = Sha256::new();
                outer.update(opad);
                outer.update(&inner_h);
                let outer_h = outer.finalize();
                let mut out = [0u8; 32];
                out.copy_from_slice(&outer_h);
                out
            }
        }

        #[async_trait]
        impl SignatureProvider for HmacTestSigner {
            async fn sign(
                &self,
                domain: &str,
                canonical_payload: &[u8],
            ) -> Result<SignatureEnvelope, SignatureError> {
                Ok(SignatureEnvelope {
                    algorithm: "test-hmac-sha256".to_string(),
                    key_id: self.key_id.clone(),
                    signed_domain: domain.to_string(),
                    value: self.mac(domain, canonical_payload).to_vec(),
                })
            }
            async fn verify(
                &self,
                domain: &str,
                canonical_payload: &[u8],
                envelope: &SignatureEnvelope,
            ) -> Result<domain::trust::SignatureVerification, SignatureError> {
                use domain::trust::SignatureVerification;
                if envelope.signed_domain != domain {
                    return Ok(SignatureVerification {
                        state: SignatureVerificationState::Invalid,
                        key_id: envelope.key_id.clone(),
                        signer_label: None,
                        explanation: "signed_domain mismatch".to_string(),
                    });
                }
                if !self.trusted_key_ids.contains(&envelope.key_id) {
                    return Ok(SignatureVerification {
                        state: SignatureVerificationState::UnknownKey,
                        key_id: envelope.key_id.clone(),
                        signer_label: None,
                        explanation: "unknown key id".to_string(),
                    });
                }
                let expected = self.mac(domain, canonical_payload);
                if envelope.value.as_slice() == expected.as_slice() {
                    Ok(SignatureVerification {
                        state: SignatureVerificationState::Verified,
                        key_id: envelope.key_id.clone(),
                        signer_label: None,
                        explanation: "verified".to_string(),
                    })
                } else {
                    Ok(SignatureVerification {
                        state: SignatureVerificationState::Invalid,
                        key_id: envelope.key_id.clone(),
                        signer_label: None,
                        explanation: "value mismatch".to_string(),
                    })
                }
            }
        }
    }

    fn fresh_coupon() -> Coupon {
        Coupon {
            coupon_id: "promo-001".to_string(),
            grants: vec![Grant {
                capability_key: "annotations.rich_media".to_string(),
                grant_kind: GrantKind::Term,
                quantity: None,
                valid_until_ms: Some(50_000),
            }],
            issuer_key_id: "issuer-7".to_string(),
            not_before_ms: 1_000,
            expires_at_ms: Some(20_000),
            coupon_secret: generate_coupon_secret().unwrap(),
            signature: ArtifactSignature {
                algorithm: String::new(),
                key_id: String::new(),
                signed_domain: String::new(),
                value: Vec::new(),
            },
        }
    }

    #[test]
    fn receipt_is_deterministic_and_device_specific() {
        let secret = vec![0xabu8; COUPON_SECRET_LEN];
        let a = derive_coupon_receipt("promo-001", "device-a", &secret);
        let a2 = derive_coupon_receipt("promo-001", "device-a", &secret);
        let b = derive_coupon_receipt("promo-001", "device-b", &secret);
        let c = derive_coupon_receipt("promo-002", "device-a", &secret);
        assert_eq!(a, a2, "receipt must be deterministic for same inputs");
        assert_ne!(a, b, "receipt must depend on device_id");
        assert_ne!(a, c, "receipt must depend on coupon_id");
        assert_eq!(a.len(), 64, "receipt must be SHA-256 hex (64 chars)");
    }

    #[test]
    fn receipt_matches_known_vector() {
        // Independent SHA-256 of the spec byte sequence, used as the
        // oracle. Computed inline so the test is self-contained.
        let secret = vec![0x01u8; COUPON_SECRET_LEN];
        let mut h = Sha256::new();
        h.update(COUPON_RECEIPT_DOMAIN.as_bytes());
        h.update(b"promo-001");
        h.update(b"device-a");
        h.update(&secret);
        let expected = hex::encode(h.finalize());
        let got = derive_coupon_receipt("promo-001", "device-a", &secret);
        assert_eq!(expected, got);
    }

    #[tokio::test]
    async fn redeem_succeeds_for_valid_coupon() {
        let signer = hmac_signer::HmacTestSigner::new("issuer-7", b"key");
        let mut coupon = fresh_coupon();
        sign_coupon(&mut coupon, &signer).await.unwrap();
        let mut ledger = CouponLedger::new();
        let revoked = HashSet::new();
        let r = ledger
            .redeem(&coupon, "device-a", &signer, &revoked, 5_000)
            .await
            .unwrap();
        assert_eq!(r.coupon_id, "promo-001");
        assert_eq!(r.grants.len(), 1);
        assert_eq!(ledger.len(), 1);
        assert!(ledger.is_redeemed(&r.receipt_sha256));
    }

    #[tokio::test]
    async fn redeem_rejects_repeat_on_same_device() {
        let signer = hmac_signer::HmacTestSigner::new("issuer-7", b"key");
        let mut coupon = fresh_coupon();
        sign_coupon(&mut coupon, &signer).await.unwrap();
        let mut ledger = CouponLedger::new();
        let revoked = HashSet::new();
        ledger
            .redeem(&coupon, "device-a", &signer, &revoked, 5_000)
            .await
            .unwrap();
        let err = ledger
            .redeem(&coupon, "device-a", &signer, &revoked, 6_000)
            .await
            .unwrap_err();
        assert!(matches!(err, EntitlementError::CouponAlreadyRedeemed));
    }

    #[tokio::test]
    async fn redeem_allows_same_coupon_on_different_device() {
        let signer = hmac_signer::HmacTestSigner::new("issuer-7", b"key");
        let mut coupon = fresh_coupon();
        sign_coupon(&mut coupon, &signer).await.unwrap();
        let mut ledger = CouponLedger::new();
        let revoked = HashSet::new();
        let r1 = ledger
            .redeem(&coupon, "device-a", &signer, &revoked, 5_000)
            .await
            .unwrap();
        // Same ledger tracks this device only; device-b is a different
        // device and the per-device receipt is different.
        let r2 = ledger
            .redeem(&coupon, "device-b", &signer, &revoked, 5_000)
            .await
            .unwrap();
        assert_ne!(r1.receipt_sha256, r2.receipt_sha256);
        // The single-device ledger records both receipts because they are
        // different strings; cross-device reuse detection is a server-side
        // concern that this crate cannot (and per A-09 must not) block.
        assert_eq!(ledger.len(), 2);
    }

    #[tokio::test]
    async fn redeem_rejects_tampered_grant() {
        let signer = hmac_signer::HmacTestSigner::new("issuer-7", b"key");
        let mut coupon = fresh_coupon();
        sign_coupon(&mut coupon, &signer).await.unwrap();
        coupon.grants[0].capability_key = "ask.local_inference".to_string();
        let mut ledger = CouponLedger::new();
        let revoked = HashSet::new();
        let err = ledger
            .redeem(&coupon, "device-a", &signer, &revoked, 5_000)
            .await
            .unwrap_err();
        assert!(matches!(err, EntitlementError::InvalidArtifactSignature));
    }

    #[tokio::test]
    async fn redeem_rejects_expired_coupon() {
        let signer = hmac_signer::HmacTestSigner::new("issuer-7", b"key");
        let mut coupon = fresh_coupon();
        sign_coupon(&mut coupon, &signer).await.unwrap();
        let mut ledger = CouponLedger::new();
        let revoked = HashSet::new();
        let err = ledger
            .redeem(&coupon, "device-a", &signer, &revoked, 30_000)
            .await
            .unwrap_err();
        assert!(matches!(err, EntitlementError::ArtifactExpired));
    }

    #[tokio::test]
    async fn redeem_rejects_not_yet_valid_coupon() {
        let signer = hmac_signer::HmacTestSigner::new("issuer-7", b"key");
        let mut coupon = fresh_coupon();
        sign_coupon(&mut coupon, &signer).await.unwrap();
        let mut ledger = CouponLedger::new();
        let revoked = HashSet::new();
        let err = ledger
            .redeem(&coupon, "device-a", &signer, &revoked, 500)
            .await
            .unwrap_err();
        assert!(matches!(err, EntitlementError::Policy(_)));
    }

    #[tokio::test]
    async fn ledger_persistence_roundtrip() {
        let signer = hmac_signer::HmacTestSigner::new("issuer-7", b"key");
        let mut coupon = fresh_coupon();
        sign_coupon(&mut coupon, &signer).await.unwrap();
        let mut ledger = CouponLedger::new();
        let revoked = HashSet::new();
        let r = ledger
            .redeem(&coupon, "device-a", &signer, &revoked, 5_000)
            .await
            .unwrap();
        let snapshot = ledger.known_receipts();
        let mut restored = CouponLedger::new();
        restored.import_receipts(snapshot);
        let err = restored
            .redeem(&coupon, "device-a", &signer, &revoked, 6_000)
            .await
            .unwrap_err();
        assert!(matches!(err, EntitlementError::CouponAlreadyRedeemed));
        let _ = r;
    }
}
