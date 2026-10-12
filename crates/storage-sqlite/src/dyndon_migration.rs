//! Atomic schema upgrade preserving full-file versus partial-source trust.
use crate::migration::{sha256_hex, AppliedMigration, MigrationError};
use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use std::time::{SystemTime, UNIX_EPOCH};
pub const DYNDON_VERSION: i64 = 2;
pub const DYNDON_NAME: &str = "dyndon_sources";
pub const DYNDON_SQL: &str = include_str!("../../../schemas/sqlite/0002_dyndon_sources.sql");
struct RestoreForeignKeys<'a> {
    conn: &'a Connection,
    enabled: bool,
}
impl Drop for RestoreForeignKeys<'_> {
    fn drop(&mut self) {
        let _ = self.conn.pragma_update(None, "foreign_keys", self.enabled);
    }
}
pub fn apply_dyndon_schema(conn: &Connection) -> Result<AppliedMigration, MigrationError> {
    if !conn.is_autocommit() {
        return Err(rusqlite::Error::InvalidQuery.into());
    }
    let fingerprint = sha256_hex(DYNDON_SQL.as_bytes());
    let existing = conn
        .query_row(
            "SELECT version,name,sha256,applied_at_ms FROM schema_migrations WHERE version=?1",
            [DYNDON_VERSION],
            |r| {
                Ok(AppliedMigration {
                    version: r.get(0)?,
                    name: r.get(1)?,
                    sha256: r.get(2)?,
                    applied_at_ms: r.get(3)?,
                })
            },
        )
        .optional()?;
    if let Some(record) = existing {
        if record.sha256 != fingerprint {
            return Err(MigrationError::FingerprintMismatch {
                version: DYNDON_VERSION,
                recorded: record.sha256,
                current: fingerprint,
            });
        }
        return Ok(record);
    }
    let enabled: bool = conn.pragma_query_value(None, "foreign_keys", |r| r.get(0))?;
    conn.pragma_update(None, "foreign_keys", false)?;
    let restore = RestoreForeignKeys { conn, enabled };
    let result = (|| {
        let transaction = Transaction::new_unchecked(conn, TransactionBehavior::Immediate)?;
        transaction.execute_batch(DYNDON_SQL)?;
        let violation = transaction
            .query_row("SELECT 1 FROM pragma_foreign_key_check LIMIT 1", [], |r| {
                r.get::<_, i64>(0)
            })
            .optional()?;
        if violation.is_some() {
            return Err(MigrationError::Sqlite(rusqlite::Error::InvalidQuery));
        }
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| MigrationError::ClockBeforeEpoch)?
            .as_millis() as i64;
        transaction.execute(
            "INSERT INTO schema_migrations(version,name,sha256,applied_at_ms) VALUES (?1,?2,?3,?4)",
            params![DYNDON_VERSION, DYNDON_NAME, fingerprint, time],
        )?;
        transaction.commit()?;
        Ok(AppliedMigration {
            version: DYNDON_VERSION,
            name: DYNDON_NAME.into(),
            sha256: fingerprint,
            applied_at_ms: time,
        })
    })();
    drop(restore);
    // Also surface restoration failure on a successful migration.
    conn.pragma_update(None, "foreign_keys", enabled)?;
    result
}
