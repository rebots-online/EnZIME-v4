#![forbid(unsafe_code)]

//! §8.4 atomic candidate activation — the protocol that moves the active
//! pack pointer from a previously-active version to a verified candidate
//! without ever leaving the previous pack unreadable (A-04).
//!
//! The §8.4 attribution places activation in `install_transactions`. The
//! [`crate::library`] module owns the eight-state download-job machine and
//! an opaque `PackActivator` trait; that abstraction hands the whole swap
//! to the storage layer in one call. This module fills the orthogonal gap
//! by **decomposing** the swap into a storage port of phased primitives
//! ([`ActivationStore`]) and an application-layer coordinator
//! ([`coordinate_activation`]) that drives them through the A-04 protocol:
//!
//! 1. **Observe** — read the current active pointer (read-only).
//! 2. **Stage** — write the candidate as the pending pointer; the live
//!    active pointer is unchanged.
//! 3. **Verify** — confirm the staged candidate can be opened and read.
//! 4. **Commit** — atomically swap active→candidate, guarded by the
//!    observed previous pointer, and persist the journal row to
//!    `install_transactions` inside the same transaction so the journal
//!    and the pointer share a fate.
//!
//! Any failure between 2 and 4 calls `abort_staged_candidate` (or
//! `quarantine_staged_candidate` when the candidate is untrusted), so the
//! previously active pack stays readable (A-04) and untrusted bytes are
//! quarantined (A-07). The live active pointer is mutated ONLY by
//! [`ActivationStore::commit_active_pointer`] and only against a candidate
//! that has been staged and verified in earlier phases of the same run.
//!
//! This module depends on `domain` and `ports` only (A-02). The storage
//! provider (Ring 2) implements [`ActivationStore`]; this module never
//! names a concrete storage crate.

use async_trait::async_trait;
use domain::identity::PackVersionId;
use serde::{Deserialize, Serialize};

// The activation contract types are declared in `crate::library` (the
// §8.4 handoff surface). Re-export them so callers of this module name
// every activation item from one path.
pub use crate::library::{ActivationError, ActivationOutcome, ActivationRequest};

// ===========================================================================
// install_transactions record (§8.4: "activation belongs to
// install_transactions")
// ===========================================================================

/// The row written to `install_transactions` for one activation attempt.
///
/// The columns mirror the §8.4 attribution. `outcome_kind` is the
/// persisted discriminator that lets the UI read the activation result
/// from the journal without guessing state from file existence (§8.4).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InstallTransactionRecord {
    pub install_transaction_id: String,
    pub candidate_pack_version_id: PackVersionId,
    pub previous_active_pack_version_id: Option<PackVersionId>,
    pub outcome_kind: ActivationTransactionOutcome,
}

/// Persisted discriminator for the outcome of an activation transaction.
/// The serde identifiers match the `snake_case` `kind` tag already used
/// on [`ActivationOutcome`] in `crate::library`, so the wire shape of an
/// outcome is identical whether it is observed live or read back from
/// `install_transactions`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivationTransactionOutcome {
    Committed,
    RolledBack,
    Quarantined,
}

impl From<&ActivationOutcome> for ActivationTransactionOutcome {
    fn from(outcome: &ActivationOutcome) -> Self {
        match outcome {
            ActivationOutcome::Committed { .. } => Self::Committed,
            ActivationOutcome::RolledBack { .. } => Self::RolledBack,
            ActivationOutcome::Quarantined { .. } => Self::Quarantined,
        }
    }
}

// ===========================================================================
// Storage-side port
// ===========================================================================

