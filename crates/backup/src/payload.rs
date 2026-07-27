//! The snapshot of user data a backup serialises and a restore replays.
//!
//! Per §A-06 ("user work outlives sources and engines") and §13.5 ("never
//! destroy or hide existing user data") the backup payload is a pure,
//! self-contained serialisation of primary user data — annotations,
//! notebooks, conversations and accepted lore. Indexes, embeddings, chunk
//! projections and caches are rebuildable (§Data posture) and are not
//! included.
//!
//! None of the entity types below is invented: they are local, serialisable
//! projections of types whose canonical homes live in `domain::annotation`,
//! `domain::lore`, and the (not-yet-landed) conversation / notebook domain
//! modules. The local-projection fields are exactly the fields the §14.3
//! `BackupRestorePlan` reports counts for: `packBindings`, `annotations`,
//! `conversations`, `loreMemories`. They are marked
//! `// I-10(b): swap when <crate> lands` and the field set matches what
//! those modules will export.

use serde::{Deserialize, Serialize};

use domain::identity::{ArticleKey, ContentDigest, EntryKey, PackId, PackVersionId};

// The import above is the only types this module needs from `domain`. The
// payload is a local, self-contained projection — see the per-type
// I-10(b) markers for which `domain` type each struct will become.

/// Selects which slices of primary user data are included in a backup.
/// Matches the §14.3 `BackupCreateRequest` booleans 1:1. `includePacks`
/// (pack bindings, i.e. user-named curated packs) is always included —
/// §A-06 says user work outlives engines.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackupFilter {
    pub include_packs: bool,
    pub include_annotations: bool,
    pub include_conversations: bool,
    pub include_lore: bool,
}

impl BackupFilter {
    pub fn from_request(include_conversations: bool, include_lore: bool) -> Self {
        Self {
            include_packs: true,
            include_annotations: true,
            include_conversations,
            include_lore,
        }
    }
}

/// A user pack binding — i.e. "this user has named / curated this pack and
/// pinned it to this storage root". The fields are exactly those required
/// to recreate the user-visible binding after a restore (pack identity +
/// display name). Source / engine state (download URLs, etc.) is rebuildable
/// and not stored.
///
/// // I-10(b): swap for the real `domain::pack::UserPackBinding` when it
/// lands. Field names are stable projections of the §6.2 `packs` table.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PackBinding {
    pub pack_id: PackId,
    pub active_version_id: Option<PackVersionId>,
    pub display_name: String,
}

/// One annotation, inlined. The §6.2 `annotations` row's user-visible
/// fields; everything else (revision graph, etc.) lives in `revisions`.
///
/// // I-10(b): swap for `domain::annotation::AnnotationRecord` when its
/// serialisable shape lands. Field names are the same as the §6.2 columns.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AnnotationRecord {
    pub article_key: ArticleKey,
    pub entry_key: EntryKey,
    pub start_char: u32,
    pub end_char: u32,
    pub body_text: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

/// One accepted lore memory.
///
/// // I-10(b): swap for `domain::lore::LoreMemory` when it lands.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LoreMemory {
    pub memory_id: String,
    pub summary: String,
    pub source_digest: ContentDigest,
    pub accepted_at_ms: i64,
}

/// One user-authored conversation message. Conversation turn structure is
/// not in `ARCHITECTURE.md` §6.2 verbatim; this is the minimal shape required
/// for the `BackupRestorePlan.conversations` count to be honest and for the
/// round-trip to be lossless.
///
/// // I-10(b): swap for `domain::conversation::ConversationMessage` when it
/// lands.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub message_id: String,
    pub role: String,
    pub body_text: String,
    pub created_at_ms: i64,
}

/// The full, self-contained user-data snapshot a backup serialises.
///
/// Order is stable so the manifest SHA-256 is deterministic for the same
/// inputs. Each `Vec` is sorted by its primary key before serialisation.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct UserDataSnapshot {
    pub pack_bindings: Vec<PackBinding>,
    pub annotations: Vec<AnnotationRecord>,
    pub conversations: Vec<ConversationMessage>,
    pub lore_memories: Vec<LoreMemory>,
}

impl UserDataSnapshot {
    /// Sort every collection by its primary key so two backups of the same
    /// data produce byte-identical payloads (and therefore the same SHA-256).
    pub fn sort_for_serialization(&mut self) {
        self.pack_bindings
            .sort_by(|a, b| a.pack_id.as_str().cmp(b.pack_id.as_str()));
        self.annotations.sort_by(|a, b| {
            a.article_key
                .as_str()
                .cmp(b.article_key.as_str())
                .then_with(|| a.entry_key.as_str().cmp(b.entry_key.as_str()))
                .then(a.start_char.cmp(&b.start_char))
                .then(a.end_char.cmp(&b.end_char))
        });
        self.conversations.sort_by(|a, b| {
            a.created_at_ms
                .cmp(&b.created_at_ms)
                .then_with(|| a.message_id.cmp(&b.message_id))
        });
        self.lore_memories.sort_by(|a, b| {
            a.accepted_at_ms
                .cmp(&b.accepted_at_ms)
                .then_with(|| a.memory_id.cmp(&b.memory_id))
        });
    }

    /// Apply the filter: drop conversations / lore if the user opted out.
    pub fn apply_filter(&mut self, filter: BackupFilter) {
        if !filter.include_conversations {
            self.conversations.clear();
        }
        if !filter.include_lore {
            self.lore_memories.clear();
        }
    }

    pub fn pack_binding_count(&self) -> usize {
        self.pack_bindings.len()
    }
    pub fn annotation_count(&self) -> usize {
        self.annotations.len()
    }
    pub fn conversation_count(&self) -> usize {
        self.conversations.len()
    }
    pub fn lore_memory_count(&self) -> usize {
        self.lore_memories.len()
    }
}
