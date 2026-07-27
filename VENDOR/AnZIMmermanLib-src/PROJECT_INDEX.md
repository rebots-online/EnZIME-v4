# Project Index: EnZIM

**Generated:** 2026-02-04
**Last Updated:** 2026-02-10
**Repository Size:** 5.9 MB
**Repository:** https://github.com/rebots-online/EnZIMe
**Type:** Tauri 2 Desktop Application
**Language:** TypeScript (Frontend) + Rust (Backend)
**Copyright:** © 2025–2026 Robin L. M. Cheung, MBA. All rights reserved.

---

## Quick Summary

EnZIM is an offline-first ZIM (Wikipedia) reader built with Tauri 2 + React. Features semantic mesh visualization, annotations, AI assistance, and 11 themes. Uses clean-room AnZimmermanLib for ZIM handling.

**Status:** Early implementation phase (~40% complete)
**Architecture:** Exceptional planning, solid foundation
**Maturity:** 6.5/10 overall

---

## Directory Structure

```
EnZIMe/
├── src/                          # React frontend (53 files, ~6,998 lines)
│   ├── components/               # UI components
│   │   ├── layout/              # AppShell, Header, Sidebar, MenuBar, MainContent, StatusBar
│   │   ├── library/             # LibraryView, ArchiveCard, DropZone
│   │   ├── reader/              # ReaderView for articles
│   │   ├── search/              # SearchInput, SearchView
│   │   ├── mesh/                # MeshPanel (semantic graph)
│   │   ├── annotations/         # AnnotationPanel, InkCanvas, TextNoteEditor, VoiceRecorder, PhotoCapture, VideoCapture, TextHighlighter, CanvasAnnotationLayer
│   │   ├── views/               # BookmarksView, HistoryView, SettingsView
│   │   └── dialogs/             # AboutDialog
│   ├── contexts/                # React contexts
│   │   └── ThemeContext.tsx     # Theme provider (11 themes)
│   ├── services/                # Business logic
│   │   ├── zimService.ts        # ZIM archive handling
│   │   └── semanticEngine.ts    # Link extraction, mesh generation
│   ├── stores/                  # Zustand state management
│   │   └── archiveStore.ts      # Archive & article state
│   ├── entitlements/            # Auth/billing system (incomplete)
│   │   ├── gatekeeper.ts        # Feature gate enforcement
│   │   ├── providers/           # Mock & remote providers
│   │   ├── token.ts             # JWT handling
│   │   └── __tests__/           # 1 test file (14 tests passing)
│   ├── App.tsx                  # Root component
│   ├── main.tsx                 # Entry point
│   └── router.tsx               # React Router config
├── src-tauri/                   # Rust backend
│   ├── src/main.rs              # Tauri commands
│   ├── Cargo.toml               # Rust dependencies
│   └── build.rs                 # Build script
├── AnZimmermanLIB/              # Clean-room ZIM libraries (147MB)
│   ├── AnZimmermanLib/
│   │   ├── zimlib.ts            # TypeScript ZIM library (29KB)
│   │   ├── zimlib.py            # Python ZIM library (18KB)
│   │   ├── zimlib.go            # Go ZIM library (21KB)
│   │   ├── zimlib.php           # PHP ZIM library (23KB)
│   │   ├── api/                 # REST API implementations
│   │   ├── examples/            # Usage examples
│   │   └── TOOLS/               # zim-downloader, zimmer-extension
│   ├── api/                     # Multi-language APIs
│   ├── docs/                    # Language-specific architecture docs
│   └── tests/                   # Cross-language tests
├── DOCS/
│   ├── ARCHITECTURE/            # 15 diagrams (.puml, .mmd)
│   │   ├── ARCHITECTURE.md      # Main architecture (22KB)
│   │   ├── UI/                  # Auth/billing flow diagrams
│   │   └── UI_COMPONENTS.md     # Component specs (21KB)
│   ├── CHECKLISTS/              # Phase-by-phase checklists
│   ├── PLANNING/                # PRD documents
│   └── UI-Samples/              # HTML theme prototypes
├── .github/workflows/           # CI/CD (build.yml)
├── build.sh                     # Cross-platform build script
├── CHECKLIST.md                 # Development checklist (SSoT)
├── CLAUDE.md                    # Project config & rules
└── README.md                    # Project overview
```

