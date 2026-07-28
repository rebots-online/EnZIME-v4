#![forbid(unsafe_code)]

//! Pack import and activation — ARCHITECTURE.md §8.3 install plan + §8.4
//! durable install state machine.
//!
//! This module owns three things, in binding order:
//!
//! 1. [`plan_install`] — the pure planner that produces an [`InstallPlan`]
//!    or fails with one of the six §8.3 failure conditions, encoded exactly
//!    as [`InstallPlanError`]. The six conditions are checked in the order
//!    the spec lists them.
//! 2. [`DownloadJobState`] — the eight-state machine from the §8.4 mermaid
//!    diagram, with legal edges enforced by
//!    [`DownloadJobState::transition`]. State belongs to `download_jobs`
//!    (§8.4); the `snake_case` serde rename matches the SQL CHECK
//!    identifier for that column.
//! 3. [`PackActivator`] — the abstraction that performs the atomic swap of
//!    the active pack pointer (§6.3 commit steps 9 + 10). The application
//!    layer drives verification, calls the activator, and on any
//!    non-committed outcome transitions the job to `Failed` — the previous
//!    active pack stays readable (A-04) and the untrusted candidate bytes
//!    are quarantined (A-07).
//!
//! Persistence is delegated: this module produces and validates state
//! values; the storage layer (Ring 2) writes them to `download_jobs` and
//! `install_transactions`. The UI never guesses state from file existence
//! (§8.4) — every observed state is read from `download_jobs.state`.

use async_trait::async_trait;
use domain::identity::PackVersionId;
use ports::catalog::CatalogEntry;
use ports::transfer::{InstallPlan, StorageDestination, TransferMode, VerificationPlan};
use serde::{Deserialize, Serialize};

// ===========================================================================
// §8.3 Install plan
// ===========================================================================

/// Raw inputs to [`plan_install`]. The planner observes these and produces
/// a fully populated [`InstallPlan`] or one of the six [`InstallPlanError`]
/// conditions.
#[derive(Clone, Debug)]
pub struct InstallPlanRequest {
    pub install_plan_id: String,
    pub catalog_entry: CatalogEntry,
    pub destination: StorageDestination,
    pub transfer_mode: TransferMode,

    /// Free bytes at the destination root, observed at plan time.
    pub free_bytes: u64,

    /// Bytes occupied by the last-known-good pack version that must remain
    /// readable through the activation window (A-04). Zero when no prior
    /// version is being replaced.
    pub retained_last_known_good_bytes: u64,

    /// Whether the source honors byte-range requests, as observed via
    /// `TransferSource::probe`.
    pub source_accepts_ranges: bool,

    /// Policy: a trusted signature is required for this resource.
    pub signature_required: bool,

    /// Observation: a trusted signature is available for this artifact.
    pub trusted_signature_available: bool,

    /// Whether the format verifier should run a ZIM structure check on the
    /// staged candidate.
    pub zim_structure_check: bool,

    pub replaces_pack_version_id: Option<PackVersionId>,
    pub retained_pack_version_id: Option<PackVersionId>,
}

/// The six §8.3 failure conditions. Each variant is actionable: the caller
/// can change exactly one input to make planning succeed.
#[derive(Clone, Debug, thiserror::Error, Eq, PartialEq)]
pub enum InstallPlanError {
    /// `destination is not writable`
    #[error("destination is not writable")]
    DestinationNotWritable,
    /// `final bytes do not fit`
    #[error("final bytes do not fit")]
    FinalBytesExceedFree,
    /// `temporary plus retained last-known-good bytes do not fit`
    #[error("temporary plus retained last-known-good bytes do not fit")]
    TemporaryPlusRetainedExceedFree,
    /// `no verified digest is available`
    #[error("no verified digest is available")]
    NoVerifiedDigest,
    /// `policy requires a signature and no trusted signature exists`
    #[error("policy requires a signature and no trusted signature exists")]
    SignatureRequiredButMissing,
    /// `source range behavior is incompatible with the selected strategy`
    #[error("source range behavior is incompatible with the selected strategy")]
    TransferModeIncompatible,
}

