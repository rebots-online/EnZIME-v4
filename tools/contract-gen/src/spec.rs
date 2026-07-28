//! §24 entity-to-UI binding table and §16.6 React component identifiers,
//! transcribed verbatim from `DOCS/ARCHITECTURE.md`.
//!
//! Per §24 ("The following table is normative") and §16.6 ("Handwritten
//! aliases may re-export but may not rename them"), these constants are the
//! canonical source the validator checks drift against. They are not derived
//! from any other file in the tree; they ARE the manifest.
//!
//! The transcription is line-for-line from the pak; do not edit unless the
//! pak itself changes.

/// How a binding row refers to its SQLite tables.
///
/// §24 footnote: "'None' means the entity is derived from a ZIM/runtime
/// response and has no canonical SQLite row." The literal string
/// "projection across tables" appears for `HomeView` and is preserved here
/// as a distinct variant — it is neither "None" nor a named-table list, and
/// must not be conflated with either.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TablesRef {
    /// §24 literal cell value `None`.
    None,

    /// §24 literal cell value `projection across tables` (read across many
    /// tables, no single canonical source row).
    Projection,

    /// §24 literal cell value is a comma-separated list of table names.
    Named(&'static [&'static str]),
}

impl TablesRef {
    /// Return the explicit table names this reference asserts, or an empty
    /// slice for `None` / `Projection`. Used by the validator to enumerate
    /// tables that must appear in the migration.
    pub fn names(&self) -> &'static [&'static str] {
        match self {
            TablesRef::None | TablesRef::Projection => &[],
            TablesRef::Named(names) => names,
        }
    }
}

/// One row of the §24 entity-to-UI binding table.
///
/// Fields preserve the verbatim text of each §24 column so a future
/// architecture amendment is a mechanical replacement.
#[derive(Clone, Copy, Debug)]
pub struct BindingRow {
    /// §24 column "UI component" — the React component public name.
    pub component: &'static str,

    /// §24 column "TypeScript entity" — base entity name (array suffix and
    /// generics stripped at validation time, not here).
    pub entity: &'static str,

    /// §24 column "Tauri command(s)" — every command the component invokes.
    pub commands: &'static [&'static str],

    /// §24 column "SQLite table(s)".
    pub tables: TablesRef,

    /// §24 column "Zustand slice" — full dotted path, e.g.
    /// `homeStore.snapshot.continueReading`.
    pub store_slice: &'static str,
}

