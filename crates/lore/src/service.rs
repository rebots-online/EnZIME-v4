#![forbid(unsafe_code)]

//! SQLite-backed implementation of [`ports::lore::LoreService`] for
//! ARCHITECTURE.md §12 (memory lifecycle, contracts, extraction policy,
//! consolidation, deletion).
//!
//! The service owns a single writer [`Connection`] guarded by a `Mutex`.
//! Trait methods do not hold the mutex across `.await` points: each method
//! locks the connection, performs all SQL work synchronously inside one
//! transaction, commits, and returns the materialised result. This is the
//! right shape for a single-process offline reader where the SQL work per
//! call is bounded and the alternative (background executor + IPC) would
//! add complexity the spec does not ask for.
//!
//! The schema (`schemas/sqlite/0001_initial.sql`) already provides the five
//! §12.1 tables — `lore_evidence`, `lore_candidates`, `lore_candidate_evidence`,
//! `lore_memories`, `lore_memory_revisions`, `lore_memory_evidence` — and a
//! trigger (`trg_lore_revision_insert`) that propagates each new revision's
//! `revision` and `created_at_ms` into `lore_memories.current_revision` and
//! `.updated_at_ms` respectively. This service relies on both.

use std::fmt::Write as _;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use domain::lore::{
    LoreCandidate, LoreCorrection, LoreEvidence, LoreMemory, LoreMemoryStatus, LoreQuery,
};
use ports::lore::{LoreError, LoreService};
use rusqlite::{params, Connection, OptionalExtension, Transaction};

use crate::policy::{self, PromotionPolicy};

/// SQLite-backed lore service.
pub struct SqliteLoreService {
    conn: Arc<Mutex<Connection>>,
    promotion_policy: PromotionPolicy,
}

impl SqliteLoreService {
    /// Wrap an open connection (with §6.1 pragmas and the §6.2 schema already
    /// applied) and a promotion policy. The service takes ownership of the
    /// connection.
    pub fn new(conn: Connection, promotion_policy: PromotionPolicy) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
            promotion_policy,
        }
    }

    /// Convenience constructor for the common case where no auto-promotion
    /// policy is configured — every acceptance is an explicit user action,
    /// exactly the §A-08 default.
    pub fn without_promotion(conn: Connection) -> Self {
        Self::new(conn, PromotionPolicy::default())
    }
}

/// Wall-clock milliseconds since the unix epoch. Falls back to 0 if the system
/// clock is before the epoch (the same convention as `storage-sqlite`).
fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Lowercase hexadecimal SHA-256 of a byte slice. Used for deterministic,
/// idempotent identifier generation so re-running `extract_candidates`
/// against the same conversation does not produce duplicate candidates.
fn sha256_hex(bytes: &[u8]) -> String {
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

/// Convert a stored status string into the typed enum. The schema's CHECK
/// constraint limits the value to the three named variants, so the fallback
/// is unreachable in practice.
fn parse_status(s: &str) -> LoreMemoryStatus {
    match s {
        "active" => LoreMemoryStatus::Active,
        "superseded" => LoreMemoryStatus::Superseded,
        "deleted" => LoreMemoryStatus::Deleted,
        _ => LoreMemoryStatus::Active,
    }
}

/// A user message used as extraction source material.
struct SourceMessage {
    message_id: String,
    body: String,
    created_at_ms: i64,
}

fn load_user_messages(
    tx: &Transaction,
    conversation_id: &str,
) -> Result<Vec<SourceMessage>, LoreError> {
    let mut stmt = tx
        .prepare(
            "SELECT message_id, body_markdown, created_at_ms\n\
             \x20\x20FROM messages\n\
             \x20\x20WHERE conversation_id = ?1 AND role = 'user'\n\
             \x20\x20ORDER BY sequence_no ASC",
        )
        .map_err(|e| LoreError::Storage(e.to_string()))?;
    let rows = stmt
        .query_map(params![conversation_id], |r| {
            Ok(SourceMessage {
                message_id: r.get(0)?,
                body: r.get(1)?,
                created_at_ms: r.get(2)?,
            })
        })
        .map_err(|e| LoreError::Storage(e.to_string()))?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| LoreError::Storage(e.to_string()))?);
    }
    Ok(out)
}

fn load_evidence_for_candidate(
    tx: &Transaction,
    candidate_id: &str,
) -> Result<Vec<LoreEvidence>, LoreError> {
    let mut stmt = tx
        .prepare(
            "SELECT e.lore_evidence_id, e.source_kind, e.source_id, e.excerpt,\n\
             \x20\x20\x20\x20\x20\x20\x20 e.locator_json, e.content_digest\n\
             \x20\x20FROM lore_evidence e\n\
             \x20\x20JOIN lore_candidate_evidence ce\n\
             \x20\x20\x20\x20ON ce.lore_evidence_id = e.lore_evidence_id\n\
             \x20\x20WHERE ce.lore_candidate_id = ?1",
        )
        .map_err(|e| LoreError::Storage(e.to_string()))?;
    let rows = stmt
        .query_map(params![candidate_id], |r| {
            let locator_json: String = r.get(4)?;
            Ok(LoreEvidence {
                lore_evidence_id: r.get(0)?,
                source_kind: r.get(1)?,
                source_id: r.get(2)?,
                excerpt: r.get(3)?,
                locator: serde_json::from_str(&locator_json)
                    .unwrap_or(serde_json::Value::Null),
                content_digest: r.get(5)?,
            })
        })
        .map_err(|e| LoreError::Storage(e.to_string()))?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| LoreError::Storage(e.to_string()))?);
    }
    Ok(out)
}

fn load_evidence_for_memory(
    tx: &Transaction,
    memory_id: &str,
    revision: u32,
) -> Result<Vec<LoreEvidence>, LoreError> {
    let mut stmt = tx
        .prepare(
            "SELECT e.lore_evidence_id, e.source_kind, e.source_id, e.excerpt,\n\
             \x20\x20\x20\x20\x20\x20\x20 e.locator_json, e.content_digest\n\
             \x20\x20FROM lore_evidence e\n\
             \x20\x20JOIN lore_memory_evidence me\n\
             \x20\x20\x20\x20ON me.lore_evidence_id = e.lore_evidence_id\n\
             \x20\x20WHERE me.memory_id = ?1 AND me.memory_revision = ?2",
        )
        .map_err(|e| LoreError::Storage(e.to_string()))?;
    let rows = stmt
        .query_map(params![memory_id, revision as i64], |r| {
            let locator_json: String = r.get(4)?;
            Ok(LoreEvidence {
                lore_evidence_id: r.get(0)?,
                source_kind: r.get(1)?,
                source_id: r.get(2)?,
                excerpt: r.get(3)?,
                locator: serde_json::from_str(&locator_json)
                    .unwrap_or(serde_json::Value::Null),
                content_digest: r.get(5)?,
            })
        })
        .map_err(|e| LoreError::Storage(e.to_string()))?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| LoreError::Storage(e.to_string()))?);
    }
    Ok(out)
}