---

## Entry Points

| Entry Point | Path | Purpose |
|-------------|------|---------|
| **Frontend** | `src/main.tsx` | React app initialization |
| **Backend** | `src-tauri/src/main.rs` | Tauri command handlers |
| **Build** | `build.sh` | Cross-platform build script |
| **Tests** | `src/entitlements/__tests__/gatekeeper.test.ts` | Gatekeeper tests |
| **CI/CD** | `.github/workflows/build.yml` | GitHub Actions |

---

## Core Modules

### Frontend Services

| Module | Path | Exports | Purpose |
|--------|------|---------|---------|
| **ZIMService** | `src/services/zimService.ts` | `zimService` | Load ZIM archives, read articles, title search |
| **SemanticEngine** | `src/services/semanticEngine.ts` | `semanticEngine` | Extract links, generate mesh graph |

### State Management (Zustand)

| Store | Path | Key State | Purpose |
|-------|------|-----------|---------|
| **ArchiveStore** | `src/stores/archiveStore.ts` | `archives[]`, `currentArchive`, `currentArticle`, `recentArticles[]` (max 50), `bookmarks[]` | Archive management, navigation, history |
| **AnnotationStore** | `src/stores/annotationStore.ts` | Annotations per article | Annotation CRUD, persistence |
| **EntitlementsStore** | `src/entitlements/store.ts` | `gatekeeper`, `ready`, `lastUpdated` | Auth & billing state, capability checks |

**ArchiveStore Actions:**
- `addArchive(archive)`, `removeArchive(id)`, `setCurrentArchive(archive)`
- `setCurrentArticle(article)`, `addToRecent(article)`, `addBookmark(article)`, `removeBookmark(url)`
- `setLoading(loading)`, `setError(error)`

**Persistence:** Zustand middleware persists `archives`, `recentArticles`, `bookmarks` to localStorage.

---

## Tauri Backend Commands (RPC)

**Backend:** Rust (Tauri 2.0)
**IPC Layer:** `@tauri-apps/api` (frontend) ↔ Tauri command handlers (backend)

| Command | Signature | Implementation | Purpose |
|---------|-----------|----------------|---------|
| `get_version()` | `() → String` | `src-tauri/src/main.rs:24` | Returns version with epoch build number (`v0.1.0.84721`) |
| `get_copyright()` | `() → String` | `src-tauri/src/main.rs:30` | Returns copyright notice |
| `get_app_info()` | `() → JSON` | `src-tauri/src/main.rs:37` | Returns app metadata (name, version, description, copyright, license) |

**Versioning Algorithm:**
- **Format:** `v<major>.<minor>.<patch>.<build>`
- **Build Number:** `floor(epoch_seconds / 60) % 100000` (last 5 digits of epoch minutes)
- **Example:** `v0.1.0.84721`

**Future Commands (Phase 8-9):**
- `open_archive(path: String) → Archive` - Load ZIM file
- `search_archive(query: String, archiveId: String) → SearchResults` - Full-text search
- `save_bookmark(articleId: String, note: String) → void` - Persist bookmark

---

### Entitlements System (Incomplete)

| Module | Path | Exports | Purpose |
|--------|------|---------|---------|
| **Gatekeeper** | `src/entitlements/gatekeeper.ts` | `Gatekeeper` | Feature gate enforcement |
| **Token** | `src/entitlements/token.ts` | `TokenManager` | JWT token handling |
| **Store** | `src/entitlements/store.ts` | `useEntitlementsStore` | Entitlements state |

### UI Components