/// The §24 binding table, transcribed from `DOCS/ARCHITECTURE.md` lines
/// 6359–6425 (65 rows).
pub static BINDING_TABLE: &[BindingRow] = &[
    BindingRow {
        component: "HomeView",
        entity: "HomeSnapshot",
        commands: &["home_snapshot"],
        tables: TablesRef::Projection,
        store_slice: "homeStore.snapshot",
    },
    BindingRow {
        component: "ContinueReadingCard",
        entity: "ReaderHistoryItem",
        commands: &["reader_history"],
        tables: TablesRef::Named(&["reader_history"]),
        store_slice: "homeStore.snapshot.continueReading",
    },
    BindingRow {
        component: "RecentPacksList",
        entity: "PackManifest",
        commands: &["pack_list"],
        tables: TablesRef::Named(&["packs", "pack_versions"]),
        store_slice: "homeStore.snapshot.recentPacks",
    },
    BindingRow {
        component: "TaskInbox",
        entity: "UserTask",
        commands: &["task_list", "task_resolve"],
        tables: TablesRef::Named(&["user_tasks"]),
        store_slice: "taskStore.tasks",
    },
    BindingRow {
        component: "MemoryInboxCard",
        entity: "number",
        commands: &["lore_inbox"],
        tables: TablesRef::Named(&["lore_candidates"]),
        store_slice: "homeStore.snapshot.memoryInboxCount",
    },
    BindingRow {
        component: "ReadinessPanel",
        entity: "SystemReadiness",
        commands: &["readiness_get"],
        tables: TablesRef::Named(&["capability_reports", "settings"]),
        store_slice: "homeStore.snapshot.readiness",
    },
    BindingRow {
        component: "ReadinessIndicator",
        entity: "SystemReadiness",
        commands: &["readiness_get"],
        tables: TablesRef::Named(&["capability_reports", "settings"]),
        store_slice: "appStore.bootstrap.readiness",
    },
    BindingRow {
        component: "JobStrip",
        entity: "DownloadJob",
        commands: &["job_list"],
        tables: TablesRef::Named(&["download_jobs"]),
        store_slice: "libraryStore.downloads",
    },
    BindingRow {
        component: "LibraryView",
        entity: "PackManifest",
        commands: &["pack_list"],
        tables: TablesRef::Named(&["packs", "pack_versions", "pack_locations"]),
        store_slice: "libraryStore.packs",
    },
    BindingRow {
        component: "LibraryToolbar",
        entity: "CatalogSearchRequest",
        commands: &["catalog_search", "pack_import_choose"],
        tables: TablesRef::Named(&["catalog_entries"]),
        store_slice: "libraryStore.catalogQuery",
    },
    BindingRow {
        component: "PackGrid",
        entity: "PackManifest",
        commands: &["pack_list"],
        tables: TablesRef::Named(&["packs", "pack_versions"]),
        store_slice: "libraryStore.packs",
    },
    BindingRow {
        component: "PackCard",
        entity: "PackManifest",
        commands: &["pack_list", "pack_metadata"],
        tables: TablesRef::Named(&["packs", "pack_versions", "pack_locations"]),
        store_slice: "libraryStore.packs",
    },
    BindingRow {
        component: "CatalogBrowser",
        entity: "CatalogEntry",
        commands: &["catalog_search", "catalog_refresh"],
        tables: TablesRef::Named(&["catalog_sources", "catalog_entries"]),
        store_slice: "libraryStore.catalogEntries",
    },
    BindingRow {
        component: "CatalogEntryCard",
        entity: "CatalogEntry",
        commands: &["install_plan"],
        tables: TablesRef::Named(&["catalog_entries"]),
        store_slice: "libraryStore.catalogEntries",
    },
    BindingRow {
        component: "DownloadQueue",
        entity: "DownloadJob",
        commands: &["job_list", "job_pause", "job_resume", "job_cancel"],
        tables: TablesRef::Named(&["download_jobs", "download_chunks"]),
        store_slice: "libraryStore.downloads",
    },
    BindingRow {
        component: "DownloadJobRow",
        entity: "DownloadJob",
        commands: &["job_get", "job_pause", "job_resume", "job_cancel"],
        tables: TablesRef::Named(&["download_jobs", "download_chunks"]),
        store_slice: "libraryStore.downloads",
    },
    BindingRow {
        component: "StoragePlanner",
        entity: "StorageDestination",
        commands: &["storage_roots_list"],
        tables: TablesRef::Named(&["storage_roots"]),
        store_slice: "libraryStore.storageRoots",
    },
    BindingRow {
        component: "DownloadPlanDialog",
        entity: "InstallPlan",
        commands: &["install_plan", "install_start"],
        tables: TablesRef::Named(&["install_transactions", "download_jobs"]),
        store_slice: "libraryStore.installPlan",
    },
    BindingRow {
        component: "InstallPlanSummary",
        entity: "InstallPlan",
        commands: &["install_plan"],
        tables: TablesRef::Named(&["catalog_entries", "storage_roots"]),
        store_slice: "libraryStore.installPlan",
    },
    BindingRow {
        component: "StorageImpactBar",
        entity: "InstallPlan",
        commands: &["install_plan"],
        tables: TablesRef::Named(&["storage_roots"]),
        store_slice: "libraryStore.installPlan",
    },
    BindingRow {
        component: "ReaderView",
        entity: "ArticleResponse",
        commands: &["reader_open", "reader_article"],
        tables: TablesRef::Named(&["reader_history", "reader_positions"]),
        store_slice: "readerStore.currentArticle",
    },
    BindingRow {
        component: "BreadcrumbNav",
        entity: "BreadcrumbItem",
        commands: &["reader_article"],
        tables: TablesRef::None,
        store_slice: "readerStore.currentArticle.breadcrumb",
    },
    BindingRow {
        component: "ArticleToolbar",
        entity: "ArticleResponse",
        commands: &["bookmark_set", "reader_article"],
        tables: TablesRef::Named(&["bookmarks", "annotations"]),
        store_slice: "readerStore.currentArticle",
    },
    BindingRow {
        component: "ReaderPane",
        entity: "ArticleResponse",
        commands: &["reader_article"],
        tables: TablesRef::None,
        store_slice: "readerStore.currentArticle",
    },
    BindingRow {
        component: "ArticleContent",
        entity: "ArticleResponse",
        commands: &["reader_article", "reader_position_save"],
        tables: TablesRef::Named(&["reader_positions"]),
        store_slice: "readerStore.currentArticle",
    },
    BindingRow {
        component: "AnnotationWorkspace",
        entity: "AnnotationRecord",
        commands: &["annotation_list", "annotation_create", "annotation_update"],
        tables: TablesRef::Named(&["annotations", "annotation_revisions"]),
        store_slice: "annotationStore.annotations",
    },
    BindingRow {
        component: "AnnotationPanel",
        entity: "AnnotationRecord",
        commands: &["annotation_list", "annotation_delete", "annotation_restore"],
        tables: TablesRef::Named(&["annotations", "annotation_revisions"]),
        store_slice: "annotationStore.annotations",
    },
    BindingRow {
        component: "TextHighlighter",
        entity: "SelectorBundle",
        commands: &["annotation_create"],
        tables: TablesRef::Named(&["annotations", "annotation_revisions"]),
        store_slice: "readerStore.selection",
    },
    BindingRow {
        component: "InkCanvas",
        entity: "AnnotationAssetBeginResponse",
        commands: &[
            "annotation_asset_begin",
            "annotation_asset_chunk",
            "annotation_asset_commit",
        ],
        tables: TablesRef::Named(&["annotation_assets"]),
        store_slice: "annotationStore.annotations",
    },
    BindingRow {
        component: "VoiceRecorder",
        entity: "AnnotationAssetBeginResponse",
        commands: &[
            "annotation_asset_begin",
            "annotation_asset_chunk",
            "annotation_asset_commit",
        ],
        tables: TablesRef::Named(&["annotation_assets"]),
        store_slice: "annotationStore.annotations",
    },
    BindingRow {
        component: "SidecarImportDialog",
        entity: "SidecarImportPlan",
        commands: &[
            "sidecar_import_choose",
            "sidecar_import_plan",
            "sidecar_import_commit",
        ],
        tables: TablesRef::Named(&["sidecar_imports", "sidecar_layers"]),
        store_slice: "annotationStore.layers",
    },
    BindingRow {
        component: "ReaderCompanionDrawer",
        entity: "ConversationDetail",
        commands: &["conversation_create", "ask_start"],
        tables: TablesRef::Named(&["conversations", "messages"]),
        store_slice: "askStore.activeConversation",
    },
    BindingRow {
        component: "AskView",
        entity: "ConversationDetail",
        commands: &[
            "conversation_list",
            "conversation_get",
            "ask_start",
            "ask_cancel",
        ],
        tables: TablesRef::Named(&["conversations", "messages", "inference_runs"]),
        store_slice: "askStore.activeConversation",
    },
    BindingRow {
        component: "ConversationSidebar",
        entity: "ConversationSummary",
        commands: &["conversation_list", "conversation_create"],
        tables: TablesRef::Named(&["conversations"]),
        store_slice: "askStore.conversations",
    },
    BindingRow {
        component: "ScopeSelector",
        entity: "RetrievalScope",
        commands: &["pack_list", "notebook_tree"],
        tables: TablesRef::Named(&["packs", "notebooks"]),
        store_slice: "askStore.scope",
    },
    BindingRow {
        component: "BackendStatusBadge",
        entity: "BackendStatus",
        commands: &["backend_status", "backend_select"],
        tables: TablesRef::Named(&["inference_backends", "model_installations"]),
        store_slice: "askStore.backendStatus",
    },
    BindingRow {
        component: "ConversationThread",
        entity: "ConversationMessage",
        commands: &["conversation_get"],
        tables: TablesRef::Named(&["messages", "message_citations"]),
        store_slice: "askStore.activeConversation.messages",
    },
    BindingRow {
        component: "MessageBubble",
        entity: "ConversationMessage",
        commands: &["conversation_get"],
        tables: TablesRef::Named(&["messages", "message_citations"]),
        store_slice: "askStore.activeConversation.messages",
    },
    BindingRow {
        component: "CitationChip",
        entity: "CitationRecord",
        commands: &["citation_open"],
        tables: TablesRef::Named(&["message_citations", "retrieval_chunks"]),
        store_slice: "askStore.activeConversation.messages",
    },
    BindingRow {
        component: "Composer",
        entity: "AskStartRequest",
        commands: &["ask_start", "ask_cancel"],
        tables: TablesRef::Named(&["messages", "inference_runs"]),
        store_slice: "askStore.streamingRunId",
    },
    BindingRow {
        component: "NotesView",
        entity: "AnnotationRecord",
        commands: &["annotation_list"],
        tables: TablesRef::Named(&["annotations", "annotation_revisions"]),
        store_slice: "annotationStore.annotations",
    },
    BindingRow {
        component: "NotebookTree",
        entity: "NotebookNode",
        commands: &["notebook_tree", "notebook_create", "notebook_update"],
        tables: TablesRef::Named(&["notebooks"]),
        store_slice: "annotationStore.notebooks",
    },
    BindingRow {
        component: "NotesToolbar",
        entity: "AnnotationListRequest",
        commands: &["annotation_list"],
        tables: TablesRef::Named(&["annotations", "annotation_tags"]),
        store_slice: "annotationStore.filter",
    },
    BindingRow {
        component: "TagFilter",
        entity: "TagSummary",
        commands: &["tag_list", "tag_create", "annotation_list"],
        tables: TablesRef::Named(&["tags", "annotation_tags"]),
        store_slice: "annotationStore.tags",
    },
    BindingRow {
        component: "SidecarLayerToggle",
        entity: "SidecarLayer",
        commands: &["sidecar_layers", "annotation_list"],
        tables: TablesRef::Named(&["sidecar_layers"]),
        store_slice: "annotationStore.layers",
    },
    BindingRow {
        component: "AnnotationList",
        entity: "AnnotationRecord",
        commands: &["annotation_list"],
        tables: TablesRef::Named(&["annotations", "annotation_revisions"]),
        store_slice: "annotationStore.annotations",
    },
    BindingRow {
        component: "AnnotationListItem",
        entity: "AnnotationRecord",
        commands: &["annotation_list"],
        tables: TablesRef::Named(&["annotations", "annotation_revisions"]),
        store_slice: "annotationStore.annotations",
    },
    BindingRow {
        component: "AnnotationDetailPanel",
        entity: "AnnotationRecord",
        commands: &["annotation_update", "annotation_delete"],
        tables: TablesRef::Named(&["annotations", "annotation_revisions"]),
        store_slice: "annotationStore.selectedAnnotationId",
    },
    BindingRow {
        component: "MemoryView",
        entity: "LoreMemory",
        commands: &["lore_search", "lore_inbox"],
        tables: TablesRef::Named(&["lore_memories", "lore_candidates"]),
        store_slice: "memoryStore.memories",
    },
    BindingRow {
        component: "MemoryInbox",
        entity: "LoreCandidate",
        commands: &["lore_inbox", "lore_accept", "lore_reject"],
        tables: TablesRef::Named(&["lore_candidates", "lore_candidate_evidence"]),
        store_slice: "memoryStore.candidates",
    },
    BindingRow {
        component: "MemoryWorkspace",
        entity: "LoreMemory",
        commands: &["lore_search"],
        tables: TablesRef::Named(&["lore_memories", "lore_memory_revisions"]),
        store_slice: "memoryStore.memories",
    },
    BindingRow {
        component: "EntityGraph",
        entity: "EntityGraphNode",
        commands: &["lore_search"],
        tables: TablesRef::Named(&["lore_memories", "lore_memory_evidence"]),
        store_slice: "memoryStore.memories",
    },
    BindingRow {
        component: "EpisodeTimeline",
        entity: "LoreMemory",
        commands: &["lore_search"],
        tables: TablesRef::Named(&["lore_memories", "lore_memory_revisions"]),
        store_slice: "memoryStore.memories",
    },
    BindingRow {
        component: "MemoryDetailPanel",
        entity: "LoreMemory",
        commands: &["lore_correct", "lore_pin", "lore_delete"],
        tables: TablesRef::Named(&["lore_memories", "lore_memory_revisions"]),
        store_slice: "memoryStore.selectedMemoryId",
    },
    BindingRow {
        component: "SettingsView",
        entity: "SettingsSnapshot",
        commands: &["settings_get", "settings_update"],
        tables: TablesRef::Named(&["settings"]),
        store_slice: "settingsStore.settings",
    },
    BindingRow {
        component: "NetworkPolicyPanel",
        entity: "NetworkPolicy",
        commands: &["settings_get", "settings_update"],
        tables: TablesRef::Named(&["settings"]),
        store_slice: "settingsStore.settings.networkPolicy",
    },
    BindingRow {
        component: "ComputePanel",
        entity: "ComputePolicy",
        commands: &[
            "settings_update",
            "backend_status",
            "model_list",
            "model_install_plan",
            "model_remove",
        ],
        tables: TablesRef::Named(&[
            "settings",
            "models",
            "model_installations",
        ]),
        store_slice: "settingsStore.settings.computePolicy",
    },
    BindingRow {
        component: "AccessibilityPanel",
        entity: "AccessibilitySettings",
        commands: &["settings_get", "settings_update"],
        tables: TablesRef::Named(&["settings"]),
        store_slice: "settingsStore.settings.accessibility",
    },
    BindingRow {
        component: "EntitlementPanel",
        entity: "CapabilityDecision",
        commands: &[
            "entitlement_decisions",
            "entitlement_refresh",
            "entitlement_restore",
            "coupon_redeem",
        ],
        tables: TablesRef::Named(&[
            "entitlement_artifacts",
            "entitlement_grants",
            "coupon_ledger",
        ]),
        store_slice: "settingsStore.settings.capabilities",
    },
    BindingRow {
        component: "BackupRestorePanel",
        entity: "BackupSettings",
        commands: &["backup_create", "backup_restore_plan", "backup_restore_commit"],
        tables: TablesRef::Named(&["settings", "backups"]),
        store_slice: "settingsStore.settings.backup",
    },
    BindingRow {
        component: "SetupRoute",
        entity: "CapabilityReport",
        commands: &["capability_probe", "model_list"],
        tables: TablesRef::Named(&["capability_reports", "models"]),
        store_slice: "appStore.bootstrap",
    },
    BindingRow {
        component: "SetupWizard",
        entity: "CapabilityReport",
        commands: &["capability_probe", "settings_update"],
        tables: TablesRef::Named(&["capability_reports", "app_meta"]),
        store_slice: "appStore.bootstrap",
    },
    BindingRow {
        component: "CapabilityReport",
        entity: "CapabilityReport",
        commands: &["capability_probe"],
        tables: TablesRef::Named(&["capability_reports"]),
        store_slice: "appStore.bootstrap",
    },
    BindingRow {
        component: "SetupPathChooser",
        entity: "SetupPath",
        commands: &["pack_import_choose", "catalog_search"],
        tables: TablesRef::Named(&["app_meta"]),
        store_slice: "appStore.bootstrap",
    },
    BindingRow {
        component: "ModelRecommendation",
        entity: "ModelSummary",
        commands: &["model_list", "model_install_plan"],
        tables: TablesRef::Named(&["models", "model_installations"]),
        store_slice: "settingsStore.models",
    },
];