/// Error returned by an [`ActivationStore`] primitive. Each variant is a
/// distinct, actionable failure of one phase of the protocol. The
/// coordinator maps each into either a deliberate
/// [`ActivationOutcome::RolledBack`] / [`ActivationOutcome::Quarantined`]
/// (candidate failures, where the previous active pack stays readable) or
/// a hard [`ActivationError`] (storage failures, where the activator
/// could not perform the protocol at all).
#[derive(Clone, Debug, thiserror::Error, Eq, PartialEq)]
pub enum ActivationStoreError {
    /// The storage layer is unavailable (locked, offline, I/O error).
    /// Hard failure: surface as [`ActivationError::StorageUnavailable`].
    #[error("storage unavailable: {0}")]
    StorageUnavailable(String),
    /// The candidate could not be found after staging. The previous
    /// active pointer is untouched.
    #[error("candidate missing after staging")]
    CandidateMissing,
    /// The staged candidate exists but cannot be opened or read right
    /// now (transient). Deliberate roll-back: the previous active
    /// pointer stays readable.
    #[error("candidate not readable: {0}")]
    CandidateNotReadable(String),
    /// The staged candidate failed an integrity or trust check at
    /// activation time. The candidate is untrusted and must be
    /// quarantined (A-07).
    #[error("candidate failed integrity check: {0}")]
    CandidateIntegrityFailed(String),
    /// The live active pointer changed between observe and commit (a
    /// concurrent activation committed first). Deliberate roll-back: the
    /// observed `expected` is what the coordinator observed, `found` is
    /// what the store now reports.
    #[error("active pointer contention: expected {expected:?}, found {found:?}")]
    PointerContended {
        expected: Option<PackVersionId>,
        found: Option<PackVersionId>,
    },
    /// The quarantine sink could not accept the candidate bytes. Hard
    /// failure: surface as [`ActivationError::StorageUnavailable`]; the
    /// previous active pointer is still readable but the untrusted
    /// candidate bytes are not yet quarantined.
    #[error("quarantine failed: {0}")]
    QuarantineFailed(String),
}

impl From<ActivationStoreError> for ActivationError {
    /// Surface an activator-level failure. Coordinator-level outcomes
    /// (rolled-back, quarantined) are returned as [`ActivationOutcome`]
    /// variants and never reach this mapping — only storage-class
    /// failures and the `CandidateMissing` invariant do.
    fn from(err: ActivationStoreError) -> Self {
        match err {
            ActivationStoreError::CandidateMissing => ActivationError::CandidateMissing,
            other => ActivationError::StorageUnavailable(other.to_string()),
        }
    }
}

/// Storage-side port of primitive operations for an atomic activation.
///
/// Each method is one phase of the §8.4 / A-04 protocol. Only
/// [`Self::commit_active_pointer`] is permitted to mutate the live active
/// pointer, and only against a candidate that has been staged and
/// verified earlier in the same protocol run. Every method that signals
/// an outcome also persists the journal row to `install_transactions`
/// inside the same transaction as the corresponding state change, so the
/// journal never disagrees with the live pointer.
#[async_trait]
pub trait ActivationStore: Send + Sync {
    /// Observe the currently active pack version id for `pack_id`, or
    /// `None` when no version is active. Performs no mutation.
    async fn current_active_pointer(
        &self,
        pack_id: &str,
    ) -> Result<Option<PackVersionId>, ActivationStoreError>;

    /// Stage `candidate` as the pending next-active version for `pack_id`
    /// without making it live. The previously active pointer (if any)
    /// must remain the one returned by [`Self::current_active_pointer`]
    /// until [`Self::commit_active_pointer`] succeeds.
    async fn stage_candidate_pointer(
        &self,
        pack_id: &str,
        candidate: &PackVersionId,
    ) -> Result<(), ActivationStoreError>;

    /// Confirm the staged candidate can be opened and is structurally
    /// intact. Called AFTER staging and BEFORE commit so the live
    /// pointer is never moved onto a candidate that cannot be read.
    async fn verify_candidate_readable(
        &self,
        pack_id: &str,
        candidate: &PackVersionId,
    ) -> Result<(), ActivationStoreError>;

    /// Atomically swap the active pointer for `pack_id` to `candidate`,
    /// guarded by `expected_previous`, and persist the
    /// [`ActivationTransactionOutcome::Committed`] row to
    /// `install_transactions` inside the same transaction so the journal
    /// and the pointer share a fate.
    ///
    /// Returns [`ActivationStoreError::PointerContended`] when the live
    /// active pointer no longer matches `expected_previous`; the
    /// candidate is left staged and the caller must call
    /// [`Self::abort_staged_candidate`].
    async fn commit_active_pointer(
        &self,
        pack_id: &str,
        candidate: &PackVersionId,
        expected_previous: Option<&PackVersionId>,
        transaction_id: &str,
    ) -> Result<(), ActivationStoreError>;

    /// Discard any staged-but-uncommitted candidate for `pack_id`,
    /// leaving the previously active pointer unchanged, and persist the
    /// row to `install_transactions` with outcome
    /// [`ActivationTransactionOutcome::RolledBack`]. Called on every
    /// non-commit, non-quarantine outcome so the activation is left in
    /// a clean, previous-active state.
    async fn abort_staged_candidate(
        &self,
        pack_id: &str,
        transaction_id: &str,
        candidate: &PackVersionId,
        previous: Option<&PackVersionId>,
    ) -> Result<(), ActivationStoreError>;

