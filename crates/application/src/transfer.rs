//! Resumable download worker — ARCHITECTURE.md §8.4 durable install state
//! machine and §8.5 transfer limits.
//!
//! This module owns the application-layer driver that moves a download through
//! the §8.4 state machine (`Planned → Fetching → Verifying → Staged →
//! Activating → Completed`, with the `Paused` and `Failed` branches). State
//! belongs to `download_jobs`; activation belongs to `install_transactions`.
//! The UI never guesses state from file existence — every state read goes
//! through [`DownloadJobStore::load_job`].
//!
//! Chunk state is persisted so that a process kill mid-download resumes
//! without re-fetching any chunk whose stored bytes already hash to its
//! recorded digest. State transitions obey §8.4 exactly — an invalid
//! transition is a [`StateTransitionError`], never a silent correction.
//!
//! A-02 (ports before providers): this module depends only on `domain`,
//! `ports`, and standard utility crates. The concrete transfer provider
//! (`transfer-http`, `transfer-file`) is injected as a `dyn TransferSource`,
//! and the concrete persistence provider (the future SQL backend for
//! `download_jobs`) is injected as a `dyn DownloadJobStore`.

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::stream::{self, StreamExt};
use ports::transfer::{
    InstallPlan, TransferError, TransferMode, TransferRange, TransferSource, TRANSFER_CHUNK_BYTES,
    TRANSFER_MAX_PARALLEL_CHUNKS_ANDROID, TRANSFER_MAX_PARALLEL_CHUNKS_DESKTOP,
    TRANSFER_RETRY_BASE_MS, TRANSFER_RETRY_COUNT, TRANSFER_RETRY_MAX_MS,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

// ============================================================================
// §8.4 — Durable install state machine
// ============================================================================

/// Durable state of a single download job, persisted in the `download_jobs`
/// table. Variants and edges are normative — see ARCHITECTURE.md §8.4.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadJobState {
    Planned,
    Fetching,
    Paused,
    Verifying,
    Staged,
    Activating,
    Completed,
    Failed,
}

impl DownloadJobState {
    /// Initial state for a freshly created job. The state machine has
    /// exactly one entry point (`[*] --> Planned`).
    pub const fn initial() -> Self {
        DownloadJobState::Planned
    }

    /// True iff the §8.4 diagram draws a directed edge `self --> target`.
    pub fn can_transition_to(self, target: DownloadJobState) -> bool {
        use DownloadJobState::*;
        matches!(
            (self, target),
            (Planned, Fetching)
                | (Fetching, Paused)
                | (Paused, Fetching)
                | (Fetching, Verifying)
                | (Verifying, Staged)
                | (Staged, Activating)
                | (Activating, Completed)
                | (Fetching, Failed)
                | (Verifying, Failed)
                | (Activating, Failed)
                | (Failed, Fetching)
        )
    }

    /// Every state the diagram permits as a direct successor of `self`.
    pub fn allowed_targets(self) -> &'static [DownloadJobState] {
        use DownloadJobState::*;
        match self {
            Planned => &[Fetching],
            Fetching => &[Paused, Verifying, Failed],
            Paused => &[Fetching],
            Verifying => &[Staged, Failed],
            Staged => &[Activating],
            Activating => &[Completed, Failed],
            Completed => &[],
            Failed => &[Fetching],
        }
    }

    /// Terminal state per §8.4 — `Completed` has no outbound edge.
    pub fn is_terminal(self) -> bool {
        matches!(self, DownloadJobState::Completed)
    }
}

/// Rejected state transition. The §8.4 diagram is the sole authority for
/// which edges exist; this error is returned whenever a caller attempts an
/// edge the diagram does not draw.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum StateTransitionError {
    #[error("invalid state transition: {from:?} -> {to:?}")]
    Invalid {
        from: DownloadJobState,
        to: DownloadJobState,
    },
}

/// Validate `from --> to` against §8.4 and return `to` on success.
pub fn transition(
    from: DownloadJobState,
    to: DownloadJobState,
) -> Result<DownloadJobState, StateTransitionError> {
    if from.can_transition_to(to) {
        Ok(to)
    } else {
        Err(StateTransitionError::Invalid { from, to })
    }
}

// ============================================================================
// §8.5 — Chunk planning & retry policy
// ============================================================================

/// Description of a single byte-range chunk planned for a download.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChunkDescriptor {
    pub index: u32,
    pub range: TransferRange,
    pub expected_bytes: u64,
}

/// The full set of chunks a download is partitioned into.
#[derive(Clone, Debug)]
pub struct ChunkPlan {
    pub chunks: Vec<ChunkDescriptor>,
}

