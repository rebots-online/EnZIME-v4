# Stitch MCP Runbook — EnZIME Reader

## 1. Purpose

Create and verify one Stitch project and ten contract-bound UI screens.
Stitch drafts layout and visual treatment.
It does not create or rename product entities.

Inputs:

- `design-system.json`;
- `screen-manifest.json`;
- `../ARCHITECTURE.md` §§14–16 and §24.

Output evidence:

- project ID;
- design-system application evidence;
- ten screen IDs;
- successful `get_screen` result for every screen;
- updated `generation-evidence.json`.

Never place an API key in this pak.
The Stitch server credential belongs only in the operator’s MCP secret
configuration.

## 2. Execution sequence

1. Connect the workstation MCP client to the `stitch` server.
2. Create a project titled exactly `EnZIME Reader`.
3. Record the returned project ID.
4. Create/apply the design system `EnZIME Aurora Glass` from
   `design-system.json`.
5. Generate screens in this order and batching:
   - batch A: Home, Library, Reader;
   - batch B: Annotation, Ask, Notes;
   - batch C: Memory, Settings;
   - batch D: Setup, Download Plan Dialog.
6. Use `GEMINI_3_FLASH`.
7. Escalate an individual screen to `GEMINI_3_1_PRO` only when:
   - the hierarchy is materially wrong;
   - required entities are absent;
   - responsive behavior is incoherent;
   - accessibility state cannot be repaired through edit.
8. After generation, apply the design system to every screen.
9. Call `get_screen` for every returned screen ID.
10. Reject a screen when a required entity is absent or renamed.
11. Record the verified IDs in `generation-evidence.json`.
12. Do not mark the project verified until all ten `get_screen` calls succeed.

Tool names may be surfaced with a provider prefix.
Use the Stitch tools whose operations correspond to:

```text
create_project
create_or_update_design_system
generate_screen
apply_design_system
get_screen
```

Do not invent a successful return when a tool is absent or fails.

## 3. Global screen prompt

Prefix every screen prompt with:

```text
Design a production desktop-first responsive screen for the offline-first
EnZIME Reader application. Use the attached EnZIME Aurora Glass design system:
dark canvas #090B13, restrained glass chrome, Inter UI text, JetBrains Mono for
telemetry, Literata for article reading, periwinkle #7C8CFF primary, teal
#34E0D4 verified/ready accent. Glass is for chrome/overlays, not long reading
surfaces. Meet WCAG 2.2 AA, visible keyboard focus, 200% text scaling,
non-colour-only status, reduced motion. Use the exact component/entity names
provided. Do not rename, merge or invent backend entities. Design real
populated, empty, loading, offline and actionable-error states; no lorem ipsum,
“coming soon,” disabled decorative controls or placeholder product behavior.
Frame: 1440×1024 desktop, with a coherent 412×915 mobile adaptation.
```

## 4. Screen prompts

### 4.1 Home / Dashboard

```text
Screen ID: home-dashboard
Route: /
Root: HomeView

Create HomeView inside AppShell with NavigationRail and compact JobStrip.
Required named regions:
- ContinueReadingCard with article title, pack, progress and one Continue action;
- RecentPacksList containing PackCard items with integrity and availability text;
- TaskInbox with prioritized UserTask rows and resolve/dismiss actions;
- MemoryInboxCard with candidate count and Review action;
- ReadinessPanel showing reader, storage, inference, backup and entitlement as
  ready/limited/blocked using icon plus text.

Show a useful no-pack state that offers Import existing ZIM and Browse catalog.
Show inference-limited state without implying Reader is broken.
On mobile, place continue reading first, then readiness, tasks and recent packs.
Exact entities: HomeView, RecentPacksList, TaskInbox, ReadinessPanel.
```

### 4.2 Library / Pack Manager

