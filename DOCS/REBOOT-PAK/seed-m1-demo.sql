-- M1 DynDon demo seed — applied to the AppImage's enzime.db after first boot.
-- DB path: ~/.local/share/mba.robin.enzime/enzime.db  (identifier mba.robin.enzime)
-- Apply:   sqlite3 ~/.local/share/mba.robin.enzime/enzime.db < DOCS/REBOOT-PAK/seed-m1-demo.sql
--
-- Purpose: one catalog entry (the real 2.09 GB urban-prepper ZIM) + one storage
-- root whose budget (900 MB) is DELIBERATELY smaller than the pack, so DynDon's
-- plan_fit must return a TRUNCATED subset that fits — the R-PLAN USP moment.
-- (install_plan's free-bytes ladder reads storage_roots.free_bytes before the
--  statvfs syscall, so this seed controls the demo budget deterministically.)

INSERT OR IGNORE INTO catalog_sources (
  catalog_source_id, source_kind, display_name, endpoint,
  trust_key_id, enabled, priority, last_refresh_at_ms, etag,
  created_at_ms, updated_at_ms
) VALUES (
  'src.local','local_manifest','Local Demo Catalog','file:///home/robin/Downloads',
  NULL,1,1,NULL,NULL,
  (CAST(strftime('%s','now') AS INTEGER))*1000,
  (CAST(strftime('%s','now') AS INTEGER))*1000
);

INSERT OR IGNORE INTO catalog_entries (
  catalog_entry_id, catalog_source_id, corpus_id, upstream_version,
  title, description, language_bcp47, category, publisher,
  artifact_url, artifact_size_bytes, artifact_sha256,
  signature_url, published_at_ms, metadata_json, discovered_at_ms
) VALUES (
  'ce.urban-prepper','src.local','urban-prepper.en','2025-11',
  'Urban Prepper (en)','Prepper knowledge reference — full 2.09 GB pack for the DynDon subset demo.',
  'en','reference','openZIM',
  'file:///home/robin/Downloads/urban-prepper_en_all_2025-11.zim',
  2245236272,
  '4bebfb84004bd84457837e5cb2ffe6bd54febd0e174242e1a1a7337a5f9f3557',
  NULL,
  (CAST(strftime('%s','now') AS INTEGER))*1000,
  '{}',
  (CAST(strftime('%s','now') AS INTEGER))*1000
);

INSERT OR IGNORE INTO storage_roots (
  storage_root_id, kind, platform_locator, display_name, writable,
  capacity_bytes, free_bytes, last_probed_at_ms, created_at_ms, updated_at_ms
) VALUES (
  'root.app','app_private','/home/robin/.local/share/mba.robin.enzime/packs','App Storage',1,
  900000000, 900000000,
  (CAST(strftime('%s','now') AS INTEGER))*1000,
  (CAST(strftime('%s','now') AS INTEGER))*1000,
  (CAST(strftime('%s','now') AS INTEGER))*1000
);
