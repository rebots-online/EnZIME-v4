//! SQLite migration runner.
//!
//! Applies the canonical schema from `schemas/sqlite/0001_initial.sql`
//! (ARCHITECTURE.md §6.2, transcribed verbatim) and records each migration
//! in the `schema_migrations` table defined by that same schema (§6.4).
//!
//! §6.1 connection pragmas (`journal_mode = WAL`, `foreign_keys = ON`,
//! `synchronous = FULL`, `busy_timeout = 5000`, `trusted_schema = OFF`,
//! `temp_store = MEMORY`) are applied here at connection time.

use std::fmt::Write as _;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension};

/// Integer version of the initial migration (§6.4: "integer version").
pub const INITIAL_VERSION: i64 = 1;

/// Stable name of the initial migration (§6.4: "stable name").
pub const INITIAL_NAME: &str = "initial";

/// The verbatim §6.2 schema. Bound at compile time via `include_str!` so a
/// drift between the shipped SQL and the source tree fails the build rather
/// than silently shipping a stale schema.
pub const INITIAL_SQL: &str = include_str!("../../../schemas/sqlite/0001_initial.sql");

/// Errors produced by the migration runner.
#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    /// The system clock reported a time before the unix epoch.
    #[error("system clock is before the unix epoch")]
    ClockBeforeEpoch,

    /// The migration `version` is already recorded in `schema_migrations`
    /// with a different SHA-256 than the SQL the runner is currently asked
    /// to apply. Per §6.4 ("SHA-256 registered in `schema_migrations`")
    /// fingerprint drift is a hard stop — never silently re-apply.
    #[error(
        "migration {version} is already recorded with a different sha256 (recorded={recorded}, current={current})"
    )]
    FingerprintMismatch {
        version: i64,
        recorded: String,
        current: String,
    },

    /// A `rusqlite` call failed.
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

/// A row of `schema_migrations` (§6.2).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedMigration {
    pub version: i64,
    pub name: String,
    pub sha256: String,
    pub applied_at_ms: i64,
}

/// Lowercase hexadecimal SHA-256 of a byte slice.
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for byte in digest {
        let _ = write!(out, "{:02x}", byte);
    }
    out
}

fn now_ms() -> Result<i64, MigrationError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| MigrationError::ClockBeforeEpoch)
        .map(|d| d.as_millis() as i64)
}

/// Apply the §6.1 connection pragmas to an open connection.
///
/// Per §6.1 these run at connection time, before any migration does.
/// `journal_mode = WAL` is persisted in the database file; the remaining
/// pragmas are per-connection.
pub fn apply_pragmas(conn: &Connection) -> Result<(), MigrationError> {
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;\n\
         PRAGMA journal_mode = WAL;\n\
         PRAGMA synchronous = FULL;\n\
         PRAGMA busy_timeout = 5000;\n\
         PRAGMA trusted_schema = OFF;\n\
         PRAGMA temp_store = MEMORY;",
    )?;
    Ok(())
}

/// Open a read/write connection to `path` and immediately apply §6.1 pragmas.
pub fn open_with_pragmas<P: AsRef<Path>>(path: P) -> Result<Connection, MigrationError> {
    let conn = Connection::open(path)?;
    apply_pragmas(&conn)?;
    Ok(conn)
}

/// Migration runner.
///
/// Borrows a [`Connection`]; the caller owns its lifecycle. All methods are
/// idempotent and may be called repeatedly against the same database.
pub struct MigrationRunner<'a> {
    conn: &'a Connection,
}

