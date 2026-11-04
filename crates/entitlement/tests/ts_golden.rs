//! Cross-language contract: an artifact issued by the TypeScript billing
//! function (services/billing) must decode and verify here byte-for-byte.
//! Regenerate the fixture with `node services/billing/scripts/golden.ts`.

use std::collections::HashSet;

use ed25519_dalek::VerifyingKey;
use entitlement::{
    canonical_bytes_for_artifact, decode_artifact, encode_artifact, verify_artifact, GrantKind,
    SubjectKind,
};
use sidecar::provider::{Ed25519Keystore, Ed25519Provider};

const FIXTURE: &str = include_str!("fixtures/ts_golden.txt");

fn fixture() -> (String, VerifyingKey, String, Vec<u8>) {
    let mut lines = FIXTURE.lines();
    let key_id = lines.next().unwrap().to_string();
    let pk: [u8; 32] = hex::decode(lines.next().unwrap()).unwrap().try_into().unwrap();
    let device = lines.next().unwrap().to_string();
    let artifact = hex::decode(lines.next().unwrap()).unwrap();
    (key_id, VerifyingKey::from_bytes(&pk).unwrap(), device, artifact)
}

#[tokio::test]
async fn typescript_issued_artifact_verifies() {
    let (key_id, vk, device, bytes) = fixture();
    let artifact = decode_artifact(&bytes).expect("decodes");
    assert_eq!(artifact.issuer_key_id, key_id);
    assert_eq!(artifact.subject.kind, SubjectKind::Device);
    assert_eq!(artifact.subject.subject_id, device);
    assert_eq!(artifact.expires_at_ms, None, "lifetime purchase makes artifact perpetual");
    assert!(artifact
        .grants
        .iter()
        .any(|g| g.capability_key == "models.premium_catalog" && g.grant_kind == GrantKind::Term));
    assert!(artifact
        .grants
        .iter()
        .any(|g| g.capability_key == "annotations.rich_media" && g.grant_kind == GrantKind::Perpetual));

    // Byte-identical re-encoding proves the encoders agree, not just the signature.
    assert_eq!(encode_artifact(&artifact).unwrap(), bytes);

    let mut ks = Ed25519Keystore::default();
    ks.add_verifying(key_id, vk, Some("golden".into()));
    let provider = Ed25519Provider::new(ks);
    verify_artifact(&artifact, &provider, &HashSet::new())
        .await
        .expect("signature verifies");
}

#[tokio::test]
async fn tampered_artifact_is_rejected() {
    let (key_id, vk, _device, bytes) = fixture();
    let mut artifact = decode_artifact(&bytes).unwrap();
    artifact.subject.subject_id = "someone-else".into();
    assert_ne!(canonical_bytes_for_artifact(&artifact).unwrap().len(), 0);
    let mut ks = Ed25519Keystore::default();
    ks.add_verifying(key_id, vk, None);
    let provider = Ed25519Provider::new(ks);
    assert!(verify_artifact(&artifact, &provider, &HashSet::new()).await.is_err());
}
