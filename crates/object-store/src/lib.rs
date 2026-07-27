#![forbid(unsafe_code)]

//! Ring-1 content-addressed object store (ARCHITECTURE.md §6.3 + §6.4).
//!
//! This crate implements the durable, content-addressed object store that
//! backs pack artifacts, model artifacts, encrypted backups and signed
//! sidecars. The layout and the 11-step commit algorithm are fixed by
//! `ARCHITECTURE.md` §6.3; the migration contract is fixed by §6.4.
//!
//! ## Layout
//!
//! ```text
//! <storage-root>/EnZIME/
//! ├── objects/sha256/ab/<62 remaining hex characters>
//! ├── staging/<install-transaction-id>/
//! ├── quarantine/<audit-event-id>/
//! ├── packs/<pack-id>/<pack-version-id>/
//! ├── models/<model-id>/<artifact-sha256>/
//! ├── sidecars/
//! ├── backups/
//! └── locks/
//! ```
//!
//! Path-construction helpers live in [`layout`].
//!
//! ## Commit algorithm (§6.3)
//!
//! [`ObjectStore::commit`](store::ObjectStore::commit) executes:
//!
//! 1. create a transaction-scoped staging file with exclusive create;
//! 2. stream bytes while enforcing declared maximum size;
//! 3. flush file content (`fsync`);
//! 4. calculate SHA-256 from staged bytes;
//! 5. compare length and digest against the caller's declared values;
//! 6. run format verifier;
//! 7. create the content-addressed object directory;
//! 8. atomically rename into the object store;
//! 9. _(not in this primitive)_ commit metadata transaction — owned by the
//!    application layer that holds the SQLite connection;
//! 10. _(not in this primitive)_ atomically update the active pointer — owned
//!     by the pack-version activation logic;
//! 11. remove the staging directory.
//!
//! Steps 9 and 10 are deliberately out of scope: they require the SQLite
//! metadata database and the pack-active-version pointer, neither of which
//! is owned by this crate. The crate that wires the full pipeline together
//! calls `commit` and then performs 9 and 10 with the returned
//! [`CommittedObject`](store::CommittedObject).
//!
//! On any verification failure (steps 2, 3, 4, 5 or 6) the staged bytes are
//! moved to `quarantine/<audit-event-id>/` and never become an active object
//! — this is the operational form of invariant A-07.
//!
//! Object files are immutable (§6.3): a second commit of the same content is
//! a no-op that returns the existing object path; the content-addressed path
//! is never overwritten.
//!
//! ## Migrations (§6.4)
//!
//! [`migration`] applies the §6.4 migration contract to the filesystem
//! layout. SQL migrations — where `schema_migrations` is a literal SQLite
//! table — live in the `storage-sqlite` crate; the object store records its
//! own applied migrations in a marker file at
//! `<storage-root>/EnZIME/schema_migrations`, which is the filesystem
//! analogue.

pub mod digest;
pub mod error;
pub mod layout;
pub mod migration;
pub mod store;
pub mod verifier;

pub use error::{FormatRejection, MigrationError, ObjectStoreError};
pub use store::{CommittedObject, CommitRequest, ObjectStore};
pub use verifier::{AcceptRawBytes, MagicPrefixVerifier, ObjectFormatVerifier};

pub use migration::{
    known_migrations, migration_sha256, run_pending, v1_initial_layout, AppliedMigration, Migration,
    LATEST_VERSION,
};