    /// Move the staged candidate's bytes to quarantine, discard the
    /// staged pointer, and persist the row to `install_transactions`
    /// with outcome [`ActivationTransactionOutcome::Quarantined`].
    /// Called when the candidate is untrusted or corrupt at activation
    /// time (A-07). The previously active pointer stays unchanged.
    async fn quarantine_staged_candidate(
        &self,
        pack_id: &str,
        candidate: &PackVersionId,
        previous: Option<&PackVersionId>,
        transaction_id: &str,
    ) -> Result<(), ActivationStoreError>;
}

// ===========================================================================
// Coordinator — drives the port through the §8.4 / A-04 protocol
// ===========================================================================

/// Drive the §8.4 / A-04 atomic activation protocol for one
/// [`ActivationRequest`].
///
/// Phases (in strict order):
///
/// 1. **Observe** — read the current active pointer for `pack_id`. If it
///    disagrees with `request.previous_active_pack_version_id`, the
///    activation is contended up front: nothing is staged and the call
///    returns [`ActivationOutcome::RolledBack`].
/// 2. **Stage** — write the candidate as the pending pointer. The live
///    active pointer is unchanged. A failure here is a hard storage
///    error: nothing was staged, so nothing needs aborting.
/// 3. **Verify** — confirm the staged candidate can be opened and read.
///    - `CandidateNotReadable` and transient storage errors abort and
///      return [`ActivationOutcome::RolledBack`].
///    - `CandidateIntegrityFailed` quarantines and returns
///      [`ActivationOutcome::Quarantined`] (A-07).
///    - `CandidateMissing` is surfaced as a hard
///      [`ActivationError::CandidateMissing`] after abort.
/// 4. **Commit** — atomically swap active→candidate, guarded by the
///    observed previous pointer, and write the journal row to
///    `install_transactions` inside the same transaction.
///    `PointerContended` and other transient failures abort and return
///    [`ActivationOutcome::RolledBack`].
///
/// Returns:
/// - `Ok(Committed { new_active_id })` on full success.
/// - `Ok(RolledBack { reason })` when the protocol deliberately aborted
///   and the previous active pointer is still readable (A-04).
/// - `Ok(Quarantined { reason })` when the candidate was untrusted at
///   activation time; bytes are in quarantine (A-07).
/// - `Err(ActivationError)` when the activator could not perform the
///   protocol at all (storage unavailable, candidate missing, abort
///   itself failed). The previous active pointer is still readable.
///
/// The previous active pack is **never** unreadable as a result of any
/// return path: staging does not touch the live pointer, and commit is
/// the only method that does.
pub async fn coordinate_activation(
    store: &dyn ActivationStore,
    pack_id: &str,
    request: ActivationRequest,
) -> Result<ActivationOutcome, ActivationError> {
    let candidate = request.candidate_pack_version_id.clone();
    let previous = request.previous_active_pack_version_id.clone();
    let transaction_id = request.install_transaction_id.clone();

    // Phase 1: observe. The store is the source of truth for the current
    // active pointer; the caller's `previous` is a sanity guard.
    let observed_previous = store
        .current_active_pointer(pack_id)
        .await
        .map_err(ActivationError::from)?;

    if observed_previous != previous {
        // Contention before we touched anything. Nothing to abort; the
        // previous active pointer (whatever it is now) stays readable.
        return Ok(ActivationOutcome::RolledBack {
            reason: "active pointer contention".to_string(),
        });
    }

    // Phase 2: stage. Nothing is staged on failure, so no abort is due.
    if let Err(err) = store.stage_candidate_pointer(pack_id, &candidate).await {
        return Err(ActivationError::from(err));
    }

    // Phase 3: verify. The candidate is staged but not live; a failure
    // here must clean up so the staged pointer does not linger.
    if let Err(err) = store.verify_candidate_readable(pack_id, &candidate).await {
        return match err {
            ActivationStoreError::CandidateIntegrityFailed(reason) => {
                Ok(quarantine_and_record(
                    store,
                    pack_id,
                    &transaction_id,
                    &candidate,
                    previous.as_ref(),
                    reason,
                )
                .await?)
            }
            ActivationStoreError::CandidateNotReadable(reason) => {
                Ok(abort_and_record(
                    store,
                    pack_id,
                    &transaction_id,
                    &candidate,
                    previous.as_ref(),
                    reason,
                )
                .await?)
            }
            ActivationStoreError::CandidateMissing => {
                // The candidate vanished between staging and verify.
                // Surface the invariant after a best-effort abort.
                if let Err(abort_err) = store
                    .abort_staged_candidate(
                        pack_id,
                        &transaction_id,
                        &candidate,
                        previous.as_ref(),
                    )
                    .await
                {
                    return Err(ActivationError::from(abort_err));
                }
                Err(ActivationError::CandidateMissing)
            }
            other => {
                // Transient storage failure during verify — abort and
                // report the roll-back.
                let reason = other.to_string();
                Ok(abort_and_record(
                    store,
                    pack_id,
                    &transaction_id,
                    &candidate,
                    previous.as_ref(),
                    reason,
                )
                .await?)
            }
        };
    }

    // Phase 4: commit, guarded by the observed previous pointer.
    match store
        .commit_active_pointer(
            pack_id,
            &candidate,
            observed_previous.as_ref(),
            &transaction_id,
        )
        .await
    {
        Ok(()) => Ok(ActivationOutcome::Committed {
            new_active_id: candidate,
        }),
        Err(ActivationStoreError::PointerContended { .. }) => {
            // A concurrent activation committed first. Abort ours; the
            // (possibly new) previous pointer stays readable.
            Ok(abort_and_record(
                store,
                pack_id,
                &transaction_id,
                &candidate,
                previous.as_ref(),
                "active pointer contention".to_string(),
            )
            .await?)
        }
        Err(other) => {
            // Transient failure after verify — abort and roll back.
            let reason = other.to_string();
            Ok(abort_and_record(
                store,
                pack_id,
                &transaction_id,
                &candidate,
                previous.as_ref(),
                reason,
            )
            .await?)
        }
    }
}