impl ChunkPlan {
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}

/// Partition `total_bytes` according to §8.5 `transfer.chunk_bytes` and the
/// chosen transfer mode.
///
/// - `Ranged` is split into `TRANSFER_CHUNK_BYTES`-sized ranges. The final
///   chunk is shorter when `total_bytes` is not a multiple of the chunk size.
/// - `Full` is a single chunk spanning `[0, total_bytes)` because §8.5 only
///   permits a bounded full transfer after explicit user approval (see
///   [`DownloadWorker::approve_full_transfer`]).
/// - `LocalCopy` is also a single chunk: the local-file provider does not
///   speak ranges, but the worker still records one chunk so byte-count and
///   digest verification remain uniform.
///
/// Returns an empty plan when `total_bytes == 0`; the caller must reject
/// zero-byte artifacts at planning time, not here.
pub fn plan_chunks(total_bytes: u64, mode: TransferMode) -> ChunkPlan {
    if total_bytes == 0 {
        return ChunkPlan { chunks: Vec::new() };
    }
    match mode {
        TransferMode::Full | TransferMode::LocalCopy => ChunkPlan {
            chunks: vec![ChunkDescriptor {
                index: 0,
                range: TransferRange {
                    start_byte: 0,
                    end_byte_exclusive: total_bytes,
                },
                expected_bytes: total_bytes,
            }],
        },
        TransferMode::Ranged => {
            let mut chunks = Vec::new();
            let mut offset = 0u64;
            let mut index = 0u32;
            while offset < total_bytes {
                let end = offset.saturating_add(TRANSFER_CHUNK_BYTES).min(total_bytes);
                chunks.push(ChunkDescriptor {
                    index,
                    range: TransferRange {
                        start_byte: offset,
                        end_byte_exclusive: end,
                    },
                    expected_bytes: end - offset,
                });
                offset = end;
                index = index.wrapping_add(1);
            }
            ChunkPlan { chunks }
        }
    }
}

/// Exponential backoff delay for a retry attempt, per §8.5
/// `transfer.retry_base_ms` and `transfer.retry_max_ms`. `attempt` is the
/// 1-based retry number, so the first retry waits `base`, the second waits
/// `2·base`, and so on, capped at `transfer.retry_max_ms`.
pub fn backoff_delay_ms(attempt: u32) -> u64 {
    if attempt == 0 {
        return TRANSFER_RETRY_BASE_MS;
    }
    let shift = attempt.min(63);
    let multiplier = 1u64.checked_shl(shift).unwrap_or(u64::MAX);
    let raw = TRANSFER_RETRY_BASE_MS.saturating_mul(multiplier);
    raw.min(TRANSFER_RETRY_MAX_MS)
}

/// Maximum number of retry attempts per chunk before the chunk fails, per
/// §8.5 `transfer.retry_count`.
pub const RETRY_COUNT: u32 = TRANSFER_RETRY_COUNT;

/// Resolve the §8.5 `transfer.max_parallel_chunks.*` limit from the platform
/// label of the storage destination. The application crate cannot import
/// `ports::platform::PlatformKind` because that module does not yet compile;
/// callers pass the label they already hold from their platform adapter.
pub fn max_parallel_chunks_for(platform_label: &str) -> u32 {
    if platform_label.eq_ignore_ascii_case("android") {
        TRANSFER_MAX_PARALLEL_CHUNKS_ANDROID
    } else {
        TRANSFER_MAX_PARALLEL_CHUNKS_DESKTOP
    }
}

// ============================================================================
// Durable records — the application-layer view of `download_jobs`
// ============================================================================

/// Durable record of one download job. Mirrors the columns the §8.4 worker
/// needs; the concrete SQL schema for `download_jobs` (owned by the
/// `storage-sqlite` provider) will materialize this view.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DownloadJobRecord {
    pub job_id: String,
    pub install_plan_id: String,
    pub state: DownloadJobState,
    pub source_uri: String,
    pub expected_etag: Option<String>,
    pub total_bytes: u64,
    pub expected_sha256: String,
    pub transfer_mode: TransferMode,
    pub signature_required: bool,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

/// Durable record of one chunk within a download job. A chunk counts as
/// "already fetched" iff `verified == true` AND `stored_bytes ==
/// expected_bytes`; on resume the worker re-reads its bytes, recomputes the
/// sha256, and only re-fetches when the recomputed digest does not match
/// `sha256_hex`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChunkRecord {
    pub index: u32,
    pub start_byte: u64,
    pub end_byte_exclusive: u64,
    pub expected_bytes: u64,
    pub stored_bytes: u64,
    pub sha256_hex: Option<String>,
    pub verified: bool,
}

