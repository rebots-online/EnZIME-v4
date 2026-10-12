#![forbid(unsafe_code)]

//! Ring-1 SQLite storage: schema migration runner.
//!
//! The schema is the verbatim transcription of ARCHITECTURE.md §6.2 in
//! `schemas/sqlite/0001_initial.sql`. The runner applies the §6.1 connection
//! pragmas, applies the schema idempotently, and records each migration in
//! the `schema_migrations` table defined by §6.2 itself (per §6.4).

pub mod annotations;
pub mod migration;
pub mod query;
pub mod write;

pub use migration::{
    apply_pragmas, open_with_pragmas, sha256_hex, AppliedMigration, MigrationError,
    MigrationRunner, INITIAL_NAME, INITIAL_SQL, INITIAL_VERSION,
};
pub mod dyndon_migration;
pub use dyndon_migration::{apply_dyndon_schema,DYNDON_VERSION,DYNDON_NAME,DYNDON_SQL};