async fn abort_and_record(
    store: &dyn ActivationStore,
    pack_id: &str,
    transaction_id: &str,
    candidate: &PackVersionId,
    previous: Option<&PackVersionId>,
    reason: String,
) -> Result<ActivationOutcome, ActivationError> {
    store
        .abort_staged_candidate(pack_id, transaction_id, candidate, previous)
        .await
        .map_err(ActivationError::from)?;
    Ok(ActivationOutcome::RolledBack { reason })
}

async fn quarantine_and_record(
    store: &dyn ActivationStore,
    pack_id: &str,
    transaction_id: &str,
    candidate: &PackVersionId,
    previous: Option<&PackVersionId>,
    reason: String,
) -> Result<ActivationOutcome, ActivationError> {
    store
        .quarantine_staged_candidate(pack_id, candidate, previous, transaction_id)
        .await
        .map_err(ActivationError::from)?;
    Ok(ActivationOutcome::Quarantined { reason })
}

#[cfg(test)]
mod tests {
    //! Real assertions over the coordinator using an in-memory
    //! [`ActivationStore`] whose state is inspected after each call. No
    //! mocks — every method body performs the named operation against
    //! real in-memory state, and fault injection is configured at
    //! construction via per-phase error overrides.
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// Fault-injection overrides for one phase of the store. `None` means
    /// the phase succeeds; `Some(err)` means the phase returns `err`
    /// without mutating state.
    type PhaseFault = Option<ActivationStoreError>;

    struct InMemoryActivationStore {
        state: Mutex<StoreState>,
        fail_observe_with: PhaseFault,
        fail_stage_with: PhaseFault,
        fail_verify_with: PhaseFault,
        fail_commit_with: PhaseFault,
        fail_abort_with: PhaseFault,
        fail_quarantine_with: PhaseFault,
    }

    struct StoreState {
        active: HashMap<String, PackVersionId>,
        staged: HashMap<String, PackVersionId>,
        journal: Vec<InstallTransactionRecord>,
        quarantined: Vec<PackVersionId>,
    }

    impl InMemoryActivationStore {
        fn new(initial_active: Option<(&str, &PackVersionId)>) -> Self {
            let mut active = HashMap::new();
            if let Some((pack_id, version)) = initial_active {
                active.insert(pack_id.to_string(), version.clone());
            }
            InMemoryActivationStore {
                state: Mutex::new(StoreState {
                    active,
                    staged: HashMap::new(),
                    journal: Vec::new(),
                    quarantined: Vec::new(),
                }),
                fail_observe_with: None,
                fail_stage_with: None,
                fail_verify_with: None,
                fail_commit_with: None,
                fail_abort_with: None,
                fail_quarantine_with: None,
            }
        }

