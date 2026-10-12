use rusqlite::{types::Value, Connection};
use storage_sqlite::{apply_dyndon_schema, MigrationError, MigrationRunner};
fn initial() -> Connection {
    let c = Connection::open_in_memory().unwrap();
    storage_sqlite::apply_pragmas(&c).unwrap();
    MigrationRunner::new(&c).apply_initial().unwrap();
    c
}
fn seed(c: &Connection) {
    c.execute_batch("INSERT INTO storage_roots VALUES('root','app_private','/storage','Storage',1,999999,999999,1,1,1);
 INSERT INTO catalog_sources VALUES('source','local_manifest','Source','https://example.org',NULL,1,1,1,NULL,1,1);
 INSERT INTO catalog_entries VALUES('entry','source','corpus','edition','Title','Description','eng','reference','Publisher','https://example.org/test.zim',100,printf('%064d',1),NULL,NULL,'{}',1);
 INSERT INTO packs VALUES('pack','corpus','Title','Description','eng','full_zim',NULL,1,1);
 INSERT INTO pack_versions VALUES('version','pack','edition',printf('%064d',2),printf('%064d',1),100,'zim','verified',1,1);
 UPDATE packs SET active_pack_version_id='version' WHERE pack_id='pack';
 INSERT INTO pack_locations VALUES('location','version','root','pack.zim','active',100,1,1);
 INSERT INTO download_jobs VALUES('job','entry','https://example.org/test.zim','root',100,printf('%064d',1),'completed',100,NULL,NULL,NULL,NULL,1,1,1);
 INSERT INTO download_chunks VALUES('job',0,0,100,NULL,printf('%064d',3),'verified',1,1);
 INSERT INTO sidecar_layers VALUES('layer','Local',NULL,'private','local',NULL,1,1);
 INSERT INTO annotations VALUES('annotation','comment','pack','A/Page',NULL,'layer',1,'resolved',0,NULL,1,1);
 INSERT INTO annotation_revisions VALUES('annotation',1,'Saved note','{}','{}','context',NULL,NULL,1);").unwrap();
}
fn rows(c: &Connection, sql: &str) -> Vec<Vec<Value>> {
    let mut s = c.prepare(sql).unwrap();
    let n = s.column_count();
    s.query_map([], |r| (0..n).map(|i| r.get(i)).collect())
        .unwrap()
        .map(Result::unwrap)
        .collect()
}
#[test]
fn preserves_rows_relations_indexes_triggers_and_is_idempotent() {
    let c = initial();
    seed(&c);
    let tables = [
        "catalog_sources",
        "catalog_entries",
        "packs",
        "pack_versions",
        "pack_locations",
        "download_jobs",
        "download_chunks",
        "annotations",
        "annotation_revisions",
    ];
    let before: Vec<_> = tables
        .iter()
        .map(|t| {
            let cols: Vec<String> = c
                .prepare(&format!("PRAGMA table_info({t})"))
                .unwrap()
                .query_map([], |r| r.get(1))
                .unwrap()
                .map(Result::unwrap)
                .collect();
            let query = format!("SELECT {} FROM {t}", cols.join(","));
            let data = rows(&c, &query);
            (query, data)
        })
        .collect();
    let objects = rows(
        &c,
        "SELECT type,name FROM sqlite_schema WHERE type IN ('index','trigger') ORDER BY name",
    );
    let first = apply_dyndon_schema(&c).unwrap();
    assert_eq!(first.version, 2);
    assert_eq!(first, apply_dyndon_schema(&c).unwrap());
    for (query, expected) in before {
        assert_eq!(rows(&c, &query), expected, "{query}");
    }
    assert_eq!(
        rows(
            &c,
            "SELECT type,name FROM sqlite_schema WHERE type IN ('index','trigger') ORDER BY name"
        ),
        objects
    );
    assert!(rows(&c, "PRAGMA foreign_key_check").is_empty());
    assert_eq!(
        rows(&c, "PRAGMA foreign_keys"),
        vec![vec![Value::Integer(1)]]
    );
    assert!(c
        .execute(
            "UPDATE download_jobs SET state='fetching' WHERE download_job_id='job'",
            []
        )
        .is_err());
    assert_eq!(
        c.query_row(
            "SELECT sha256 FROM schema_migrations WHERE version=1",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        storage_sqlite::sha256_hex(storage_sqlite::INITIAL_SQL.as_bytes())
    );
}
#[test]
fn null_digest_and_partial_activation_are_truthful() {
    let c = initial();
    seed(&c);
    apply_dyndon_schema(&c).unwrap();
    c.execute(
        "UPDATE catalog_entries SET artifact_sha256=NULL,artifact_size_bytes=0",
        [],
    )
    .unwrap();
    let insert="INSERT INTO pack_versions(pack_version_id,pack_id,upstream_version,edition_manifest_sha256,artifact_sha256,artifact_size_bytes,format,verification_state,created_at_ms,source_identity_sha256,verification_scope) VALUES(?1,'pack','edition',printf('%064d',2),?2,100,'zim',?3,1,?4,?5)";
    for (id, state, digest, identity, scope, valid) in [
        (
            "partial",
            "source_pinned",
            None,
            Some("a".repeat(64)),
            "partial",
            true,
        ),
        ("false-full", "verified", None, None, "full", false),
        (
            "unidentified",
            "source_pinned",
            None,
            None,
            "partial",
            false,
        ),
        (
            "wrong-scope",
            "source_pinned",
            None,
            Some("a".repeat(64)),
            "full",
            false,
        ),
        (
            "wrong-verified",
            "verified",
            Some("b".repeat(64)),
            None,
            "partial",
            false,
        ),
        ("unverified", "unverified", None, None, "partial", true),
        ("quarantined", "quarantined", None, None, "partial", true),
    ] {
        let result = c.execute(
            insert,
            rusqlite::params![id, digest, state, identity, scope],
        );
        assert_eq!(result.is_ok(), valid, "{id}: {result:?}");
    }
    c.execute("UPDATE packs SET active_pack_version_id='partial'", [])
        .unwrap();
    for id in ["unverified", "quarantined"] {
        assert!(c
            .execute("UPDATE packs SET active_pack_version_id=?1", [id])
            .is_err());
    }
    c.execute("UPDATE download_jobs SET expected_sha256=NULL,bytes_retained=50,bytes_verified=0,transfer_scope='partial'",[]).unwrap();
    assert!(c
        .execute("UPDATE download_jobs SET expected_sha256='missing'", [])
        .is_err());
}
#[test]
fn fingerprint_transaction_and_failed_upgrade_restore_original_state() {
    let c = initial();
    seed(&c);
    c.execute_batch("BEGIN").unwrap();
    assert!(apply_dyndon_schema(&c).is_err());
    assert!(!c.is_autocommit());
    c.execute_batch("ROLLBACK").unwrap();
    c.execute_batch("CREATE TABLE catalog_sources_dyndon(x INTEGER)")
        .unwrap();
    assert!(apply_dyndon_schema(&c).is_err());
    assert_eq!(
        rows(&c, "PRAGMA foreign_keys"),
        vec![vec![Value::Integer(1)]]
    );
    assert_eq!(
        rows(&c, "SELECT COUNT(*) FROM catalog_entries"),
        vec![vec![Value::Integer(1)]]
    );
    assert_eq!(
        rows(&c, "SELECT COUNT(*) FROM schema_migrations WHERE version=2"),
        vec![vec![Value::Integer(0)]]
    );
    assert_eq!(
        rows(
            &c,
            "SELECT COUNT(*) FROM sqlite_schema WHERE name='trg_pack_active_version_same_pack'"
        ),
        vec![vec![Value::Integer(1)]]
    );
    c.execute_batch("DROP TABLE catalog_sources_dyndon")
        .unwrap();
    apply_dyndon_schema(&c).unwrap();
    c.execute(
        "UPDATE schema_migrations SET sha256=printf('%064d',9) WHERE version=2",
        [],
    )
    .unwrap();
    assert!(matches!(
        apply_dyndon_schema(&c),
        Err(MigrationError::FingerprintMismatch { version: 2, .. })
    ));
}
#[test]
fn foreign_key_violation_rolls_back_every_rebuild() {
    let c = initial();
    seed(&c);
    c.pragma_update(None, "foreign_keys", false).unwrap();
    c.execute("UPDATE pack_locations SET pack_version_id='missing'", [])
        .unwrap();
    c.pragma_update(None, "foreign_keys", true).unwrap();
    assert!(apply_dyndon_schema(&c).is_err());
    assert_eq!(
        rows(&c, "SELECT COUNT(*) FROM schema_migrations WHERE version=2"),
        vec![vec![Value::Integer(0)]]
    );
    assert!(c
        .prepare("SELECT verification_scope FROM pack_versions")
        .is_err());
    assert_eq!(
        rows(&c, "PRAGMA foreign_keys"),
        vec![vec![Value::Integer(1)]]
    );
}
