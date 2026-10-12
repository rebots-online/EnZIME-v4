-- Schema 2: explicit upstream digest absence and partial source-pinned acquisition.

DROP TRIGGER trg_pack_active_version_same_pack;

DROP TRIGGER trg_download_state_guard;

CREATE TABLE catalog_sources_dyndon (
  catalog_source_id TEXT PRIMARY KEY,
  source_kind TEXT NOT NULL CHECK(source_kind IN ('openzim_xml','enzime_json','local_manifest','kiwix_opds')),
  display_name TEXT NOT NULL,
  endpoint TEXT NOT NULL,
  trust_key_id TEXT REFERENCES trust_keys(trust_key_id),
  enabled INTEGER NOT NULL CHECK(enabled IN (0,1)),
  priority INTEGER NOT NULL,
  last_refresh_at_ms INTEGER,
  etag TEXT,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
) STRICT;

INSERT INTO catalog_sources_dyndon (catalog_source_id, source_kind, display_name, endpoint, trust_key_id, enabled, priority, last_refresh_at_ms, etag, created_at_ms, updated_at_ms) SELECT catalog_source_id, source_kind, display_name, endpoint, trust_key_id, enabled, priority, last_refresh_at_ms, etag, created_at_ms, updated_at_ms FROM catalog_sources;

DROP TABLE catalog_sources;

ALTER TABLE catalog_sources_dyndon RENAME TO catalog_sources;

CREATE TABLE catalog_entries_dyndon (
  catalog_entry_id TEXT PRIMARY KEY,
  catalog_source_id TEXT NOT NULL REFERENCES catalog_sources(catalog_source_id) ON DELETE CASCADE,
  corpus_id TEXT NOT NULL,
  upstream_version TEXT NOT NULL,
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  language_bcp47 TEXT NOT NULL,
  category TEXT NOT NULL,
  publisher TEXT NOT NULL,
  artifact_url TEXT NOT NULL,
  artifact_size_bytes INTEGER NOT NULL CHECK(artifact_size_bytes >= 0),
  artifact_sha256 TEXT CHECK(artifact_sha256 IS NULL OR length(artifact_sha256) = 64),
  signature_url TEXT,
  published_at_ms INTEGER,
  metadata_json TEXT NOT NULL CHECK(json_valid(metadata_json)),
  discovered_at_ms INTEGER NOT NULL,
  UNIQUE(catalog_source_id, corpus_id, upstream_version, artifact_sha256)
) STRICT;

INSERT INTO catalog_entries_dyndon (catalog_entry_id, catalog_source_id, corpus_id, upstream_version, title, description, language_bcp47, category, publisher, artifact_url, artifact_size_bytes, artifact_sha256, signature_url, published_at_ms, metadata_json, discovered_at_ms) SELECT catalog_entry_id, catalog_source_id, corpus_id, upstream_version, title, description, language_bcp47, category, publisher, artifact_url, artifact_size_bytes, artifact_sha256, signature_url, published_at_ms, metadata_json, discovered_at_ms FROM catalog_entries;

DROP TABLE catalog_entries;

ALTER TABLE catalog_entries_dyndon RENAME TO catalog_entries;

CREATE TABLE pack_versions_dyndon (
  pack_version_id TEXT PRIMARY KEY,
  pack_id TEXT NOT NULL REFERENCES packs(pack_id) ON DELETE CASCADE,
  upstream_version TEXT NOT NULL,
  edition_manifest_sha256 TEXT NOT NULL CHECK(length(edition_manifest_sha256) = 64),
  artifact_sha256 TEXT CHECK(artifact_sha256 IS NULL OR length(artifact_sha256) = 64),
  artifact_size_bytes INTEGER NOT NULL CHECK(artifact_size_bytes > 0),
  format TEXT NOT NULL CHECK(format IN ('zim','enzime_pack')),
  verification_state TEXT NOT NULL CHECK(verification_state IN ('unverified','verified','quarantined','source_pinned')),
  installed_at_ms INTEGER,
  created_at_ms INTEGER NOT NULL,
  source_identity_sha256 TEXT CHECK(source_identity_sha256 IS NULL OR length(source_identity_sha256) = 64),
  verification_scope TEXT NOT NULL DEFAULT 'full' CHECK(verification_scope IN ('full','partial')),
  CHECK(verification_state <> 'source_pinned' OR (verification_scope = 'partial' AND source_identity_sha256 IS NOT NULL)),
  CHECK(verification_state <> 'verified' OR (verification_scope = 'full' AND artifact_sha256 IS NOT NULL)),
  UNIQUE(pack_id, upstream_version, artifact_sha256, edition_manifest_sha256)
) STRICT;

