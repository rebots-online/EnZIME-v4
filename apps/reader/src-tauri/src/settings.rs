//! Typed access to the §6.2 `settings` table (JSON-encoded values).

use rusqlite::{params, Connection, OptionalExtension};

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub fn get_string(conn: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = ?1 AND value_type = 'string'",
            params![key],
            |r| r.get(0),
        )
        .optional()?;
    Ok(raw.and_then(|j| serde_json::from_str::<String>(&j).ok()))
}

pub fn set_string(conn: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
    let json = serde_json::to_string(value).expect("string serializes");
    conn.execute(
        "INSERT INTO settings (key, value_json, value_type, updated_at_ms)
         VALUES (?1, ?2, 'string', ?3)
         ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json,
             value_type = 'string', updated_at_ms = excluded.updated_at_ms",
        params![key, json, now_ms()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_overwrite() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE settings (key TEXT PRIMARY KEY,
               value_json TEXT NOT NULL CHECK(json_valid(value_json)),
               value_type TEXT NOT NULL, updated_at_ms INTEGER NOT NULL) STRICT;",
        )
        .unwrap();
        assert_eq!(get_string(&conn, "a").unwrap(), None);
        set_string(&conn, "a", "x\"y").unwrap();
        assert_eq!(get_string(&conn, "a").unwrap().as_deref(), Some("x\"y"));
        set_string(&conn, "a", "z").unwrap();
        assert_eq!(get_string(&conn, "a").unwrap().as_deref(), Some("z"));
    }
}