impl<'a> MigrationRunner<'a> {
    /// Construct a runner that runs migrations through `conn`.
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Ensure the `schema_migrations` table exists.
    ///
    /// On a fresh database the table is normally created by the initial
    /// migration's SQL. This method creates it idempotently so the runner can
    /// query state before that migration has run. The definition matches the
    /// §6.2 `schema_migrations` table byte-for-byte.
    pub fn ensure_schema_migrations_table(&self) -> Result<(), MigrationError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (\n\
             \x20\x20version INTEGER PRIMARY KEY,\n\
             \x20\x20name TEXT NOT NULL UNIQUE,\n\
             \x20\x20sha256 TEXT NOT NULL CHECK(length(sha256) = 64),\n\
             \x20\x20applied_at_ms INTEGER NOT NULL\n\
             ) STRICT;",
        )?;
        Ok(())
    }

    /// List all recorded migrations ordered ascending by `version`.
    pub fn list_applied(&self) -> Result<Vec<AppliedMigration>, MigrationError> {
        self.ensure_schema_migrations_table()?;
        let mut stmt = self.conn.prepare(
            "SELECT version, name, sha256, applied_at_ms\n\
             \x20\x20FROM schema_migrations\n\
             \x20\x20ORDER BY version",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(AppliedMigration {
                version: row.get(0)?,
                name: row.get(1)?,
                sha256: row.get(2)?,
                applied_at_ms: row.get(3)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// Return whether migration `version` has been recorded.
    pub fn is_applied(&self, version: i64) -> Result<bool, MigrationError> {
        self.ensure_schema_migrations_table()?;
        let recorded: Option<i64> = self
            .conn
            .query_row(
                "SELECT version FROM schema_migrations WHERE version = ?1",
                params![version],
                |row| row.get(0),
            )
            .optional()?;
        Ok(recorded.is_some())
    }

    /// Apply the initial migration (version 1, §6.2 schema) idempotently.
    pub fn apply_initial(&self) -> Result<AppliedMigration, MigrationError> {
        self.apply_versioned(INITIAL_VERSION, INITIAL_NAME, INITIAL_SQL)
    }

    /// Apply `sql` as migration `version`/`name`, recording its SHA-256.
    ///
    /// Idempotent: if `version` is already recorded with the same fingerprint,
    /// returns the existing record without re-running the SQL. If recorded
    /// with a different fingerprint, returns [`MigrationError::FingerprintMismatch`].
    ///
    /// The SQL must contain its own `BEGIN IMMEDIATE; ... COMMIT;` (the §6.2
    /// schema does); the runner writes the `schema_migrations` row after the
    /// schema's own transaction has committed. Per §6.4 a migration never
    /// silently repairs ambiguous primary data; this runner does not catch
    /// schema errors and substitute a repair.
    pub fn apply_versioned(
        &self,
        version: i64,
        name: &str,
        sql: &str,
    ) -> Result<AppliedMigration, MigrationError> {
        self.ensure_schema_migrations_table()?;
        let fingerprint = sha256_hex(sql.as_bytes());

        if let Some(existing) = self.fetch(version)? {
            if existing.sha256 != fingerprint {
                return Err(MigrationError::FingerprintMismatch {
                    version,
                    recorded: existing.sha256,
                    current: fingerprint,
                });
            }
            return Ok(existing);
        }

        // The §6.2 SQL opens its own BEGIN IMMEDIATE transaction and commits
        // it; rusqlite's execute_batch executes the multi-statement script
        // verbatim, including those transaction controls.
        self.conn.execute_batch(sql)?;

        let applied_at_ms = now_ms()?;
        self.conn.execute(
            "INSERT INTO schema_migrations (version, name, sha256, applied_at_ms)\n\
             \x20\x20VALUES (?1, ?2, ?3, ?4)",
            params![version, name, fingerprint, applied_at_ms],
        )?;

        Ok(AppliedMigration {
            version,
            name: name.to_string(),
            sha256: fingerprint,
            applied_at_ms,
        })
    }

    fn fetch(&self, version: i64) -> Result<Option<AppliedMigration>, MigrationError> {
        let row = self
            .conn
            .query_row(
                "SELECT version, name, sha256, applied_at_ms\n\
                 \x20\x20FROM schema_migrations\n\
                 \x20\x20WHERE version = ?1",
                params![version],
                |row| {
                    Ok(AppliedMigration {
                        version: row.get(0)?,
                        name: row.get(1)?,
                        sha256: row.get(2)?,
                        applied_at_ms: row.get(3)?,
                    })
                },
            )
            .optional()?;
        Ok(row)
    }
}