```text
Screen ID: library-pack-manager
Route: /library
Root: LibraryView

Create LibraryView with LibraryToolbar and a segmented Installed / Catalog /
Downloads / Storage organization. Desktop may show installed packs plus a
right-side queue; mobile uses tabs.

Required:
- PackGrid with PackCard grid/list toggle;
- PackCard shows displayName, language, version, size, verification state,
  storage location and Open/menu actions;
- CatalogBrowser with search, language/category/max-size filters and
  CatalogEntryCard results;
- DownloadQueue with DownloadJobRow progress, speed, ETA, pause/resume/cancel;
- StoragePlanner with roots, used/free/reserve and consequence preview;
- Import and catalog refresh actions.

Never label an unverified pack as installed/ready.
Exact entities: LibraryView, PackCard, CatalogBrowser, DownloadQueue,
StoragePlanner.
```

### 4.3 Reader

```text
Screen ID: reader
Route: /reader/:packId/:articleKey
Root: ReaderView

Create a calm editorial ReaderView. Keep ArticleContent on an opaque surface.
Use glass only for AppShell, ArticleToolbar and optional side panels.

Required:
- BreadcrumbNav with back/forward hierarchy;
- ArticleToolbar with bookmark, annotate, Ask, typography and overflow actions;
- ArticleContent using Literata, generous measure and visible internal links;
- article title, pack/version and local availability;
- persisted reading progress;
- collapsed annotation rail with counts;
- source unavailable state that preserves note access;
- safe resource failure treatment that does not break the article.

Desktop supports an optional right drawer; mobile gives the article full width
with bottom contextual actions.
Exact entities: ReaderView, ArticleToolbar, ArticleContent, BreadcrumbNav.
```

### 4.4 Annotation Workspace

```text
Screen ID: annotation-workspace
Route: /reader/:packId/:articleKey
Root: AnnotationWorkspace

Design the ReaderView with AnnotationWorkspace open, showing selected text and
existing placed annotations without obscuring reading.

Required:
- AnnotationPanel with list, resolution icon/text, notebook, tags and layer;
- TextHighlighter colour choices distinct from trust/status colours;
- Comment editor bound to the current SelectorBundle;
- InkCanvas tools with colour, width, undo and explicit Commit/Cancel;
- VoiceRecorder with idle/record/review/upload states and elapsed time;
- SidecarImportDialog with signer/trust, counts, conflicts and unsigned warning;
- placement outcomes resolved, ambiguous, source unavailable and orphaned.

No annotation is lost when the source is unavailable.
Exact entities: AnnotationPanel, TextHighlighter, InkCanvas, VoiceRecorder,
SidecarImportDialog.
```

### 4.5 Ask / AI Conversation

```text
Screen ID: ask-ai-conversation
Route: /ask/:conversationId
Root: AskView

Create AskView as a local cited conversation, not a generic chatbot.

Required:
- ConversationSidebar with searchable local conversations;
- ScopeSelector visibly set to passage/article/packs/notebook/all local;
- BackendStatusBadge naming local backend/model/accelerator;
- ConversationThread with MessageBubble items;
- assistant MessageBubble with inline CitationChip items [S1], [S2];
- Composer with Send and streaming Cancel;
- streaming text state;
- citation invalid/retry state;
- model-not-installed state with Install a model action while existing
  conversations remain readable;
- explicit offline/local status without privacy marketing clutter.

Exact entities: AskView, ConversationThread, MessageBubble, CitationChip,
BackendStatusBadge, ScopeSelector.
```

### 4.6 Notes Browser

```text
Screen ID: notes-browser
Route: /notes
Root: NotesView

Create a cross-pack NotesView with three-pane desktop structure and focused
mobile navigation.

Required:
- NotebookTree with hierarchy, counts and create action;
- NotesToolbar search and resolution/type filters;
- TagFilter with multi-select;
- SidecarLayerToggle with trust and visibility text;
- AnnotationList with source title, excerpt, annotation type, tags, layer,
  updated time and resolution;
- AnnotationDetailPanel with full note, evidence context, revisions and
  Open source action;
- source-unavailable annotation remains fully inspectable.

Exact entities: NotesView, AnnotationList, NotebookTree, TagFilter,
SidecarLayerToggle.
```

### 4.7 Memory / Lore Browser