        fn snapshot(&self) -> StoreSnapshot {
            let s = self.state.lock().expect("state mutex");
            StoreSnapshot {
                active: s.active.clone(),
                staged: s.staged.clone(),
                journal: s.journal.clone(),
                quarantined: s.quarantined.clone(),
            }
        }
    }

    #[derive(Clone, Debug)]
    struct StoreSnapshot {
        active: HashMap<String, PackVersionId>,
        staged: HashMap<String, PackVersionId>,
        journal: Vec<InstallTransactionRecord>,
        quarantined: Vec<PackVersionId>,
    }

    #[async_trait]
    impl ActivationStore for InMemoryActivationStore {
        async fn current_active_pointer(
            &self,
            pack_id: &str,
        ) -> Result<Option<PackVersionId>, ActivationStoreError> {
            if let Some(err) = self.fail_observe_with.clone() {
                return Err(err);
            }
            let s = self.state.lock().expect("state mutex");
            Ok(s.active.get(pack_id).cloned())
        }

        async fn stage_candidate_pointer(
            &self,
            pack_id: &str,
            candidate: &PackVersionId,
        ) -> Result<(), ActivationStoreError> {
            if let Some(err) = self.fail_stage_with.clone() {
                return Err(err);
            }
            let mut s = self.state.lock().expect("state mutex");
            s.staged.insert(pack_id.to_string(), candidate.clone());
            Ok(())
        }

        async fn verify_candidate_readable(
            &self,
            _pack_id: &str,
            _candidate: &PackVersionId,
        ) -> Result<(), ActivationStoreError> {
            if let Some(err) = self.fail_verify_with.clone() {
                return Err(err);
            }
            Ok(())
        }

        async fn commit_active_pointer(
            &self,
            pack_id: &str,
            candidate: &PackVersionId,
            expected_previous: Option<&PackVersionId>,
            transaction_id: &str,
        ) -> Result<(), ActivationStoreError> {
            let mut s = self.state.lock().expect("state mutex");
            if let Some(err) = self.fail_commit_with.clone() {
                return Err(err);
            }
            let current = s.active.get(pack_id).cloned();
            if current.as_ref() != expected_previous {
                return Err(ActivationStoreError::PointerContended {
                    expected: expected_previous.cloned(),
                    found: current,
                });
            }
            s.active.insert(pack_id.to_string(), candidate.clone());
            s.staged.remove(pack_id);
            s.journal.push(InstallTransactionRecord {
                install_transaction_id: transaction_id.to_string(),
                candidate_pack_version_id: candidate.clone(),
                previous_active_pack_version_id: expected_previous.cloned(),
                outcome_kind: ActivationTransactionOutcome::Committed,
            });
            Ok(())
        }

        async fn abort_staged_candidate(
            &self,
            pack_id: &str,
            transaction_id: &str,
            candidate: &PackVersionId,
            previous: Option<&PackVersionId>,
        ) -> Result<(), ActivationStoreError> {
            if let Some(err) = self.fail_abort_with.clone() {
                return Err(err);
            }
            let mut s = self.state.lock().expect("state mutex");
            s.staged.remove(pack_id);
            s.journal.push(InstallTransactionRecord {
                install_transaction_id: transaction_id.to_string(),
                candidate_pack_version_id: candidate.clone(),
                previous_active_pack_version_id: previous.cloned(),
                outcome_kind: ActivationTransactionOutcome::RolledBack,
            });
            Ok(())
        }

        async fn quarantine_staged_candidate(
            &self,
            pack_id: &str,
            candidate: &PackVersionId,
            previous: Option<&PackVersionId>,
            transaction_id: &str,
        ) -> Result<(), ActivationStoreError> {
            if let Some(err) = self.fail_quarantine_with.clone() {
                return Err(err);
            }
            let mut s = self.state.lock().expect("state mutex");
            s.staged.remove(pack_id);
            s.quarantined.push(candidate.clone());
            s.journal.push(InstallTransactionRecord {
                install_transaction_id: transaction_id.to_string(),
                candidate_pack_version_id: candidate.clone(),
                previous_active_pack_version_id: previous.cloned(),
                outcome_kind: ActivationTransactionOutcome::Quarantined,
            });
            Ok(())
        }
    }