| Component | Path | Purpose |
|-----------|------|---------|
| **AppShell** | `src/components/layout/AppShell.tsx` | Main layout wrapper |
| **LibraryView** | `src/components/library/LibraryView.tsx` | ZIM archive browser |
| **ReaderView** | `src/components/reader/ReaderView.tsx` | Article content display |
| **MeshPanel** | `src/components/mesh/MeshPanel.tsx` | Semantic graph sidebar |
| **SearchView** | `src/components/search/SearchView.tsx` | Search results |

### Annotation System (New)

| Component | Path | Purpose |
|-----------|------|---------|
| **AnnotationPanel** | `src/components/annotations/AnnotationPanel.tsx` | Annotation toolbar & panel |
| **InkCanvas** | `src/components/annotations/InkCanvas.tsx` | Freehand ink drawing |
| **TextNoteEditor** | `src/components/annotations/TextNoteEditor.tsx` | Text note creation/editing |
| **VoiceRecorder** | `src/components/annotations/VoiceRecorder.tsx` | Voice note recording |
| **PhotoCapture** | `src/components/annotations/PhotoCapture.tsx` | Photo annotation capture |
| **VideoCapture** | `src/components/annotations/VideoCapture.tsx` | Video annotation capture |
| **TextHighlighter** | `src/components/annotations/TextHighlighter.tsx` | Text selection highlighting |
| **CanvasAnnotationLayer** | `src/components/annotations/CanvasAnnotationLayer.tsx` | Canvas overlay for annotations |

**Types:** `src/types/annotation.ts`
**Store:** `src/stores/annotationStore.ts`

### Theme System (11 Themes + Light/Dark/System Modes)

| Theme | Light | Dark | Aesthetic | Color Palette | Phase |
|-------|:-----:|:----:|-----------|---------------|-------|
| **Minimal** (default) | ✅ | ✅ | Clean Swiss design | Neutral grays, blue accent | 6.4 |
| **Synaptic Cartography Veil** | ✅ | ✅ | Neon, glassmorphic, neural | Cyan/magenta/electric blue | 6.3 |
| **Brutalist Archive Monolith** | ✅ | ✅ | Bold, print-like, utilitarian | Black/concrete grays | 6.3 |
| **Prismatic Swiss Utility** | ✅ | ✅ | Swiss design, multi-color | Cyan/magenta/gold | 6.3 |
| **Spectral ZIM Reader** | ❌ | ✅ | Sci-fi, spectral glows | Vibrant highlights | 6.3 |
| **Kinetic** | ✅ | ✅ | Colorful, dynamic, Gumroad | Rainbow gradient | 6.4 |
| **Retro** | ❌ | ✅ | CRT terminal, scanlines | Green phosphor/amber | 6.4 |
| **Neumorphism** | ✅ | ✅ | Soft shadows, extruded | Pastel gradients | 6.4 |
| **Glassmorphism** | ✅ | ✅ | Frosted glass, depth | Semi-transparent overlays | 6.4 |
| **Y2K** | ✅ | ✅ | Web maximalism, gradients | Hot pink/cyan/chrome | 6.4 |
| **Cyberpunk** | ❌ | ✅ | Neon dystopian | Neon pink/cyan/yellow | 6.4 |

**Implementation:** CSS custom properties (`--color-primary`, `--bg-surface`) + Tailwind config
**Context Provider:** `src/contexts/ThemeContext.tsx`
**Mode Toggle:** Light / Dark / System (auto-detects OS preference)

---

## Dependencies

### External Libraries

| Library | Type | Purpose | Documentation |
|---------|------|---------|---------------|
| **AnZimmermanLib** | Reference Library (External) | Clean-room ZIM file format implementation | See `AnZimmermanLIB/AnZimmermanLib/README.md` |

**Note:** AnZimmermanLib is a **complete, separate project** used as reference for ZIM format implementation in EnZIM. It is not part of the EnZIM codebase.