/// The §16.6 React component identifier table (68 names), transcribed from
/// `DOCS/ARCHITECTURE.md` lines 5316–5383. Per §16.6 these are the public
/// component names; handwritten aliases may re-export but may not rename
/// them.
pub static COMPONENT_NAMES: &[&str] = &[
    "App",
    "AppShell",
    "NavigationRail",
    "JobStrip",
    "ReadinessIndicator",
    "HomeView",
    "ContinueReadingCard",
    "RecentPacksList",
    "TaskInbox",
    "MemoryInboxCard",
    "ReadinessPanel",
    "LibraryView",
    "LibraryToolbar",
    "PackGrid",
    "PackCard",
    "CatalogBrowser",
    "CatalogEntryCard",
    "DownloadQueue",
    "DownloadJobRow",
    "StoragePlanner",
    "DownloadPlanDialog",
    "InstallPlanSummary",
    "StorageImpactBar",
    "ReaderView",
    "BreadcrumbNav",
    "ArticleToolbar",
    "ReaderPane",
    "ArticleContent",
    "AnnotationWorkspace",
    "AnnotationPanel",
    "TextHighlighter",
    "InkCanvas",
    "VoiceRecorder",
    "SidecarImportDialog",
    "ReaderCompanionDrawer",
    "AskView",
    "ConversationSidebar",
    "ScopeSelector",
    "BackendStatusBadge",
    "ConversationThread",
    "MessageBubble",
    "CitationChip",
    "Composer",
    "NotesView",
    "NotebookTree",
    "NotesToolbar",
    "TagFilter",
    "SidecarLayerToggle",
    "AnnotationList",
    "AnnotationListItem",
    "AnnotationDetailPanel",
    "MemoryView",
    "MemoryInbox",
    "MemoryWorkspace",
    "EntityGraph",
    "EpisodeTimeline",
    "MemoryDetailPanel",
    "SettingsView",
    "NetworkPolicyPanel",
    "ComputePanel",
    "AccessibilityPanel",
    "EntitlementPanel",
    "BackupRestorePanel",
    "SetupRoute",
    "SetupWizard",
    "CapabilityReport",
    "SetupPathChooser",
    "ModelRecommendation",
];

