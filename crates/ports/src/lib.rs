#![forbid(unsafe_code)]

//! Ring-1 ports: the traits every replaceable engine implements.
//!
//! A-02 (ports before providers): this crate depends on `domain` and on plain
//! serialization/async utility crates only. It must never depend on a
//! concrete provider crate.

pub mod catalog;
pub mod embedding;
pub mod entitlement;
pub mod inference;
pub mod lore;
pub mod platform;
pub mod retrieval;
pub mod signature;
pub mod transfer;
pub mod zim;
