#![forbid(unsafe_code)]

//! §9.3 ink and voice annotation assets: chunked upload with
//! digest-on-commit.
//!
//! A partial asset is NEVER referenced by an annotation. The upload
//! session is a pure value held by the caller; the only side effect on
//! the annotation→asset link happens inside
//! [`AnnotationAssetService::commit`], which delegates to an injected
//! [`CommittedAssetSink`]. The sink performs "write object + insert
//! `annotation_assets` row" as one storage transaction — the §6.3
//! steps-9-and-10 analogue for asset artefacts. On any failure —
//! interruption, cancellation, size-limit breach, process death — the
//! session is dropped and no annotation ever references the partial
//! bytes.
//!
//! ## Why digest-on-commit
//!
//! The SHA-256 of the asset bytes is computed by this service from the
//! fully-assembled buffer at commit time and passed to the sink as the
//! `declared_sha256`. The sink re-hashes the bytes it receives and
//! rejects on mismatch (defence in depth), then writes the object under
//! that content-addressed key and inserts the metadata row. A partial
//! digest computed mid-stream is never exposed.
//!
//! ## Why the session is a value, not a registry
//!
//! Resumable-upload state is owned by the host (the layer with the
//! durable session table). This service transforms session values
//! deterministically; it holds no maps, no locks, no background tasks.
//! A process death between two [`AnnotationAssetService::append_chunk`]
//! calls leaves nothing for this service to clean up — the host's
//! session table is the authoritative resume point, and an unfinished
//! session there is not an annotation reference.
//!
//! ## Relationship to the §9.3 `AnnotationService::attach_asset` contract
//!
//! [`AnnotationAssetService::attach`] is the single-shot form of that
//! contract: validate, hash, delegate to the sink. The chunked path
//! (`begin` → `append_chunk` → `commit`) is the resumable form for
//! larger voice and ink payloads. Both paths produce the same
//! `AnnotationAssetRecord` for the same bytes (see
//! `chunked_commit_and_attach_produce_same_digest`).

use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use domain::annotation::{AnnotationAssetInput, AnnotationAssetRecord, AnnotationError};
use sha2::{Digest, Sha256};

/// Per-upload declaration. `max_byte_length` is the inclusive ceiling the
/// host has decided for this asset kind (ink stroke, voice clip). The
/// service refuses any chunk that would push the accumulated length
/// above it; nothing about the limit is hardcoded here.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetUploadInit {
    pub upload_id: String,
    pub asset_kind: String,
    pub mime_type: String,
    pub duration_ms: Option<u64>,
    pub max_byte_length: u64,
}

/// Pure state of one upload. Held by the caller between chunks; this
/// service holds no session map.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetUploadSession {
    pub upload_id: String,
    pub asset_kind: String,
    pub mime_type: String,
    pub duration_ms: Option<u64>,
    pub max_byte_length: u64,
    pub accumulated: Vec<u8>,
    pub state: UploadState,
}

/// Lifecycle of an upload session.
///
/// ```text
///   begin() ──► Open ──append_chunk()──► Open ──commit()──► Committed
///                    │                                         │
///                    └─append_chunk() breach / abort()──────► Aborted
/// ```
///
/// `Committed` and `Aborted` are terminal. The service refuses any
/// further `append_chunk` or `commit` call on a non-`Open` session —
/// the only legal continuation is to drop the value and start a fresh
/// `begin()` with a new `upload_id`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UploadState {
    Open,
    Committed,
    Aborted,
}