/// TypeScript built-in types that appear in the §24 entity column and are
/// therefore not undeclared — they do not need a generated DTO. §24's
/// `MemoryInboxCard` row uses `number` for its `memoryInboxCount` projection.
pub static TS_BUILTIN_ENTITIES: &[&str] = &["number", "string", "boolean"];

#[cfg(test)]
mod spec_internal_tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn component_names_are_unique() {
        let mut seen = HashSet::new();
        for name in COMPONENT_NAMES {
            assert!(seen.insert(*name), "duplicate component name: {name}");
        }
    }

    #[test]
    fn binding_components_are_unique() {
        let mut seen = HashSet::new();
        for row in BINDING_TABLE {
            assert!(
                seen.insert(row.component),
                "duplicate binding row component: {}",
                row.component
            );
        }
    }

    #[test]
    fn every_binding_component_exists_in_section_16_6() {
        // §24.1: "a component public name differs from §16.6".
        let names: HashSet<&str> = COMPONENT_NAMES.iter().copied().collect();
        for row in BINDING_TABLE {
            assert!(
                names.contains(row.component),
                "§24 references component {} but §16.6 does not declare it",
                row.component
            );
        }
    }

    #[test]
    fn binding_row_count_matches_section_24() {
        // 65 rows in the §24 table (ARCHITECTURE.md lines 6361–6425).
        assert_eq!(BINDING_TABLE.len(), 65, "transcription drift");
        assert_eq!(COMPONENT_NAMES.len(), 68, "transcription drift");
    }
}