**Usage in EnZIM:** EnZIM's `src/services/zimService.ts` implements ZIM reading based on AnZimmermanLib's TypeScript implementation patterns.

---

## Configuration Files

| File | Purpose |
|------|---------|
| `package.json` | npm dependencies, scripts |
| `src-tauri/Cargo.toml` | Rust dependencies, build config |
| `tsconfig.json` | TypeScript compiler options |
| `vite.config.ts` | Vite bundler config |
| `tailwind.config.js` | Tailwind CSS config |
| `postcss.config.js` | PostCSS plugins |

---

## Key Dependencies

### Frontend
- `@tauri-apps/api@^2.0.0` - Tauri 2 APIs
- `react@^18.3.1` - UI framework
- `react-router-dom@^6.30.3` - Routing
- `zustand@^5.0.0` - State management
- `lucide-react@^0.468.0` - Icons

### Backend (Rust)
- `tauri@^2` - Desktop framework
- `tokio@^1` - Async runtime
- `serde@^1` - Serialization
- `tracing@^0.1` - Logging

### Dev
- `vite@^6.0.0` - Bundler
- `vitest@^2.1.0` - Testing
- `tailwindcss@^3.4.17` - Styling
- `typescript@^5.6.0` - Type checking

---

## Implementation Status

**Overall Progress:** ~40% complete (MVP-ready foundation)

| Phase | Name | Completion | MVP Critical? | Blocker |
|-------|------|:----------:|:-------------:|---------|
| **1-4** | Architecture & Design | 100% | ✅ Core | None |
| **5** | Tauri 2 Scaffolding | 90% | ✅ Core | None |
| **6** | Theme System (11 themes) | 95% | ✅ Core | None |
| **7** | Core UI Components | 60% | ✅ Core | In Progress |
| **8** | ZIM Core Integration | 40% | ✅ Core | **LZMA/ZSTD decompression** |
| **9** | Auth & Billing | 10% | ❌ Post-MVP | None (deferred) |
| **10** | Testing & QA | 5% | ❌ Post-MVP | Coverage target |
| **11** | Release Prep | 20% | ❌ Post-MVP | Code signing |

**MVP Scope:** Phases 1-8 (offline reader with search, bookmarks, mesh visualization)
**Post-MVP Scope:** Phases 9-11 (auth, billing, comprehensive testing, release)

**Current Sprint:** Phase 7 (UI components) + Phase 8 (ZIM integration)
**Next Milestone:** Complete Phase 8 → MVP release candidate

**Estimated Time to MVP:** 6-8 weeks (assumes dedicated full-time development, phases 1-8 only)

---

## Documentation

| Document | Path | Size | Topic |
|----------|------|------|-------|
| **Architecture** | `DOCS/ARCHITECTURE/ARCHITECTURE.md` | 22KB | System design |
| **UI Components** | `DOCS/ARCHITECTURE/UI_COMPONENTS.md` | 21KB | Component specs |
| **Checklist** | `CHECKLIST.md` | - | Development SSoT |
| **Diagrams** | `DOCS/ARCHITECTURE/*.puml, *.mmd` | 21 files | Architecture visuals |

---

## Architecture Diagrams Index

**Total:** 21 diagrams (15 PlantUML `.puml` + 6 Mermaid `.mmd`)
**Location:** `DOCS/ARCHITECTURE/`

### Core System Architecture (5 PlantUML)

| Diagram | File | C4 Level | Purpose | Phases |
|---------|------|----------|---------|--------|
| **System Context** | `01_system_context.puml` | Level 1 | User personas, external systems | 1-2 |
| **Container** | `02_container_diagram.puml` | Level 2 | Desktop, Web, Mobile containers | 1-2 |
| **Component** | `03_component_diagram.puml` | Level 3 | Services, state management, auth/billing | 3-4 |
| **Data Models** | `04_class_diagram.puml` | Class | ZIM format, User, Subscription schemas | 3-4 |
| **UI Layout** | `09_ui_layout.puml` | UI | 3-column responsive grid | 6 |