fn load_candidate(
    tx: &Transaction,
    candidate_id: &str,
) -> Result<LoreCandidate, LoreError> {
    let row = tx
        .query_row(
            "SELECT lore_candidate_id, statement, subject_key, predicate_key,\n\
             \x20\x20\x20\x20\x20\x20 object_json, confidence, created_at_ms\n\
             \x20\x20FROM lore_candidates\n\
             \x20\x20WHERE lore_candidate_id = ?1",
            params![candidate_id],
            |r| {
                let object_json: String = r.get(4)?;
                Ok(LoreCandidate {
                    lore_candidate_id: r.get(0)?,
                    statement: r.get(1)?,
                    subject_key: r.get(2)?,
                    predicate_key: r.get(3)?,
                    object: serde_json::from_str(&object_json)
                        .unwrap_or(serde_json::Value::Null),
                    confidence: r.get(5)?,
                    evidence: Vec::new(),
                    created_at_ms: r.get(6)?,
                })
            },
        )
        .optional()
        .map_err(|e| LoreError::Storage(e.to_string()))?;
    let mut candidate = row.ok_or(LoreError::CandidateNotFound)?;
    candidate.evidence = load_evidence_for_candidate(tx, candidate_id)?;
    Ok(candidate)
}

fn load_memory(tx: &Transaction, memory_id: &str) -> Result<LoreMemory, LoreError> {
    let row = tx
        .query_row(
            "SELECT m.memory_id, m.subject_key, m.predicate_key, m.current_revision,\n\
             \x20\x20\x20\x20\x20\x20 m.pinned, m.status, m.created_at_ms, m.updated_at_ms,\n\
             \x20\x20\x20\x20\x20\x20 r.statement, r.object_json\n\
             \x20\x20FROM lore_memories m\n\
             \x20\x20JOIN lore_memory_revisions r\n\
             \x20\x20\x20\x20ON r.memory_id = m.memory_id AND r.revision = m.current_revision\n\
             \x20\x20WHERE m.memory_id = ?1",
            params![memory_id],
            |r| {
                let pinned: i64 = r.get(4)?;
                let status: String = r.get(5)?;
                let object_json: String = r.get(9)?;
                let revision: i64 = r.get(3)?;
                Ok(LoreMemory {
                    memory_id: r.get(0)?,
                    subject_key: r.get(1)?,
                    predicate_key: r.get(2)?,
                    statement: r.get(8)?,
                    object: serde_json::from_str(&object_json)
                        .unwrap_or(serde_json::Value::Null),
                    revision: revision as u32,
                    pinned: pinned != 0,
                    status: parse_status(&status),
                    evidence: Vec::new(),
                    created_at_ms: r.get(6)?,
                    updated_at_ms: r.get(7)?,
                })
            },
        )
        .optional()
        .map_err(|e| LoreError::Storage(e.to_string()))?;
    let mut memory = row.ok_or(LoreError::MemoryNotFound)?;
    memory.evidence = load_evidence_for_memory(tx, memory_id, memory.revision)?;
    Ok(memory)
}

/// One classified, pre-consolidation assertion pulled out of a message body.
struct RawCandidate {
    statement: String,
    subject_key: String,
    predicate_key: String,
    object: serde_json::Value,
    confidence: f32,
    excerpt: String,
    message_id: String,
    created_at_ms: i64,
}

/// A consolidated candidate ready to persist. Built by grouping raw
/// candidates by `(subject_key, predicate_key, object_json)` per §12.4.
struct GroupedCandidate {
    statement: String,
    subject_key: String,
    predicate_key: String,
    object: serde_json::Value,
    confidence: f32,
    evidence: Vec<(String, String, i64, serde_json::Value)>, // (excerpt, message_id, created_at_ms, locator)
}

/// Promote a candidate into a memory with `authored_by = 'user'` (explicit
/// user action) or `authored_by = 'policy'` (user-configured deterministic
/// promotion policy per §12.3). The optional `policy_reason` is recorded as
/// the revision's `correction_reason` so the Memory Centre can display which
/// rule promoted which candidate.
fn promote_candidate(
    tx: &Transaction,
    candidate_id: &str,
    authored_by: &str,
    policy_reason: Option<&str>,
    now: i64,
) -> Result<LoreMemory, LoreError> {
    let candidate = load_candidate(tx, candidate_id)?;
    let object_json =
        serde_json::to_string(&candidate.object).unwrap_or_else(|_| "null".into());

    // If a memory already exists with the same (subject, predicate, object)
    // and is active, link this candidate to it rather than duplicating.
    let existing_same: Option<String> = tx
        .query_row(
            "SELECT m.memory_id\n\
             \x20\x20FROM lore_memories m\n\
             \x20\x20JOIN lore_memory_revisions r\n\
             \x20\x20\x20\x20ON r.memory_id = m.memory_id\n\
             \x20\x20\x20\x20   AND r.revision = m.current_revision\n\
             \x20\x20WHERE m.subject_key = ?1\n\
             \x20\x20\x20\x20 AND m.predicate_key = ?2\n\
             \x20\x20\x20\x20 AND r.object_json = ?3\n\
             \x20\x20\x20\x20 AND m.status = 'active'\n\
             \x20\x20LIMIT 1",
            params![candidate.subject_key, candidate.predicate_key, object_json],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| LoreError::Storage(e.to_string()))?;

    if let Some(memory_id) = existing_same {
        tx.execute(
            "UPDATE lore_candidates\n\
             \x20\x20SET status = 'accepted', reviewed_at_ms = ?1\n\
             \x20\x20WHERE lore_candidate_id = ?2",
            params![now, candidate_id],
        )
        .map_err(|e| LoreError::Storage(e.to_string()))?;
        return load_memory(tx, &memory_id);
    }

    // §12.1 transition: Accepted → Superseded. Mark every active memory with
    // the same (subject, predicate) — but a different object — as superseded
    // by the incoming candidate. The supersession is recorded in `status`
    // only; revision metadata is preserved so sync cannot resurrect the old
    // statement as `active`.
    tx.execute(
        "UPDATE lore_memories\n\
         \x20\x20SET status = 'superseded', updated_at_ms = ?1\n\
         \x20\x20WHERE subject_key = ?2\n\
         \x20\x20\x20\x20 AND predicate_key = ?3\n\
         \x20\x20\x20\x20 AND status = 'active'",
        params![now, candidate.subject_key, candidate.predicate_key],
    )
    .map_err(|e| LoreError::Storage(e.to_string()))?;

    // Create the new memory at revision 1.
    let memory_id = sha256_hex(
        format!(
            "memory|{}|{}|{}|{}|{}",
            candidate.subject_key,
            candidate.predicate_key,
            object_json,
            authored_by,
            now
        )
        .as_bytes(),
    );

    tx.execute(
        "INSERT INTO lore_memories\n\
         \x20\x20(memory_id, subject_key, predicate_key, current_revision,\n\
         \x20\x20 pinned, status, created_at_ms, updated_at_ms, deleted_at_ms)\n\
         \x20\x20VALUES (?1, ?2, ?3, 1, 0, 'active', ?4, ?4, NULL)",
        params![memory_id, candidate.subject_key, candidate.predicate_key, now],
    )
    .map_err(|e| LoreError::Storage(e.to_string()))?;

    tx.execute(
        "INSERT INTO lore_memory_revisions\n\
         \x20\x20(memory_id, revision, statement, object_json,\n\
         \x20\x20 correction_reason, authored_by, created_at_ms)\n\
         \x20\x20VALUES (?1, 1, ?2, ?3, ?4, ?5, ?6)",
        params![
            memory_id,
            candidate.statement,
            object_json,
            policy_reason,
            authored_by,
            now,
        ],
    )
    .map_err(|e| LoreError::Storage(e.to_string()))?;

    // Carry the candidate's evidence into the new memory's first revision.
    for ev in &candidate.evidence {
        tx.execute(
            "INSERT OR IGNORE INTO lore_memory_evidence\n\
             \x20\x20(memory_id, memory_revision, lore_evidence_id)\n\
             \x20\x20VALUES (?1, 1, ?2)",
            params![memory_id, ev.lore_evidence_id],
        )
        .map_err(|e| LoreError::Storage(e.to_string()))?;
    }

    tx.execute(
        "UPDATE lore_candidates\n\
         \x20\x20SET status = 'accepted', reviewed_at_ms = ?1\n\
         \x20\x20WHERE lore_candidate_id = ?2",
        params![now, candidate_id],
    )
    .map_err(|e| LoreError::Storage(e.to_string()))?;

    load_memory(tx, &memory_id)
}

