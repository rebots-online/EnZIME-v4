#![forbid(unsafe_code)]

//! §8.3 install planner.
//!
//! Pure decision function over facts the caller has already gathered from
//! the catalog, transfer and platform ports. Produces an [`InstallPlan`]
//! or an actionable [`InstallPlanError`] matching one of the §8.3 failure
//! conditions. Performs no I/O of its own and depends on no provider crate
//! (A-02): every external fact arrives in [`InstallPlanRequest`].
//!
//! The plan types themselves live in [`ports::transfer`] and are re-exported
//! here so consumers can name them from the application crate.

use domain::identity::PackVersionId;
use ports::catalog::CatalogEntry;
use serde::{Deserialize, Serialize};

pub use ports::transfer::{InstallPlan, StorageDestination, TransferMode, VerificationPlan};

/// Trust-state token the openzim catalog provider writes when an artifact
/// signature has been verified against a trusted signer. Mirrors the value
/// produced by `catalog-openzim` for `TrustPresentation::state`.
const TRUST_STATE_VERIFIED: &str = "verified";

/// Signature policy applied by the planner. `Required` corresponds to the
/// §8.3 failure "policy requires a signature and no trusted signature
/// exists".
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignaturePolicy {
    Required,
    Optional,
}

/// Every fact the planner needs, gathered by the caller from the catalog,
/// transfer and platform ports. The planner does not perform I/O.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstallPlanRequest {
    pub install_plan_id: String,
    pub catalog_entry: CatalogEntry,
    pub destination: StorageDestination,
    pub free_bytes_before: u64,
    /// Reported by `TransferSource::probe` (`TransferProbe::size_bytes`).
    /// Ignored for [`TransferMode::LocalCopy`].
    pub source_size_bytes: u64,
    /// Reported by `TransferSource::probe` (`TransferProbe::accepts_ranges`).
    pub source_accepts_ranges: bool,
    pub desired_mode: TransferMode,
    pub signature_policy: SignaturePolicy,
    pub replaces_pack_version_id: Option<PackVersionId>,
    pub retained_pack_version_id: Option<PackVersionId>,
    /// On-disk footprint of `retained_pack_version_id` (the last-known-good
    /// that must stay readable during the swap, per A-04). Zero when no pack
    /// is retained.
    pub retained_pack_bytes: u64,
}

/// Actionable planning failure. One variant per §8.3 failure condition; the
/// planner returns the first match in the order the spec lists them.
#[derive(Clone, Debug, thiserror::Error)]
pub enum InstallPlanError {
    /// §8.3: "destination is not writable".
    #[error("destination is not writable")]
    DestinationNotWritable,
    /// §8.3: "final bytes do not fit".
    #[error("final bytes ({final_bytes}) do not fit in free space ({free_bytes})")]
    FinalBytesDoNotFit {
        final_bytes: u64,
        free_bytes: u64,
    },
    /// §8.3: "temporary plus retained last-known-good bytes do not fit".
    #[error(
        "temporary ({temporary_bytes}) plus retained last-known-good ({retained_bytes}) \
         bytes do not fit in free space ({free_bytes})"
    )]
    TemporaryAndRetainedDoNotFit {
        temporary_bytes: u64,
        retained_bytes: u64,
        free_bytes: u64,
    },
    /// §8.3: "no verified digest is available".
    #[error("no verified digest is available for the artifact")]
    NoVerifiedDigest,
    /// §8.3: "policy requires a signature and no trusted signature exists".
    #[error("signature is required by policy but no trusted signature exists")]
    SignatureRequiredButMissing,
    /// §8.3: "source range behavior is incompatible with the selected strategy".
    #[error("source range behavior is incompatible with the selected {mode:?} strategy")]
    RangeStrategyIncompatible { mode: TransferMode },
}

/// Decide an install plan from the gathered facts.
///
/// The planner is synchronous and pure: every async fact (catalog lookup,
/// transfer probe, destination free-space query, retained-pack footprint)
/// arrives in `request` and the planner only decides. The order of checks
/// matches the §8.3 bullet order so the first actionable failure is the one
/// surfaced.
pub fn plan_install(request: InstallPlanRequest) -> Result<InstallPlan, InstallPlanError> {
    let InstallPlanRequest {
        install_plan_id,
        catalog_entry,
        destination,
        free_bytes_before,
        source_size_bytes,
        source_accepts_ranges,
        desired_mode,
        signature_policy,
        replaces_pack_version_id,
        retained_pack_version_id,
        retained_pack_bytes,
    } = request;

    // §8.3 failure 1: destination is not writable.
    if !destination.writable {
        return Err(InstallPlanError::DestinationNotWritable);
    }

    let final_bytes = catalog_entry.artifact_size_bytes;

    // §8.3 failure 2: final bytes do not fit.
    if final_bytes > free_bytes_before {
        return Err(InstallPlanError::FinalBytesDoNotFit {
            final_bytes,
            free_bytes: free_bytes_before,
        });
    }

    // `download_bytes` is what the wire carries. Local copies don't fetch.
    let download_bytes = match desired_mode {
        TransferMode::LocalCopy => 0,
        TransferMode::Ranged | TransferMode::Full => source_size_bytes,
    };
    // `temporary_bytes` is the on-disk footprint of the in-flight download
    // before activation. The retained last-known-good is tracked separately
    // (it must stay readable during the swap, per A-04).
    let temporary_bytes = download_bytes;

    // §8.3 failure 3: temporary plus retained last-known-good bytes do not fit.
    let install_footprint = temporary_bytes.saturating_add(retained_pack_bytes);
    if install_footprint > free_bytes_before {
        return Err(InstallPlanError::TemporaryAndRetainedDoNotFit {
            temporary_bytes,
            retained_bytes: retained_pack_bytes,
            free_bytes: free_bytes_before,
        });
    }

    // §8.3 failure 4: no verified digest is available. The catalog source is
    // responsible for provenance; presence is what the planner can check.
    if catalog_entry.artifact_sha256.trim().is_empty() {
        return Err(InstallPlanError::NoVerifiedDigest);
    }

    // §8.3 failure 5: policy requires a signature and no trusted signature exists.
    if signature_policy == SignaturePolicy::Required
        && catalog_entry.trust.state != TRUST_STATE_VERIFIED
    {
        return Err(InstallPlanError::SignatureRequiredButMissing);
    }

    // §8.3 failure 6: source range behavior is incompatible with the selected strategy.
    if matches!(desired_mode, TransferMode::Ranged) && !source_accepts_ranges {
        return Err(InstallPlanError::RangeStrategyIncompatible {
            mode: desired_mode,
        });
    }

    // Steady-state free space once the new pack is at rest. Conservative lower
    // bound: the planner does not assume any bytes are reclaimed (A-04 keeps
    // the last-known-good readable until a successful activation, and reclaims
    // happen after `Staged`).
    let free_bytes_after = free_bytes_before.saturating_sub(final_bytes);

    let verification = VerificationPlan {
        expected_size_bytes: final_bytes,
        expected_sha256: catalog_entry.artifact_sha256.clone(),
        signature_required: signature_policy == SignaturePolicy::Required,
        // Catalog entries are ZIM artifacts; the install verifier always
        // structural-checks them.
        zim_structure_check: true,
    };

    Ok(InstallPlan {
        install_plan_id,
        catalog_entry,
        destination,
        transfer_mode: desired_mode,
        download_bytes,
        temporary_bytes,
        final_bytes,
        free_bytes_before,
        free_bytes_after,
        verification,
        replaces_pack_version_id,
        retained_pack_version_id,
    })
}