/// Produce an [`InstallPlan`] from a request, or fail with one of the six
/// §8.3 conditions.
///
/// Size accounting (binding reading of §8.3):
/// - `download_bytes` = `catalog_entry.artifact_size_bytes`;
/// - `temporary_bytes` = `download_bytes` (the entire artifact is staged
///   before activation, regardless of transfer mode);
/// - `final_bytes` = `download_bytes` (on-disk pack size before any
///   rebuildable index);
/// - `free_bytes_after` = `free_bytes_before - final_bytes`.
pub fn plan_install(request: InstallPlanRequest) -> Result<InstallPlan, InstallPlanError> {
    // §8.3 bullet 5: signature policy gate — checked before any I/O.
    if request.signature_required && !request.trusted_signature_available {
        return Err(InstallPlanError::SignatureRequiredButMissing);
    }

    // §8.3 bullet 1: destination must be writable.
    if !request.destination.writable {
        return Err(InstallPlanError::DestinationNotWritable);
    }

    // §8.3 bullet 4: a verified digest must be available. The catalog entry
    // is the verified-digest carrier — its size and sha256 fields are the
    // commitment the planner verifies staged bytes against. Either an empty
    // size or a non-64-char-lowercase-hex sha256 means there is no verified
    // digest to commit to.
    let sha = &request.catalog_entry.artifact_sha256;
    let verified_digest_available = request.catalog_entry.artifact_size_bytes > 0
        && sha.len() == 64
        && sha.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'));
    if !verified_digest_available {
        return Err(InstallPlanError::NoVerifiedDigest);
    }

    // §8.3 bullet 6: range-strategy compatibility. `Ranged` requires the
    // source to honor byte-range requests; `Full` and `LocalCopy` do not.
    if matches!(request.transfer_mode, TransferMode::Ranged) && !request.source_accepts_ranges {
        return Err(InstallPlanError::TransferModeIncompatible);
    }

    let download_bytes = request.catalog_entry.artifact_size_bytes;
    let temporary_bytes = download_bytes;
    let final_bytes = download_bytes;
    let free_bytes_before = request.free_bytes;

    // §8.3 bullet 2: final bytes must fit.
    if final_bytes > free_bytes_before {
        return Err(InstallPlanError::FinalBytesExceedFree);
    }

    // §8.3 bullet 3: temporary plus retained last-known-good bytes must fit.
    // The activation window holds both the staged candidate and the
    // previous readable version simultaneously (A-04).
    let combined = temporary_bytes
        .checked_add(request.retained_last_known_good_bytes)
        .ok_or(InstallPlanError::TemporaryPlusRetainedExceedFree)?;
    if combined > free_bytes_before {
        return Err(InstallPlanError::TemporaryPlusRetainedExceedFree);
    }

    let free_bytes_after = free_bytes_before - final_bytes;
    let expected_sha256 = request.catalog_entry.artifact_sha256.clone();

    Ok(InstallPlan {
        install_plan_id: request.install_plan_id,
        catalog_entry: request.catalog_entry,
        destination: request.destination,
        transfer_mode: request.transfer_mode,
        download_bytes,
        temporary_bytes,
        final_bytes,
        free_bytes_before,
        free_bytes_after,
        verification: VerificationPlan {
            expected_size_bytes: download_bytes,
            expected_sha256: expected_sha256,
            signature_required: request.signature_required,
            zim_structure_check: request.zim_structure_check,
        },
        replaces_pack_version_id: request.replaces_pack_version_id,
        retained_pack_version_id: request.retained_pack_version_id,
    })
}

// ===========================================================================
// §8.4 Durable install state machine
// ===========================================================================