#[async_trait]
impl LoreService for SqliteLoreService {
    async fn extract_candidates(
        &self,
        conversation_id: &str,
        inference_run_id: &str,
    ) -> Result<Vec<LoreCandidate>, LoreError> {
        let mut conn = self.conn.lock().map_err(|e| LoreError::Storage(e.to_string()))?;
        let tx = conn
            .transaction()
            .map_err(|e| LoreError::Storage(e.to_string()))?;

        // §12.3: opt-in per conversation. We always extract when called; the
        // "global review-inbox policy" is the user's UI choice to call this
        // method or not — there is no separate flag in the contract.
        let messages = load_user_messages(&tx, conversation_id)?;

        let mut raw: Vec<RawCandidate> = Vec::new();
        for msg in &messages {
            for sentence in policy::split_sentences(&msg.body) {
                if let Some(a) = policy::classify(&sentence) {
                    raw.push(RawCandidate {
                        statement: a.statement,
                        subject_key: "user".to_string(),
                        predicate_key: a.class.predicate_key().to_string(),
                        object: a.object,
                        confidence: a.class.base_confidence(),
                        excerpt: sentence.clone(),
                        message_id: msg.message_id.clone(),
                        created_at_ms: msg.created_at_ms,
                    });
                }
            }
        }

        // §12.4 consolidation. Group by (subject_key, predicate_key, object).
        // Two raw assertions with identical triples merge their evidence;
        // two with contradictory objects stay separate (per §12.4 "may not
        // merge contradictory objects").
        let mut grouped: Vec<GroupedCandidate> = Vec::new();
        for c in raw {
            let locator = serde_json::json!({
                "conversation_id": conversation_id,
                "message_id": c.message_id,
                "inference_run_id": inference_run_id,
            });
            let mut merged = false;
            for g in grouped.iter_mut() {
                if g.subject_key == c.subject_key
                    && g.predicate_key == c.predicate_key
                    && g.object == c.object
                {
                    if c.confidence > g.confidence {
                        g.confidence = c.confidence;
                    }
                    g.evidence.push((c.excerpt.clone(), c.message_id.clone(), c.created_at_ms, locator.clone()));
                    merged = true;
                    break;
                }
            }
            if !merged {
                grouped.push(GroupedCandidate {
                    statement: c.statement,
                    subject_key: c.subject_key,
                    predicate_key: c.predicate_key,
                    object: c.object,
                    confidence: c.confidence,
                    evidence: vec![(c.excerpt.clone(), c.message_id.clone(), c.created_at_ms, locator)],
                });
            }
        }

        // Resolve the inference_run_id FK target. The schema's
        // `proposed_by_run_id REFERENCES inference_runs(inference_run_id)`
        // means we cannot blindly insert a foreign run id; if it is not yet
        // persisted (e.g. extraction is invoked before the run row is
        // committed), store NULL.
        let run_exists = if inference_run_id.is_empty() {
            false
        } else {
            tx.query_row(
                "SELECT 1 FROM inference_runs WHERE inference_run_id = ?1",
                params![inference_run_id],
                |_| Ok(1i64),
            )
            .optional()
            .map_err(|e| LoreError::Storage(e.to_string()))?
            .is_some()
        };
        let run_ref: Option<&str> = if run_exists {
            Some(inference_run_id)
        } else {
            None
        };

        let mut created: Vec<LoreCandidate> = Vec::new();
        let now = now_ms();

        for g in &grouped {
            let object_json =
                serde_json::to_string(&g.object).unwrap_or_else(|_| "null".into());

            // Persist each evidence row (idempotent on PK).
            let mut evidence_ids: Vec<String> = Vec::new();
            for (excerpt, message_id, created_at_ms, locator) in &g.evidence {
                let locator_json =
                    serde_json::to_string(locator).unwrap_or_else(|_| "{}".into());
                let evidence_id = sha256_hex(
                    format!("message|{message_id}|{excerpt}|{locator_json}").as_bytes(),
                );
                let content_digest = sha256_hex(excerpt.as_bytes());
                tx.execute(
                    "INSERT OR IGNORE INTO lore_evidence\n\
                     \x20\x20(lore_evidence_id, source_kind, source_id, excerpt,\n\
                     \x20\x20 locator_json, content_digest, created_at_ms)\n\
                     \x20\x20VALUES (?1, 'message', ?2, ?3, ?4, ?5, ?6)",
                    params![evidence_id, message_id, excerpt, locator_json, content_digest, *created_at_ms],
                )
                .map_err(|e| LoreError::Storage(e.to_string()))?;
                evidence_ids.push(evidence_id);
            }

            // Deterministic candidate id so re-extraction is a no-op for
            // identical inputs.
            let first_excerpt = g
                .evidence
                .first()
                .map(|(e, _, _, _)| e.as_str())
                .unwrap_or("");
            let candidate_id = sha256_hex(
                format!(
                    "{}\n{}\n{}\n{}\n{}\n{}",
                    g.statement,
                    g.subject_key,
                    g.predicate_key,
                    object_json,
                    run_ref.unwrap_or(""),
                    first_excerpt,
                )
                .as_bytes(),
            );

            let inserted = tx
                .execute(
                    "INSERT OR IGNORE INTO lore_candidates\n\
                     \x20\x20(lore_candidate_id, statement, subject_key, predicate_key,\n\
                     \x20\x20 object_json, confidence, status, proposed_by_run_id,\n\
                     \x20\x20 created_at_ms, reviewed_at_ms)\n\
                     \x20\x20VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'inbox', ?7, ?8, NULL)",
                    params![
                        candidate_id,
                        g.statement,
                        g.subject_key,
                        g.predicate_key,
                        object_json,
                        g.confidence,
                        run_ref,
                        g.evidence
                            .first()
                            .map(|(_, _, ts, _)| *ts)
                            .unwrap_or(now),
                    ],
                )
                .map_err(|e| LoreError::Storage(e.to_string()))?;

            for eid in &evidence_ids {
                tx.execute(
                    "INSERT OR IGNORE INTO lore_candidate_evidence\n\
                     \x20\x20(lore_candidate_id, lore_evidence_id)\n\
                     \x20\x20VALUES (?1, ?2)",
                    params![candidate_id, eid],
                )
                .map_err(|e| LoreError::Storage(e.to_string()))?;
            }

            if inserted > 0 {
                let candidate = load_candidate(&tx, &candidate_id)?;
                // §12.3: user-configured deterministic promotion policy. The
                // rule's `reason` is recorded as the resulting revision's
                // `correction_reason` so the Memory Centre surfaces exactly
                // which rule promoted which candidate.
                if let Some(rule) = self.promotion_policy.matches(
                    &candidate.predicate_key,
                    candidate.confidence,
                    candidate.evidence.len() as u32,
                ) {
                    promote_candidate(
                        &tx,
                        &candidate_id,
                        "policy",
                        Some(&rule.reason),
                        now,
                    )?;
                }
                created.push(load_candidate(&tx, &candidate_id)?);
            }
        }

        tx.commit().map_err(|e| LoreError::Storage(e.to_string()))?;
        Ok(created)
    }