/// I-10(b): swap when the committed-asset-sink task lands. The trait
/// encodes exactly the §9.3 + §6.3 contract: the storage layer writes
/// the asset's bytes to the content-addressed object store, recomputes
/// the SHA-256 from the received bytes, rejects on mismatch with
/// `declared_sha256`, and inserts the `annotation_assets` row linking
/// the annotation to the new asset — all in one storage transaction.
/// Until that lands in `ports`, the application crate owns the trait;
/// the host supplies a real implementation backed by SQLite + the
/// object store. The trait is shaped so the storage side can reject
/// without ever producing a partial link, which is the operational form
/// of "a partial asset is never referenced by an annotation".
#[async_trait]
pub trait CommittedAssetSink: Send + Sync {
    /// Atomically: (a) recompute SHA-256 of `bytes` and reject on
    /// mismatch with `declared_sha256`, (b) write `bytes` to the
    /// content-addressed object store under `declared_sha256`,
    /// (c) insert the `annotation_assets` row linking `annotation_id`
    /// to the new asset, (d) return the resulting
    /// [`AnnotationAssetRecord`]. Steps (a)–(c) are one storage
    /// transaction — either the annotation gets a fully-verified,
    /// durably-stored asset, or it gets nothing.
    async fn commit_linked(
        &self,
        annotation_id: &str,
        asset_kind: &str,
        declared_sha256: &str,
        mime_type: &str,
        byte_length: u64,
        duration_ms: Option<u64>,
        bytes: Bytes,
    ) -> Result<AnnotationAssetRecord, AnnotationError>;
}

/// Orchestrates §9.3 asset upload. Stateless across sessions; safe to
/// share by reference (the only state is the injected sink).
pub struct AnnotationAssetService {
    sink: Arc<dyn CommittedAssetSink>,
}

impl AnnotationAssetService {
    pub fn new(sink: Arc<dyn CommittedAssetSink>) -> Self {
        Self { sink }
    }

    /// Allocate a fresh open session. No I/O, no side effects, no
    /// annotation reference.
    pub fn begin(&self, init: AssetUploadInit) -> Result<AssetUploadSession, AnnotationError> {
        if init.max_byte_length == 0 {
            return Err(AnnotationError::InvalidAsset(
                "max_byte_length must be positive".to_string(),
            ));
        }
        validate_asset_kind(&init.asset_kind)?;
        validate_mime_type(&init.mime_type)?;
        Ok(AssetUploadSession {
            upload_id: init.upload_id,
            asset_kind: init.asset_kind,
            mime_type: init.mime_type,
            duration_ms: init.duration_ms,
            max_byte_length: init.max_byte_length,
            accumulated: Vec::new(),
            state: UploadState::Open,
        })
    }

    /// Append a chunk to an open session. Consumes the session and
    /// returns the updated one on success. On size-limit breach,
    /// consumes the session and returns
    /// [`AnnotationError::InvalidAsset`] — the bytes are gone and no
    /// annotation reference ever existed.
    pub fn append_chunk(
        &self,
        session: AssetUploadSession,
        chunk: Bytes,
    ) -> Result<AssetUploadSession, AnnotationError> {
        if session.state != UploadState::Open {
            return Err(AnnotationError::InvalidAsset(format!(
                "append_chunk on non-open session: state={:?}",
                session.state
            )));
        }
        let chunk_len: u64 = u64::try_from(chunk.len())
            .map_err(|_| AnnotationError::InvalidAsset("chunk length overflow".to_string()))?;
        let current_len: u64 = u64::try_from(session.accumulated.len())
            .map_err(|_| AnnotationError::InvalidAsset("accumulated length overflow".to_string()))?;
        let new_len = current_len
            .checked_add(chunk_len)
            .ok_or_else(|| AnnotationError::InvalidAsset("byte length overflow".to_string()))?;
        if new_len > session.max_byte_length {
            return Err(AnnotationError::InvalidAsset(format!(
                "size-limit breach: {} + {} > {}",
                session.accumulated.len(),
                chunk.len(),
                session.max_byte_length
            )));
        }

        let mut next = session;
        next.accumulated.extend_from_slice(&chunk);
        Ok(next)
    }