### Workflow Sequences (4 PlantUML)

| Diagram | File | Purpose | Target Performance |
|---------|------|---------|-------------------|
| **Content Load** | `05_seq_content_load.puml` | Archive → Article rendering flow | < 2 seconds |
| **Search** | `06_seq_search.puml` | Query → Results with ranking | < 1 sec (title), < 2 sec (full-text) |
| **Semantic Mesh** | `07_seq_semantic_mesh.puml` | Link extraction → 2.5D graph | < 500ms (10-15 nodes) |
| **Theme System** | `08_ui_theme_system.puml` | CSS variables + context provider | Instant (CSS-only) |

### Auth & Billing (3 PlantUML)

| Diagram | File | Purpose | Phase |
|---------|------|---------|-------|
| **Auth Flow** | `UI/auth_flow.puml` | Login/signup/OAuth sequences | 9 |
| **Billing Flow** | `UI/billing_flow.puml` | Subscription CRUD + Stripe | 9 |
| **Feature Gates** | `UI/feature_gate_flow.puml` | Capability checks, upsell prompts | 9 |

### Mermaid Diagrams (6 .mmd)

| Diagram | File | Type | Purpose |
|---------|------|------|---------|
| **Architecture Overview** | `architecture_overview.mmd` | Flowchart | High-level system flow |
| **Data Flow** | `data_flow.mmd` | Flowchart | Component communication paths |
| **Entity-Relationship** | `erd_models.mmd` | ERD | Database schema (SQLite/IndexedDB) |
| **State Machine** | `state_diagram.mmd` | State | UI state transitions |
| **Deployment** | `deployment_diagram.mmd` | Deployment | Cross-platform deployment targets |
| **Component Hierarchy** | `ui_component_hierarchy.mmd` | Tree | React component structure |

**Viewing:** Use PlantUML extension (VS Code) or [plantuml.com/plantuml](https://www.plantuml.com/plantuml) for `.puml`. Mermaid: [mermaid.live](https://mermaid.live) or GitHub preview.

---

## Test Coverage

| Type | Files | Status |
|------|-------|--------|
| **Unit Tests** | 1 file (gatekeeper) | 14 tests passing |
| **Component Tests** | 0 files | Not implemented |
| **E2E Tests** | 0 files | Not configured |
| **Coverage** | ~2% | Minimal |

---

## Build & Run

```bash
# Install dependencies
npm install

# Development mode
npm run tauri dev

# Build frontend
npm run build

# Full build (cross-platform)
./build.sh

# Run tests
npm test
```

---

## Critical Gaps

1. **ZIM Decompression** - LZMA/ZSTD not implemented
2. **Full-Text Search** - Only title search exists
3. **Auth System** - AuthService not implemented
4. **Billing** - Stripe integration not started
5. **Test Coverage** - Only 2% coverage

---

## Estimated Time to MVP

**6-8 weeks** with dedicated development effort

---

**Token Savings:**
- Full codebase read: ~58,000 tokens
- This index: ~3,500 tokens
- **Savings: 94% per session**

---

## Index Metadata

| Field | Value |
|-------|-------|
| **Generated** | 2026-02-04 |
| **Last Updated** | 2026-02-10 |
| **Index Version** | 2.3 |
| **Repository** | [EnZIMe](https://github.com/rebots-online/EnZIMe) |
| **Repository Size** | 5.9 MB |
| **Current Build** | `v0.1.0.84721` (epoch-based) |
| **Phases Complete** | 1-6 (partial 7-8) |
| **Documentation Status** | Living, per CLAUDE.md requirements |

**Disclaimer:** This index is a **snapshot**. For real-time phase status, see [`CHECKLIST.md`](CHECKLIST.md) (Single Source of Truth).

**Next Update:** When Phase 8 (ZIM Integration) reaches 80% completion or major architecture changes occur.

---

*EnZIM — Bringing offline knowledge to life.*