    async fn list_inbox(
        &self,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<LoreCandidate>, LoreError> {
        let conn = self.conn.lock().map_err(|e| LoreError::Storage(e.to_string()))?;
        let tx = conn.unchecked_transaction().map_err(|e| LoreError::Storage(e.to_string()))?;
        let limit_clamped = if limit == 0 { 50 } else { limit };
        let mut stmt = tx
            .prepare(
                "SELECT lore_candidate_id\n\
                 \x20\x20FROM lore_candidates\n\
                 \x20\x20WHERE status = 'inbox'\n\
                 \x20\x20ORDER BY created_at_ms DESC, lore_candidate_id ASC\n\
                 \x20\x20LIMIT ?1 OFFSET ?2",
            )
            .map_err(|e| LoreError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map(params![limit_clamped as i64, offset as i64], |r| {
                let id: String = r.get(0)?;
                Ok(id)
            })
            .map_err(|e| LoreError::Storage(e.to_string()))?;
        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(|e| LoreError::Storage(e.to_string()))?);
        }
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            out.push(load_candidate(&tx, &id)?);
        }
        Ok(out)
    }

    async fn accept(&self, candidate_id: &str) -> Result<LoreMemory, LoreError> {
        let mut conn = self.conn.lock().map_err(|e| LoreError::Storage(e.to_string()))?;
        let tx = conn.transaction().map_err(|e| LoreError::Storage(e.to_string()))?;
        // Verify the candidate exists and is in an actionable state.
        let status: Option<String> = tx
            .query_row(
                "SELECT status FROM lore_candidates WHERE lore_candidate_id = ?1",
                params![candidate_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| LoreError::Storage(e.to_string()))?;
        match status {
            None => return Err(LoreError::CandidateNotFound),
            Some(s) if s == "accepted" => {
                // Already promoted: return the linked memory if any active
                // memory for this (subject, predicate) exists.
                let candidate = load_candidate(&tx, candidate_id)?;
                let object_json = serde_json::to_string(&candidate.object)
                    .unwrap_or_else(|_| "null".into());
                let mid: Option<String> = tx
                    .query_row(
                        "SELECT m.memory_id\n\
                         \x20\x20FROM lore_memories m\n\
                         \x20\x20JOIN lore_memory_revisions r\n\
                         \x20\x20\x20\x20ON r.memory_id = m.memory_id\n\
                         \x20\x20\x20\x20   AND r.revision = m.current_revision\n\
                         \x20\x20WHERE m.subject_key = ?1\n\
                         \x20\x20\x20\x20 AND m.predicate_key = ?2\n\
                         \x20\x20\x20\x20 AND r.object_json = ?3\n\
                         \x20\x20\x20\x20 AND m.status = 'active'\n\
                         \x20\x20LIMIT 1",
                        params![candidate.subject_key, candidate.predicate_key, object_json],
                        |r| r.get(0),
                    )
                    .optional()
                    .map_err(|e| LoreError::Storage(e.to_string()))?;
                if let Some(mid) = mid {
                    return load_memory(&tx, &mid);
                }
                // Fall through: re-promote (rare; legacy row state).
            }
            Some(s) if s == "expired" => return Err(LoreError::CandidateNotFound),
            _ => {}
        }
        let memory = promote_candidate(&tx, candidate_id, "user", None, now_ms())?;
        tx.commit().map_err(|e| LoreError::Storage(e.to_string()))?;
        Ok(memory)
    }

    async fn reject(&self, candidate_id: &str) -> Result<(), LoreError> {
        let mut conn = self.conn.lock().map_err(|e| LoreError::Storage(e.to_string()))?;
        let tx = conn.transaction().map_err(|e| LoreError::Storage(e.to_string()))?;
        let affected = tx
            .execute(
                "UPDATE lore_candidates\n\
                 \x20\x20SET status = 'rejected', reviewed_at_ms = ?1\n\
                 \x20\x20WHERE lore_candidate_id = ?2",
                params![now_ms(), candidate_id],
            )
            .map_err(|e| LoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(LoreError::CandidateNotFound);
        }
        tx.commit().map_err(|e| LoreError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn correct(
        &self,
        memory_id: &str,
        expected_revision: u32,
        correction: LoreCorrection,
    ) -> Result<LoreMemory, LoreError> {
        let mut conn = self.conn.lock().map_err(|e| LoreError::Storage(e.to_string()))?;
        let tx = conn.transaction().map_err(|e| LoreError::Storage(e.to_string()))?;

        // Load and validate the memory's current state.
        let row = tx
            .query_row(
                "SELECT current_revision, status\n\
                 \x20\x20FROM lore_memories\n\
                 \x20\x20WHERE memory_id = ?1",
                params![memory_id],
                |r| Ok((r.get::<_, i64>(0)? as u32, r.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|e| LoreError::Storage(e.to_string()))?;
        let (current_revision, status) = row.ok_or(LoreError::MemoryNotFound)?;

        if status != "active" {
            return Err(LoreError::PolicyRejected(format!(
                "memory is not active (status = {status})"
            )));
        }
        if current_revision != expected_revision {
            return Err(LoreError::RevisionConflict);
        }

        // §A-08: every memory needs cited evidence. Validate the supplied
        // evidence ids against `lore_evidence` before mutating anything.
        for eid in &correction.evidence_ids {
            let exists: Option<i64> = tx
                .query_row(
                    "SELECT 1 FROM lore_evidence WHERE lore_evidence_id = ?1",
                    params![eid],
                    |_| Ok(1),
                )
                .optional()
                .map_err(|e| LoreError::Storage(e.to_string()))?;
            if exists.is_none() {
                return Err(LoreError::EvidenceMissing);
            }
        }

        let new_revision = expected_revision
            .checked_add(1)
            .ok_or_else(|| LoreError::Storage("revision overflow".into()))?;
        let object_json = serde_json::to_string(&correction.object)
            .unwrap_or_else(|_| "null".into());
        let now = now_ms();

        tx.execute(
            "INSERT INTO lore_memory_revisions\n\
             \x20\x20(memory_id, revision, statement, object_json,\n\
             \x20\x20 correction_reason, authored_by, created_at_ms)\n\
             \x20\x20VALUES (?1, ?2, ?3, ?4, ?5, 'user', ?6)",
            params![
                memory_id,
                new_revision as i64,
                correction.statement,
                object_json,
                correction.correction_reason,
                now,
            ],
        )
        .map_err(|e| LoreError::Storage(e.to_string()))?;

        // The trg_lore_revision_insert trigger has already bumped
        // `current_revision` and `updated_at_ms` on `lore_memories`. The
        // UPDATE below is defensive — it is a no-op when the trigger fired.
        tx.execute(
            "UPDATE lore_memories\n\
             \x20\x20SET current_revision = ?1, updated_at_ms = ?2\n\
             \x20\x20WHERE memory_id = ?3 AND current_revision < ?1",
            params![new_revision as i64, now, memory_id],
        )
        .map_err(|e| LoreError::Storage(e.to_string()))?;

        // Link the cited evidence to the new revision. If the caller did not
        // supply any evidence ids, carry forward the previous revision's
        // evidence so the corrected statement stays cited (§A-08).
        if correction.evidence_ids.is_empty() {
            tx.execute(
                "INSERT OR IGNORE INTO lore_memory_evidence\n\
                 \x20\x20(memory_id, memory_revision, lore_evidence_id)\n\
                 \x20\x20SELECT ?1, ?2, lore_evidence_id\n\
                 \x20\x20FROM lore_memory_evidence\n\
                 \x20\x20WHERE memory_id = ?1 AND memory_revision = ?3",
                params![memory_id, new_revision as i64, expected_revision as i64],
            )
            .map_err(|e| LoreError::Storage(e.to_string()))?;
        } else {
            for eid in &correction.evidence_ids {
                tx.execute(
                    "INSERT OR IGNORE INTO lore_memory_evidence\n\
                     \x20\x20(memory_id, memory_revision, lore_evidence_id)\n\
                     \x20\x20VALUES (?1, ?2, ?3)",
                    params![memory_id, new_revision as i64, eid],
                )
                .map_err(|e| LoreError::Storage(e.to_string()))?;
            }
        }

        let memory = load_memory(&tx, memory_id)?;
        tx.commit().map_err(|e| LoreError::Storage(e.to_string()))?;
        Ok(memory)
    }

    async fn set_pinned(
        &self,
        memory_id: &str,
        pinned: bool,
    ) -> Result<LoreMemory, LoreError> {
        let mut conn = self.conn.lock().map_err(|e| LoreError::Storage(e.to_string()))?;
        let tx = conn.transaction().map_err(|e| LoreError::Storage(e.to_string()))?;
        let affected = tx
            .execute(
                "UPDATE lore_memories\n\
                 \x20\x20SET pinned = ?1, updated_at_ms = ?2\n\
                 \x20\x20WHERE memory_id = ?3",
                params![if pinned { 1i64 } else { 0 }, now_ms(), memory_id],
            )
            .map_err(|e| LoreError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(LoreError::MemoryNotFound);
        }
        let memory = load_memory(&tx, memory_id)?;
        tx.commit().map_err(|e| LoreError::Storage(e.to_string()))?;
        Ok(memory)
    }

    async fn delete(
        &self,
        memory_id: &str,
        expected_revision: u32,
    ) -> Result<(), LoreError> {
        let mut conn = self.conn.lock().map_err(|e| LoreError::Storage(e.to_string()))?;
        let tx = conn.transaction().map_err(|e| LoreError::Storage(e.to_string()))?;

        let row = tx
            .query_row(
                "SELECT current_revision, status\n\
                 \x20\x20FROM lore_memories\n\
                 \x20\x20WHERE memory_id = ?1",
                params![memory_id],
                |r| Ok((r.get::<_, i64>(0)? as u32, r.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|e| LoreError::Storage(e.to_string()))?;
        let (current_revision, _status) = row.ok_or(LoreError::MemoryNotFound)?;

        if current_revision != expected_revision {
            return Err(LoreError::RevisionConflict);
        }

        let now = now_ms();

        // §12.5 steps 1, 2, 6: mark `status = 'deleted'`, set `deleted_at_ms`,
        // and KEEP all revision metadata so sync cannot resurrect the memory
        // as `active` later.
        tx.execute(
            "UPDATE lore_memories\n\
             \x20\x20SET status = 'deleted', deleted_at_ms = ?1, updated_at_ms = ?1\n\
             \x20\x20WHERE memory_id = ?2",
            params![now, memory_id],
        )
        .map_err(|e| LoreError::Storage(e.to_string()))?;

        // §12.5 steps 3, 4, 5: remove the memory from the retrieval projection.
        // Deleting the retrieval_documents row cascades (via the schema's FK
        // `ON DELETE CASCADE`) to retrieval_chunks, which in turn cascades to
        // chunk_embeddings; the `trg_retrieval_chunk_fts_delete` trigger
        // removes the text from retrieval_chunks_fts. Together this makes
        // the deleted statement unreachable by retrieval, by lexical search
        // and by vector search, and therefore excluded from inference.
        tx.execute(
            "DELETE FROM retrieval_documents\n\
             \x20\x20WHERE source_kind = 'lore' AND source_id = ?1",
            params![memory_id],
        )
        .map_err(|e| LoreError::Storage(e.to_string()))?;

        tx.commit().map_err(|e| LoreError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn query(&self, query: LoreQuery) -> Result<Vec<LoreMemory>, LoreError> {
        let conn = self.conn.lock().map_err(|e| LoreError::Storage(e.to_string()))?;
        let tx = conn.unchecked_transaction().map_err(|e| LoreError::Storage(e.to_string()))?;

        // §12.5 step 7: exclude deleted text from ordinary export/diagnostics.
        // Only explicit `include_deleted: true` returns tombstones — and
        // tombstones never carry the statement text into the result.
        let mut sql = String::from(
            "SELECT m.memory_id\n\
             \x20\x20FROM lore_memories m\n\
             \x20\x20JOIN lore_memory_revisions r\n\
             \x20\x20\x20\x20ON r.memory_id = m.memory_id AND r.revision = m.current_revision\n\
             \x20\x20WHERE 1 = 1\n",
        );
        if !query.include_deleted {
            sql.push_str("  AND m.status = 'active'\n");
        }
        if !query.subject_keys.is_empty() {
            sql.push_str("  AND m.subject_key IN (");
            for (i, _) in query.subject_keys.iter().enumerate() {
                if i > 0 {
                    sql.push(',');
                }
                sql.push_str(&format!("?{}", i + 1));
            }
            sql.push_str(")\n");
        }
        let mut param_idx = query.subject_keys.len();
        if !query.text.trim().is_empty() {
            param_idx += 1;
            sql.push_str(&format!(
                "  AND (LOWER(r.statement) LIKE LOWER(?{})\n\
                 \x20\x20\x20\x20\x20\x20\x20OR LOWER(r.object_json) LIKE LOWER(?{}))\n",
                param_idx, param_idx
            ));
        }
        sql.push_str("  ORDER BY m.pinned DESC, m.updated_at_ms DESC, m.memory_id ASC\n");
        let limit_clamped = if query.limit == 0 { 50 } else { query.limit };
        param_idx += 1;
        sql.push_str(&format!("  LIMIT ?{}\n", param_idx));

        let mut stmt = tx.prepare(&sql).map_err(|e| LoreError::Storage(e.to_string()))?;
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        for sk in &query.subject_keys {
            params_vec.push(Box::new(sk.clone()));
        }
        if !query.text.trim().is_empty() {
            let like = format!("%{}%", query.text.trim());
            params_vec.push(Box::new(like));
        }
        params_vec.push(Box::new(limit_clamped as i64));
        let param_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();
        let rows = stmt
            .query_map(param_refs.as_slice(), |r| {
                let id: String = r.get(0)?;
                Ok(id)
            })
            .map_err(|e| LoreError::Storage(e.to_string()))?;

        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(|e| LoreError::Storage(e.to_string()))?);
        }
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            out.push(load_memory(&tx, &id)?);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use storage_sqlite::apply_pragmas;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        apply_pragmas(&conn).expect("apply pragmas");
        // Apply the verbatim §6.2 schema directly. (We bypass the runner here
        // because its `ensure_schema_migrations_table` step pre-creates the
        // `schema_migrations` table, which then conflicts with the schema's
        // own `CREATE TABLE schema_migrations` — a storage-sqlite
        // bookkeeping quirk that is irrelevant to lore's behaviour.)
        conn.execute_batch(storage_sqlite::INITIAL_SQL)
            .expect("apply schema");
        conn
    }

    fn seed_conversation(conn: &Connection, conversation_id: &str, user_messages: &[&str]) {
        let now = now_ms();
        conn.execute(
            "INSERT INTO conversations\n\
             \x20\x20(conversation_id, title, scope_json, backend_id,\n\
             \x20\x20 created_at_ms, updated_at_ms, archived_at_ms)\n\
             \x20\x20VALUES (?1, ?2, '{}', NULL, ?3, ?3, NULL)",
            params![conversation_id, format!("Conversation {conversation_id}"), now],
        )
        .expect("insert conversation");
        for (seq, body) in user_messages.iter().enumerate() {
            let mid = sha256_hex(format!("{conversation_id}|{seq}|{body}").as_bytes());
            conn.execute(
                "INSERT INTO messages\n\
                 \x20\x20(message_id, conversation_id, role, body_markdown,\n\
                 \x20\x20 sequence_no, inference_run_id, created_at_ms)\n\
                 \x20\x20VALUES (?1, ?2, 'user', ?3, ?4, NULL, ?5)",
                params![mid, conversation_id, body, seq as i64, now + seq as i64],
            )
            .expect("insert message");
        }
    }

    fn insert_retrieval_doc_for_memory(conn: &Connection, memory_id: &str, text: &str) {
        let doc_id = sha256_hex(format!("doc|lore|{memory_id}").as_bytes());
        conn.execute(
            "INSERT INTO retrieval_documents\n\
             \x20\x20(retrieval_document_id, source_kind, source_id, pack_id,\n\
             \x20\x20 article_key, title, content_digest, producer_version,\n\
             \x20\x20 updated_at_ms)\n\
             \x20\x20VALUES (?1, 'lore', ?2, NULL, NULL, ?3, ?4, 'test', ?5)",
            params![
                doc_id,
                memory_id,
                format!("Memory {memory_id}"),
                sha256_hex(text.as_bytes()),
                now_ms(),
            ],
        )
        .expect("insert retrieval document");
        let chunk_id = sha256_hex(format!("chunk|{doc_id}").as_bytes());
        conn.execute(
            "INSERT INTO retrieval_chunks\n\
             \x20\x20(retrieval_chunk_id, retrieval_document_id, ordinal,\n\
             \x20\x20 start_char, end_char, text_content, text_digest,\n\
             \x20\x20 locator_json, token_count)\n\
             \x20\x20VALUES (?1, ?2, 0, 0, ?3, ?4, ?5, '{}', 1)",
            params![
                chunk_id,
                doc_id,
                text.len() as i64,
                text,
                sha256_hex(text.as_bytes()),
            ],
        )
        .expect("insert retrieval chunk");
    }

    fn count_retrieval_for_memory(conn: &Connection, memory_id: &str) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM retrieval_documents\n\
             \x20\x20WHERE source_kind = 'lore' AND source_id = ?1",
            params![memory_id],
            |r| r.get(0),
        )
        .unwrap_or(0)
    }

    #[tokio::test]
    async fn extracts_stable_preference() {
        let conn = setup_db();
        seed_conversation(&conn, "c1", &["I prefer Vim editor. Hello world."]);
        let svc = SqliteLoreService::without_promotion(conn);
        let cands = svc.extract_candidates("c1", "run-1").await.unwrap();
        let pref = cands
            .iter()
            .find(|c| c.predicate_key == "preference")
            .expect("preference candidate present");
        assert_eq!(pref.subject_key, "user");
        assert_eq!(pref.object["value"], "Vim editor");
        assert!(!pref.evidence.is_empty());
        assert_eq!(pref.evidence[0].source_kind, "message");
    }

    #[tokio::test]
    async fn extracts_no_forbidden_medical() {
        let conn = setup_db();
        seed_conversation(&conn, "c1", &["I am depressed today."]);
        let svc = SqliteLoreService::without_promotion(conn);
        let cands = svc.extract_candidates("c1", "run-1").await.unwrap();
        assert!(cands.is_empty(), "no candidates should be extracted");
    }

    #[tokio::test]
    async fn extracts_no_secret() {
        let conn = setup_db();
        seed_conversation(&conn, "c1", &["My password is hunter2."]);
        let svc = SqliteLoreService::without_promotion(conn);
        let cands = svc.extract_candidates("c1", "run-1").await.unwrap();
        assert!(cands.is_empty());
    }

    #[tokio::test]
    async fn consolidates_duplicate_evidence() {
        let conn = setup_db();
        seed_conversation(
            &conn,
            "c1",
            &["I prefer Vim editor.", "I prefer Vim editor."],
        );
        let svc = SqliteLoreService::without_promotion(conn);
        let cands = svc.extract_candidates("c1", "run-1").await.unwrap();
        // Both messages collapse to one candidate carrying two evidence rows
        // (identical subject_key + predicate_key + object_value).
        assert_eq!(cands.len(), 1, "expected one consolidated candidate");
        let c = &cands[0];
        assert!(
            c.evidence.len() >= 2,
            "expected merged evidence (got {})",
            c.evidence.len()
        );
    }

    #[tokio::test]
    async fn does_not_consolidate_contradictory_objects() {
        let conn = setup_db();
        seed_conversation(
            &conn,
            "c1",
            &["I prefer Vim editor.", "I prefer Emacs editor."],
        );
        let svc = SqliteLoreService::without_promotion(conn);
        let cands = svc.extract_candidates("c1", "run-1").await.unwrap();
        assert_eq!(
            cands.len(),
            2,
            "contradictory objects must remain separate candidates"
        );
    }

    #[tokio::test]
    async fn extract_is_idempotent() {
        let conn = setup_db();
        seed_conversation(&conn, "c1", &["I prefer Vim editor."]);
        let svc = SqliteLoreService::without_promotion(conn);
        let first = svc.extract_candidates("c1", "run-1").await.unwrap();
        let second = svc.extract_candidates("c1", "run-1").await.unwrap();
        assert_eq!(first.len(), 1);
        assert!(second.is_empty(), "second extraction must be a no-op");
    }

    #[tokio::test]
    async fn accept_promotes_candidate() {
        let conn = setup_db();
        seed_conversation(&conn, "c1", &["I prefer Vim editor."]);
        let svc = SqliteLoreService::without_promotion(conn);
        let cands = svc.extract_candidates("c1", "run-1").await.unwrap();
        let cid = cands[0].lore_candidate_id.clone();
        let mem = svc.accept(&cid).await.unwrap();
        assert_eq!(mem.status, LoreMemoryStatus::Active);
        assert_eq!(mem.revision, 1);
        assert_eq!(mem.subject_key, "user");
        assert_eq!(mem.predicate_key, "preference");
        assert!(!mem.evidence.is_empty());
    }

    #[tokio::test]
    async fn accept_supersedes_prior_active_memory() {
        let conn = setup_db();
        seed_conversation(
            &conn,
            "c1",
            &["I prefer Vim editor.", "I prefer Emacs editor."],
        );
        let svc = SqliteLoreService::without_promotion(conn);
        let cands = svc.extract_candidates("c1", "run-1").await.unwrap();
        let vim = cands
            .iter()
            .find(|c| c.object["value"] == "Vim editor")
            .unwrap();
        let emacs = cands
            .iter()
            .find(|c| c.object["value"] == "Emacs editor")
            .unwrap();
        let m1 = svc.accept(&vim.lore_candidate_id).await.unwrap();
        let m2 = svc.accept(&emacs.lore_candidate_id).await.unwrap();
        assert_eq!(m2.status, LoreMemoryStatus::Active);
        // Different memory ids confirm supersession rather than duplicate-drop.
        assert_ne!(m1.memory_id, m2.memory_id);
        // Reloading m1 reflects the supersession that happened during m2's
        // accept: Vim's memory should now be Superseded by the Emacs memory.
        let m1_after = svc
            .query(LoreQuery {
                text: String::new(),
                subject_keys: vec![],
                include_deleted: true,
                limit: 10,
            })
            .await
            .unwrap()
            .into_iter()
            .find(|m| m.memory_id == m1.memory_id)
            .expect("vim memory still present as tombstoned history");
        assert_eq!(m1_after.status, LoreMemoryStatus::Superseded);
    }

    #[tokio::test]
    async fn reject_marks_candidate() {
        let conn = setup_db();
        seed_conversation(&conn, "c1", &["I prefer Vim editor."]);
        let svc = SqliteLoreService::without_promotion(conn);
        let cands = svc.extract_candidates("c1", "run-1").await.unwrap();
        svc.reject(&cands[0].lore_candidate_id).await.unwrap();
        let inbox = svc.list_inbox(0, 10).await.unwrap();
        assert!(inbox.is_empty());
    }

    #[tokio::test]
    async fn reject_unknown_candidate_errors() {
        let conn = setup_db();
        let svc = SqliteLoreService::without_promotion(conn);
        let err = svc.reject("does-not-exist").await.unwrap_err();
        assert!(matches!(err, LoreError::CandidateNotFound));
    }

    #[tokio::test]
    async fn correct_updates_memory() {
        let conn = setup_db();
        seed_conversation(&conn, "c1", &["I prefer Vim editor."]);
        let svc = SqliteLoreService::without_promotion(conn);
        let cands = svc.extract_candidates("c1", "run-1").await.unwrap();
        let cid = cands[0].lore_candidate_id.clone();
        let m1 = svc.accept(&cid).await.unwrap();
        let evidence_ids: Vec<String> =
            m1.evidence.iter().map(|e| e.lore_evidence_id.clone()).collect();
        let m2 = svc
            .correct(
                &m1.memory_id,
                1,
                LoreCorrection {
                    statement: "User now prefers Emacs.".into(),
                    object: serde_json::json!({"value": "Emacs"}),
                    correction_reason: "user changed mind".into(),
                    evidence_ids,
                },
            )
            .await
            .unwrap();
        assert_eq!(m2.revision, 2);
        assert_eq!(m2.statement, "User now prefers Emacs.");
        assert_eq!(m2.object["value"], "Emacs");
    }

    #[tokio::test]
    async fn correct_revision_conflict() {
        let conn = setup_db();
        seed_conversation(&conn, "c1", &["I prefer Vim editor."]);
        let svc = SqliteLoreService::without_promotion(conn);
        let cands = svc.extract_candidates("c1", "run-1").await.unwrap();
        let cid = cands[0].lore_candidate_id.clone();
        let m1 = svc.accept(&cid).await.unwrap();
        // First a successful correction, then a stale revision=1 attempt.
        let _ = svc
            .correct(
                &m1.memory_id,
                1,
                LoreCorrection {
                    statement: "v2".into(),
                    object: serde_json::json!({"value": "v2"}),
                    correction_reason: "first".into(),
                    evidence_ids: vec![],
                },
            )
            .await
            .unwrap();
        let err = svc
            .correct(
                &m1.memory_id,
                1,
                LoreCorrection {
                    statement: "stale".into(),
                    object: serde_json::json!({"value": "stale"}),
                    correction_reason: "stale".into(),
                    evidence_ids: vec![],
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, LoreError::RevisionConflict));
    }

    #[tokio::test]
    async fn correct_rejects_missing_evidence() {
        let conn = setup_db();
        seed_conversation(&conn, "c1", &["I prefer Vim editor."]);
        let svc = SqliteLoreService::without_promotion(conn);
        let cands = svc.extract_candidates("c1", "run-1").await.unwrap();
        let cid = cands[0].lore_candidate_id.clone();
        let m1 = svc.accept(&cid).await.unwrap();
        let err = svc
            .correct(
                &m1.memory_id,
                1,
                LoreCorrection {
                    statement: "x".into(),
                    object: serde_json::json!({"value": "x"}),
                    correction_reason: "x".into(),
                    evidence_ids: vec!["nonexistent-evidence-id".into()],
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, LoreError::EvidenceMissing));
    }

    #[tokio::test]
    async fn set_pinned_toggles_flag() {
        let conn = setup_db();
        seed_conversation(&conn, "c1", &["I prefer Vim editor."]);
        let svc = SqliteLoreService::without_promotion(conn);
        let cands = svc.extract_candidates("c1", "run-1").await.unwrap();
        let m1 = svc.accept(&cands[0].lore_candidate_id).await.unwrap();
        assert!(!m1.pinned);
        let m2 = svc.set_pinned(&m1.memory_id, true).await.unwrap();
        assert!(m2.pinned);
        let m3 = svc.set_pinned(&m1.memory_id, false).await.unwrap();
        assert!(!m3.pinned);
    }

    #[tokio::test]
    async fn set_pinned_unknown_memory_errors() {
        let conn = setup_db();
        let svc = SqliteLoreService::without_promotion(conn);
        let err = svc.set_pinned("ghost", true).await.unwrap_err();
        assert!(matches!(err, LoreError::MemoryNotFound));
    }

    #[tokio::test]
    async fn delete_removes_retrieval_projection() {
        let conn = setup_db();
        seed_conversation(&conn, "c1", &["I prefer Vim editor."]);
        let svc = SqliteLoreService::without_promotion(conn);
        let cands = svc.extract_candidates("c1", "run-1").await.unwrap();
        let m1 = svc.accept(&cands[0].lore_candidate_id).await.unwrap();
        // Insert the retrieval projection through the service's own connection
        // (the connection has been moved into `svc`).
        {
            let conn = svc.conn.lock().unwrap();
            insert_retrieval_doc_for_memory(&conn, &m1.memory_id, "User prefers Vim editor.");
            assert_eq!(count_retrieval_for_memory(&conn, &m1.memory_id), 1);
        }
        svc.delete(&m1.memory_id, m1.revision).await.unwrap();
        // Post-condition: projection gone, memory tombstoned.
        {
            let conn = svc.conn.lock().unwrap();
            assert_eq!(count_retrieval_for_memory(&conn, &m1.memory_id), 0);
            let status: String = conn
                .query_row(
                    "SELECT status FROM lore_memories WHERE memory_id = ?1",
                    params![m1.memory_id],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(status, "deleted");
            let deleted_at: i64 = conn
                .query_row(
                    "SELECT deleted_at_ms FROM lore_memories WHERE memory_id = ?1",
                    params![m1.memory_id],
                    |r| r.get(0),
                )
                .unwrap();
            assert!(deleted_at > 0);
        }
    }

    #[tokio::test]
    async fn delete_revision_conflict() {
        let conn = setup_db();
        seed_conversation(&conn, "c1", &["I prefer Vim editor."]);
        let svc = SqliteLoreService::without_promotion(conn);
        let cands = svc.extract_candidates("c1", "run-1").await.unwrap();
        let m1 = svc.accept(&cands[0].lore_candidate_id).await.unwrap();
        let err = svc.delete(&m1.memory_id, 999).await.unwrap_err();
        assert!(matches!(err, LoreError::RevisionConflict));
    }

    #[tokio::test]
    async fn query_excludes_deleted_by_default() {
        let conn = setup_db();
        seed_conversation(&conn, "c1", &["I prefer Vim editor."]);
        let svc = SqliteLoreService::without_promotion(conn);
        let cands = svc.extract_candidates("c1", "run-1").await.unwrap();
        let m1 = svc.accept(&cands[0].lore_candidate_id).await.unwrap();
        svc.delete(&m1.memory_id, m1.revision).await.unwrap();
        let result = svc
            .query(LoreQuery {
                text: String::new(),
                subject_keys: vec![],
                include_deleted: false,
                limit: 10,
            })
            .await
            .unwrap();
        assert!(result.is_empty());
        // Tombstone is reachable only with include_deleted = true.
        let result = svc
            .query(LoreQuery {
                text: String::new(),
                subject_keys: vec![],
                include_deleted: true,
                limit: 10,
            })
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].status, LoreMemoryStatus::Deleted);
    }

    #[tokio::test]
    async fn query_filters_by_subject_and_text() {
        let conn = setup_db();
        seed_conversation(&conn, "c1", &["I prefer Vim editor."]);
        seed_conversation(&conn, "c2", &["I live in Tokyo."]);
        let svc = SqliteLoreService::without_promotion(conn);
        let c1 = svc.extract_candidates("c1", "run-1").await.unwrap();
        let c2 = svc.extract_candidates("c2", "run-2").await.unwrap();
        let _ = svc.accept(&c1[0].lore_candidate_id).await.unwrap();
        let _ = svc.accept(&c2[0].lore_candidate_id).await.unwrap();
        // Subject filter (always "user" in our extractor).
        let by_subject = svc
            .query(LoreQuery {
                text: String::new(),
                subject_keys: vec!["user".into()],
                include_deleted: false,
                limit: 10,
            })
            .await
            .unwrap();
        assert_eq!(by_subject.len(), 2);
        // Text filter.
        let by_text = svc
            .query(LoreQuery {
                text: "Vim".into(),
                subject_keys: vec![],
                include_deleted: false,
                limit: 10,
            })
            .await
            .unwrap();
        assert_eq!(by_text.len(), 1);
        assert!(by_text[0].statement.contains("Vim"));
    }

    #[tokio::test]
    async fn list_inbox_paginates() {
        let conn = setup_db();
        seed_conversation(
            &conn,
            "c1",
            &[
                "I prefer Vim editor.",
                "I prefer Emacs editor.",
                "I prefer Nano editor.",
            ],
        );
        let svc = SqliteLoreService::without_promotion(conn);
        let _ = svc.extract_candidates("c1", "run-1").await.unwrap();
        let page1 = svc.list_inbox(0, 2).await.unwrap();
        let page2 = svc.list_inbox(2, 2).await.unwrap();
        assert_eq!(page1.len(), 2);
        assert_eq!(page2.len(), 1);
    }

    #[tokio::test]
    async fn promotion_policy_auto_accepts() {
        let conn = setup_db();
        seed_conversation(&conn, "c1", &["I prefer Vim editor."]);
        let policy = PromotionPolicy {
            rules: vec![crate::policy::PromotionRule {
                predicate_key: "preference".into(),
                min_confidence: 0.5,
                min_evidence_count: 1,
                reason: "user enabled auto-promote preferences".into(),
            }],
        };
        let svc = SqliteLoreService::new(conn, policy);
        let cands = svc.extract_candidates("c1", "run-1").await.unwrap();
        // After extraction with a matching policy, the candidate is no longer
        // in the inbox — it was auto-accepted.
        assert_eq!(cands.len(), 1);
        let inbox = svc.list_inbox(0, 10).await.unwrap();
        assert!(inbox.is_empty(), "auto-promoted candidate should not linger in inbox");
        let memories = svc
            .query(LoreQuery {
                text: String::new(),
                subject_keys: vec![],
                include_deleted: false,
                limit: 10,
            })
            .await
            .unwrap();
        assert_eq!(memories.len(), 1);
        // The promotion's `reason` must be visible in the revision's
        // correction_reason so the Memory Centre can display which rule
        // promoted the candidate.
        let conn = svc.conn.lock().unwrap();
        let reason: Option<String> = conn
            .query_row(
                "SELECT correction_reason\n\
                 \x20\x20FROM lore_memory_revisions\n\
                 \x20\x20WHERE memory_id = ?1 AND revision = 1",
                params![memories[0].memory_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            reason.as_deref(),
            Some("user enabled auto-promote preferences")
        );
    }

    #[tokio::test]
    async fn missing_conversation_yields_no_candidates() {
        let conn = setup_db();
        let svc = SqliteLoreService::without_promotion(conn);
        let cands = svc.extract_candidates("no-such-conv", "run-1").await.unwrap();
        assert!(cands.is_empty());
    }
}
