BEGIN IMMEDIATE;

-- Idempotent: the migration runner also bootstraps this table via
-- ensure_schema_migrations_table() before executing this script, so the
-- table already exists when this CREATE runs. IF NOT EXISTS keeps the §6.2
-- "schema self-defines its bookkeeping table" design without colliding.
CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  sha256 TEXT NOT NULL CHECK(length(sha256) = 64),
  applied_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE app_meta (
  key TEXT PRIMARY KEY,
  value_json TEXT NOT NULL CHECK(json_valid(value_json)),
  updated_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE devices (
  device_id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  platform TEXT NOT NULL CHECK(platform IN ('android','linux','windows')),
  installation_public_key BLOB NOT NULL,
  created_at_ms INTEGER NOT NULL,
  last_seen_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value_json TEXT NOT NULL CHECK(json_valid(value_json)),
  value_type TEXT NOT NULL CHECK(value_type IN ('boolean','integer','number','string','object','array')),
  updated_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE storage_roots (
  storage_root_id TEXT PRIMARY KEY,
  kind TEXT NOT NULL CHECK(kind IN ('app_private','user_selected','removable')),
  platform_locator TEXT NOT NULL,
  display_name TEXT NOT NULL,
  writable INTEGER NOT NULL CHECK(writable IN (0,1)),
  capacity_bytes INTEGER CHECK(capacity_bytes IS NULL OR capacity_bytes >= 0),
  free_bytes INTEGER CHECK(free_bytes IS NULL OR free_bytes >= 0),
  last_probed_at_ms INTEGER,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE capability_reports (
  capability_report_id TEXT PRIMARY KEY,
  device_id TEXT NOT NULL REFERENCES devices(device_id) ON DELETE CASCADE,
  report_json TEXT NOT NULL CHECK(json_valid(report_json)),
  app_version TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE catalog_sources (
  catalog_source_id TEXT PRIMARY KEY,
  source_kind TEXT NOT NULL CHECK(source_kind IN ('openzim_xml','enzime_json','local_manifest')),
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

CREATE TABLE catalog_entries (
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
  artifact_size_bytes INTEGER NOT NULL CHECK(artifact_size_bytes > 0),
  artifact_sha256 TEXT NOT NULL CHECK(length(artifact_sha256) = 64),
  signature_url TEXT,
  published_at_ms INTEGER,
  metadata_json TEXT NOT NULL CHECK(json_valid(metadata_json)),
  discovered_at_ms INTEGER NOT NULL,
  UNIQUE(catalog_source_id, corpus_id, upstream_version, artifact_sha256)
) STRICT;

CREATE TABLE packs (
  pack_id TEXT PRIMARY KEY,
  corpus_id TEXT NOT NULL,
  display_name TEXT NOT NULL,
  description TEXT NOT NULL,
  language_bcp47 TEXT NOT NULL,
  pack_kind TEXT NOT NULL CHECK(pack_kind IN ('full_zim','tailored_zim','enzime_pack')),
  active_pack_version_id TEXT,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE pack_versions (
  pack_version_id TEXT PRIMARY KEY,
  pack_id TEXT NOT NULL REFERENCES packs(pack_id) ON DELETE CASCADE,
  upstream_version TEXT NOT NULL,
  edition_manifest_sha256 TEXT NOT NULL CHECK(length(edition_manifest_sha256) = 64),
  artifact_sha256 TEXT NOT NULL CHECK(length(artifact_sha256) = 64),
  artifact_size_bytes INTEGER NOT NULL CHECK(artifact_size_bytes > 0),
  format TEXT NOT NULL CHECK(format IN ('zim','enzime_pack')),
  verification_state TEXT NOT NULL CHECK(verification_state IN ('unverified','verified','quarantined')),
  installed_at_ms INTEGER,
  created_at_ms INTEGER NOT NULL,
  UNIQUE(pack_id, upstream_version, artifact_sha256)
) STRICT;

CREATE TABLE pack_locations (
  pack_location_id TEXT PRIMARY KEY,
  pack_version_id TEXT NOT NULL REFERENCES pack_versions(pack_version_id) ON DELETE CASCADE,
  storage_root_id TEXT NOT NULL REFERENCES storage_roots(storage_root_id) ON DELETE RESTRICT,
  relative_path TEXT NOT NULL,
  locator_kind TEXT NOT NULL CHECK(locator_kind IN ('active','retained','staging','quarantine')),
  byte_length INTEGER NOT NULL CHECK(byte_length >= 0),
  last_verified_at_ms INTEGER,
  created_at_ms INTEGER NOT NULL,
  UNIQUE(storage_root_id, relative_path)
) STRICT;

CREATE TABLE selection_profiles (
  profile_id TEXT PRIMARY KEY,
  pack_id TEXT NOT NULL REFERENCES packs(pack_id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  current_revision INTEGER NOT NULL CHECK(current_revision >= 1),
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE selection_profile_revisions (
  profile_id TEXT NOT NULL REFERENCES selection_profiles(profile_id) ON DELETE CASCADE,
  revision INTEGER NOT NULL CHECK(revision >= 1),
  canonical_json TEXT NOT NULL CHECK(json_valid(canonical_json)),
  profile_sha256 TEXT NOT NULL CHECK(length(profile_sha256) = 64),
  created_at_ms INTEGER NOT NULL,
  PRIMARY KEY(profile_id, revision)
) STRICT, WITHOUT ROWID;

CREATE TABLE download_jobs (
  download_job_id TEXT PRIMARY KEY,
  catalog_entry_id TEXT REFERENCES catalog_entries(catalog_entry_id) ON DELETE SET NULL,
  source_uri TEXT NOT NULL,
  destination_storage_root_id TEXT NOT NULL REFERENCES storage_roots(storage_root_id) ON DELETE RESTRICT,
  expected_size_bytes INTEGER NOT NULL CHECK(expected_size_bytes > 0),
  expected_sha256 TEXT NOT NULL CHECK(length(expected_sha256) = 64),
  state TEXT NOT NULL CHECK(state IN ('planned','fetching','paused','verifying','staged','activating','completed','failed','cancelled')),
  bytes_verified INTEGER NOT NULL DEFAULT 0 CHECK(bytes_verified >= 0),
  etag TEXT,
  last_modified TEXT,
  error_code TEXT,
  error_detail TEXT,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  completed_at_ms INTEGER
) STRICT;

CREATE TABLE download_chunks (
  download_job_id TEXT NOT NULL REFERENCES download_jobs(download_job_id) ON DELETE CASCADE,
  chunk_index INTEGER NOT NULL CHECK(chunk_index >= 0),
  start_byte INTEGER NOT NULL CHECK(start_byte >= 0),
  end_byte_exclusive INTEGER NOT NULL CHECK(end_byte_exclusive > start_byte),
  expected_sha256 TEXT CHECK(expected_sha256 IS NULL OR length(expected_sha256) = 64),
  actual_sha256 TEXT CHECK(actual_sha256 IS NULL OR length(actual_sha256) = 64),
  state TEXT NOT NULL CHECK(state IN ('pending','fetching','verified','failed')),
  attempts INTEGER NOT NULL DEFAULT 0 CHECK(attempts >= 0),
  updated_at_ms INTEGER NOT NULL,
  PRIMARY KEY(download_job_id, chunk_index)
) STRICT, WITHOUT ROWID;

CREATE TABLE install_transactions (
  install_transaction_id TEXT PRIMARY KEY,
  resource_kind TEXT NOT NULL CHECK(resource_kind IN ('pack','model','restore')),
  resource_id TEXT NOT NULL,
  previous_active_id TEXT,
  candidate_id TEXT NOT NULL,
  state TEXT NOT NULL CHECK(state IN ('planned','staging','verifying','ready','activating','committed','rolled_back','failed')),
  journal_json TEXT NOT NULL CHECK(json_valid(journal_json)),
  error_code TEXT,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  committed_at_ms INTEGER
) STRICT;

CREATE TABLE reader_positions (
  pack_id TEXT NOT NULL REFERENCES packs(pack_id) ON DELETE CASCADE,
  article_key TEXT NOT NULL,
  scroll_anchor TEXT NOT NULL,
  progress_fraction REAL NOT NULL CHECK(progress_fraction >= 0.0 AND progress_fraction <= 1.0),
  updated_at_ms INTEGER NOT NULL,
  PRIMARY KEY(pack_id, article_key)
) STRICT, WITHOUT ROWID;

CREATE TABLE reader_history (
  history_id TEXT PRIMARY KEY,
  pack_id TEXT NOT NULL REFERENCES packs(pack_id) ON DELETE CASCADE,
  article_key TEXT NOT NULL,
  title TEXT NOT NULL,
  visited_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE bookmarks (
  bookmark_id TEXT PRIMARY KEY,
  pack_id TEXT NOT NULL REFERENCES packs(pack_id) ON DELETE CASCADE,
  article_key TEXT NOT NULL,
  title TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL,
  UNIQUE(pack_id, article_key)
) STRICT;

CREATE TABLE notebooks (
  notebook_id TEXT PRIMARY KEY,
  parent_notebook_id TEXT REFERENCES notebooks(notebook_id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  description TEXT NOT NULL,
  sort_order INTEGER NOT NULL DEFAULT 0,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  UNIQUE(parent_notebook_id, name)
) STRICT;

CREATE TABLE tags (
  tag_id TEXT PRIMARY KEY,
  name TEXT NOT NULL COLLATE NOCASE UNIQUE,
  color_token TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE sidecar_layers (
  sidecar_layer_id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  owner_device_id TEXT REFERENCES devices(device_id) ON DELETE SET NULL,
  visibility TEXT NOT NULL CHECK(visibility IN ('private','shared','imported')),
  trust_state TEXT NOT NULL CHECK(trust_state IN ('local','verified','unverified','rejected')),
  source_artifact_id TEXT,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE annotations (
  annotation_id TEXT PRIMARY KEY,
  annotation_type TEXT NOT NULL CHECK(annotation_type IN ('highlight','comment','ink','voice','bookmark_note')),
  pack_id TEXT NOT NULL REFERENCES packs(pack_id) ON DELETE CASCADE,
  article_key TEXT NOT NULL,
  notebook_id TEXT REFERENCES notebooks(notebook_id) ON DELETE SET NULL,
  sidecar_layer_id TEXT NOT NULL REFERENCES sidecar_layers(sidecar_layer_id) ON DELETE RESTRICT,
  current_revision INTEGER NOT NULL CHECK(current_revision >= 1),
  resolution_state TEXT NOT NULL CHECK(resolution_state IN ('resolved','ambiguous','source_unavailable','orphaned')),
  pinned INTEGER NOT NULL DEFAULT 0 CHECK(pinned IN (0,1)),
  deleted_at_ms INTEGER,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE annotation_revisions (
  annotation_id TEXT NOT NULL REFERENCES annotations(annotation_id) ON DELETE CASCADE,
  revision INTEGER NOT NULL CHECK(revision >= 1),
  body_text TEXT,
  selector_json TEXT NOT NULL CHECK(json_valid(selector_json)),
  style_json TEXT NOT NULL CHECK(json_valid(style_json)),
  context_text TEXT NOT NULL,
  source_revision_digest TEXT,
  authored_by_device_id TEXT REFERENCES devices(device_id) ON DELETE SET NULL,
  created_at_ms INTEGER NOT NULL,
  PRIMARY KEY(annotation_id, revision)
) STRICT, WITHOUT ROWID;

CREATE TABLE annotation_assets (
  annotation_asset_id TEXT PRIMARY KEY,
  annotation_id TEXT NOT NULL REFERENCES annotations(annotation_id) ON DELETE CASCADE,
  asset_kind TEXT NOT NULL CHECK(asset_kind IN ('ink_svg','voice_opus','image','file')),
  object_sha256 TEXT NOT NULL CHECK(length(object_sha256) = 64),
  mime_type TEXT NOT NULL,
  byte_length INTEGER NOT NULL CHECK(byte_length > 0),
  duration_ms INTEGER CHECK(duration_ms IS NULL OR duration_ms >= 0),
  created_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE annotation_tags (
  annotation_id TEXT NOT NULL REFERENCES annotations(annotation_id) ON DELETE CASCADE,
  tag_id TEXT NOT NULL REFERENCES tags(tag_id) ON DELETE CASCADE,
  PRIMARY KEY(annotation_id, tag_id)
) STRICT, WITHOUT ROWID;

CREATE TABLE sidecar_imports (
  sidecar_import_id TEXT PRIMARY KEY,
  artifact_sha256 TEXT NOT NULL UNIQUE CHECK(length(artifact_sha256) = 64),
  signature_state TEXT NOT NULL CHECK(signature_state IN ('verified','unsigned','invalid','unknown_key')),
  imported_layer_id TEXT REFERENCES sidecar_layers(sidecar_layer_id) ON DELETE SET NULL,
  record_count INTEGER NOT NULL CHECK(record_count >= 0),
  conflict_count INTEGER NOT NULL CHECK(conflict_count >= 0),
  imported_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE conversations (
  conversation_id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  scope_json TEXT NOT NULL CHECK(json_valid(scope_json)),
  backend_id TEXT,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  archived_at_ms INTEGER
) STRICT;

CREATE TABLE messages (
  message_id TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL REFERENCES conversations(conversation_id) ON DELETE CASCADE,
  role TEXT NOT NULL CHECK(role IN ('user','assistant','system_notice')),
  body_markdown TEXT NOT NULL,
  sequence_no INTEGER NOT NULL CHECK(sequence_no >= 0),
  inference_run_id TEXT,
  created_at_ms INTEGER NOT NULL,
  UNIQUE(conversation_id, sequence_no)
) STRICT;

CREATE TABLE retrieval_documents (
  retrieval_document_id TEXT PRIMARY KEY,
  source_kind TEXT NOT NULL CHECK(source_kind IN ('article','annotation','lore','message')),
  source_id TEXT NOT NULL,
  pack_id TEXT REFERENCES packs(pack_id) ON DELETE CASCADE,
  article_key TEXT,
  title TEXT NOT NULL,
  content_digest TEXT NOT NULL CHECK(length(content_digest) = 64),
  producer_version TEXT NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  UNIQUE(source_kind, source_id, content_digest)
) STRICT;

CREATE TABLE retrieval_chunks (
  retrieval_chunk_id TEXT PRIMARY KEY,
  retrieval_document_id TEXT NOT NULL REFERENCES retrieval_documents(retrieval_document_id) ON DELETE CASCADE,
  ordinal INTEGER NOT NULL CHECK(ordinal >= 0),
  start_char INTEGER NOT NULL CHECK(start_char >= 0),
  end_char INTEGER NOT NULL CHECK(end_char > start_char),
  text_content TEXT NOT NULL,
  text_digest TEXT NOT NULL CHECK(length(text_digest) = 64),
  locator_json TEXT NOT NULL CHECK(json_valid(locator_json)),
  token_count INTEGER NOT NULL CHECK(token_count > 0),
  UNIQUE(retrieval_document_id, ordinal)
) STRICT;

CREATE VIRTUAL TABLE retrieval_chunks_fts USING fts5(
  title,
  text_content,
  content='',
  tokenize='unicode61 remove_diacritics 2'
);

CREATE TABLE embedding_spaces (
  embedding_space_id TEXT PRIMARY KEY,
  model_id TEXT NOT NULL,
  model_sha256 TEXT NOT NULL CHECK(length(model_sha256) = 64),
  dimensions INTEGER NOT NULL CHECK(dimensions > 0),
  distance_metric TEXT NOT NULL CHECK(distance_metric IN ('cosine','dot','l2')),
  normalized INTEGER NOT NULL CHECK(normalized IN (0,1)),
  created_at_ms INTEGER NOT NULL,
  UNIQUE(model_sha256, dimensions, distance_metric, normalized)
) STRICT;

CREATE TABLE chunk_embeddings (
  retrieval_chunk_id TEXT NOT NULL REFERENCES retrieval_chunks(retrieval_chunk_id) ON DELETE CASCADE,
  embedding_space_id TEXT NOT NULL REFERENCES embedding_spaces(embedding_space_id) ON DELETE CASCADE,
  vector_le_f32 BLOB NOT NULL,
  vector_sha256 TEXT NOT NULL CHECK(length(vector_sha256) = 64),
  created_at_ms INTEGER NOT NULL,
  PRIMARY KEY(retrieval_chunk_id, embedding_space_id)
) STRICT, WITHOUT ROWID;

CREATE TABLE inference_backends (
  backend_id TEXT PRIMARY KEY,
  backend_kind TEXT NOT NULL CHECK(backend_kind IN ('litert_lm','gguf')),
  display_name TEXT NOT NULL,
  enabled INTEGER NOT NULL CHECK(enabled IN (0,1)),
  priority INTEGER NOT NULL,
  config_json TEXT NOT NULL CHECK(json_valid(config_json)),
  last_health_json TEXT CHECK(last_health_json IS NULL OR json_valid(last_health_json)),
  updated_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE models (
  model_id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  family TEXT NOT NULL,
  model_kind TEXT NOT NULL CHECK(model_kind IN ('chat','embedding','reranker')),
  format TEXT NOT NULL CHECK(format IN ('litert','gguf')),
  manifest_json TEXT NOT NULL CHECK(json_valid(manifest_json)),
  artifact_sha256 TEXT NOT NULL CHECK(length(artifact_sha256) = 64),
  artifact_size_bytes INTEGER NOT NULL CHECK(artifact_size_bytes > 0),
  created_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE model_installations (
  model_installation_id TEXT PRIMARY KEY,
  model_id TEXT NOT NULL REFERENCES models(model_id) ON DELETE CASCADE,
  backend_id TEXT NOT NULL REFERENCES inference_backends(backend_id) ON DELETE RESTRICT,
  storage_root_id TEXT NOT NULL REFERENCES storage_roots(storage_root_id) ON DELETE RESTRICT,
  relative_path TEXT NOT NULL,
  state TEXT NOT NULL CHECK(state IN ('staged','verified','active','retained','quarantined')),
  installed_at_ms INTEGER NOT NULL,
  last_used_at_ms INTEGER,
  UNIQUE(model_id, backend_id, storage_root_id, relative_path)
) STRICT;

CREATE TABLE inference_runs (
  inference_run_id TEXT PRIMARY KEY,
  conversation_id TEXT REFERENCES conversations(conversation_id) ON DELETE SET NULL,
  backend_id TEXT NOT NULL REFERENCES inference_backends(backend_id) ON DELETE RESTRICT,
  model_id TEXT NOT NULL REFERENCES models(model_id) ON DELETE RESTRICT,
  scope_json TEXT NOT NULL CHECK(json_valid(scope_json)),
  prompt_digest TEXT NOT NULL CHECK(length(prompt_digest) = 64),
  status TEXT NOT NULL CHECK(status IN ('running','completed','cancelled','failed')),
  input_tokens INTEGER NOT NULL DEFAULT 0 CHECK(input_tokens >= 0),
  output_tokens INTEGER NOT NULL DEFAULT 0 CHECK(output_tokens >= 0),
  started_at_ms INTEGER NOT NULL,
  completed_at_ms INTEGER,
  error_code TEXT
) STRICT;

CREATE TABLE message_citations (
  message_id TEXT NOT NULL REFERENCES messages(message_id) ON DELETE CASCADE,
  citation_index INTEGER NOT NULL CHECK(citation_index >= 1),
  retrieval_chunk_id TEXT NOT NULL REFERENCES retrieval_chunks(retrieval_chunk_id) ON DELETE RESTRICT,
  quote_text TEXT NOT NULL,
  start_char INTEGER NOT NULL CHECK(start_char >= 0),
  end_char INTEGER NOT NULL CHECK(end_char > start_char),
  confidence REAL NOT NULL CHECK(confidence >= 0.0 AND confidence <= 1.0),
  PRIMARY KEY(message_id, citation_index)
) STRICT, WITHOUT ROWID;

CREATE TABLE lore_evidence (
  lore_evidence_id TEXT PRIMARY KEY,
  source_kind TEXT NOT NULL CHECK(source_kind IN ('message','annotation','article','user_assertion')),
  source_id TEXT NOT NULL,
  excerpt TEXT NOT NULL,
  locator_json TEXT NOT NULL CHECK(json_valid(locator_json)),
  content_digest TEXT NOT NULL CHECK(length(content_digest) = 64),
  created_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE lore_candidates (
  lore_candidate_id TEXT PRIMARY KEY,
  statement TEXT NOT NULL,
  subject_key TEXT NOT NULL,
  predicate_key TEXT NOT NULL,
  object_json TEXT NOT NULL CHECK(json_valid(object_json)),
  confidence REAL NOT NULL CHECK(confidence >= 0.0 AND confidence <= 1.0),
  status TEXT NOT NULL CHECK(status IN ('inbox','accepted','rejected','expired')),
  proposed_by_run_id TEXT REFERENCES inference_runs(inference_run_id) ON DELETE SET NULL,
  created_at_ms INTEGER NOT NULL,
  reviewed_at_ms INTEGER
) STRICT;

CREATE TABLE lore_candidate_evidence (
  lore_candidate_id TEXT NOT NULL REFERENCES lore_candidates(lore_candidate_id) ON DELETE CASCADE,
  lore_evidence_id TEXT NOT NULL REFERENCES lore_evidence(lore_evidence_id) ON DELETE RESTRICT,
  PRIMARY KEY(lore_candidate_id, lore_evidence_id)
) STRICT, WITHOUT ROWID;

CREATE TABLE lore_memories (
  memory_id TEXT PRIMARY KEY,
  subject_key TEXT NOT NULL,
  predicate_key TEXT NOT NULL,
  current_revision INTEGER NOT NULL CHECK(current_revision >= 1),
  pinned INTEGER NOT NULL DEFAULT 0 CHECK(pinned IN (0,1)),
  status TEXT NOT NULL CHECK(status IN ('active','superseded','deleted')),
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  deleted_at_ms INTEGER
) STRICT;

CREATE TABLE lore_memory_revisions (
  memory_id TEXT NOT NULL REFERENCES lore_memories(memory_id) ON DELETE CASCADE,
  revision INTEGER NOT NULL CHECK(revision >= 1),
  statement TEXT NOT NULL,
  object_json TEXT NOT NULL CHECK(json_valid(object_json)),
  correction_reason TEXT,
  authored_by TEXT NOT NULL CHECK(authored_by IN ('user','policy')),
  created_at_ms INTEGER NOT NULL,
  PRIMARY KEY(memory_id, revision)
) STRICT, WITHOUT ROWID;

CREATE TABLE lore_memory_evidence (
  memory_id TEXT NOT NULL REFERENCES lore_memories(memory_id) ON DELETE CASCADE,
  memory_revision INTEGER NOT NULL,
  lore_evidence_id TEXT NOT NULL REFERENCES lore_evidence(lore_evidence_id) ON DELETE RESTRICT,
  PRIMARY KEY(memory_id, memory_revision, lore_evidence_id),
  FOREIGN KEY(memory_id, memory_revision)
    REFERENCES lore_memory_revisions(memory_id, revision)
    ON DELETE CASCADE
) STRICT, WITHOUT ROWID;

CREATE TABLE trust_keys (
  trust_key_id TEXT PRIMARY KEY,
  key_kind TEXT NOT NULL CHECK(key_kind IN ('root','catalog','sidecar','entitlement','release')),
  label TEXT NOT NULL,
  ed25519_public_key BLOB NOT NULL CHECK(length(ed25519_public_key) = 32),
  trust_state TEXT NOT NULL CHECK(trust_state IN ('trusted','untrusted','revoked')),
  valid_from_ms INTEGER NOT NULL,
  valid_until_ms INTEGER,
  created_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE entitlement_artifacts (
  entitlement_artifact_id TEXT PRIMARY KEY,
  subject_device_id TEXT REFERENCES devices(device_id) ON DELETE SET NULL,
  artifact_cbor BLOB NOT NULL,
  artifact_sha256 TEXT NOT NULL UNIQUE CHECK(length(artifact_sha256) = 64),
  issuer_key_id TEXT NOT NULL REFERENCES trust_keys(trust_key_id) ON DELETE RESTRICT,
  issued_at_ms INTEGER NOT NULL,
  not_before_ms INTEGER NOT NULL,
  expires_at_ms INTEGER,
  verification_state TEXT NOT NULL CHECK(verification_state IN ('verified','expired','invalid','revoked')),
  imported_at_ms INTEGER NOT NULL
) STRICT;

CREATE TABLE entitlement_grants (
  entitlement_artifact_id TEXT NOT NULL REFERENCES entitlement_artifacts(entitlement_artifact_id) ON DELETE CASCADE,
  capability_key TEXT NOT NULL,
  grant_kind TEXT NOT NULL CHECK(grant_kind IN ('perpetual','term','quantity')),
  quantity INTEGER,
  valid_until_ms INTEGER,
  PRIMARY KEY(entitlement_artifact_id, capability_key)
) STRICT, WITHOUT ROWID;

CREATE TABLE coupon_ledger (
  coupon_redemption_id TEXT PRIMARY KEY,
  coupon_id TEXT NOT NULL,
  entitlement_artifact_id TEXT REFERENCES entitlement_artifacts(entitlement_artifact_id) ON DELETE SET NULL,
  redeemed_on_device_id TEXT NOT NULL REFERENCES devices(device_id) ON DELETE RESTRICT,
  redeemed_at_ms INTEGER NOT NULL,
  redemption_nonce BLOB NOT NULL,
  receipt_sha256 TEXT NOT NULL UNIQUE CHECK(length(receipt_sha256) = 64),
  status TEXT NOT NULL CHECK(status IN ('accepted','reconciled','rejected')),
  UNIQUE(coupon_id, redeemed_on_device_id)
) STRICT;

CREATE TABLE backups (
  backup_id TEXT PRIMARY KEY,
  destination_kind TEXT NOT NULL CHECK(destination_kind IN ('file','directory','android_document')),
  destination_locator TEXT NOT NULL,
  manifest_sha256 TEXT NOT NULL CHECK(length(manifest_sha256) = 64),
  encrypted INTEGER NOT NULL CHECK(encrypted IN (0,1)),
  byte_length INTEGER NOT NULL CHECK(byte_length > 0),
  state TEXT NOT NULL CHECK(state IN ('creating','verified','failed')),
  created_at_ms INTEGER NOT NULL,
  verified_at_ms INTEGER
) STRICT;

CREATE TABLE user_tasks (
  task_id TEXT PRIMARY KEY,
  task_kind TEXT NOT NULL CHECK(task_kind IN ('download_attention','annotation_ambiguous','memory_review','backup_due','model_attention')),
  title TEXT NOT NULL,
  detail_json TEXT NOT NULL CHECK(json_valid(detail_json)),
  status TEXT NOT NULL CHECK(status IN ('open','resolved','dismissed')),
  priority INTEGER NOT NULL CHECK(priority BETWEEN 0 AND 100),
  created_at_ms INTEGER NOT NULL,
  resolved_at_ms INTEGER
) STRICT;

CREATE TABLE audit_events (
  audit_event_id TEXT PRIMARY KEY,
  event_type TEXT NOT NULL,
  entity_kind TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  detail_json TEXT NOT NULL CHECK(json_valid(detail_json)),
  severity TEXT NOT NULL CHECK(severity IN ('info','warning','error','security')),
  created_at_ms INTEGER NOT NULL
) STRICT;

CREATE INDEX idx_catalog_entries_search
  ON catalog_entries(language_bcp47, category, title);
CREATE INDEX idx_catalog_entries_corpus_version
  ON catalog_entries(corpus_id, upstream_version);
CREATE INDEX idx_pack_versions_pack_created
  ON pack_versions(pack_id, created_at_ms DESC);
CREATE INDEX idx_pack_locations_version_kind
  ON pack_locations(pack_version_id, locator_kind);
CREATE INDEX idx_download_jobs_state_updated
  ON download_jobs(state, updated_at_ms DESC);
CREATE INDEX idx_download_chunks_state
  ON download_chunks(download_job_id, state, chunk_index);
CREATE INDEX idx_install_transactions_state
  ON install_transactions(state, updated_at_ms DESC);
CREATE INDEX idx_reader_history_recent
  ON reader_history(visited_at_ms DESC);
CREATE INDEX idx_annotations_article
  ON annotations(pack_id, article_key, deleted_at_ms);
CREATE INDEX idx_annotations_notebook
  ON annotations(notebook_id, updated_at_ms DESC);
CREATE INDEX idx_annotations_layer
  ON annotations(sidecar_layer_id, updated_at_ms DESC);
CREATE INDEX idx_annotation_revisions_created
  ON annotation_revisions(annotation_id, created_at_ms DESC);
CREATE INDEX idx_conversations_recent
  ON conversations(updated_at_ms DESC);
CREATE INDEX idx_messages_conversation_sequence
  ON messages(conversation_id, sequence_no);
CREATE INDEX idx_retrieval_documents_source
  ON retrieval_documents(source_kind, source_id);
CREATE INDEX idx_retrieval_chunks_document
  ON retrieval_chunks(retrieval_document_id, ordinal);
CREATE INDEX idx_inference_runs_status
  ON inference_runs(status, started_at_ms DESC);
CREATE INDEX idx_lore_candidates_inbox
  ON lore_candidates(status, created_at_ms DESC);
CREATE INDEX idx_lore_memories_subject
  ON lore_memories(subject_key, predicate_key, status);
CREATE INDEX idx_entitlement_state
  ON entitlement_artifacts(verification_state, expires_at_ms);
CREATE INDEX idx_user_tasks_open
  ON user_tasks(status, priority DESC, created_at_ms);
CREATE INDEX idx_audit_entity
  ON audit_events(entity_kind, entity_id, created_at_ms DESC);

CREATE TRIGGER trg_pack_active_version_same_pack
BEFORE UPDATE OF active_pack_version_id ON packs
WHEN NEW.active_pack_version_id IS NOT NULL
BEGIN
  SELECT CASE WHEN NOT EXISTS (
    SELECT 1
    FROM pack_versions pv
    WHERE pv.pack_version_id = NEW.active_pack_version_id
      AND pv.pack_id = NEW.pack_id
      AND pv.verification_state = 'verified'
  ) THEN RAISE(ABORT, 'active pack version must be verified and belong to pack') END;
END;

CREATE TRIGGER trg_pack_active_version_insert
BEFORE INSERT ON packs
WHEN NEW.active_pack_version_id IS NOT NULL
BEGIN
  SELECT RAISE(
    ABORT,
    'packs must be inserted without an active version; activate after verified version insert'
  );
END;

CREATE TRIGGER trg_annotation_revision_insert
AFTER INSERT ON annotation_revisions
BEGIN
  UPDATE annotations
  SET current_revision = NEW.revision,
      updated_at_ms = NEW.created_at_ms
  WHERE annotation_id = NEW.annotation_id
    AND NEW.revision > current_revision;
END;

CREATE TRIGGER trg_lore_revision_insert
AFTER INSERT ON lore_memory_revisions
BEGIN
  UPDATE lore_memories
  SET current_revision = NEW.revision,
      updated_at_ms = NEW.created_at_ms
  WHERE memory_id = NEW.memory_id
    AND NEW.revision > current_revision;
END;

CREATE TRIGGER trg_retrieval_chunk_fts_insert
AFTER INSERT ON retrieval_chunks
BEGIN
  INSERT INTO retrieval_chunks_fts(rowid, title, text_content)
  SELECT NEW.rowid, d.title, NEW.text_content
  FROM retrieval_documents d
  WHERE d.retrieval_document_id = NEW.retrieval_document_id;
END;

CREATE TRIGGER trg_retrieval_chunk_fts_delete
AFTER DELETE ON retrieval_chunks
BEGIN
  INSERT INTO retrieval_chunks_fts(retrieval_chunks_fts, rowid, title, text_content)
  VALUES('delete', OLD.rowid, '', OLD.text_content);
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

CREATE TRIGGER trg_install_state_guard
BEFORE UPDATE OF state ON install_transactions
WHEN OLD.state <> NEW.state
BEGIN
  SELECT CASE WHEN NOT (
    (OLD.state = 'planned' AND NEW.state IN ('staging','failed')) OR
    (OLD.state = 'staging' AND NEW.state IN ('verifying','failed','rolled_back')) OR
    (OLD.state = 'verifying' AND NEW.state IN ('ready','failed','rolled_back')) OR
    (OLD.state = 'ready' AND NEW.state IN ('activating','rolled_back')) OR
    (OLD.state = 'activating' AND NEW.state IN ('committed','failed','rolled_back')) OR
    (OLD.state = 'failed' AND NEW.state = 'rolled_back')
  ) THEN RAISE(ABORT, 'invalid install state transition') END;
END;

COMMIT;