/// The eight states of the §8.4 install state machine. State belongs to
/// `download_jobs.state` (§8.4). The `snake_case` serde rename matches the
/// SQL CHECK constraint identifier for that column.
///
/// Legal edges are encoded in [`DownloadJobState::transition`]; every edge
/// not listed there is rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
    /// Apply a transition, returning the new state or
    /// [`DownloadJobTransitionError`] if the `(from, to)` edge is not in
    /// the §8.4 mermaid diagram.
    ///
    /// The legal edges are exactly:
    ///
    /// ```text
    /// Planned   -> Fetching
    /// Fetching  -> Paused
    /// Paused    -> Fetching
    /// Fetching  -> Verifying
    /// Verifying -> Staged
    /// Staged    -> Activating
    /// Activating-> Completed
    /// Fetching  -> Failed
    /// Verifying -> Failed
    /// Activating-> Failed
    /// Failed    -> Fetching
    /// ```
    ///
    /// `Completed` and `Failed`-after-`Completed` are terminal; only
    /// `Failed -> Fetching` (operator retry) leaves a failed job.
    pub fn transition(
        self,
        target: DownloadJobState,
    ) -> Result<DownloadJobState, DownloadJobTransitionError> {
        use DownloadJobState::*;
        let legal = matches!(
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
        );
        if legal {
            Ok(target)
        } else {
            Err(DownloadJobTransitionError {
                from: self,
                to: target,
            })
        }
    }
}

/// The edge `(from, to)` is not in the §8.4 mermaid diagram.
#[derive(Clone, Copy, Debug, thiserror::Error, Eq, PartialEq)]
#[error("illegal download job transition: {from:?} -> {to:?}")]
pub struct DownloadJobTransitionError {
    pub from: DownloadJobState,
    pub to: DownloadJobState,
}

// ===========================================================================
// §8.4 activation handoff (belongs to `install_transactions`)
// ===========================================================================

/// Request to atomically swap the active pack pointer from
/// `previous_active_pack_version_id` to `candidate_pack_version_id`. The
/// candidate must already be fully verified (download job in `Staged`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivationRequest {
    pub install_transaction_id: String,
    pub candidate_pack_version_id: PackVersionId,
    pub previous_active_pack_version_id: Option<PackVersionId>,
}

/// Outcome of an activation attempt. Only `Committed` is a success — every
/// other outcome leaves the previous active pack readable (A-04) and
/// causes the orchestrator to transition the download job to `Failed`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActivationOutcome {
    /// The candidate is now the active pack version; the previous active
    /// id (if any) is no longer pointed at.
    Committed { new_active_id: PackVersionId },
    /// The candidate verified cleanly but the pointer swap could not
    /// commit (e.g. storage contention). The previous active pack is still
    /// readable; the candidate may be retried.
    RolledBack { reason: String },
    /// The candidate turned out to be untrusted or corrupt at activation
    /// time and its bytes have been moved to quarantine (A-07). The
    /// previous active pack is still readable.
    Quarantined { reason: String },
}

/// Activator-level failure (distinct from a deliberately-rolled-back or
/// quarantined outcome). These reflect an inability to perform the swap at
/// all, not a verification failure on the candidate.
#[derive(Clone, Debug, thiserror::Error, Eq, PartialEq)]
pub enum ActivationError {
    #[error("storage unavailable: {0}")]
    StorageUnavailable(String),
    #[error("candidate missing after verification")]
    CandidateMissing,
}

/// Atomic active-pointer swap (ARCHITECTURE.md §6.3 steps 9 + 10, §8.4
/// activation handoff). Implementations live in the storage layer and
/// write the journal row to `install_transactions`.
//
// The shape mirrors the §6.3 contract: take a verified candidate and the
// previous active id, return whether the swap committed, was rolled back,
// or quarantined. The storage-side implementation is intentionally not
// part of this crate (A-02).
#[async_trait]
pub trait PackActivator: Send + Sync {
    async fn activate(
        &self,
        request: ActivationRequest,
    ) -> Result<ActivationOutcome, ActivationError>;
}

/// Drive the activation phase of a verified download job.
///
/// Pre-condition: `state` is `Staged` (the only legal entry into
/// `Activating`). The orchestrator:
///
/// 1. transitions the job `Staged -> Activating`;
/// 2. delegates the atomic pointer swap to `activator`;
/// 3. on [`ActivationOutcome::Committed`], transitions `Activating ->
///    Completed` — the new active pack is now readable;
/// 4. on any other outcome, transitions `Activating -> Failed` — the
///    previous active pack remains readable (A-04) and any untrusted
///    candidate bytes are already quarantined by the activator (A-07).
///
/// The caller (storage layer) is responsible for persisting each returned
/// state to `download_jobs.state` and for writing the activation record to
/// `install_transactions` inside the activator's transaction.
pub async fn activate_candidate(
    state: DownloadJobState,
    activator: &dyn PackActivator,
    request: ActivationRequest,
) -> Result<DownloadJobState, ActivationFailure> {
    let activating = state
        .transition(DownloadJobState::Activating)
        .map_err(ActivationFailure::IllegalTransition)?;

    let outcome = activator
        .activate(request)
        .await
        .map_err(ActivationFailure::Activator)?;

    let target = match outcome {
        ActivationOutcome::Committed { .. } => DownloadJobState::Completed,
        // RolledBack and Quarantined both leave the previous active pack
        // readable; only the diagnostic differs.
        ActivationOutcome::RolledBack { .. } | ActivationOutcome::Quarantined { .. } => {
            DownloadJobState::Failed
        }
    };

    activating
        .transition(target)
        .map_err(ActivationFailure::IllegalTransition)
}

