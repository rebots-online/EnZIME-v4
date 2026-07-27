#![forbid(unsafe_code)]

//! Ring-2 lore engine — SQLite-backed implementation of the
//! `ports::lore::LoreService` trait defined by ARCHITECTURE.md §12.
//!
//! §12.1 lifecycle: Evidence → Candidate → {Accepted | Rejected};
//! Accepted → Corrected → Accepted; Accepted → {Superseded | Deleted}.
//!
//! §A-08 binding invariant: a candidate NEVER becomes accepted memory
//! implicitly. Acceptance is always an explicit, evidence-bearing transition
//! driven either by user action ([`ports::lore::LoreService::accept`]) or by
//! a user-configured deterministic promotion policy whose rule and evidence
//! are surfaced next to the promoted memory via the revision's
//! `correction_reason` (see [`policy::PromotionPolicy`]). Deletion (§12.5)
//! is a tombstone plus removal of the retrieval projection so deleted text
//! becomes unreachable, not merely hidden.

pub mod policy;
pub mod service;

pub use policy::{CandidateClass, PromotionPolicy, PromotionRule};
pub use service::SqliteLoreService;