// ============================================================================
// Persistence boundary — A-02 indirection for `download_jobs`
// ============================================================================

/// Failure modes of the durable job store. Implementations translate their
/// backend-specific errors into these variants; the worker never sees SQL,
/// filesystem paths, or provider internals.
#[derive(Clone, Debug, thiserror::Error)]
pub enum JobStoreError {
    #[error("job not found: {0}")]
    NotFound(String),
    #[error("chunk not found: job={0} index={1}")]
    ChunkNotFound(String, u32),
    /// A conditional `save_state` update matched zero rows because the row's
    /// persisted `from` state differs from what the worker expected — i.e.
    /// another process or a concurrent resume mutated the job.
    #[error("state precondition mismatch: job={job_id} expected={expected:?} actual={actual:?}")]
    StatePrecondition {
        job_id: String,
        expected: DownloadJobState,
        actual: DownloadJobState,
    },
    #[error("backend error: {0}")]
    Backend(String),
}

/// Durable persistence boundary for the resumable download worker.
///
/// This is the application-layer contract for the `download_jobs` table (and
/// its per-job chunk blob storage). The concrete implementation will live in
/// the `storage-sqlite` provider; the worker itself depends only on this
/// trait so that A-02 (ports before providers) holds.
///
/// All methods must flush durably before returning — a process kill
/// immediately after a successful return must not lose data.
#[async_trait]
pub trait DownloadJobStore: Send + Sync {
    /// Atomically create a new job. Reject with
    /// [`JobStoreError::Backend`] if `job_id` already exists. Implementations
    /// that allow in-place metadata rewrites (e.g. for
    /// [`DownloadWorker::approve_full_transfer`]) accept a row whose `job_id`
    /// matches an existing row and overwrite it atomically.
    async fn create_job(&self, record: DownloadJobRecord) -> Result<(), JobStoreError>;

    /// Load the full durable record for a job.
    async fn load_job(&self, job_id: &str) -> Result<DownloadJobRecord, JobStoreError>;

    /// Conditionally transition a job's persisted state from `from` to `to`.
    /// Implementations MUST perform this as an atomic compare-and-set on the
    /// `from` value and return [`JobStoreError::StatePrecondition`] if the
    /// row's current state is not `from`. The §8.4 edge `from --> to` is
    /// validated by the caller via [`transition`] before this is invoked.
    async fn save_state(
        &self,
        job_id: &str,
        from: DownloadJobState,
        to: DownloadJobState,
    ) -> Result<(), JobStoreError>;

    /// Touch `updated_at_ms` without changing state. Used to keep the row
    /// fresh while fetching so that monitors can detect a stalled worker.
    async fn touch(&self, job_id: &str, updated_at_ms: i64) -> Result<(), JobStoreError>;

    /// Persist or update a chunk record. Called after [`append_chunk_bytes`]
    /// and [`seal_chunk`] so the row reflects reality on disk.
    async fn upsert_chunk(&self, job_id: &str, chunk: ChunkRecord) -> Result<(), JobStoreError>;

    /// List chunk records in ascending `index` order. Returns an empty vec
    /// for a job that has not started fetching.
    async fn list_chunks(&self, job_id: &str) -> Result<Vec<ChunkRecord>, JobStoreError>;

    /// Append `bytes` to the staged blob for chunk `chunk_index`. Bytes must
    /// be flushed durably before return.
    async fn append_chunk_bytes(
        &self,
        job_id: &str,
        chunk_index: u32,
        bytes: &[u8],
    ) -> Result<(), JobStoreError>;

    /// Finalize a chunk: record its final byte count and sha256 digest and
    /// mark it verified. After this returns the bytes MUST be re-readable
    /// via [`read_chunk_bytes`].
    async fn seal_chunk(
        &self,
        job_id: &str,
        chunk_index: u32,
        byte_count: u64,
        sha256_hex: &str,
    ) -> Result<(), JobStoreError>;

    /// Re-read the full staged blob for a chunk. Used by the worker on
    /// resume to confirm an existing chunk's digest and at the `Verifying`
    /// stage to compute the file-level digest.
    async fn read_chunk_bytes(
        &self,
        job_id: &str,
        chunk_index: u32,
    ) -> Result<Vec<u8>, JobStoreError>;

    /// Discard any staged bytes for a chunk and clear its record. Used
    /// before re-fetching a chunk whose stored bytes failed digest
    /// verification.
    async fn discard_chunk(&self, job_id: &str, chunk_index: u32) -> Result<(), JobStoreError>;
}

// ============================================================================
// Worker errors
// ============================================================================