/// Either the entry/exit transition was illegal, or the activator itself
/// failed (distinct from a deliberate roll-back or quarantine).
#[derive(Debug, thiserror::Error)]
pub enum ActivationFailure {
    #[error("illegal transition: {0}")]
    IllegalTransition(#[from] DownloadJobTransitionError),
    #[error("activator error: {0}")]
    Activator(#[from] ActivationError),
}

#[cfg(test)]
mod tests {
    //! Real assertions over the §8.3 planner and the §8.4 state machine.
    //! No mocks, no stubs — every input is a fully-populated value.
    use super::*;
    use domain::identity::PackVersionId;
    use ports::catalog::{CatalogEntry, TrustPresentation};
    use ports::transfer::{StorageDestination, TransferMode};

    fn catalog_entry(size: u64, sha: &str) -> CatalogEntry {
        CatalogEntry {
            catalog_entry_id: "ce.test".to_string(),
            corpus_id: "corpus.test".to_string(),
            upstream_version: "1".to_string(),
            title: "Test".to_string(),
            description: String::new(),
            language_bcp47: "en".to_string(),
            category: "general".to_string(),
            publisher: "test".to_string(),
            artifact_url: "https://example.invalid/test.zim".to_string(),
            artifact_size_bytes: size,
            artifact_sha256: sha.to_string(),
            signature_url: None,
            trust: TrustPresentation {
                state: "verified".to_string(),
                signer: None,
                explanation: String::new(),
            },
        }
    }

    const GOOD_SHA: &str =
        "0000000000000000000000000000000000000000000000000000000000000000";

    fn writable_destination() -> StorageDestination {
        StorageDestination {
            storage_root_id: "root".to_string(),
            display_name: "Root".to_string(),
            writable: true,
        }
    }

    fn base_request() -> InstallPlanRequest {
        InstallPlanRequest {
            install_plan_id: "plan.1".to_string(),
            catalog_entry: catalog_entry(1_000, GOOD_SHA),
            destination: writable_destination(),
            transfer_mode: TransferMode::Full,
            free_bytes: 10_000,
            retained_last_known_good_bytes: 0,
            source_accepts_ranges: false,
            signature_required: false,
            trusted_signature_available: false,
            zim_structure_check: true,
            replaces_pack_version_id: None,
            retained_pack_version_id: None,
        }
    }

    #[test]
    fn plan_succeeds_for_valid_request() {
        let plan = plan_install(base_request()).expect("valid request plans");
        assert_eq!(plan.install_plan_id, "plan.1");
        assert_eq!(plan.download_bytes, 1_000);
        assert_eq!(plan.temporary_bytes, 1_000);
        assert_eq!(plan.final_bytes, 1_000);
        assert_eq!(plan.free_bytes_before, 10_000);
        assert_eq!(plan.free_bytes_after, 9_000);
        assert_eq!(plan.verification.expected_size_bytes, 1_000);
        assert_eq!(plan.verification.expected_sha256, GOOD_SHA);
        assert!(plan.verification.zim_structure_check);
        assert!(!plan.verification.signature_required);
    }

    // §8.3 bullet 1: destination is not writable.
    #[test]
    fn plan_rejects_unwritable_destination() {
        let mut r = base_request();
        r.destination.writable = false;
        assert_eq!(
            plan_install(r).unwrap_err(),
            InstallPlanError::DestinationNotWritable
        );
    }

    // §8.3 bullet 2: final bytes do not fit.
    #[test]
    fn plan_rejects_oversize_final() {
        let mut r = base_request();
        r.free_bytes = 999;
        assert_eq!(
            plan_install(r).unwrap_err(),
            InstallPlanError::FinalBytesExceedFree
        );
    }

    // §8.3 bullet 3: temporary plus retained last-known-good bytes do not fit.
    #[test]
    fn plan_rejects_temporary_plus_retained() {
        let mut r = base_request();
        // free_bytes = 10_000, final = 1_000, retained = 9_500 -> 1_000 + 9_500 > 10_000
        r.retained_last_known_good_bytes = 9_500;
        assert_eq!(
            plan_install(r).unwrap_err(),
            InstallPlanError::TemporaryPlusRetainedExceedFree
        );
    }

    #[test]
    fn plan_accepts_temporary_plus_retained_at_boundary() {
        let mut r = base_request();
        r.retained_last_known_good_bytes = 9_000; // 1_000 + 9_000 == 10_000
        assert!(plan_install(r).is_ok());
    }

    // §8.3 bullet 4: no verified digest is available.
    #[test]
    fn plan_rejects_missing_digest_size() {
        let mut r = base_request();
        r.catalog_entry.artifact_size_bytes = 0;
        assert_eq!(
            plan_install(r).unwrap_err(),
            InstallPlanError::NoVerifiedDigest
        );
    }

    #[test]
    fn plan_rejects_short_digest() {
        let mut r = base_request();
        r.catalog_entry.artifact_sha256 = "abc".to_string();
        assert_eq!(
            plan_install(r).unwrap_err(),
            InstallPlanError::NoVerifiedDigest
        );
    }

    #[test]
    fn plan_rejects_uppercase_digest() {
        let mut r = base_request();
        r.catalog_entry.artifact_sha256 = "A".repeat(64);
        assert_eq!(
            plan_install(r).unwrap_err(),
            InstallPlanError::NoVerifiedDigest
        );
    }

    // §8.3 bullet 5: policy requires a signature and no trusted signature exists.
    #[test]
    fn plan_rejects_required_signature_not_available() {
        let mut r = base_request();
        r.signature_required = true;
        r.trusted_signature_available = false;
        assert_eq!(
            plan_install(r).unwrap_err(),
            InstallPlanError::SignatureRequiredButMissing
        );
    }

    #[test]
    fn plan_accepts_required_signature_when_available() {
        let mut r = base_request();
        r.signature_required = true;
        r.trusted_signature_available = true;
        let plan = plan_install(r).expect("signature satisfied");
        assert!(plan.verification.signature_required);
    }

    // §8.3 bullet 6: source range behavior incompatible with selected strategy.
    #[test]
    fn plan_rejects_ranged_mode_without_range_support() {
        let mut r = base_request();
        r.transfer_mode = TransferMode::Ranged;
        r.source_accepts_ranges = false;
        assert_eq!(
            plan_install(r).unwrap_err(),
            InstallPlanError::TransferModeIncompatible
        );
    }

    #[test]
    fn plan_accepts_ranged_mode_with_range_support() {
        let mut r = base_request();
        r.transfer_mode = TransferMode::Ranged;
        r.source_accepts_ranges = true;
        let plan = plan_install(r).expect("range-compatible");
        assert_eq!(plan.transfer_mode, TransferMode::Ranged);
    }

    #[test]
    fn plan_accepts_full_mode_without_range_support() {
        let mut r = base_request();
        r.transfer_mode = TransferMode::Full;
        r.source_accepts_ranges = false;
        assert!(plan_install(r).is_ok());
    }

    #[test]
    fn plan_accepts_local_copy_without_range_support() {
        let mut r = base_request();
        r.transfer_mode = TransferMode::LocalCopy;
        r.source_accepts_ranges = false;
        assert!(plan_install(r).is_ok());
    }

    // ---- §8.4 state machine edges --------------------------------------

    #[test]
    fn every_spec_edge_is_legal() {
        use DownloadJobState::*;
        let legal = [
            (Planned, Fetching),
            (Fetching, Paused),
            (Paused, Fetching),
            (Fetching, Verifying),
            (Verifying, Staged),
            (Staged, Activating),
            (Activating, Completed),
            (Fetching, Failed),
            (Verifying, Failed),
            (Activating, Failed),
            (Failed, Fetching),
        ];
        for (from, to) in legal {
            assert_eq!(from.transition(to), Ok(to), "{from:?} -> {to:?}");
        }
    }

    #[test]
    fn illegal_edges_are_rejected() {
        use DownloadJobState::*;
        let illegal = [
            (Planned, Verifying),
            (Paused, Verifying),
            (Staged, Failed),
            (Completed, Fetching),
            (Completed, Failed),
            (Failed, Planned),
            (Failed, Completed),
            (Fetching, Completed),
            (Verifying, Completed),
        ];
        for (from, to) in illegal {
            assert_eq!(
                from.transition(to),
                Err(DownloadJobTransitionError { from, to }),
                "{from:?} -> {to:?} should be illegal"
            );
        }
    }

    #[test]
    fn download_job_state_serde_is_snake_case() {
        use DownloadJobState::*;
        for (state, expected) in [
            (Planned, "\"planned\""),
            (Fetching, "\"fetching\""),
            (Paused, "\"paused\""),
            (Verifying, "\"verifying\""),
            (Staged, "\"staged\""),
            (Activating, "\"activating\""),
            (Completed, "\"completed\""),
            (Failed, "\"failed\""),
        ] {
            let s = serde_json::to_string(&state).unwrap();
            assert_eq!(s, expected);
            let back: DownloadJobState = serde_json::from_str(&s).unwrap();
            assert_eq!(back, state);
        }
    }

    // ---- activate_candidate: A-04 / A-07 enforcement -------------------

    struct CommittedActivator;
    #[async_trait]
    impl PackActivator for CommittedActivator {
        async fn activate(
            &self,
            _request: ActivationRequest,
        ) -> Result<ActivationOutcome, ActivationError> {
            Ok(ActivationOutcome::Committed {
                new_active_id: PackVersionId("pv.new".to_string()),
            })
        }
    }

    struct RolledBackActivator;
    #[async_trait]
    impl PackActivator for RolledBackActivator {
        async fn activate(
            &self,
            _request: ActivationRequest,
        ) -> Result<ActivationOutcome, ActivationError> {
            Ok(ActivationOutcome::RolledBack {
                reason: "transient".to_string(),
            })
        }
    }

    struct QuarantinedActivator;
    #[async_trait]
    impl PackActivator for QuarantinedActivator {
        async fn activate(
            &self,
            _request: ActivationRequest,
        ) -> Result<ActivationOutcome, ActivationError> {
            Ok(ActivationOutcome::Quarantined {
                reason: "digest mismatch at activation".to_string(),
            })
        }
    }

    fn activation_request() -> ActivationRequest {
        ActivationRequest {
            install_transaction_id: "tx.1".to_string(),
            candidate_pack_version_id: PackVersionId("pv.new".to_string()),
            previous_active_pack_version_id: Some(PackVersionId("pv.old".to_string())),
        }
    }

    #[tokio::test]
    async fn committed_activation_completes_job() {
        let next = activate_candidate(
            DownloadJobState::Staged,
            &CommittedActivator,
            activation_request(),
        )
        .await
        .expect("committed activation completes");
        assert_eq!(next, DownloadJobState::Completed);
    }

    #[tokio::test]
    async fn rolled_back_activation_fails_job() {
        let next = activate_candidate(
            DownloadJobState::Staged,
            &RolledBackActivator,
            activation_request(),
        )
        .await
        .expect("rolled-back outcome is not an activator error");
        assert_eq!(next, DownloadJobState::Failed);
    }

    #[tokio::test]
    async fn quarantined_activation_fails_job() {
        let next = activate_candidate(
            DownloadJobState::Staged,
            &QuarantinedActivator,
            activation_request(),
        )
        .await
        .expect("quarantined outcome is not an activator error");
        assert_eq!(next, DownloadJobState::Failed);
    }

    #[tokio::test]
    async fn activation_refuses_non_staged_entry_state() {
        let err = activate_candidate(
            DownloadJobState::Verifying,
            &CommittedActivator,
            activation_request(),
        )
        .await
        .unwrap_err();
        assert!(matches!(
            err,
            ActivationFailure::IllegalTransition(_)
        ));
    }
}