    /// Commit the session: compute SHA-256 over the assembled bytes,
    /// delegate to the sink for the atomic write+link. Consumes the
    /// session. On any sink failure the session is consumed and the
    /// annotation remains unreferenced — the caller must start a fresh
    /// `begin()` with a new `upload_id`.
    pub async fn commit(
        &self,
        session: AssetUploadSession,
        annotation_id: &str,
    ) -> Result<AnnotationAssetRecord, AnnotationError> {
        if session.state != UploadState::Open {
            return Err(AnnotationError::InvalidAsset(format!(
                "commit on non-open session: state={:?}",
                session.state
            )));
        }
        if session.accumulated.is_empty() {
            return Err(AnnotationError::InvalidAsset("empty asset".to_string()));
        }
        let mut hasher = Sha256::new();
        hasher.update(&session.accumulated);
        let digest = hex::encode(hasher.finalize());
        // Safe cast: append_chunk enforces accumulated.len() <= max_byte_length <= u64::MAX.
        let byte_length: u64 = session.accumulated.len() as u64;

        self.sink
            .commit_linked(
                annotation_id,
                &session.asset_kind,
                &digest,
                &session.mime_type,
                byte_length,
                session.duration_ms,
                Bytes::from(session.accumulated),
            )
            .await
    }

    /// Abort: drop the session. No side effects to undo because the
    /// only side-effecting step is [`Self::commit`], which was never
    /// reached. The explicit `abort` exists so callers can document
    /// intent at the call site; dropping the value directly has the
    /// identical effect.
    pub fn abort(&self, _session: AssetUploadSession) {}

    /// Single-shot convenience implementing the §9.3
    /// `AnnotationService::attach_asset` contract: validate, hash,
    /// delegate to the sink. Internally equivalent to
    /// `begin → append_chunk → commit`, but does not surface the
    /// session value to the caller. Produces the same digest for the
    /// same bytes as the chunked path.
    pub async fn attach(
        &self,
        annotation_id: &str,
        asset: AnnotationAssetInput,
    ) -> Result<AnnotationAssetRecord, AnnotationError> {
        validate_asset_kind(&asset.asset_kind)?;
        validate_mime_type(&asset.mime_type)?;
        if asset.bytes.is_empty() {
            return Err(AnnotationError::InvalidAsset("empty asset".to_string()));
        }
        let byte_length: u64 = u64::try_from(asset.bytes.len())
            .map_err(|_| AnnotationError::InvalidAsset("byte length overflow".to_string()))?;
        let mut hasher = Sha256::new();
        hasher.update(&asset.bytes);
        let digest = hex::encode(hasher.finalize());

        self.sink
            .commit_linked(
                annotation_id,
                &asset.asset_kind,
                &digest,
                &asset.mime_type,
                byte_length,
                asset.duration_ms,
                asset.bytes,
            )
            .await
    }
}

fn validate_asset_kind(kind: &str) -> Result<(), AnnotationError> {
    // §9.3 names ink and voice as the asset kinds backed by this
    // service. The string is preserved verbatim — the sink stores
    // whatever the host declared — but we refuse empty so a typo
    // cannot silently produce an untyped asset row.
    if kind.is_empty() {
        return Err(AnnotationError::InvalidAsset(
            "empty asset_kind".to_string(),
        ));
    }
    Ok(())
}