/// Failure modes of the resumable download worker. Maps cleanly onto the
/// §8.4 branches that lead to `Failed` plus the two §8.5 conditions that
/// require user judgement (`RangeUnsupported`, `RangeResponseInvalid`).
#[derive(Clone, Debug, thiserror::Error)]
pub enum DownloadWorkerError {
    #[error("job not found: {0}")]
    JobNotFound(String),
    #[error("state transition rejected: {0}")]
    StateTransition(#[from] StateTransitionError),
    #[error("store error: {0}")]
    Store(#[from] JobStoreError),
    #[error("transfer error: {0}")]
    Transfer(#[from] TransferError),
    #[error("chunk {chunk_index} failed after {attempts} attempts: {inner}")]
    ChunkFailed {
        chunk_index: u32,
        attempts: u32,
        inner: TransferError,
    },
    #[error("digest mismatch for chunk {chunk_index}: expected {expected}, computed {computed}")]
    ChunkDigestMismatch {
        chunk_index: u32,
        expected: String,
        computed: String,
    },
    #[error("size mismatch for chunk {chunk_index}: expected {expected}, got {actual}")]
    ChunkSizeMismatch {
        chunk_index: u32,
        expected: u64,
        actual: u64,
    },
    #[error("file digest mismatch: expected {expected}, computed {computed}")]
    FileDigestMismatch { expected: String, computed: String },
    /// §8.5: a ranged request received an invalid response. The worker
    /// surfaces this so the operator can decide whether to approve a
    /// bounded full transfer via
    /// [`DownloadWorker::approve_full_transfer`].
    #[error(
        "ranged response invalid at chunk {chunk_index}; user may approve a bounded full transfer"
    )]
    RangeResponseInvalid { chunk_index: u32 },
    /// §8.5: the source does not accept ranges at all. Same user-decision
    /// path as [`Self::RangeResponseInvalid`].
    #[error("source does not accept ranges; user may approve a bounded full transfer")]
    RangeUnsupported,
    /// The job is currently paused; the caller must invoke `resume()` to
    /// drive it forward. Returned (not returned as a hard error) so callers
    /// can distinguish "paused" from "failed".
    #[error("job is paused; call resume() to continue: {0}")]
    Paused(String),
}

// ============================================================================
// The worker
// ============================================================================

/// Abstraction over the runtime sleep so the worker is deterministic in
/// tests. Production code constructs this from `tokio::time::sleep`.
pub trait Pause: Send + Sync {
    fn pause(&self, duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}

/// Default `Pause` backed by `tokio::time::sleep`.
pub struct TokioPause;

impl Pause for TokioPause {
    fn pause(&self, duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(tokio::time::sleep(duration))
    }
}

/// A monotonic clock used to stamp `created_at_ms` / `updated_at_ms`.
pub trait Clock: Send + Sync {
    fn now_ms(&self) -> i64;
}

/// Wall-clock `Clock` backed by `std::time::SystemTime`.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> i64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }
}

/// Resumable download worker. Drives the §8.4 state machine for one or many
/// jobs against an injected [`TransferSource`] and [`DownloadJobStore`].
///
/// The worker owns no per-job state — every transition is durably persisted
/// via the store before the worker continues, so a process kill at any point
/// resumes cleanly when `resume()` is next invoked.
pub struct DownloadWorker {
    source: Arc<dyn TransferSource>,
    max_parallel_chunks: u32,
    pause: Arc<dyn Pause>,
    clock: Arc<dyn Clock>,
}

impl DownloadWorker {
    /// Construct a new worker. `max_parallel_chunks` should be derived from
    /// the storage destination's platform via
    /// [`max_parallel_chunks_for`].
    pub fn new(
        source: Arc<dyn TransferSource>,
        max_parallel_chunks: u32,
        pause: Arc<dyn Pause>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            source,
            max_parallel_chunks: max_parallel_chunks.max(1),
            pause,
            clock,
        }
    }

    /// Construct with the production `tokio` pause and `SystemClock`.
    pub fn with_production_runtime(
        source: Arc<dyn TransferSource>,
        max_parallel_chunks: u32,
    ) -> Self {
        Self::new(
            source,
            max_parallel_chunks,
            Arc::new(TokioPause),
            Arc::new(SystemClock),
        )
    }

