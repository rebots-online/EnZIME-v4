//! Library entry for `contract-gen`, the §24.1 entity-to-UI binding
//! validator.
//!
//! Per `DOCS/ARCHITECTURE.md` §26 AMD-4 the entity-table check is a local
//! script under `tools/`, not a CI gate. The binary in [`main.rs`] is the
//! operator-facing surface; this library exposes the validation logic so a
//! future build script can invoke it programmatically without spawning a
//! subprocess.

pub mod diagnostic;
pub mod extract;
pub mod spec;
pub mod validate;

pub use validate::{validate, Diagnostic, Inputs};