```text
Screen ID: memory-lore-browser
Route: /memory
Root: MemoryView

Create MemoryView as an evidence-governance workspace.

Required:
- MemoryInbox showing LoreCandidate statement, confidence, evidence count and
  Accept/Reject;
- EntityGraph as a legible secondary view, not decorative wallpaper;
- EpisodeTimeline showing revisions, corrections and supersession;
- MemoryDetailPanel with current statement, subject/predicate, evidence,
  revision, pin, correct and delete;
- deletion warning that retrieval will forget the content;
- contradictory candidate state;
- empty inbox that explains nothing is awaiting review.

Do not imply the model can accept memory by itself.
Exact entities: MemoryView, MemoryInbox, EntityGraph, EpisodeTimeline,
MemoryDetailPanel.
```

### 4.8 Settings

```text
Screen ID: settings
Route: /settings
Root: SettingsView

Create SettingsView with stable left categories on desktop and stacked mobile
sections.

Required:
- NetworkPolicyPanel with catalog/download/metered/Wi-Fi policies;
- ComputePanel with backends, models, compatibility, storage and
  install/remove/select actions;
- AccessibilityPanel with live font-scale/typeface/line-height/reduced-motion/
  high-contrast controls;
- EntitlementPanel showing CapabilityDecision source, validity, Refresh,
  Restore and Redeem coupon;
- BackupRestorePanel with destination, encryption, scope, Create and Restore;
- diagnostics/privacy export actions;
- offline provider state that does not disable signed perpetual capabilities.

Exact entities: SettingsView, ComputePanel, AccessibilityPanel,
EntitlementPanel, BackupRestorePanel.
```

### 4.9 First Launch / Setup

```text
Screen ID: first-launch-setup
Route: /setup
Root: SetupWizard

Create a short, honest SetupWizard.

Required:
- CapabilityReport with CPU, RAM, free storage, accelerators, zimReady and
  inferenceReady;
- warnings with concrete actions;
- SetupPathChooser cards: Existing files, Choose packs, Explore without AI;
- ModelRecommendation with compatible recommendation, size, RAM and backend;
- explicit Skip model action;
- progress steps Capabilities / Starting path / Model / Complete;
- completion screen leading to HomeView.

Never imply AI is required.
Exact entities: SetupWizard, CapabilityReport, SetupPathChooser,
ModelRecommendation.
```

### 4.10 Download Plan Dialog

```text
Screen ID: download-plan-dialog
Route context: /library
Root: DownloadPlanDialog

Create a modal over LibraryView that makes transfer consequences explicit.

Required:
- item title, corpus, language, upstream version and trust;
- InstallPlanSummary with source, ranged/full/local-copy mode and destination;
- StorageImpactBar showing free before, temporary peak, final use and free after;
- exact download/final/temporary byte values;
- VerificationPlan showing SHA-256, signature policy and ZIM structure check;
- previous active version retained until candidate verifies;
- Cancel and Confirm download actions;
- storage-insufficient variant with Choose another location action;
- no deceptively bright confirm before the plan is valid.

Exact entities: DownloadPlanDialog, InstallPlanSummary, StorageImpactBar.
```

## 5. Verification rubric

For each `get_screen` result record:

- returned screen ID;
- title;
- root component name present;
- all required entity names present in metadata/description;
- desktop frame;
- mobile frame or responsive evidence;
- design system applied;
- no forbidden placeholder captions;
- actionable empty/error/offline state;
- acceptance or rejection reason.

Reject when:

- a required entity was renamed;
- a decorative control has no contract action;
- long article text is placed on low-contrast glass;
- status relies on colour;
- model absence breaks Reader;
- unverified content appears active;
- entitlement outage appears to revoke perpetual access;
- a “future” feature is shown.

## 6. Evidence update

Only after successful remote calls:

```json
{
  "status": "verified",
  "project": {
    "expectedTitle": "EnZIME Reader",
    "projectId": "<returned-id>",
    "verified": true
  }
}
```

Replace every screen’s `remoteScreenId` and set `verified: true` only from its
own successful `get_screen` result.