    /// Create a new job in `Planned` and immediately drive it forward. The
    /// job is durably `Planned` before any network activity so a kill between
    /// `create_job` and the first fetch still resumes cleanly.
    pub async fn start(
        &self,
        store: &dyn DownloadJobStore,
        plan: &InstallPlan,
        job_id: String,
    ) -> Result<String, DownloadWorkerError> {
        let now = self.clock.now_ms();
        let record = DownloadJobRecord {
            job_id: job_id.clone(),
            install_plan_id: plan.install_plan_id.clone(),
            state: DownloadJobState::Planned,
            source_uri: plan.catalog_entry.artifact_url.clone(),
            expected_etag: None,
            total_bytes: plan.verification.expected_size_bytes,
            expected_sha256: plan.verification.expected_sha256.clone(),
            transfer_mode: plan.transfer_mode,
            signature_required: plan.verification.signature_required,
            created_at_ms: now,
            updated_at_ms: now,
        };
        store.create_job(record).await?;

        // Drive forward as far as possible without blocking on activation
        // (activation belongs to `install_transactions`).
        self.resume(store, &job_id).await?;
        Ok(job_id)
    }

    /// Drive an existing job forward from whatever durable state it is in,
    /// stopping at the first state that requires external input:
    ///   * `Paused` — operator action required.
    ///   * `Staged` — the install layer must call `begin_activation`.
    ///   * `Completed` — nothing to do.
    ///   * `Failed` — operator must trigger a retry (resume auto-retries
    ///     once via the `Failed --> Fetching` edge).
    pub async fn resume(
        &self,
        store: &dyn DownloadJobStore,
        job_id: &str,
    ) -> Result<(), DownloadWorkerError> {
        let record = store.load_job(job_id).await.map_err(|e| match e {
            JobStoreError::NotFound(id) => DownloadWorkerError::JobNotFound(id),
            other => DownloadWorkerError::Store(other),
        })?;

        match record.state {
            DownloadJobState::Planned => {
                self.transition_durably(store, job_id, record.state, DownloadJobState::Fetching)
                    .await?;
                self.drive_fetching(store, job_id).await
            }
            DownloadJobState::Fetching => self.drive_fetching(store, job_id).await,
            DownloadJobState::Paused => Err(DownloadWorkerError::Paused(job_id.to_string())),
            DownloadJobState::Failed => {
                // §8.4 edge: Failed --> Fetching.
                self.transition_durably(store, job_id, record.state, DownloadJobState::Fetching)
                    .await?;
                self.drive_fetching(store, job_id).await
            }
            DownloadJobState::Verifying => self.drive_verifying(store, job_id).await,
            DownloadJobState::Staged
            | DownloadJobState::Activating
            | DownloadJobState::Completed => Ok(()),
        }
    }

    /// Cooperatively pause a `Fetching` job. Transitions the persisted state
    /// to `Paused`; in-flight chunk streams finish writing their current
    /// batch but the fetch loop notices the new state before issuing the
    /// next chunk and stops.
    pub async fn pause(
        &self,
        store: &dyn DownloadJobStore,
        job_id: &str,
    ) -> Result<(), DownloadWorkerError> {
        let record = store.load_job(job_id).await?;
        self.transition_durably(store, job_id, record.state, DownloadJobState::Paused)
            .await
    }

    /// §8.5 fallback: the user has explicitly approved converting a stuck
    /// ranged download into a bounded full transfer. Discards any partial
    /// ranged chunks, rewrites the job's `transfer_mode` to `Full`, and
    /// re-enters `Fetching`.
    pub async fn approve_full_transfer(
        &self,
        store: &dyn DownloadJobStore,
        job_id: &str,
    ) -> Result<(), DownloadWorkerError> {
        let mut record = store.load_job(job_id).await?;
        record.transfer_mode = TransferMode::Full;
        record.updated_at_ms = self.clock.now_ms();

        // Discard any partial ranged chunks so the worker re-fetches as a
        // single full chunk.
        let chunks = store.list_chunks(job_id).await.unwrap_or_default();
        for chunk in chunks {
            store.discard_chunk(job_id, chunk.index).await?;
        }

        // §8.4 edges: Fetching/Paused/Failed can all move to Fetching (Paused
        // and Failed directly; Fetching is already there).
        record.state = match record.state {
            DownloadJobState::Fetching => DownloadJobState::Fetching,
            DownloadJobState::Paused | DownloadJobState::Failed => DownloadJobState::Fetching,
            other => other,
        };
        // create_job with an existing id must overwrite per the trait contract.
        store.create_job(record).await?;
        self.drive_fetching(store, job_id).await
    }

    /// Install-layer entry point: transition `Staged --> Activating`. The
    /// actual activation work (atomic rename, `install_transactions` row
    /// insert) is performed by the install layer between this call and
    /// [`complete_activation`].
    pub async fn begin_activation(
        &self,
        store: &dyn DownloadJobStore,
        job_id: &str,
    ) -> Result<(), DownloadWorkerError> {
        let record = store.load_job(job_id).await?;
        self.transition_durably(store, job_id, record.state, DownloadJobState::Activating)
            .await
    }