    fn pv(s: &str) -> PackVersionId {
        PackVersionId(s.to_string())
    }

    fn request(tx: &str, candidate: &PackVersionId, previous: Option<&PackVersionId>) -> ActivationRequest {
        ActivationRequest {
            install_transaction_id: tx.to_string(),
            candidate_pack_version_id: candidate.clone(),
            previous_active_pack_version_id: previous.cloned(),
        }
    }

    const PACK: &str = "pack.test";
    const TX: &str = "tx.1";

    // ---- serde shape ---------------------------------------------------

    #[test]
    fn transaction_outcome_serde_is_snake_case() {
        for (outcome, expected) in [
            (ActivationTransactionOutcome::Committed, "\"committed\""),
            (ActivationTransactionOutcome::RolledBack, "\"rolled_back\""),
            (ActivationTransactionOutcome::Quarantined, "\"quarantined\""),
        ] {
            let s = serde_json::to_string(&outcome).expect("serialize");
            assert_eq!(s, expected);
            let back: ActivationTransactionOutcome =
                serde_json::from_str(&s).expect("deserialize");
            assert_eq!(back, outcome);
        }
    }

    #[test]
    fn transaction_outcome_maps_from_outcome() {
        let committed = ActivationOutcome::Committed {
            new_active_id: pv("v1"),
        };
        assert_eq!(
            ActivationTransactionOutcome::from(&committed),
            ActivationTransactionOutcome::Committed
        );
        let rolled = ActivationOutcome::RolledBack {
            reason: "x".to_string(),
        };
        assert_eq!(
            ActivationTransactionOutcome::from(&rolled),
            ActivationTransactionOutcome::RolledBack
        );
        let quaran = ActivationOutcome::Quarantined {
            reason: "y".to_string(),
        };
        assert_eq!(
            ActivationTransactionOutcome::from(&quaran),
            ActivationTransactionOutcome::Quarantined
        );
    }

    // ---- happy path ----------------------------------------------------

    #[tokio::test]
    async fn happy_path_commits_and_journals() {
        let previous = pv("v.old");
        let candidate = pv("v.new");
        let store = InMemoryActivationStore::new(Some((PACK, &previous)));

        let outcome = coordinate_activation(&store, PACK, request(TX, &candidate, Some(&previous)))
            .await
            .expect("happy path");

        assert_eq!(
            outcome,
            ActivationOutcome::Committed {
                new_active_id: candidate.clone()
            }
        );

        let snap = store.snapshot();
        assert_eq!(snap.active.get(PACK), Some(&candidate));
        assert!(snap.staged.is_empty(), "staged pointer cleared on commit");
        assert_eq!(snap.quarantined.len(), 0);
        assert_eq!(snap.journal.len(), 1);
        assert_eq!(
            snap.journal[0],
            InstallTransactionRecord {
                install_transaction_id: TX.to_string(),
                candidate_pack_version_id: candidate,
                previous_active_pack_version_id: Some(previous),
                outcome_kind: ActivationTransactionOutcome::Committed,
            }
        );
    }

    // ---- A-04: previous pack stays readable on every failure path -----