INSERT INTO pack_versions_dyndon (pack_version_id, pack_id, upstream_version, edition_manifest_sha256, artifact_sha256, artifact_size_bytes, format, verification_state, installed_at_ms, created_at_ms) SELECT pack_version_id, pack_id, upstream_version, edition_manifest_sha256, artifact_sha256, artifact_size_bytes, format, verification_state, installed_at_ms, created_at_ms FROM pack_versions;

DROP TABLE pack_versions;

ALTER TABLE pack_versions_dyndon RENAME TO pack_versions;

CREATE TABLE download_jobs_dyndon (
  download_job_id TEXT PRIMARY KEY,
  catalog_entry_id TEXT REFERENCES catalog_entries(catalog_entry_id) ON DELETE SET NULL,
  source_uri TEXT NOT NULL,
  destination_storage_root_id TEXT NOT NULL REFERENCES storage_roots(storage_root_id) ON DELETE RESTRICT,
  expected_size_bytes INTEGER NOT NULL CHECK(expected_size_bytes > 0),
  expected_sha256 TEXT CHECK(expected_sha256 IS NULL OR length(expected_sha256) = 64),
  state TEXT NOT NULL CHECK(state IN ('planned','fetching','paused','verifying','staged','activating','completed','failed','cancelled')),
  bytes_verified INTEGER NOT NULL DEFAULT 0 CHECK(bytes_verified >= 0),
  etag TEXT,
  last_modified TEXT,
  error_code TEXT,
  error_detail TEXT,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  completed_at_ms INTEGER,
  bytes_retained INTEGER NOT NULL DEFAULT 0 CHECK(bytes_retained >= 0),
  transfer_scope TEXT NOT NULL DEFAULT 'full' CHECK(transfer_scope IN ('full','partial'))
) STRICT;

INSERT INTO download_jobs_dyndon (download_job_id, catalog_entry_id, source_uri, destination_storage_root_id, expected_size_bytes, expected_sha256, state, bytes_verified, etag, last_modified, error_code, error_detail, created_at_ms, updated_at_ms, completed_at_ms) SELECT download_job_id, catalog_entry_id, source_uri, destination_storage_root_id, expected_size_bytes, expected_sha256, state, bytes_verified, etag, last_modified, error_code, error_detail, created_at_ms, updated_at_ms, completed_at_ms FROM download_jobs;

DROP TABLE download_jobs;

ALTER TABLE download_jobs_dyndon RENAME TO download_jobs;

CREATE INDEX idx_catalog_entries_search
  ON catalog_entries(language_bcp47, category, title);

CREATE INDEX idx_catalog_entries_corpus_version
  ON catalog_entries(corpus_id, upstream_version);

CREATE INDEX idx_pack_versions_pack_created
  ON pack_versions(pack_id, created_at_ms DESC);

CREATE INDEX idx_download_jobs_state_updated
  ON download_jobs(state, updated_at_ms DESC);

CREATE TRIGGER trg_pack_active_version_same_pack
BEFORE UPDATE OF active_pack_version_id ON packs
WHEN NEW.active_pack_version_id IS NOT NULL
BEGIN
  SELECT CASE WHEN NOT EXISTS (
    SELECT 1
    FROM pack_versions pv
    WHERE pv.pack_version_id = NEW.active_pack_version_id
      AND pv.pack_id = NEW.pack_id
      AND ((pv.verification_state = 'verified' AND pv.verification_scope = 'full' AND pv.artifact_sha256 IS NOT NULL)
        OR (pv.verification_state = 'source_pinned' AND pv.verification_scope = 'partial' AND pv.source_identity_sha256 IS NOT NULL))
  ) THEN RAISE(ABORT, 'active pack version must have verified full content or pinned partial source and belong to pack') END;
END;

CREATE TRIGGER trg_download_state_guard
BEFORE UPDATE OF state ON download_jobs
WHEN OLD.state <> NEW.state
BEGIN
  SELECT CASE WHEN NOT (
    (OLD.state = 'planned' AND NEW.state IN ('fetching','cancelled')) OR
    (OLD.state = 'fetching' AND NEW.state IN ('paused','verifying','failed','cancelled')) OR
    (OLD.state = 'paused' AND NEW.state IN ('fetching','cancelled')) OR
    (OLD.state = 'verifying' AND NEW.state IN ('staged','failed','cancelled')) OR
    (OLD.state = 'staged' AND NEW.state IN ('activating','cancelled')) OR
    (OLD.state = 'activating' AND NEW.state IN ('completed','failed')) OR
    (OLD.state = 'failed' AND NEW.state IN ('fetching','cancelled'))
  ) THEN RAISE(ABORT, 'invalid download state transition') END;
END;