    /// Install-layer completion: transition `Activating --> Completed`.
    pub async fn complete_activation(
        &self,
        store: &dyn DownloadJobStore,
        job_id: &str,
    ) -> Result<(), DownloadWorkerError> {
        let record = store.load_job(job_id).await?;
        self.transition_durably(store, job_id, record.state, DownloadJobState::Completed)
            .await
    }

    /// Mark a job `Failed`. Valid from `Fetching`, `Verifying`, or
    /// `Activating` per §8.4.
    pub async fn fail(
        &self,
        store: &dyn DownloadJobStore,
        job_id: &str,
    ) -> Result<(), DownloadWorkerError> {
        let record = store.load_job(job_id).await?;
        self.transition_durably(store, job_id, record.state, DownloadJobState::Failed)
            .await
    }

    // ------------------------------------------------------------------
    // Internal: state-machine helpers
    // ------------------------------------------------------------------

    async fn transition_durably(
        &self,
        store: &dyn DownloadJobStore,
        job_id: &str,
        from: DownloadJobState,
        to: DownloadJobState,
    ) -> Result<(), DownloadWorkerError> {
        transition(from, to)?;
        store.save_state(job_id, from, to).await?;
        let _ = store.touch(job_id, self.clock.now_ms()).await;
        Ok(())
    }

    /// Drive `Fetching`: discover remaining chunks, fetch them with bounded
    /// parallelism, and once all are verified transition to `Verifying`.
    async fn drive_fetching(
        &self,
        store: &dyn DownloadJobStore,
        job_id: &str,
    ) -> Result<(), DownloadWorkerError> {
        let record = store.load_job(job_id).await?;
        debug_assert!(
            matches!(
                record.state,
                DownloadJobState::Fetching | DownloadJobState::Paused
            ),
            "drive_fetching called from non-fetching state"
        );

        let plan = plan_chunks(record.total_bytes, record.transfer_mode);

        // Build the per-chunk state map from durable records.
        let persisted: BTreeMap<u32, ChunkRecord> = store
            .list_chunks(job_id)
            .await?
            .into_iter()
            .map(|c| (c.index, c))
            .collect();

        // For each planned chunk, decide whether it needs work.
        let mut pending: Vec<ChunkDescriptor> = Vec::new();
        for descriptor in &plan.chunks {
            match persisted.get(&descriptor.index) {
                Some(rec) if rec.verified && rec.stored_bytes == descriptor.expected_bytes => {
                    // Verify the persisted bytes still hash to the recorded
                    // digest. If not, discard and re-fetch.
                    let stored = store.read_chunk_bytes(job_id, descriptor.index).await?;
                    let computed = hex::encode(Sha256::digest(&stored));
                    match &rec.sha256_hex {
                        Some(recorded) if recorded == &computed => continue,
                        _ => {
                            store.discard_chunk(job_id, descriptor.index).await?;
                            pending.push(descriptor.clone());
                        }
                    }
                }
                _ => pending.push(descriptor.clone()),
            }
        }

        if pending.is_empty() {
            // Nothing left to fetch — move to verifying.
            self.transition_durably(
                store,
                job_id,
                DownloadJobState::Fetching,
                DownloadJobState::Verifying,
            )
            .await?;
            return self.drive_verifying(store, job_id).await;
        }

        // Fetch with bounded parallelism via a concurrent stream. Each chunk
        // future borrows the injected source/store/pause references directly,
        // so no tokio task spawning is required (the `rt` feature is not
        // needed) and the store no longer has to be an `Arc<dyn …>`.
        // `buffer_unordered(max_parallel_chunks)` enforces §8.5's
        // `transfer.max_parallel_chunks.*` limit the same way a semaphore
        // would: at most `max_parallel_chunks` chunk futures are polled at
        // once, and a new one starts as soon as one completes.
        let source: &dyn TransferSource = &*self.source;
        let pause: &dyn Pause = &*self.pause;
        let expected_etag = record.expected_etag.clone();
        let source_uri = record.source_uri.clone();

        let mut chunk_results = stream::iter(pending.into_iter().map(|descriptor| {
            let source_uri = source_uri.clone();
            let expected_etag = expected_etag.clone();
            async move {
                fetch_one_chunk(
                    source,
                    store,
                    pause,
                    job_id,
                    &source_uri,
                    &descriptor,
                    expected_etag.as_deref(),
                )
                .await
            }
        }))
        .buffer_unordered(self.max_parallel_chunks as usize);

        // Surface the first hard failure. Dropping the stream cancels any
        // in-flight chunk futures (equivalent to JoinSet::shutdown).
        while let Some(res) = chunk_results.next().await {
            match res {
                Ok(()) => {}
                Err(e) => {
                    drop(chunk_results);
                    if matches!(
                        e,
                        DownloadWorkerError::RangeResponseInvalid { .. }
                            | DownloadWorkerError::RangeUnsupported
                    ) {
                        let _ = self
                            .transition_durably(
                                store,
                                job_id,
                                DownloadJobState::Fetching,
                                DownloadJobState::Failed,
                            )
                            .await;
                        return Err(e);
                    }
                    let _ = self.fail(store, job_id).await;
                    return Err(e);
                }
            }
        }

        // Between batches, check whether the operator paused the job. If so,
        // leave it parked; resume() will pick it up.
        let record = store.load_job(job_id).await?;
        if record.state == DownloadJobState::Paused {
            return Err(DownloadWorkerError::Paused(job_id.to_string()));
        }

        // All chunks succeeded — move to verifying.
        self.transition_durably(
            store,
            job_id,
            DownloadJobState::Fetching,
            DownloadJobState::Verifying,
        )
        .await?;
        self.drive_verifying(store, job_id).await
    }