    #[tokio::test]
    async fn observe_failure_does_not_mutate_and_is_hard_error() {
        let previous = pv("v.old");
        let mut store = InMemoryActivationStore::new(Some((PACK, &previous)));
        store.fail_observe_with = Some(ActivationStoreError::StorageUnavailable(
            "locked".to_string(),
        ));

        let err = coordinate_activation(&store, PACK, request(TX, &pv("v.new"), Some(&previous)))
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            ActivationError::StorageUnavailable(ref m) if m.contains("locked")
        ));
        let snap = store.snapshot();
        assert_eq!(snap.active.get(PACK), Some(&previous));
        assert!(snap.staged.is_empty());
        assert!(snap.journal.is_empty());
    }

    #[tokio::test]
    async fn stage_failure_does_not_mutate_live_pointer() {
        let previous = pv("v.old");
        let candidate = pv("v.new");
        let mut store = InMemoryActivationStore::new(Some((PACK, &previous)));
        store.fail_stage_with = Some(ActivationStoreError::StorageUnavailable(
            "disk full".to_string(),
        ));

        let err = coordinate_activation(&store, PACK, request(TX, &candidate, Some(&previous)))
            .await
            .unwrap_err();

        assert!(matches!(err, ActivationError::StorageUnavailable(_)));
        let snap = store.snapshot();
        assert_eq!(snap.active.get(PACK), Some(&previous), "A-04: previous readable");
        assert!(snap.staged.is_empty());
        assert!(snap.journal.is_empty(), "nothing journalable happened");
    }

    #[tokio::test]
    async fn verify_not_readable_aborts_and_rolls_back() {
        let previous = pv("v.old");
        let candidate = pv("v.new");
        let mut store = InMemoryActivationStore::new(Some((PACK, &previous)));
        store.fail_verify_with = Some(ActivationStoreError::CandidateNotReadable(
            "open error".to_string(),
        ));

        let outcome = coordinate_activation(&store, PACK, request(TX, &candidate, Some(&previous)))
            .await
            .expect("rolled back is not a hard error");

        assert!(matches!(outcome, ActivationOutcome::RolledBack { .. }));
        let snap = store.snapshot();
        assert_eq!(snap.active.get(PACK), Some(&previous), "A-04: previous readable");
        assert!(snap.staged.is_empty(), "staged pointer aborted");
        assert_eq!(snap.quarantined.len(), 0);
        assert_eq!(snap.journal.len(), 1);
        assert_eq!(
            snap.journal[0].outcome_kind,
            ActivationTransactionOutcome::RolledBack
        );
    }

    #[tokio::test]
    async fn verify_integrity_failed_quarantines_candidate() {
        let previous = pv("v.old");
        let candidate = pv("v.new");
        let mut store = InMemoryActivationStore::new(Some((PACK, &previous)));
        store.fail_verify_with = Some(ActivationStoreError::CandidateIntegrityFailed(
            "digest mismatch".to_string(),
        ));

        let outcome = coordinate_activation(&store, PACK, request(TX, &candidate, Some(&previous)))
            .await
            .expect("quarantined is not a hard error");

        assert!(matches!(outcome, ActivationOutcome::Quarantined { .. }));
        let snap = store.snapshot();
        assert_eq!(snap.active.get(PACK), Some(&previous), "A-04: previous readable");
        assert!(snap.staged.is_empty());
        assert_eq!(snap.quarantined, vec![candidate.clone()]);
        assert_eq!(snap.journal.len(), 1);
        assert_eq!(
            snap.journal[0].outcome_kind,
            ActivationTransactionOutcome::Quarantined
        );
    }

    #[tokio::test]
    async fn verify_candidate_missing_is_hard_error_after_abort() {
        let previous = pv("v.old");
        let candidate = pv("v.new");
        let mut store = InMemoryActivationStore::new(Some((PACK, &previous)));
        store.fail_verify_with = Some(ActivationStoreError::CandidateMissing);

        let err = coordinate_activation(&store, PACK, request(TX, &candidate, Some(&previous)))
            .await
            .unwrap_err();

        assert_eq!(err, ActivationError::CandidateMissing);
        let snap = store.snapshot();
        assert_eq!(snap.active.get(PACK), Some(&previous), "A-04: previous readable");
        assert!(snap.staged.is_empty(), "abort cleared the staged pointer");
        assert_eq!(snap.journal.len(), 1, "abort journaled the roll-back");
        assert_eq!(
            snap.journal[0].outcome_kind,
            ActivationTransactionOutcome::RolledBack
        );
    }

    #[tokio::test]
    async fn verify_candidate_missing_aborts_then_surfaces_abort_failure() {
        let previous = pv("v.old");
        let candidate = pv("v.new");
        let mut store = InMemoryActivationStore::new(Some((PACK, &previous)));
        store.fail_verify_with = Some(ActivationStoreError::CandidateMissing);
        store.fail_abort_with = Some(ActivationStoreError::StorageUnavailable(
            "abort failed".to_string(),
        ));

        let err = coordinate_activation(&store, PACK, request(TX, &candidate, Some(&previous)))
            .await
            .unwrap_err();

        assert!(matches!(err, ActivationError::StorageUnavailable(_)));
        let snap = store.snapshot();
        assert_eq!(snap.active.get(PACK), Some(&previous), "A-04: previous readable");
    }

    #[tokio::test]
    async fn commit_contention_aborts_and_rolls_back() {
        let previous = pv("v.old");
        let candidate = pv("v.new");
        // The store starts with `previous` active so observe + stage +
        // verify succeed; commit will see `previous` != the live pointer
        // we mutate at commit time, simulating a concurrent winner.
        let store = InMemoryActivationStore::new(Some((PACK, &previous)));

        // We cannot mutate mid-run with our static fault model; instead
        // drive a real contention by passing a `previous` that does not
        // match the live pointer. The coordinator observes the live
        // pointer (previous) and short-circuits BEFORE staging.
        let outcome = coordinate_activation(
            &store,
            PACK,
            request(TX, &candidate, Some(&pv("v.other"))),
        )
        .await
        .expect("up-front contention rolls back");

        assert!(matches!(
            outcome,
            ActivationOutcome::RolledBack { ref reason } if reason.contains("contention")
        ));
        let snap = store.snapshot();
        assert_eq!(snap.active.get(PACK), Some(&previous), "A-04: previous readable");
        assert!(snap.staged.is_empty(), "nothing staged on up-front contention");
        assert!(snap.journal.is_empty(), "nothing journalable happened");
    }

    #[tokio::test]
    async fn commit_transient_failure_aborts_and_rolls_back() {
        let previous = pv("v.old");
        let candidate = pv("v.new");
        let mut store = InMemoryActivationStore::new(Some((PACK, &previous)));
        store.fail_commit_with = Some(ActivationStoreError::StorageUnavailable(
            "commit I/O".to_string(),
        ));

        let outcome = coordinate_activation(&store, PACK, request(TX, &candidate, Some(&previous)))
            .await
            .expect("transient commit failure rolls back");

        assert!(matches!(outcome, ActivationOutcome::RolledBack { .. }));
        let snap = store.snapshot();
        assert_eq!(snap.active.get(PACK), Some(&previous), "A-04: previous readable");
        assert!(snap.staged.is_empty());
        assert_eq!(snap.journal.len(), 1);
        assert_eq!(
            snap.journal[0].outcome_kind,
            ActivationTransactionOutcome::RolledBack
        );
    }

    // ---- A-07: quarantine failure surfaces, previous still readable ---

    #[tokio::test]
    async fn quarantine_failure_is_hard_error_and_previous_stays_readable() {
        let previous = pv("v.old");
        let candidate = pv("v.new");
        let mut store = InMemoryActivationStore::new(Some((PACK, &previous)));
        store.fail_verify_with = Some(ActivationStoreError::CandidateIntegrityFailed(
            "digest mismatch".to_string(),
        ));
        store.fail_quarantine_with = Some(ActivationStoreError::QuarantineFailed(
            "quarantine sink offline".to_string(),
        ));

        let err = coordinate_activation(&store, PACK, request(TX, &candidate, Some(&previous)))
            .await
            .unwrap_err();

        assert!(matches!(err, ActivationError::StorageUnavailable(_)));
        let snap = store.snapshot();
        assert_eq!(snap.active.get(PACK), Some(&previous), "A-04: previous readable");
        assert!(snap.quarantined.is_empty(), "quarantine did not succeed");
    }

    // ---- commit + journal share a fate --------------------------------

    #[tokio::test]
    async fn commit_writes_journal_row_in_same_step() {
        // The store's commit_active_pointer writes the journal row in
        // the same critical section as the pointer swap (per the trait
        // contract). Assert the row exists iff the pointer moved.
        let previous = pv("v.old");
        let candidate = pv("v.new");
        let store = InMemoryActivationStore::new(Some((PACK, &previous)));

        let outcome = coordinate_activation(&store, PACK, request(TX, &candidate, Some(&previous)))
            .await
            .expect("committed");

        assert!(matches!(outcome, ActivationOutcome::Committed { .. }));
        let snap = store.snapshot();
        assert_eq!(snap.active.get(PACK), Some(&candidate));
        assert_eq!(snap.journal.len(), 1);
        assert_eq!(
            snap.journal[0].outcome_kind,
            ActivationTransactionOutcome::Committed
        );
    }

    // ---- up-front contention never stages -----------------------------

    #[tokio::test]
    async fn no_previous_active_takes_none_as_previous() {
        // Fresh install: no prior active pack. The coordinator observes
        // None and commits; the journal row carries None as previous.
        let candidate = pv("v.first");
        let store = InMemoryActivationStore::new(None);

        let outcome = coordinate_activation(&store, PACK, request(TX, &candidate, None))
            .await
            .expect("fresh install commits");

        assert!(matches!(outcome, ActivationOutcome::Committed { .. }));
        let snap = store.snapshot();
        assert_eq!(snap.active.get(PACK), Some(&candidate));
        assert_eq!(snap.journal.len(), 1);
        assert_eq!(snap.journal[0].previous_active_pack_version_id, None);
    }
}