fn validate_mime_type(mime: &str) -> Result<(), AnnotationError> {
    if mime.is_empty() {
        return Err(AnnotationError::InvalidAsset(
            "empty mime_type".to_string(),
        ));
    }
    // Minimal structural check: must contain a type/subtype separator.
    // We do not maintain a MIME allowlist here — that is a sink/host
    // responsibility because new asset kinds may add new types.
    if !mime.contains('/') {
        return Err(AnnotationError::InvalidAsset(format!(
            "malformed mime_type: {}",
            mime
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    //! Real assertions over the §9.3 chunked-upload state machine and
    //! the digest-on-commit invariant. No mocks — the in-test
    //! [`CapturingSink`] is a full, deterministic implementation of
    //! [`CommittedAssetSink`] that rehashes bytes and records every
    //! call so tests can prove side-effects did or did not happen.
    use super::*;

    // ----- deterministic in-test sink ----------------------------------

    #[derive(Clone, Debug)]
    #[allow(dead_code)] // fields retained for diagnostic introspection in tests
    struct CapturedCall {
        annotation_id: String,
        asset_kind: String,
        declared_sha256: String,
        mime_type: String,
        byte_length: u64,
        duration_ms: Option<u64>,
        bytes: Bytes,
    }

    enum SinkOutcome {
        AcceptAndRehash,
        RejectShaMismatch,
        FailStorage(&'static str),
    }

    struct CapturingSink {
        observed: std::sync::Mutex<Vec<CapturedCall>>,
        next_id: std::sync::Mutex<u64>,
        outcome: SinkOutcome,
    }

    impl CapturingSink {
        fn new(outcome: SinkOutcome) -> Self {
            Self {
                observed: std::sync::Mutex::new(Vec::new()),
                next_id: std::sync::Mutex::new(0),
                outcome,
            }
        }
    }

    #[async_trait]
    impl CommittedAssetSink for CapturingSink {
        async fn commit_linked(
            &self,
            annotation_id: &str,
            asset_kind: &str,
            declared_sha256: &str,
            mime_type: &str,
            byte_length: u64,
            duration_ms: Option<u64>,
            bytes: Bytes,
        ) -> Result<AnnotationAssetRecord, AnnotationError> {
            // Always capture so tests can assert side effects (or lack
            // thereof) even when the sink rejects.
            self.observed.lock().unwrap().push(CapturedCall {
                annotation_id: annotation_id.to_string(),
                asset_kind: asset_kind.to_string(),
                declared_sha256: declared_sha256.to_string(),
                mime_type: mime_type.to_string(),
                byte_length,
                duration_ms,
                bytes: bytes.clone(),
            });
            match self.outcome {
                SinkOutcome::AcceptAndRehash => {
                    let mut h = Sha256::new();
                    h.update(&bytes);
                    let recomputed = hex::encode(h.finalize());
                    if recomputed != declared_sha256 {
                        return Err(AnnotationError::InvalidAsset(format!(
                            "sha mismatch: declared={} recomputed={}",
                            declared_sha256, recomputed
                        )));
                    }
                    if bytes.len() as u64 != byte_length {
                        return Err(AnnotationError::InvalidAsset(format!(
                            "length mismatch: declared={} actual={}",
                            byte_length,
                            bytes.len()
                        )));
                    }
                    let mut counter = self.next_id.lock().unwrap();
                    *counter += 1;
                    let asset_id = format!("aaid-{}", *counter);
                    Ok(AnnotationAssetRecord {
                        annotation_asset_id: asset_id,
                        annotation_id: annotation_id.to_string(),
                        asset_kind: asset_kind.to_string(),
                        object_sha256: declared_sha256.to_string(),
                        mime_type: mime_type.to_string(),
                        byte_length,
                        duration_ms,
                    })
                }
                SinkOutcome::RejectShaMismatch => Err(AnnotationError::InvalidAsset(
                    "sha mismatch (simulated)".to_string(),
                )),
                SinkOutcome::FailStorage(msg) => {
                    Err(AnnotationError::Storage(msg.to_string()))
                }
            }
        }
    }

    fn service(outcome: SinkOutcome) -> AnnotationAssetService {
        AnnotationAssetService::new(Arc::new(CapturingSink::new(outcome)))
    }

    fn service_with_capturing(
        outcome: SinkOutcome,
    ) -> (AnnotationAssetService, Arc<CapturingSink>) {
        let sink = Arc::new(CapturingSink::new(outcome));
        let svc = AnnotationAssetService::new(sink.clone());
        (svc, sink)
    }

    fn init(upload_id: &str, max: u64) -> AssetUploadInit {
        AssetUploadInit {
            upload_id: upload_id.to_string(),
            asset_kind: "ink".to_string(),
            mime_type: "image/svg+xml".to_string(),
            duration_ms: None,
            max_byte_length: max,
        }
    }

    fn sha256_hex(data: &[u8]) -> String {
        let mut h = Sha256::new();
        h.update(data);
        hex::encode(h.finalize())
    }

    // ----- begin -------------------------------------------------------

    #[test]
    fn begin_rejects_zero_max() {
        let svc = service(SinkOutcome::AcceptAndRehash);
        let err = svc.begin(init("u1", 0)).unwrap_err();
        assert!(matches!(err, AnnotationError::InvalidAsset(_)));
    }

    #[test]
    fn begin_rejects_empty_asset_kind() {
        let svc = service(SinkOutcome::AcceptAndRehash);
        let mut bad = init("u1", 1024);
        bad.asset_kind = String::new();
        let err = svc.begin(bad).unwrap_err();
        assert!(matches!(err, AnnotationError::InvalidAsset(_)));
    }

    #[test]
    fn begin_rejects_malformed_mime() {
        let svc = service(SinkOutcome::AcceptAndRehash);
        let mut bad = init("u1", 1024);
        bad.mime_type = "audio".to_string();
        let err = svc.begin(bad).unwrap_err();
        assert!(matches!(err, AnnotationError::InvalidAsset(_)));
    }

    #[test]
    fn begins_in_open_state_with_empty_buffer() {
        let svc = service(SinkOutcome::AcceptAndRehash);
        let s = svc.begin(init("u1", 1024)).unwrap();
        assert_eq!(s.state, UploadState::Open);
        assert!(s.accumulated.is_empty());
        assert_eq!(s.upload_id, "u1");
        assert_eq!(s.max_byte_length, 1024);
    }

    // ----- append_chunk ------------------------------------------------

    #[test]
    fn append_chunk_accumulates_under_limit() {
        let svc = service(SinkOutcome::AcceptAndRehash);
        let s = svc.begin(init("u1", 8)).unwrap();
        let s = svc.append_chunk(s, Bytes::from_static(b"abcd")).unwrap();
        assert_eq!(s.accumulated, b"abcd");
        assert_eq!(s.state, UploadState::Open);
        let s = svc.append_chunk(s, Bytes::from_static(b"efgh")).unwrap();
        assert_eq!(s.accumulated, b"abcdefgh");
        assert_eq!(s.state, UploadState::Open);
    }

    #[test]
    fn append_chunk_rejects_breach_and_consumes_session() {
        let (svc, sink) = service_with_capturing(SinkOutcome::AcceptAndRehash);
        let s = svc.begin(init("u1", 4)).unwrap();
        let err = svc.append_chunk(s, Bytes::from_static(b"abcde")).unwrap_err();
        assert!(matches!(err, AnnotationError::InvalidAsset(_)));
        // No sink call ever occurred: the size-limit breach left no
        // path to commit.
        assert!(sink.observed.lock().unwrap().is_empty());
    }

    #[test]
    fn append_chunk_accepts_exact_limit_then_rejects_one_more() {
        let svc = service(SinkOutcome::AcceptAndRehash);
        let s = svc.begin(init("u1", 4)).unwrap();
        let s = svc.append_chunk(s, Bytes::from_static(b"abcd")).unwrap();
        assert_eq!(s.accumulated.len(), 4);
        let err = svc.append_chunk(s, Bytes::from_static(b"e")).unwrap_err();
        assert!(matches!(err, AnnotationError::InvalidAsset(_)));
    }

    #[test]
    fn append_chunk_on_terminal_session_is_rejected() {
        // We cannot easily reach Committed without going async, but we
        // can construct an Aborted-style terminal by hand to prove the
        // state guard fires.
        let svc = service(SinkOutcome::AcceptAndRehash);
        let s = AssetUploadSession {
            upload_id: "u1".to_string(),
            asset_kind: "ink".to_string(),
            mime_type: "image/svg+xml".to_string(),
            duration_ms: None,
            max_byte_length: 16,
            accumulated: vec![0u8; 4],
            state: UploadState::Committed,
        };
        let err = svc.append_chunk(s, Bytes::from_static(b"x")).unwrap_err();
        assert!(matches!(err, AnnotationError::InvalidAsset(_)));
    }

    // ----- commit ------------------------------------------------------

    #[tokio::test]
    async fn commit_rejects_empty_buffer() {
        let (svc, sink) = service_with_capturing(SinkOutcome::AcceptAndRehash);
        let s = svc.begin(init("u1", 1024)).unwrap();
        let err = svc.commit(s, "ann-1").await.unwrap_err();
        assert!(matches!(err, AnnotationError::InvalidAsset(_)));
        assert!(sink.observed.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn commit_passes_digest_that_rehashes_to_same_bytes() {
        let svc = service(SinkOutcome::AcceptAndRehash);
        let payload = b"stroke-payload";
        let s = svc.begin(init("u1", 1024)).unwrap();
        let s = svc.append_chunk(s, Bytes::from_static(payload)).unwrap();
        let record = svc.commit(s, "ann-1").await.unwrap();
        assert_eq!(record.annotation_id, "ann-1");
        assert_eq!(record.asset_kind, "ink");
        assert_eq!(record.mime_type, "image/svg+xml");
        assert_eq!(record.byte_length, payload.len() as u64);
        assert_eq!(record.object_sha256, sha256_hex(payload));
    }

    #[tokio::test]
    async fn commit_propagates_sink_storage_failure() {
        let svc = service(SinkOutcome::FailStorage("disk full"));
        let s = svc.begin(init("u1", 1024)).unwrap();
        let s = svc.append_chunk(s, Bytes::from_static(b"x")).unwrap();
        let err = svc.commit(s, "ann-1").await.unwrap_err();
        match err {
            AnnotationError::Storage(msg) => assert_eq!(msg, "disk full"),
            other => panic!("expected Storage, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn commit_propagates_sink_sha_mismatch() {
        let svc = service(SinkOutcome::RejectShaMismatch);
        let s = svc.begin(init("u1", 1024)).unwrap();
        let s = svc.append_chunk(s, Bytes::from_static(b"x")).unwrap();
        let err = svc.commit(s, "ann-1").await.unwrap_err();
        assert!(matches!(err, AnnotationError::InvalidAsset(_)));
    }

    #[tokio::test]
    async fn commit_records_voice_duration_ms() {
        let svc = service(SinkOutcome::AcceptAndRehash);
        let mut init = init("u1", 1024);
        init.asset_kind = "voice".to_string();
        init.mime_type = "audio/webm".to_string();
        init.duration_ms = Some(2400);
        let s = svc.begin(init).unwrap();
        let s = svc
            .append_chunk(s, Bytes::from_static(b"voice-bytes"))
            .unwrap();
        let record = svc.commit(s, "ann-7").await.unwrap();
        assert_eq!(record.asset_kind, "voice");
        assert_eq!(record.duration_ms, Some(2400));
    }

    // ----- abort -------------------------------------------------------

    #[tokio::test]
    async fn abort_makes_no_sink_call() {
        let (svc, sink) = service_with_capturing(SinkOutcome::AcceptAndRehash);
        let s = svc.begin(init("u1", 1024)).unwrap();
        let s = svc.append_chunk(s, Bytes::from_static(b"partial")).unwrap();
        svc.abort(s);
        assert!(sink.observed.lock().unwrap().is_empty());
    }

    // ----- single-shot attach -----------------------------------------

    #[tokio::test]
    async fn attach_rejects_empty_bytes() {
        let (svc, sink) = service_with_capturing(SinkOutcome::AcceptAndRehash);
        let err = svc
            .attach(
                "ann-1",
                AnnotationAssetInput {
                    asset_kind: "voice".to_string(),
                    mime_type: "audio/webm".to_string(),
                    bytes: Bytes::new(),
                    duration_ms: Some(0),
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AnnotationError::InvalidAsset(_)));
        assert!(sink.observed.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn attach_voice_clip_records_duration_and_digest() {
        let svc = service(SinkOutcome::AcceptAndRehash);
        let payload = Bytes::from_static(b"voice-bytes");
        let record = svc
            .attach(
                "ann-7",
                AnnotationAssetInput {
                    asset_kind: "voice".to_string(),
                    mime_type: "audio/webm".to_string(),
                    bytes: payload,
                    duration_ms: Some(1850),
                },
            )
            .await
            .unwrap();
        assert_eq!(record.annotation_id, "ann-7");
        assert_eq!(record.asset_kind, "voice");
        assert_eq!(record.mime_type, "audio/webm");
        assert_eq!(record.byte_length, 11);
        assert_eq!(record.duration_ms, Some(1850));
        assert_eq!(record.object_sha256, sha256_hex(b"voice-bytes"));
    }

    #[tokio::test]
    async fn attach_rejects_malformed_mime_before_sink_call() {
        let (svc, sink) = service_with_capturing(SinkOutcome::AcceptAndRehash);
        let err = svc
            .attach(
                "ann-1",
                AnnotationAssetInput {
                    asset_kind: "ink".to_string(),
                    mime_type: "image".to_string(),
                    bytes: Bytes::from_static(b"x"),
                    duration_ms: None,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, AnnotationError::InvalidAsset(_)));
        assert!(sink.observed.lock().unwrap().is_empty());
    }

    // ----- interruption-invariant proof cases -------------------------

    #[tokio::test]
    async fn chunked_commit_and_attach_produce_same_digest() {
        // The single-shot and chunked paths must agree on the digest
        // for identical bytes — this is what "digest-on-commit"
        // guarantees regardless of how the bytes arrived.
        let svc_chunked = service(SinkOutcome::AcceptAndRehash);
        let svc_attach = service(SinkOutcome::AcceptAndRehash);

        let payload = Bytes::from_static(b"the-same-payload");

        let s = svc_chunked
            .begin(AssetUploadInit {
                upload_id: "u".to_string(),
                asset_kind: "ink".to_string(),
                mime_type: "image/svg+xml".to_string(),
                duration_ms: None,
                max_byte_length: 1024,
            })
            .unwrap();
        let s = svc_chunked.append_chunk(s, payload.clone()).unwrap();
        let via_chunked = svc_chunked.commit(s, "ann-x").await.unwrap();

        let via_attach = svc_attach
            .attach(
                "ann-x",
                AnnotationAssetInput {
                    asset_kind: "ink".to_string(),
                    mime_type: "image/svg+xml".to_string(),
                    bytes: payload,
                    duration_ms: None,
                },
            )
            .await
            .unwrap();

        assert_eq!(via_chunked.object_sha256, via_attach.object_sha256);
        assert_eq!(via_chunked.byte_length, via_attach.byte_length);
        assert_eq!(via_chunked.asset_kind, via_attach.asset_kind);
        assert_eq!(via_chunked.mime_type, via_attach.mime_type);
    }

    #[tokio::test]
    async fn interrupted_chunked_upload_leaves_no_sink_call() {
        // Simulate interruption: the caller begins, appends a partial
        // chunk, then never commits (process death, cancellation,
        // network loss). The session value is simply dropped. The sink
        // must observe zero calls — proving no partial asset can be
        // referenced by the annotation.
        let (svc, sink) = service_with_capturing(SinkOutcome::AcceptAndRehash);
        let s = svc.begin(init("u1", 1024)).unwrap();
        let s = svc
            .append_chunk(s, Bytes::from_static(b"partial-stroke"))
            .unwrap();
        assert_eq!(s.accumulated.len(), 14);
        drop(s); // <- "interruption": the caller disappears
        assert!(sink.observed.lock().unwrap().is_empty());
    }

    #[test]
    fn aborted_after_partial_chunks_leaves_no_sink_call() {
        // Explicit abort mid-stream: same invariant, different exit.
        let (svc, sink) = service_with_capturing(SinkOutcome::AcceptAndRehash);
        let s = svc.begin(init("u1", 1024)).unwrap();
        let s = svc.append_chunk(s, Bytes::from_static(b"chunk-1")).unwrap();
        let s = svc.append_chunk(s, Bytes::from_static(b"chunk-2")).unwrap();
        assert_eq!(s.accumulated, b"chunk-1chunk-2");
        svc.abort(s);
        assert!(sink.observed.lock().unwrap().is_empty());
    }
}