    /// Drive `Verifying`: concatenate chunk bytes in order, compute the
    /// file-level sha256, and compare to the expected digest. On match,
    /// transition to `Staged`. On mismatch, transition to `Failed`.
    async fn drive_verifying(
        &self,
        store: &dyn DownloadJobStore,
        job_id: &str,
    ) -> Result<(), DownloadWorkerError> {
        let record = store.load_job(job_id).await?;
        if record.state != DownloadJobState::Verifying {
            self.transition_durably(store, job_id, record.state, DownloadJobState::Verifying)
                .await?;
        }

        let plan = plan_chunks(record.total_bytes, record.transfer_mode);
        let mut hasher = Sha256::new();
        let mut total_seen: u64 = 0;
        for descriptor in &plan.chunks {
            let bytes = store
                .read_chunk_bytes(job_id, descriptor.index)
                .await
                .map_err(|e| match e {
                    JobStoreError::ChunkNotFound(_, idx) => {
                        DownloadWorkerError::ChunkSizeMismatch {
                            chunk_index: idx,
                            expected: descriptor.expected_bytes,
                            actual: 0,
                        }
                    }
                    other => DownloadWorkerError::Store(other),
                })?;
            if bytes.len() as u64 != descriptor.expected_bytes {
                let _ = self.fail_store_ref(store, job_id).await;
                return Err(DownloadWorkerError::ChunkSizeMismatch {
                    chunk_index: descriptor.index,
                    expected: descriptor.expected_bytes,
                    actual: bytes.len() as u64,
                });
            }
            hasher.update(&bytes);
            total_seen = total_seen.saturating_add(bytes.len() as u64);
        }

        if total_seen != record.total_bytes {
            let _ = self.fail_store_ref(store, job_id).await;
            return Err(DownloadWorkerError::ChunkSizeMismatch {
                chunk_index: 0,
                expected: record.total_bytes,
                actual: total_seen,
            });
        }

        let computed = hex::encode(hasher.finalize());
        if computed != record.expected_sha256 {
            let _ = self.fail_store_ref(store, job_id).await;
            return Err(DownloadWorkerError::FileDigestMismatch {
                expected: record.expected_sha256.clone(),
                computed,
            });
        }

        self.transition_durably(
            store,
            job_id,
            DownloadJobState::Verifying,
            DownloadJobState::Staged,
        )
        .await
    }

    /// `fail()` against a borrowed store, used from internal helpers that
    /// already hold `&dyn DownloadJobStore`.
    async fn fail_store_ref(
        &self,
        store: &dyn DownloadJobStore,
        job_id: &str,
    ) -> Result<(), DownloadWorkerError> {
        let record = store.load_job(job_id).await.ok();
        if let Some(rec) = record {
            // Best-effort transition to Failed. We swallow the error here
            // because the caller already has the real error to surface.
            if rec.state != DownloadJobState::Failed {
                let _ = self
                    .transition_durably(store, job_id, rec.state, DownloadJobState::Failed)
                    .await;
            }
        }
        Ok(())
    }
}

// ============================================================================
// Internals: per-chunk fetch with retry
// ============================================================================

/// Fetch a single chunk with retries per §8.5 `transfer.retry_*`. The chunk
/// is streamed straight into the durable store; on each permanent failure
/// the staged bytes are discarded so a retry starts clean.
async fn fetch_one_chunk(
    source: &dyn TransferSource,
    store: &dyn DownloadJobStore,
    pause: &dyn Pause,
    job_id: &str,
    source_uri: &str,
    descriptor: &ChunkDescriptor,
    expected_etag: Option<&str>,
) -> Result<(), DownloadWorkerError> {
    let mut last_err: Option<TransferError> = None;
    for attempt in 0..=RETRY_COUNT {
        if attempt > 0 {
            let delay = Duration::from_millis(backoff_delay_ms(attempt));
            pause.pause(delay).await;
        }

        match fetch_one_attempt(source, store, job_id, source_uri, descriptor, expected_etag).await
        {
            Ok(()) => return Ok(()),
            Err(DownloadWorkerError::RangeResponseInvalid { .. })
            | Err(DownloadWorkerError::RangeUnsupported) => {
                // Surface immediately — these need a user decision, not a
                // retry.
                return Err(last_err
                    .map(DownloadWorkerError::Transfer)
                    .unwrap_or_else(|| DownloadWorkerError::RangeResponseInvalid {
                        chunk_index: descriptor.index,
                    }));
            }
            Err(DownloadWorkerError::Transfer(e)) => {
                last_err = Some(e);
                let _ = store.discard_chunk(job_id, descriptor.index).await;
                continue;
            }
            Err(other) => return Err(other),
        }
    }
    Err(DownloadWorkerError::ChunkFailed {
        chunk_index: descriptor.index,
        attempts: RETRY_COUNT,
        inner: last_err.unwrap_or(TransferError::Transport(
            "retry loop exhausted with no captured error".to_string(),
        )),
    })
}

/// One non-retrying attempt to fetch, store, and seal a single chunk.
async fn fetch_one_attempt(
    source: &dyn TransferSource,
    store: &dyn DownloadJobStore,
    job_id: &str,
    source_uri: &str,
    descriptor: &ChunkDescriptor,
    expected_etag: Option<&str>,
) -> Result<(), DownloadWorkerError> {
    // Discard any prior partial bytes so we start clean.
    store.discard_chunk(job_id, descriptor.index).await?;

    let stream = source
        .open_range(source_uri, descriptor.range.clone(), expected_etag)
        .await
        .map_err(map_open_range_error(descriptor.index))?;

    let mut hasher = Sha256::new();
    let mut total: u64 = 0;
    let mut chunk_stream = stream;
    while let Some(item) = chunk_stream.next().await {
        let bytes: Bytes = item.map_err(map_stream_error(descriptor.index))?;
        if !bytes.is_empty() {
            hasher.update(&bytes);
            total = total.saturating_add(bytes.len() as u64);
            store
                .append_chunk_bytes(job_id, descriptor.index, &bytes)
                .await?;
        }
    }

    if total != descriptor.expected_bytes {
        return Err(DownloadWorkerError::ChunkSizeMismatch {
            chunk_index: descriptor.index,
            expected: descriptor.expected_bytes,
            actual: total,
        });
    }

    let digest = hex::encode(hasher.finalize());
    store
        .seal_chunk(job_id, descriptor.index, total, &digest)
        .await?;

    store
        .upsert_chunk(
            job_id,
            ChunkRecord {
                index: descriptor.index,
                start_byte: descriptor.range.start_byte,
                end_byte_exclusive: descriptor.range.end_byte_exclusive,
                expected_bytes: descriptor.expected_bytes,
                stored_bytes: total,
                sha256_hex: Some(digest),
                verified: true,
            },
        )
        .await?;

    Ok(())
}

/// §8.5: HTTP `200` to a range request is not accepted as a ranged response,
/// and the source refusing ranges altogether is its own condition. The
/// transfer provider surfaces both via `TransferError`; we map them to the
/// worker's two user-decision errors so the operator can approve a bounded
/// full transfer.
fn map_open_range_error(chunk_index: u32) -> impl Fn(TransferError) -> DownloadWorkerError {
    move |e: TransferError| match e {
        TransferError::InvalidRangeResponse => {
            DownloadWorkerError::RangeResponseInvalid { chunk_index }
        }
        TransferError::RangeUnsupported => DownloadWorkerError::RangeUnsupported,
        other => DownloadWorkerError::Transfer(other),
    }
}

fn map_stream_error(chunk_index: u32) -> impl Fn(TransferError) -> DownloadWorkerError {
    move |e: TransferError| match e {
        TransferError::InvalidRangeResponse => {
            DownloadWorkerError::RangeResponseInvalid { chunk_index }
        }
        TransferError::RangeUnsupported => DownloadWorkerError::RangeUnsupported,
        other => DownloadWorkerError::Transfer(other),
    }
}
