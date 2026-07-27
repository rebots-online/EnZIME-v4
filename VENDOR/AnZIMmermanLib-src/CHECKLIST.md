Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
All rights reserved.
Unauthorized use without prior written consent is strictly prohibited.

# EnZIM Development Checklist

**Copyright (C) 2025 Robin L. M. Cheung, MBA. All rights reserved.**

> **Single Source of Truth** — All task tracking for EnZIM project

---

## Checklist Legend

- `[ ]` Not yet begun
- `[/]` Started, not complete
- `[X]` Completed, not thoroughly tested
- `✅` Tested and complete

---

## Phase 1: Architecture Foundation (Session 1 — 2026-01-16)

### 1.1 Project Setup & PRD Analysis

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Convert EnZIM PRD PDFs to text | Dev | Session 1 | ✅ | `enzim_prd1.txt`, `enzim_prd2.txt` created |
| Review CLAUDE.md directives | Dev | Session 1 | ✅ | Architecture-first approach understood |
| Analyze AnZimmermanLib architecture | Dev | Session 1 | ✅ | Library integration points identified |

### 1.2 PlantUML Architecture Diagrams

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Create `01_system_context.puml` | Dev | Session 1 | ✅ | C4 Level 1 - User personas & system |
| Create `02_container_diagram.puml` | Dev | Session 1 | ✅ | C4 Level 2 - Desktop/Web/Mobile containers |
| Create `03_component_diagram.puml` | Dev | Session 1 | ✅ | 4-layer component architecture |
| Create `04_class_diagram.puml` | Dev | Session 1 | ✅ | ZIM format + application data models |
| Create `05_seq_content_load.puml` | Dev | Session 1 | ✅ | Article loading sequence |
| Create `06_seq_search.puml` | Dev | Session 1 | ✅ | Search query workflow |
| Create `07_seq_semantic_mesh.puml` | Dev | Session 1 | ✅ | Mesh generation sequence |

### 1.3 Mermaid Diagrams

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Create `architecture_overview.mmd` | Dev | Session 1 | ✅ | System flowchart |
| Create `data_flow.mmd` | Dev | Session 1 | ✅ | Data flow through components |
| Create `erd_models.mmd` | Dev | Session 1 | ✅ | Entity-relationship diagram |
| Create `state_diagram.mmd` | Dev | Session 1 | ✅ | UI state machine |
| Create `deployment_diagram.mmd` | Dev | Session 1 | ✅ | Cross-platform targets |

### 1.4 Core Documentation

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Create `DOCS/ARCHITECTURE/ARCHITECTURE.md` | Dev | Session 1 | ✅ | Main architecture doc (~19KB) |
| Create `README.md` | Dev | Session 1 | ✅ | Project overview, references CLAUDE.md |

---

## Phase 2: UI Analysis & Documentation (Session 1 — 2026-01-16)

### 2.1 UI Prototype Analysis

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Analyze `synaptic_cartography_veil_1.html` | Dev | Session 1 | ✅ | Theme tokens extracted |
| Analyze `brutalist_archive_monolith_0.html` | Dev | Session 1 | ✅ | Theme tokens extracted |
| Analyze `prismatic_swiss_utility_0.html` | Dev | Session 1 | ✅ | Theme tokens extracted |
| Analyze `AnZimmerman-.html` (Spectral) | Dev | Session 1 | ✅ | Theme tokens extracted |

### 2.2 UI Architecture Diagrams

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Create `08_ui_theme_system.puml` | Dev | Session 1 | ✅ | Theme system with CSS vars |
| Create `09_ui_layout.puml` | Dev | Session 1 | ✅ | UI layout structure |
| Create `ui_component_hierarchy.mmd` | Dev | Session 1 | ✅ | Component tree |

### 2.3 UI Documentation

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Create `UI_COMPONENTS.md` | Dev | Session 1 | ✅ | ~690 lines, all themes documented |
| Update `ARCHITECTURE.md` diagram index | Dev | Session 1 | ✅ | New UI diagrams listed |

---

## Phase 3: Auth & Billing Architecture (Session 1 — 2026-01-16)

### 3.1 Data Model Placeholders

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Add `User` class to `04_class_diagram.puml` | Dev | Session 1 | ✅ | id, email, authProvider fields |
| Add `Subscription` class | Dev | Session 1 | ✅ | plan, status, Stripe IDs |
| Add `FeatureFlags` class | Dev | Session 1 | ✅ | maxArchives, aiAssistantEnabled, etc. |
| Add `BillingEvent` class | Dev | Session 1 | ✅ | Payment lifecycle events |
| Add auth/billing enums | Dev | Session 1 | ✅ | AuthProvider, SubscriptionPlan, etc. |

### 3.2 Component Placeholders

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Add `Auth Service` to `03_component_diagram.puml` | Dev | Session 1 | ✅ | OAuth/Email auth |
| Add `Session Manager` | Dev | Session 1 | ✅ | User session handling |
| Add `Billing Service` | Dev | Session 1 | ✅ | Stripe integration |
| Add `Feature Gate` | Dev | Session 1 | ✅ | Entitlement checking |
| Add `Auth State` slice | Dev | Session 1 | ✅ | State layer placeholder |
| Add `Subscription State` slice | Dev | Session 1 | ✅ | State layer placeholder |
| Add feature gate connections | Dev | Session 1 | ✅ | Gates AI, Mesh, Annotations |

### 3.3 Auth & Billing Documentation

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Add Section 11 to `UI_COMPONENTS.md` | Dev | Session 1 | ✅ | Auth/Billing UI components |
| Add Section 10.1 to `ARCHITECTURE.md` | Dev | Session 1 | ✅ | Auth strategy & feature gating |

---

## Phase 4: Auth & Billing UI Flows (Session 2 — 2026-01-17)

### 4.1 Directory Setup

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Create `DOCS/ARCHITECTURE/UI/` directory | Dev | Session 2 | ✅ | Directory exists |

### 4.2 Auth Flow Diagrams

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Create `auth_flow.puml` | Dev | Session 2 | ✅ | Login/signup/logout/OAuth sequences |
| Create `auth_state_machine.mmd` | Dev | Session 2 | ✅ | Auth state transitions |

### 4.3 Billing Flow Diagrams

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Create `billing_flow.puml` | Dev | Session 2 | ✅ | Subscription upgrade/downgrade/cancel |
| Create `stripe_webhook_flow.puml` | Dev | Session 2 | [X] | Included in billing_flow.puml |

### 4.4 Feature Gate Flows

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Create `feature_gate_flow.puml` | Dev | Session 2 | ✅ | Entitlement check sequences |
| Create `feature_gate_decision.mmd` | Dev | Session 2 | [X] | Included in feature_gate_flow.puml |

### 4.5 UI Wireframes & Bindings (Based on UI-Samples)

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Create `ui_wireframes.md` | Dev | Session 2 | ✅ | Annotated wireframes with variable names |
| Create `state_bindings.md` | Dev | Session 2 | ✅ | UI ↔ Backend state mapping |
| Create `api_contracts.md` | Dev | Session 2 | ✅ | Service API definitions |

### 4.6 Integration Review

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Review frontend-backend alignment | Dev | Session 2 | ✅ | No architectural blockers found |
| Create `BLOCKERS.md` (if needed) | Dev | Session 2 | N/A | No blockers identified |

---

## Phase 5: Tauri 2 Project Scaffolding (Week 1)

### 5.1 Project Initialization

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Run `npm create tauri-app@latest` with React+TS | Dev | Day 1 | [X] | `src-tauri/` created |
| Configure `tauri.conf.json` for EnZIM | Dev | Day 1 | [X] | App name, identifier, window config |
| Setup `Cargo.toml` with Tauri 2 + plugins | Dev | Day 1 | [X] | fs, shell, http, dialog plugins |
| Install frontend deps: React 18, Zustand, TailwindCSS | Dev | Day 1 | [X] | `package.json` configured |
| Install Lucide icons, shadcn/ui components | Dev | Day 1 | [X] | UI toolkit ready |

### 5.2 Version & Build System (Per Global Rules)

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Create `build.sh` / `build.ps1` scripts | Dev | Day 2 | [X] | Cross-platform build automation |
| Implement epoch-based build number | Dev | Day 2 | [X] | `floor(epoch/60) % 100000` |
| Configure executable naming | Dev | Day 2 | [X] | `enzim_v{ver}.{build}.{ext}` |
| Add version to `tauri.conf.json` bundle | Dev | Day 2 | [X] | Installer shows version |

### 5.3 Copyright & Branding (Per Global Rules)

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Create splash screen with copyright | Dev | Day 2 | [X] | Shows on app launch |
| Add `File → About` dialog | Dev | Day 2 | [X] | Copyright, License, Version+Build |
| Add version to status bar (bottom-right) | Dev | Day 2 | [X] | `v0.1a.XXXXX` visible |
| Implement menu: File \| Edit \| View \| Help | Dev | Day 2 | [X] | Platform-native menus |

### 5.4 GitHub Actions CI/CD

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Create `.github/workflows/build.yml` | Dev | Day 3 | [X] | Multi-platform trigger |
| Configure Linux job (ubuntu-latest) | Dev | Day 3 | [X] | .deb, .AppImage artifacts |
| Configure Windows job (windows-latest) | Dev | Day 3 | [X] | .msi, .exe (NSIS) artifacts |
| Configure macOS job (macos-latest) | Dev | Day 3 | [X] | .dmg, .app artifacts |
| Add artifact upload with version naming | Dev | Day 3 | [X] | `enzim_v{ver}.{build}_linux.AppImage` |
| Create release workflow (on tag push) | Dev | Day 3 | [X] | Auto-publish to GitHub Releases |

### 5.5 Mobile Targets (Tauri Mobile)

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Run `npm run tauri android init` | Dev | Day 4 | [ ] | Android project scaffolded |
| Configure Android signing | Dev | Day 4 | [ ] | Release keystore setup |
| Add Android build to CI/CD | Dev | Day 4 | [ ] | .apk artifact generated |
| Run `npm run tauri ios init` (macOS only) | Dev | Day 4 | [ ] | iOS project scaffolded |

---

## Phase 6: Theme System & Core UI Shell (Week 1-2)

### 6.1 Theme Infrastructure

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Create `src/themes/` directory structure | Dev | Day 5 | [X] | Theme files organized |
| Implement CSS custom properties system | Dev | Day 5 | [X] | `--bg-primary`, `--accent`, etc. |
| Create `ThemeProvider` React context | Dev | Day 5 | [X] | Theme state managed |
| Implement `useTheme()` hook | Dev | Day 5 | [X] | Components access theme |
| Persist theme choice to localStorage | Dev | Day 5 | [X] | Theme survives reload |

### 6.2 Light/Dark/System Toggle (Required)

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Implement Light mode variant | Dev | Day 5 | [X] | All themes have light variant |
| Implement Dark mode variant | Dev | Day 5 | [X] | All themes have dark variant |
| Implement System Auto detection | Dev | Day 5 | [X] | `prefers-color-scheme` respected |
| Create `ThemeToggle` component | Dev | Day 5 | [X] | 3-way toggle UI |

### 6.3 Theme Implementations (UI-Samples Based)

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Implement **Synaptic Cartography Veil** | Dev | Day 6 | [X] | Neon, glassmorphic, particles |
| Implement **Brutalist Archive Monolith** | Dev | Day 6 | [X] | Concrete, hard shadows, grid |
| Implement **Prismatic Swiss Utility** | Dev | Day 6 | [X] | Clean Swiss, cyan/magenta/gold |
| Implement **Spectral ZIM Reader** | Dev | Day 6 | [X] | Dark, vibrant, refraction |

### 6.4 Additional Themes (Per Global Rules)

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Implement **Kinetic** theme | Dev | Day 7 | [X] | Colorful, dynamic, Gumroad-inspired |
| Implement **Retro** theme | Dev | Day 7 | [X] | CRT terminal, scanlines |
| Implement **Neumorphism** theme | Dev | Day 7 | [X] | Soft shadows, extruded surfaces |
| Implement **Glassmorphism** theme | Dev | Day 7 | [X] | Frosted glass with depth |
| Implement **Y2K** theme | Dev | Day 7 | [X] | Early 2000s web maximalism |
| Implement **Cyberpunk** theme | Dev | Day 7 | [X] | Neon-soaked dystopian |
| Implement **Minimal** theme | Dev | Day 7 | [X] | Clean Swiss design (default) |

### 6.5 App Shell Components

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Implement `AppShell` layout component | Dev | Day 8 | [X] | 3-column responsive grid |
| Implement `Header` component | Dev | Day 8 | [X] | Logo, search, status, avatar |
| Implement `Sidebar` component | Dev | Day 8 | [X] | Navigation, archive list |
| Implement `MainContent` container | Dev | Day 8 | [X] | View router outlet |
| Implement `RightPanel` (mesh sidebar) | Dev | Day 8 | [ ] | Collapsible panel |
| Implement `StatusBar` component | Dev | Day 8 | [X] | Version, build, status |

---

## Phase 7: Core Feature UI Components (Week 2)

### 7.1 Library Management

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Implement `LibraryView` container | Dev | Week 2 | [X] | Grid/list view toggle |
| Implement `ZimCard` component | Dev | Week 2 | [X] | Archive display card |
| Implement `ZimDropzone` component | Dev | Week 2 | [X] | Drag-drop .zim files |
| Implement `ArchiveDetails` modal | Dev | Week 2 | [ ] | Metadata, size, count |
| Wire to `useAppStore.archives` state | Dev | Week 2 | [X] | State bindings working |

### 7.2 Reader View

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Implement `ReaderView` container | Dev | Week 2 | [X] | Article display area |
| Implement `ArticleHeader` component | Dev | Week 2 | [X] | Title, breadcrumb, actions |
| Implement `ArticleContent` (WebView) | Dev | Week 2 | [X] | Renders ZIM HTML |
| Implement `BookmarkButton` component | Dev | Week 2 | [X] | Add/remove bookmark |
| Implement `TTSControls` component | Dev | Week 2 | [ ] | Play/pause, speed, voice |
| Implement `AnnotationToolbar` component | Dev | Week 2 | [ ] | Highlight, note actions |

### 7.3 Search Interface

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Implement `SearchBar` component | Dev | Week 2 | [X] | ⌘K shortcut, autocomplete |
| Implement `SearchResults` list | Dev | Week 2 | [X] | Result items with snippets |
| Implement `SearchFilters` component | Dev | Week 2 | [ ] | Archive, date filters |
| Wire to search state & service | Dev | Week 2 | [X] | Real-time results |

### 7.4 Semantic Mesh

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Implement `SemanticMeshView` container | Dev | Week 2 | [X] | Graph visualization |
| Implement `GraphNode` component | Dev | Week 2 | [ ] | Interactive nodes |
| Implement `GraphEdge` SVG component | Dev | Week 2 | [ ] | Connection lines |
| Implement force-directed layout | Dev | Week 2 | [ ] | Auto-positioning |
| Implement `RelatedArticles` list | Dev | Week 2 | [X] | Text list fallback |

### 7.5 Navigation & History

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Implement `HistoryView` component | Dev | Week 2 | [X] | Recent articles list |
| Implement `BookmarksView` component | Dev | Week 2 | [X] | Saved bookmarks |
| Implement `SettingsView` component | Dev | Week 2 | [X] | Preferences UI |
| Implement routing (React Router) | Dev | Week 2 | [X] | View navigation |

---

## Phase 8: ZIM Core Integration (Week 2-3)

### 8.1 AnZimmermanLib Integration

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Import `zimlib.ts` from AnZimmermanLib | Dev | Week 2 | [X] | Library available |
| Create `ZimService` wrapper | Dev | Week 2 | [X] | API abstraction |
| Implement `loadArchive(file)` | Dev | Week 2 | [X] | Opens .zim file |
| Implement `getArticle(url)` | Dev | Week 2 | [X] | Returns HTML content |
| Implement `getMetadata()` | Dev | Week 2 | [X] | Archive info |

### 8.2 Search Engine

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Create `SearchService` class | Dev | Week 3 | [X] | Search abstraction |
| Implement title index building | Dev | Week 3 | [X] | Fast title search |
| Implement full-text indexing | Dev | Week 3 | [ ] | Web Worker background |
| Implement search result ranking | Dev | Week 3 | [ ] | Relevance sorting |
| Target: < 1s title, < 2s full-text | Dev | Week 3 | [ ] | Performance met |

### 8.3 Semantic Engine

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Create `SemanticEngine` class | Dev | Week 3 | [X] [JULES] | Mesh generation |
| Implement link extraction | Dev | Week 3 | [X] [JULES] | Parse article links |
| Implement graph building | Dev | Week 3 | [X] [JULES] | Nodes + edges |
| Implement layout algorithm | Dev | Week 3 | [ ] | Force-directed |

---

## Phase 9: Auth & Billing Implementation (Week 3)

### 9.1 Auth Service

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Implement `AuthService` class | Dev | Week 3 | [ ] | Per `api_contracts.md` |
| Implement email/password login | Dev | Week 3 | [ ] | JWT returned |
| Implement OAuth (Google, GitHub, Apple) | Dev | Week 3 | [ ] | Redirect flow works |
| Implement session persistence | Dev | Week 3 | [ ] | Token in localStorage |
| Implement token refresh | Dev | Week 3 | [ ] | Auto-refresh |

### 9.2 Auth UI Components

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Implement `LoginModal` component | Dev | Week 3 | [ ] | Email + OAuth buttons |
| Implement `SignupModal` component | Dev | Week 3 | [ ] | Registration form |
| Implement `UserAvatar` dropdown | Dev | Week 3 | [ ] | Profile menu |
| Implement `AuthGuard` HOC | Dev | Week 3 | [ ] | Route protection |
| Wire to `useAuthStore` | Dev | Week 3 | [ ] | State bindings |

### 9.3 Billing Service (Stripe)

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Implement `BillingService` class | Dev | Week 3 | [ ] | Per `api_contracts.md` |
| Integrate Stripe.js / Elements | Dev | Week 3 | [ ] | Card input component |
| Implement subscription creation | Dev | Week 3 | [ ] | Checkout flow |
| Implement subscription management | Dev | Week 3 | [ ] | Upgrade/downgrade/cancel |
| Implement webhook handler | Dev | Week 3 | [ ] | Payment events |

### 9.4 Billing UI Components

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Implement `PricingTable` component | Dev | Week 3 | [ ] | Plan comparison |
| Implement `SubscriptionCard` component | Dev | Week 3 | [ ] | Current plan display |
| Implement `PaymentMethodForm` | Dev | Week 3 | [ ] | Stripe Elements |
| Implement `InvoiceHistory` component | Dev | Week 3 | [ ] | Past payments |
| Wire to `useSubscriptionStore` | Dev | Week 3 | [ ] | State bindings |

### 9.5 Feature Gating

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Implement `FeatureGate` component | Dev | Week 3 | [ ] | Per flow diagram |
| Implement `useFeature()` hook | Dev | Week 3 | [ ] | Feature checks |
| Implement `UpgradePrompt` component | Dev | Week 3 | [ ] | Upsell modal |
| Gate AI Assistant (Starter+) | Dev | Week 3 | [ ] | Gating works |
| Gate Cloud Sync (Pro+) | Dev | Week 3 | [ ] | Gating works |
| Gate Custom Themes (Pro+) | Dev | Week 3 | [ ] | Gating works |

---

## Phase 10: Testing & QA (Week 4)

### 10.1 Unit Tests

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Setup Vitest test runner | Dev | Week 4 | [ ] | `npm test` works |
| Test ZimService methods | Dev | Week 4 | [ ] | 80%+ coverage |
| Test SearchService methods | Dev | Week 4 | [ ] | 80%+ coverage |
| Test AuthService methods | Dev | Week 4 | [ ] | 80%+ coverage |
| Test BillingService methods | Dev | Week 4 | [ ] | 80%+ coverage |
| Test Zustand stores | Dev | Week 4 | [ ] | State logic verified |

### 10.2 Integration Tests

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Setup Playwright | Dev | Week 4 | [ ] | E2E configured |
| Test archive loading flow | Dev | Week 4 | [ ] | File → display |
| Test search flow | Dev | Week 4 | [ ] | Query → results |
| Test auth flow | Dev | Week 4 | [ ] | Login/logout |
| Test billing flow | Dev | Week 4 | [ ] | Subscription |

### 10.3 Cross-Platform Testing

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Test on Windows 10/11 | Dev | Week 4 | [ ] | All features work |
| Test on macOS (Intel + ARM) | Dev | Week 4 | [ ] | All features work |
| Test on Ubuntu 22.04+ | Dev | Week 4 | [ ] | All features work |
| Test on Android (API 26+) | Dev | Week 4 | [ ] | Core features work |
| Test PWA in Chrome/Firefox/Safari | Dev | Week 4 | [ ] | Offline works |

### 10.4 Theme Testing

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Visual test all 11 themes | Dev | Week 4 | [ ] | No layout breaks |
| Test light/dark/system toggle | Dev | Week 4 | [ ] | Mode switching works |
| Test theme persistence | Dev | Week 4 | [ ] | Survives restart |
| Test responsive breakpoints | Dev | Week 4 | [ ] | Mobile/tablet/desktop |

---

## Phase 11: Release Preparation (Week 4)

### 11.1 Documentation

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Update README.md with install instructions | Dev | Week 4 | [ ] | User-ready docs |
| Create CHANGELOG.md | Dev | Week 4 | [ ] | Version history |
| Create CONTRIBUTING.md | Dev | Week 4 | [ ] | Dev guidelines |
| Update ARCHITECTURE.md | Dev | Week 4 | [ ] | Reflects final impl |

### 11.2 Release Artifacts

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Generate Linux packages | Dev | Week 4 | [ ] | .deb, .AppImage |
| Generate Windows packages | Dev | Week 4 | [ ] | .msi, .exe |
| Generate macOS package | Dev | Week 4 | [ ] | .dmg |
| Generate Android APK | Dev | Week 4 | [ ] | .apk |
| Deploy PWA to hosting | Dev | Week 4 | [ ] | URL accessible |

### 11.3 Store Submissions (Post-Release)

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Submit to Microsoft Store | Dev | Post | [ ] | Listed |
| Submit to Mac App Store | Dev | Post | [ ] | Listed |
| Submit to Flathub/Snapcraft | Dev | Post | [ ] | Listed |
| Submit to Google Play | Dev | Post | [ ] | Listed |
| Submit to Apple App Store | Dev | Post | [ ] | Requires iOS build |

### 11.4 Launch Checklist

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Version set to `v1.0.0` | Dev | Release | [ ] | No alpha/beta |
| All tests passing | Dev | Release | [ ] | CI green |
| No critical/high bugs | Dev | Release | [ ] | Issue tracker clean |
| Copyright notice verified | Dev | Release | [ ] | Shows on launch |
| Build numbers in filenames | Dev | Release | [ ] | Per global rules |
| Create GitHub Release | Dev | Release | [ ] | Tag + notes |

---

## 12. Entitlements Refactor (ChatGPT 5.2 / online webap)

### Objectives
- Move all capability checks behind a single Gatekeeper API (`can`, `limit`, `tier`, `quota`)
- Ensure modules never check plans directly (capability-first enforcement)
- Add offline-first cached entitlements with signed token verification

### Scope
- Documentation updates in `CHECKLIST.md` and `DOCS/ARCHITECTURE/*`
- Gatekeeper + provider scaffolding and minimal enforcement
- billedr (EntitlementsAuthority) stub integration with local cache

### Risks
- Breaking feature access if gatekeeper defaults or caching logic is wrong
- Architectural drift if diagrams are not updated alongside code
- Offline fallback behavior diverging from billedr-issued entitlements

### Ordered Checklist Steps

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Add Entitlements Refactor section to roadmap | Dev | Week 2 | ✅ | This section includes objectives, scope, risks, validation, rollback |
| Update ARCHITECTURE.md with Entitlements & Billing model | Dev | Week 2 | ✅ | Capability-first, module enforcement, billedr boundary, offline cache |
| Update system/context + container/component + sequence + data_flow diagrams | Dev | Week 2 | ✅ | Gatekeeper, provider, token cache, billedr added |
| Commit A: Gatekeeper scaffolding + mock provider | Dev | Week 2 | ✅ | No behavior change; defaults allow as needed |
| Commit B: Minimal enforcement on 1–2 optional features | Dev | Week 2 | ✅ | User-facing denial messaging added |
| Commit C: billedr stub + signed token v0 + cache | Dev | Week 2 | ✅ | Offline fallback to defaults |
| Commit D: Tests + remove legacy plan checks | Dev | Week 2 | ✅ | Converted modules free of plan checks |

### Validation
- Run UI/unit tests (or document missing deps) ✅
- Verify offline entitlements fallback works without network [X]
- Confirm no direct plan checks remain in converted modules ✅

### Rollback
- Revert to mock provider defaults
- Remove gatekeeper calls from converted modules
- Restore previous plan-check logic if needed

---

## Phase 13: Naming Registry (Single Source of Truth)

> **CRITICAL:** All agents MUST use these exact names. No variations allowed. This prevents duplicate implementations.

### 13.0.1 TypeScript Interfaces (Exact Definitions)

```typescript
// === Core Data Models ===

interface ZimArchive {
  id: string;                    // UUID v4
  name: string;                  // Filename (e.g., "wikipedia_en.zim")
  title: string;                 // From ZIM metadata
  description: string;           // From ZIM metadata
  language: string;              // ISO 639-1 (e.g., "en")
  articleCount: number;          // Total articles
  mediaCount: number;            // Images, videos, etc.
  size: number;                  // File size in bytes
  path: string;                  // Absolute file path
  favicon?: string;              // Base64 or data URL
  mainPage: string;              // Main article URL (e.g., "A/Main_Page")
  lastOpened?: Date;             // Last access timestamp
  addedAt: Date;                 // Import timestamp
}

interface Article {
  url: string;                   // ZIM URL (e.g., "A/Main_Page")
  archiveId: string;             // Foreign key to ZimArchive.id
  title: string;                 // Article title
  content?: string;              // HTML content (lazy loaded)
  mimeType: string;              // "text/html", "image/png", etc.
  namespace: string;             // 'A', 'I', 'M', etc. (single char)
  links: string[];               // Extracted internal links
}

interface Bookmark {
  id: string;                    // UUID v4
  archiveId: string;             // Foreign key
  articleUrl: string;            // Article identifier
  title: string;                 // Cached article title
  note?: string;                 // Optional user note (max 1000 chars)
  createdAt: Date;               // Bookmark creation time
}

interface HistoryEntry {
  id: string;                    // UUID v4
  archiveId: string;             // Foreign key
  articleUrl: string;            // Article identifier
  title: string;                 // Cached article title
  timestamp: Date;               // Visit time
  duration?: number;             // Time spent reading (seconds)
}

interface GraphNode {
  articleUrl: string;            // Article identifier
  archiveId: string;             // Archive context
  title: string;                 // Display label
  x: number;                     // Layout position X
  y: number;                     // Layout position Y
  isFocused: boolean;            // Is current focus node?
  depth: number;                 // Distance from focus (0, 1, 2)
}

interface GraphEdge {
  source: string;                // Source articleUrl
  target: string;                // Target articleUrl
  weight: number;                // Link strength (default 1.0)
  edgeType: 'internal' | 'external' | 'related';
}

interface SearchResult {
  articleUrl: string;            // Article identifier
  archiveId: string;             // Archive context
  title: string;                 // Article title
  snippet?: string;              // Search result snippet
  score: number;                 // Relevance score (0-1)
}

interface Link {
  url: string;                   // Link URL
  title: string;                 // Link text
  isInternal: boolean;           // Internal to ZIM?
}

interface GraphData {
  nodes: GraphNode[];            // All graph nodes
  edges: GraphEdge[];            // All graph edges
}

interface MeshOptions {
  maxDepth?: number;             // Default: 1
  maxNodes?: number;             // Default: 15
  includeExternal?: boolean;     // Default: false
}

// === Entitlements ===

interface EntitlementsState {
  gatekeeper: EntitlementsGatekeeper;  // Gatekeeper instance
  ready: boolean;                      // Initialization complete?
  lastUpdated: string;                 // ISO 8601 timestamp
}

interface EntitlementCapabilities {
  export_annotations: boolean;
  cloud_sync: boolean;
  ai_assistant: boolean;
  custom_themes: boolean;
  unlimited_archives: boolean;
}

interface EntitlementLimits {
  max_archives: number;          // Default: 5
  max_bookmarks: number;         // Default: 500
  max_annotations: number;       // Default: 100
}

type EntitlementTier = 'free' | 'starter' | 'pro';

// === Store States ===

interface ArchiveState {
  // State
  archives: ZimArchive[];
  currentArchive: ZimArchive | null;
  currentArticle: Article | null;
  recentArticles: Article[];     // Max 50
  bookmarks: Article[];
  isLoading: boolean;
  error: string | null;

  // Actions
  addArchive: (archive: ZimArchive) => void;
  removeArchive: (id: string) => void;
  setCurrentArchive: (archive: ZimArchive | null) => void;
  setCurrentArticle: (article: Article | null) => void;
  addToRecent: (article: Article) => void;
  addBookmark: (article: Article) => void;
  removeBookmark: (url: string) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
}

// === Component Props ===

interface LibraryViewProps {
  archives: ZimArchive[];
  onArchiveSelect: (id: string) => void;
  isLoading?: boolean;
}

interface ReaderViewProps {
  article: Article | null;
  onBookmark: () => void;
  onBack: () => void;
}

interface ArticleContentProps {
  content: string;
}

interface SearchViewProps {
  results: SearchResult[];
  onResultClick: (url: string) => void;
  isSearching?: boolean;
}

interface MeshPanelProps {
  graph: GraphData;
  onNodeClick: (url: string) => void;
}

interface ArchiveCardProps {
  archive: ZimArchive;
  onClick: () => void;
  onRemove: () => void;
}

interface DropZoneProps {
  onDrop: (file: File) => void;
}

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
}
```

### 13.0.2 Service Class Methods (Exact Signatures)

```typescript
// === ZIMService ===
class ZIMService {
  // Properties
  private archives: Map<string, ZimArchive>;

  // Methods (EXACT signatures - do not modify)
  async openArchive(path: string): Promise<ZimArchive>;
  async closeArchive(archiveId: string): Promise<void>;
  async getArticle(archiveId: string, articleUrl: string): Promise<Article>;
  async getMetadata(archiveId: string): Promise<ZimArchive>;
  async searchArchive(archiveId: string, query: string): Promise<SearchResult[]>;
  listArchives(): ZimArchive[];
  private async decompressCluster(data: Uint8Array, compression: CompressionType): Promise<Uint8Array>;
  private parseZimHeader(data: ArrayBuffer): ZimArchive;
  private extractBlob(data: Uint8Array, blobNumber: number): Uint8Array;
}

// Export singleton
export const zimService: ZIMService;

// === SemanticEngine ===
class SemanticEngine {
  // Methods (EXACT signatures)
  extractLinks(html: string): Link[];
  async generateMesh(articleUrl: string, options?: MeshOptions): Promise<GraphData>;
  computeLayout(graph: GraphData): GraphData;
  private parseInternalLink(href: string): string | null;
  private isInternalLink(href: string): boolean;
}

// Export singleton
export const semanticEngine: SemanticEngine;

// === SearchService ===
class SearchService {
  // Properties
  private indexes: Map<string, SearchIndex>;

  // Methods (EXACT signatures)
  async buildIndex(archiveId: string): Promise<void>;
  async search(archiveId: string, query: string): Promise<SearchResult[]>;
  private tokenize(text: string): string[];
  private rankResults(results: SearchResult[], query: string): SearchResult[];
}

// Export singleton
export const searchService: SearchService;
```

### 13.0.3 Zustand Store Names (Exact)

```typescript
// === Archive Store ===
export const useArchiveStore = create<ArchiveState>()(persist(...));

// State accessors (use these exact selectors)
const archives = useArchiveStore(state => state.archives);
const currentArchive = useArchiveStore(state => state.currentArchive);
const currentArticle = useArchiveStore(state => state.currentArticle);
const recentArticles = useArchiveStore(state => state.recentArticles);
const bookmarks = useArchiveStore(state => state.bookmarks);
const isLoading = useArchiveStore(state => state.isLoading);
const error = useArchiveStore(state => state.error);

// Action accessors
const addArchive = useArchiveStore(state => state.addArchive);
const removeArchive = useArchiveStore(state => state.removeArchive);
const setCurrentArchive = useArchiveStore(state => state.setCurrentArchive);
const setCurrentArticle = useArchiveStore(state => state.setCurrentArticle);
const addToRecent = useArchiveStore(state => state.addToRecent);
const addBookmark = useArchiveStore(state => state.addBookmark);
const removeBookmark = useArchiveStore(state => state.removeBookmark);
const setLoading = useArchiveStore(state => state.setLoading);
const setError = useArchiveStore(state => state.setError);

// === Entitlements Store ===
export const useEntitlementsStore = create<EntitlementsState>()(...);

// State accessors
const gatekeeper = useEntitlementsStore(state => state.gatekeeper);
const ready = useEntitlementsStore(state => state.ready);
const lastUpdated = useEntitlementsStore(state => state.lastUpdated);
```

### 13.0.4 Tauri Command Names (Rust Backend)

```rust
// EXACT command names - frontend must use these exact strings

#[tauri::command]
fn get_version() -> String;

#[tauri::command]
fn get_copyright() -> String;

#[tauri::command]
fn get_app_info() -> serde_json::Value;

#[tauri::command]
async fn open_archive(path: String) -> Result<Archive, String>;

#[tauri::command]
async fn close_archive(archive_id: String) -> Result<(), String>;

#[tauri::command]
async fn read_directory_entry(archive_id: String, url: String) -> Result<DirectoryEntry, String>;

#[tauri::command]
async fn read_cluster(archive_id: String, cluster_index: u32) -> Result<Vec<u8>, String>;

#[tauri::command]
async fn decompress_cluster(data: Vec<u8>, compression: String) -> Result<Vec<u8>, String>;

#[tauri::command]
fn sanitize_html(html: String) -> String;

#[tauri::command]
async fn search_titles(archive_id: String, query: String) -> Result<Vec<SearchResult>, String>;

#[tauri::command]
async fn build_search_index(archive_id: String) -> Result<(), String>;
```

**TypeScript Usage:**
```typescript
import { invoke } from '@tauri-apps/api/core';

// EXACT command strings
const version = await invoke<string>('get_version');
const copyright = await invoke<string>('get_copyright');
const appInfo = await invoke<AppInfo>('get_app_info');
const archive = await invoke<ZimArchive>('open_archive', { path });
await invoke<void>('close_archive', { archiveId });
const entry = await invoke<DirectoryEntry>('read_directory_entry', { archiveId, url });
const cluster = await invoke<number[]>('read_cluster', { archiveId, clusterIndex });
const decompressed = await invoke<number[]>('decompress_cluster', { data, compression });
const clean = await invoke<string>('sanitize_html', { html });
const results = await invoke<SearchResult[]>('search_titles', { archiveId, query });
await invoke<void>('build_search_index', { archiveId });
```

### 13.0.5 Component Names (React)

```typescript
// === Layout Components ===
export const AppShell: React.FC;
export const Header: React.FC;
export const MenuBar: React.FC;
export const Sidebar: React.FC;
export const StatusBar: React.FC;
export const MainContent: React.FC;

// === Library Components ===
export const LibraryView: React.FC<LibraryViewProps>;
export const ArchiveCard: React.FC<ArchiveCardProps>;
export const DropZone: React.FC<DropZoneProps>;

// === Reader Components ===
export const ReaderView: React.FC<ReaderViewProps>;
export const ArticleToolbar: React.FC;
export const ArticleContent: React.FC<ArticleContentProps>;

// === Search Components ===
export const SearchView: React.FC<SearchViewProps>;
export const SearchInput: React.FC;
export const SearchFilters: React.FC;

// === Mesh Components ===
export const MeshPanel: React.FC<MeshPanelProps>;
export const GraphNode: React.FC;
export const GraphEdge: React.FC;

// === View Components ===
export const BookmarksView: React.FC;
export const HistoryView: React.FC;
export const SettingsView: React.FC;

// === Dialog Components ===
export const AboutDialog: React.FC;
export const ErrorBoundary: React.FC<ErrorBoundaryProps>;

// === Utility Components ===
export const SplashScreen: React.FC;
export const LoadingSpinner: React.FC;
```

### 13.0.6 Custom Hook Names

```typescript
// === Custom Hooks (src/hooks/) ===
export function useArchive(archiveId: string): ZimArchive | null;
export function useArticle(archiveId: string, articleUrl: string): Article | null;
export function useSearch(archiveId: string, query: string): SearchResult[];
export function useEntitlements(): EntitlementsGatekeeper;
export function useTheme(): { theme: string; setTheme: (theme: string) => void };
export function useMesh(articleUrl: string, options?: MeshOptions): GraphData;
```

### 13.0.7 Constant Names (src/utils/constants.ts)

```typescript
// === Constants (EXACT names) ===
export const MAX_RECENT_ARTICLES = 50;
export const MAX_BOOKMARKS = 500;
export const MAX_ANNOTATIONS = 100;
export const MAX_MESH_NODES = 15;
export const MAX_MESH_DEPTH = 2;
export const SEARCH_DEBOUNCE_MS = 300;
export const ARTICLE_LOAD_TIMEOUT_MS = 2000;
export const SEARCH_TIMEOUT_MS = 1000;
export const CLUSTER_CACHE_SIZE = 100;
export const DEFAULT_THEME = 'minimal';

export const SUPPORTED_COMPRESSIONS = ['lzma', 'zstd', 'none'] as const;
export type CompressionType = typeof SUPPORTED_COMPRESSIONS[number];

export const ZIM_NAMESPACES = {
  ARTICLES: 'A',
  IMAGES: 'I',
  METADATA: 'M',
  TEXT: '-',
  CATEGORIES: 'U',
  EXTERNAL: 'X',
} as const;

export const ERROR_CODES = {
  INVALID_TYPE: 'INVALID_TYPE',
  CORRUPT_FILE: 'CORRUPT_FILE',
  ARCHIVE_NOT_FOUND: 'ARCHIVE_NOT_FOUND',
  ARTICLE_NOT_FOUND: 'ARTICLE_NOT_FOUND',
  DECOMPRESS_ERROR: 'DECOMPRESS_ERROR',
  ACCESS_DENIED: 'ACCESS_DENIED',
  INVALID_HEADER: 'INVALID_HEADER',
  NOT_FOUND: 'NOT_FOUND',
} as const;

export type ErrorCode = typeof ERROR_CODES[keyof typeof ERROR_CODES];
```

### 13.0.8 Utility Function Names

```typescript
// === Formatters (src/utils/formatters.ts) ===
export function formatBytes(bytes: number): string;
export function formatDate(date: Date): string;
export function formatDuration(seconds: number): string;
export function formatNumber(num: number): string;

// === Validators (src/utils/validators.ts) ===
export function validateZimPath(path: string): void;  // throws on invalid
export function isValidZimPath(path: string): boolean;
export function validateArticleUrl(url: string): void;
export function isInternalLink(url: string): boolean;
export function normalizeArticleUrl(url: string): string;

// === ID Generators (src/utils/ids.ts) ===
export function generateId(): string;  // UUID v4
export function generateArchiveId(path: string): string;
export function generateArticleId(archiveId: string, url: string): string;
```

### 13.0.9 Error Class Names

```typescript
// === Custom Errors (src/utils/errors.ts) ===
export class ZIMError extends Error {
  constructor(
    message: string,
    public code: ErrorCode,
    public originalError?: Error
  );
}

export class EntitlementsError extends Error {
  constructor(
    message: string,
    public code: string
  );
}

// Usage examples
throw new ZIMError('Invalid ZIM header', ERROR_CODES.INVALID_HEADER);
throw new ZIMError('Article not found', ERROR_CODES.ARTICLE_NOT_FOUND);
throw new EntitlementsError('Token expired', 'TOKEN_EXPIRED');
```

### 13.0.10 Context Names

```typescript
// === React Contexts ===
export const ThemeContext = React.createContext<ThemeContextValue | undefined>(undefined);
export const ThemeProvider: React.FC<{ children: ReactNode }>;
export function useTheme(): ThemeContextValue;

interface ThemeContextValue {
  theme: string;
  setTheme: (theme: string) => void;
  mode: 'light' | 'dark' | 'system';
  setMode: (mode: 'light' | 'dark' | 'system') => void;
}
```

### 13.0.11 Route Paths

```typescript
// === Router Paths (src/router.tsx) ===
export const ROUTES = {
  HOME: '/',
  LIBRARY: '/library',
  READER: '/reader/:archiveId/:articleUrl',
  SEARCH: '/search',
  BOOKMARKS: '/bookmarks',
  HISTORY: '/history',
  SETTINGS: '/settings',
} as const;
```

### 13.0.12 localStorage/Storage Keys

```typescript
// === Storage Keys (EXACT strings) ===
export const STORAGE_KEYS = {
  ARCHIVES: 'enzim-archive-store',
  ENTITLEMENTS: 'enzim-entitlements',
  THEME: 'enzim-theme',
  SETTINGS: 'enzim-settings',
  TOKEN: 'enzim-token',
} as const;
```

### 13.0.13 CSS Custom Property Names

```typescript
// === CSS Variables (for themes) ===
// Use these EXACT variable names in all themes

--color-primary: <color>;
--color-secondary: <color>;
--color-accent: <color>;
--color-text: <color>;
--color-text-secondary: <color>;
--color-background: <color>;
--color-surface: <color>;
--color-border: <color>;
--color-error: <color>;
--color-success: <color>;
--color-warning: <color>;

--font-family-sans: <font>;
--font-family-mono: <font>;
--font-size-base: <size>;
--font-size-lg: <size>;
--font-size-sm: <size>;

--spacing-xs: <size>;
--spacing-sm: <size>;
--spacing-md: <size>;
--spacing-lg: <size>;
--spacing-xl: <size>;

--border-radius: <size>;
--shadow-sm: <shadow>;
--shadow-md: <shadow>;
--shadow-lg: <shadow>;
```

### 13.0.14 Theme Names (Exact Strings)

```typescript
// === Theme IDs (EXACT strings) ===
export const THEMES = {
  MINIMAL: 'minimal',
  SYNAPTIC: 'synaptic-cartography-veil',
  BRUTALIST: 'brutalist-archive-monolith',
  PRISMATIC: 'prismatic-swiss-utility',
  SPECTRAL: 'spectral-zim-reader',
  KINETIC: 'kinetic',
  RETRO: 'retro',
  NEUMORPHISM: 'neumorphism',
  GLASSMORPHISM: 'glassmorphism',
  Y2K: 'y2k',
  CYBERPUNK: 'cyberpunk',
} as const;

export type ThemeId = typeof THEMES[keyof typeof THEMES];
```

---

## Phase 14: Code Standards & Conventions

> **Purpose:** Ensure consistency across all implementation work. All agents must follow these standards.

### 13.1 File Naming Conventions

| Type | Pattern | Example |
|------|---------|---------|
| **React Components** | PascalCase | `LibraryView.tsx`, `ArchiveCard.tsx` |
| **Services** | camelCase + Service suffix | `zimService.ts`, `semanticEngine.ts` |
| **Stores** | camelCase + Store suffix | `archiveStore.ts`, `entitlementsStore.ts` |
| **Hooks** | camelCase + use prefix | `useArchive.ts`, `useTheme.ts` |
| **Types** | PascalCase (interfaces) / camelCase (types) | `types.ts`, `ZimArchive.ts` |
| **Constants** | UPPER_SNAKE_CASE | `constants.ts` → `MAX_RECENT_ARTICLES = 50` |
| **Utilities** | camelCase | `formatDate.ts`, `validatePath.ts` |
| **Tauri Commands** | snake_case (Rust) | `open_archive`, `decompress_cluster` |

### 13.2 Directory Structure (Enforced)

```
src/
├── components/               # UI components (PascalCase.tsx)
│   ├── layout/              # AppShell, Header, Sidebar, StatusBar, MenuBar, MainContent
│   ├── library/             # LibraryView, ArchiveCard, DropZone
│   ├── reader/              # ReaderView, ArticleToolbar, ArticleContent
│   ├── search/              # SearchView, SearchInput, SearchFilters
│   ├── mesh/                # MeshPanel, GraphNode, GraphEdge
│   ├── views/               # BookmarksView, HistoryView, SettingsView
│   └── dialogs/             # AboutDialog, ErrorBoundary
├── contexts/                # React contexts (PascalCase.tsx)
│   └── ThemeContext.tsx
├── hooks/                   # Custom React hooks (camelCase.ts)
│   ├── useArchive.ts
│   ├── useSearch.ts
│   └── useEntitlements.ts
├── services/                # Business logic (camelCase.ts)
│   ├── zimService.ts
│   ├── searchService.ts
│   ├── semanticEngine.ts
│   └── index.ts
├── stores/                  # Zustand stores (camelCase.ts)
│   ├── archiveStore.ts
│   ├── entitlementsStore.ts
│   └── index.ts
├── entitlements/            # Entitlements system
│   ├── gatekeeper.ts
│   ├── providers/
│   ├── types.ts
│   └── index.ts
├── types/                   # Shared TypeScript types
│   ├── zim.ts
│   ├── graph.ts
│   └── index.ts
├── utils/                   # Utility functions
│   ├── formatters.ts
│   ├── validators.ts
│   └── constants.ts
├── App.tsx
├── main.tsx
└── router.tsx
```

### 13.3 TypeScript Standards (Strict Mode Required)

**tsconfig.json:**
```json
{
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,
    "strictNullChecks": true,
    "strictFunctionTypes": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true
  }
}
```

**Rules:**
- Use `interface` for object shapes
- Use `type` for unions/intersections
- **Always** specify return types for public methods
- **Avoid `any`** - use `unknown` if type is truly unknown
- Export interfaces used in props

**Example:**
```typescript
// ✅ GOOD
interface ZimArchive {
  id: string;
  name: string;
  title: string;
  path: string;
  size: number;
  addedAt: Date;
}

async function openArchive(path: string): Promise<ZimArchive> {
  // Implementation
}

const handleClick = (e: React.MouseEvent<HTMLButtonElement>): void => {
  // Implementation
};

// ❌ BAD
function openArchive(path) {  // ❌ No type
  // Implementation
}

const handleClick = (e: any) => {  // ❌ Using any
  // Implementation
};
```

### 13.4 React Component Pattern (Enforced)

```typescript
// Props interface
interface LibraryViewProps {
  archives: ZimArchive[];
  onArchiveSelect: (id: string) => void;
  isLoading?: boolean;
}

// Component (prefer explicit typing over React.FC)
export const LibraryView = ({
  archives,
  onArchiveSelect,
  isLoading = false
}: LibraryViewProps) => {
  // Implementation
};

// Export props for reuse
export type { LibraryViewProps };
```

### 13.5 Service Layer Pattern (Enforced)

```typescript
// zimService.ts
export class ZIMService {
  private cache: Map<string, ZimArchive> = new Map();

  /**
   * Opens a ZIM archive and parses metadata.
   *
   * @param path - Absolute path to .zim file
   * @returns Parsed archive metadata
   * @throws {ZIMError} If file is corrupted or unsupported
   */
  async openArchive(path: string): Promise<ZimArchive> {
    // Implementation
  }

  /**
   * Retrieves article content by URL.
   *
   * @param archiveId - Archive identifier
   * @param articleUrl - Article URL (e.g., "A/Main_Page")
   * @returns Article with HTML content
   * @throws {ZIMError} If article not found
   */
  async getArticle(archiveId: string, articleUrl: string): Promise<Article> {
    // Implementation
  }
}

// Export singleton
export const zimService = new ZIMService();
```

### 13.6 Error Handling Pattern (Enforced)

```typescript
// Custom error class
export class ZIMError extends Error {
  constructor(
    message: string,
    public code: string,
    public originalError?: Error
  ) {
    super(message);
    this.name = 'ZIMError';
  }
}

// Usage
throw new ZIMError('Invalid ZIM header', 'INVALID_HEADER');
throw new ZIMError('Article not found', 'NOT_FOUND');
```

### 13.7 Zustand Store Pattern (Enforced)

```typescript
import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface ArchiveState {
  // State
  archives: ZimArchive[];
  currentArchive: ZimArchive | null;
  isLoading: boolean;
  error: string | null;

  // Actions
  addArchive: (archive: ZimArchive) => void;
  setCurrentArchive: (archive: ZimArchive | null) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
}

export const useArchiveStore = create<ArchiveState>()(
  persist(
    (set, get) => ({
      archives: [],
      currentArchive: null,
      isLoading: false,
      error: null,

      addArchive: (archive) => {
        set((state) => ({
          archives: [...state.archives.filter(a => a.id !== archive.id), archive],
        }));
      },

      setCurrentArchive: (archive) => {
        set({ currentArchive: archive });
      },

      setLoading: (loading) => set({ isLoading: loading }),
      setError: (error) => set({ error }),
    }),
    {
      name: 'enzim-archive-store',
      partialize: (state) => ({
        archives: state.archives,
      }),
    }
  )
);
```

### 13.8 Constants Definition (Required)

```typescript
// src/utils/constants.ts
export const MAX_RECENT_ARTICLES = 50;
export const MAX_BOOKMARKS = 500;
export const MAX_MESH_NODES = 15;
export const SEARCH_DEBOUNCE_MS = 300;
export const ARTICLE_LOAD_TIMEOUT_MS = 2000;
export const SEARCH_TIMEOUT_MS = 1000;

export const SUPPORTED_COMPRESSIONS = ['lzma', 'zstd', 'none'] as const;
export type CompressionType = typeof SUPPORTED_COMPRESSIONS[number];
```

---

## Phase 14: Entity-Relationship Diagram & Data Models

### 14.1 Core Entity Definitions

```
┌──────────────────────────────┐
│       ZimArchive             │
├──────────────────────────────┤
│ id: string (PK)              │ UUID v4
│ name: string                 │ Filename (e.g., "wiki.zim")
│ title: string                │ From ZIM metadata
│ description: string          │ From ZIM metadata
│ language: string             │ ISO 639-1 (e.g., "en")
│ articleCount: number         │ Total articles
│ mediaCount: number           │ Images, videos, etc.
│ size: number                 │ File size in bytes
│ path: string (UNIQUE)        │ Absolute file path
│ favicon?: string             │ Base64 or data URL
│ mainPage: string             │ Main article URL
│ lastOpened?: Date            │ Last access
│ addedAt: Date                │ Import timestamp
└──────────┬───────────────────┘
           │ 1
           │
           │ N
┌──────────▼───────────────────┐
│         Article              │
├──────────────────────────────┤
│ url: string (PK)             │ ZIM URL (e.g., "A/Main_Page")
│ archiveId: string (FK)       │ → ZimArchive.id
│ title: string                │ Article title
│ content?: string             │ HTML (lazy loaded)
│ mimeType: string             │ "text/html", etc.
│ namespace: char              │ 'A', 'I', 'M', etc.
│ links: string[]              │ Extracted links
└──────────┬───────────────────┘
           │
           ├──── N ────┐
           │           │
    ┌──────▼──────┐ ┌──▼────────────┐
    │  Bookmark   │ │ HistoryEntry  │
    ├─────────────┤ ├───────────────┤
    │ id: string  │ │ id: string    │
    │ archiveId:FK│ │ archiveId: FK │
    │ articleUrl  │ │ articleUrl    │
    │ title       │ │ title         │
    │ note?: str  │ │ timestamp     │
    │ createdAt   │ │ duration?: #  │
    └─────────────┘ └───────────────┘

┌──────────────────────────────┐
│      Entitlements            │
├──────────────────────────────┤
│ userId?: string              │ User ID (if auth'd)
│ tier: string                 │ free/starter/pro
│ capabilities: Map<string,bool>
│ limits: Map<string, number> │
│ token?: string               │ Signed JWT
│ tokenExpiry?: Date           │
│ lastUpdated: Date            │
└──────────────────────────────┘

┌──────────────────────────────┐
│        GraphNode             │
├──────────────────────────────┤
│ articleUrl: string           │ Article ID
│ archiveId: string            │ Archive context
│ title: string                │ Display label
│ x: number                    │ Layout position
│ y: number                    │ Layout position
│ isFocused: boolean           │ Current focus?
│ depth: number                │ Distance from focus
└──────────┬───────────────────┘
           │ N
┌──────────▼───────────────────┐
│        GraphEdge             │
├──────────────────────────────┤
│ source: string               │ Source articleUrl
│ target: string               │ Target articleUrl
│ weight: number               │ Link strength
│ edgeType: string             │ internal/external
└──────────────────────────────┘
```

### 14.2 Storage Strategy

**Browser (PWA):**
- IndexedDB: archives, articles, bookmarks, history
- LocalStorage: Zustand persistence (settings, theme)
- SessionStorage: temporary state

**Desktop (Tauri):**
- SQLite: structured data (archives, bookmarks, history)
- File System: ZIM archives
- Tauri Store (encrypted): tokens, credentials

**SQLite Schema:**
```sql
CREATE TABLE archives (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    language TEXT,
    article_count INTEGER,
    media_count INTEGER,
    size INTEGER,
    path TEXT UNIQUE NOT NULL,
    favicon TEXT,
    main_page TEXT,
    last_opened DATETIME,
    added_at DATETIME NOT NULL
);

CREATE TABLE bookmarks (
    id TEXT PRIMARY KEY,
    archive_id TEXT NOT NULL,
    article_url TEXT NOT NULL,
    title TEXT NOT NULL,
    note TEXT,
    created_at DATETIME NOT NULL,
    FOREIGN KEY (archive_id) REFERENCES archives(id) ON DELETE CASCADE,
    UNIQUE(archive_id, article_url)
);

CREATE TABLE history (
    id TEXT PRIMARY KEY,
    archive_id TEXT NOT NULL,
    article_url TEXT NOT NULL,
    title TEXT NOT NULL,
    timestamp DATETIME NOT NULL,
    duration INTEGER,
    FOREIGN KEY (archive_id) REFERENCES archives(id) ON DELETE CASCADE
);

CREATE INDEX idx_bookmarks_archive ON bookmarks(archive_id);
CREATE INDEX idx_history_timestamp ON history(timestamp DESC);
```

### 14.3 Data Validation Rules

| Entity | Field | Validation |
|--------|-------|------------|
| ZimArchive | id | UUID v4 format |
| ZimArchive | name | Non-empty, max 255 chars |
| ZimArchive | path | Absolute path, ends with `.zim` |
| ZimArchive | size | Positive integer |
| Article | url | Non-empty, valid ZIM URL |
| Article | namespace | Single char: A-Z, a-z |
| Bookmark | note | Optional, max 1000 chars |
| GraphNode | x, y | Finite numbers |
| Entitlements | tier | Enum: free, starter, pro |

---

## Phase 15: Phase 8 Implementation Details (Blockers)

> **Critical:** These blockers must be resolved for MVP completion.

### 15.1 Blocker B1: LZMA/ZSTD Decompression (CRITICAL)

**Status:** Not implemented
**Severity:** CRITICAL
**Effort:** 2-3 days
**Blocks:** ZIM article loading

**Implementation Task Breakdown:**

| Task | Owner | Due | Status | Acceptance |
|------|-------|-----|--------|------------|
| Add lzma-rs crate to Cargo.toml | Dev | Day 1 | [ ] | `lzma-rs = "0.3"` |
| Add zstd crate to Cargo.toml | Dev | Day 1 | [ ] | `zstd = "0.11"` |
| Create `decompress_cluster` Tauri command | Dev | Day 1 | [ ] | Rust function signature defined |
| Implement LZMA decompression | Dev | Day 1 | [ ] | Decompress LZMA data, return bytes |
| Implement ZSTD decompression | Dev | Day 1 | [ ] | Decompress ZSTD data, return bytes |
| Handle uncompressed data passthrough | Dev | Day 1 | [ ] | Return original bytes |
| Add error handling for corrupt data | Dev | Day 2 | [ ] | Return descriptive errors |
| Test with real Wikipedia ZIM | Dev | Day 2 | [ ] | Successfully load articles |
| Integrate into zimService.ts | Dev | Day 2 | [ ] | Call from TypeScript |
| Add performance logging | Dev | Day 3 | [ ] | Log decompress time |

**Rust Implementation:**
```rust
// src-tauri/src/zim/decompression.rs
use lzma_rs::lzma_decompress;
use std::io::Read;

#[tauri::command]
pub async fn decompress_cluster(
    data: Vec<u8>,
    compression: String
) -> Result<Vec<u8>, String> {
    match compression.as_str() {
        "lzma" => {
            let mut decompressed = Vec::new();
            lzma_decompress(&mut &data[..], &mut decompressed)
                .map_err(|e| format!("LZMA error: {}", e))?;
            Ok(decompressed)
        },
        "zstd" => {
            zstd::decode_all(&data[..])
                .map_err(|e| format!("ZSTD error: {}", e))
        },
        "none" => Ok(data),
        _ => Err(format!("Unsupported compression: {}", compression))
    }
}
```

**TypeScript Integration:**
```typescript
// src/services/zimService.ts
import { invoke } from '@tauri-apps/api/core';

async decompressCluster(
  data: Uint8Array,
  compression: 'lzma' | 'zstd' | 'none'
): Promise<Uint8Array> {
  const result = await invoke<number[]>('decompress_cluster', {
    data: Array.from(data),
    compression
  });
  return new Uint8Array(result);
}
```

### 15.2 Blocker B2: HTML Sanitization (HIGH)

**Status:** Not implemented
**Severity:** HIGH (XSS vulnerability)
**Effort:** 1 day
**Blocks:** Safe article rendering

**Implementation Task Breakdown:**

| Task | Owner | Due | Status | Acceptance |
|------|-------|-----|--------|------------|
| Choose sanitization approach | Dev | Day 1 | [ ] | Decision: DOMPurify vs Rust ammonia |
| **Option A: Add DOMPurify (browser)** | Dev | Day 1 | [ ] | `npm install dompurify @types/dompurify` |
| Configure allowed tags/attributes | Dev | Day 1 | [ ] | Whitelist defined |
| Sanitize in ArticleContent component | Dev | Day 1 | [ ] | HTML sanitized before render |
| **Option B: Add ammonia crate (Rust)** | Dev | Day 1 | [ ] | `ammonia = "3.3"` in Cargo.toml |
| Create `sanitize_html` Tauri command | Dev | Day 1 | [ ] | Rust function defined |
| Call from TypeScript | Dev | Day 1 | [ ] | Integrated into zimService |
| Test XSS prevention | Dev | Day 1 | [ ] | `<script>` tags removed |
| Add CSP headers to WebView | Dev | Day 1 | [ ] | Content Security Policy configured |
| Test with malicious HTML samples | Dev | Day 1 | [ ] | No script execution |

**Option A: DOMPurify (Recommended for Browser)**
```typescript
// src/components/reader/ArticleContent.tsx
import DOMPurify from 'dompurify';

export const ArticleContent = ({ content }: { content: string }) => {
  const sanitized = DOMPurify.sanitize(content, {
    ALLOWED_TAGS: ['p', 'div', 'span', 'a', 'img', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
                   'table', 'thead', 'tbody', 'tr', 'td', 'th', 'ul', 'ol', 'li',
                   'strong', 'em', 'code', 'pre', 'blockquote'],
    ALLOWED_ATTR: ['href', 'src', 'alt', 'class', 'id', 'title'],
    ALLOW_DATA_ATTR: false,
    FORBID_TAGS: ['script', 'style', 'iframe', 'object', 'embed'],
    FORBID_ATTR: ['onerror', 'onload', 'onclick']
  });

  return <div dangerouslySetInnerHTML={{ __html: sanitized }} />;
};
```

**Option B: Rust ammonia (Recommended for Tauri)**
```rust
// src-tauri/src/zim/sanitize.rs
use ammonia::clean;

#[tauri::command]
pub fn sanitize_html(html: String) -> String {
    clean(&html)
}
```

### 15.3 Blocker B3: Complete zimService Implementation (HIGH)

**Status:** 40% complete
**Severity:** HIGH
**Effort:** 3-4 days
**Blocks:** Core functionality

**Missing Methods:**

| Method | Status | Effort | Acceptance |
|--------|--------|--------|------------|
| `openArchive(path)` | Stub | 2 days | Opens ZIM, returns metadata |
| `getArticle(archiveId, url)` | Stub | 1 day | Returns article with HTML |
| `searchArchive(archiveId, query)` | Not started | 1 day | Title search works |
| `getMetadata(archiveId)` | Partial | 0.5 day | Returns archive info |
| `closeArchive(archiveId)` | Not started | 0.5 day | Cleanup resources |

**Implementation:**
```typescript
// src/services/zimService.ts
export class ZIMService {
  private archives: Map<string, ZimArchive> = new Map();

  async openArchive(path: string): Promise<ZimArchive> {
    // 1. Validate path
    if (!path.endsWith('.zim')) {
      throw new ZIMError('Invalid file type', 'INVALID_TYPE');
    }

    // 2. Check if already open
    const existingArchive = Array.from(this.archives.values())
      .find(a => a.path === path);
    if (existingArchive) return existingArchive;

    // 3. Open file via Tauri
    const archiveData = await invoke<ZimArchive>('open_archive', { path });

    // 4. Parse ZIM header
    const metadata = this.parseZimHeader(archiveData);

    // 5. Cache and return
    this.archives.set(metadata.id, metadata);
    return metadata;
  }

  async getArticle(archiveId: string, articleUrl: string): Promise<Article> {
    // 1. Get archive
    const archive = this.archives.get(archiveId);
    if (!archive) {
      throw new ZIMError('Archive not found', 'ARCHIVE_NOT_FOUND');
    }

    // 2. Read directory entry
    const entry = await invoke<DirectoryEntry>('read_directory_entry', {
      archiveId,
      url: articleUrl
    });

    // 3. Get cluster data
    const clusterData = await invoke<Uint8Array>('read_cluster', {
      archiveId,
      clusterIndex: entry.clusterNumber
    });

    // 4. Decompress
    const decompressed = await this.decompressCluster(
      clusterData,
      entry.compression
    );

    // 5. Extract blob
    const blobData = this.extractBlob(decompressed, entry.blobNumber);

    // 6. Decode to string
    const content = new TextDecoder().decode(blobData);

    return {
      url: articleUrl,
      archiveId,
      title: entry.title,
      content,
      mimeType: entry.mimeType,
      namespace: entry.namespace,
      links: []
    };
  }

  async searchArchive(archiveId: string, query: string): Promise<SearchResult[]> {
    // Title index search (Phase 8.2)
    const results = await invoke<SearchResult[]>('search_titles', {
      archiveId,
      query
    });
    return results;
  }
}
```

### 15.4 Blocker B4: Error Boundaries (MEDIUM)

**Status:** Not implemented
**Severity:** MEDIUM
**Effort:** 0.5 day
**Blocks:** Graceful error handling

**Implementation:**
```typescript
// src/components/ErrorBoundary.tsx
import React, { Component, ReactNode } from 'react';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('ErrorBoundary caught:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return this.props.fallback || (
        <div className="error-fallback">
          <h1>Something went wrong</h1>
          <p>{this.state.error?.message}</p>
          <button onClick={() => window.location.reload()}>
            Reload App
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}
```

**App.tsx Integration:**
```typescript
// src/App.tsx
import { ErrorBoundary } from './components/ErrorBoundary';

export const App = () => (
  <ErrorBoundary>
    <Router>
      <AppShell />
    </Router>
  </ErrorBoundary>
);
```

---

## Phase 16: Interface & API Contracts

### 16.1 ZIMService API

```typescript
// src/services/zimService.ts
export interface ZIMService {
  /**
   * Opens a ZIM archive file.
   * @throws {ZIMError} INVALID_TYPE, CORRUPT_FILE, ACCESS_DENIED
   */
  openArchive(path: string): Promise<ZimArchive>;

  /**
   * Closes an archive and frees resources.
   */
  closeArchive(archiveId: string): Promise<void>;

  /**
   * Retrieves article content by URL.
   * @throws {ZIMError} ARCHIVE_NOT_FOUND, ARTICLE_NOT_FOUND, DECOMPRESS_ERROR
   */
  getArticle(archiveId: string, articleUrl: string): Promise<Article>;

  /**
   * Gets archive metadata.
   * @throws {ZIMError} ARCHIVE_NOT_FOUND
   */
  getMetadata(archiveId: string): Promise<ZimArchive>;

  /**
   * Searches article titles.
   * @returns Ranked search results (max 100)
   */
  searchArchive(archiveId: string, query: string): Promise<SearchResult[]>;

  /**
   * Lists all open archives.
   */
  listArchives(): ZimArchive[];
}
```

### 16.2 SemanticEngine API

```typescript
// src/services/semanticEngine.ts
export interface SemanticEngine {
  /**
   * Extracts links from article HTML.
   * @param html - Article HTML content
   * @returns Array of internal links
   */
  extractLinks(html: string): Link[];

  /**
   * Generates semantic mesh graph.
   * @param articleUrl - Focus article
   * @param options - Mesh options (depth, maxNodes)
   * @returns Graph data (nodes + edges)
   */
  generateMesh(
    articleUrl: string,
    options?: MeshOptions
  ): Promise<GraphData>;

  /**
   * Computes force-directed layout.
   * @param graph - Graph data
   * @returns Updated graph with positions
   */
  computeLayout(graph: GraphData): GraphData;
}

export interface MeshOptions {
  maxDepth?: number;      // Default: 1
  maxNodes?: number;      // Default: 15
  includeExternal?: boolean;  // Default: false
}

export interface Link {
  url: string;
  title: string;
  isInternal: boolean;
}

export interface GraphData {
  nodes: GraphNode[];
  edges: GraphEdge[];
}
```

### 16.3 Tauri Backend Commands

```rust
// src-tauri/src/commands/mod.rs

/// Opens a ZIM archive and returns metadata
#[tauri::command]
pub async fn open_archive(path: String) -> Result<Archive, String>;

/// Reads a directory entry from the archive
#[tauri::command]
pub async fn read_directory_entry(
    archive_id: String,
    url: String
) -> Result<DirectoryEntry, String>;

/// Reads and returns cluster data
#[tauri::command]
pub async fn read_cluster(
    archive_id: String,
    cluster_index: u32
) -> Result<Vec<u8>, String>;

/// Decompresses cluster data
#[tauri::command]
pub async fn decompress_cluster(
    data: Vec<u8>,
    compression: String
) -> Result<Vec<u8>, String>;

/// Sanitizes HTML content
#[tauri::command]
pub fn sanitize_html(html: String) -> String;

/// Searches archive titles
#[tauri::command]
pub async fn search_titles(
    archive_id: String,
    query: String
) -> Result<Vec<SearchResult>, String>;

/// Gets app version
#[tauri::command]
pub fn get_version() -> String;

/// Gets copyright notice
#[tauri::command]
pub fn get_copyright() -> String;

/// Gets app metadata
#[tauri::command]
pub fn get_app_info() -> serde_json::Value;
```

### 16.4 Zustand Store Interfaces

```typescript
// src/stores/archiveStore.ts
export interface ArchiveState {
  // State
  archives: ZimArchive[];
  currentArchive: ZimArchive | null;
  currentArticle: Article | null;
  recentArticles: Article[];  // Max 50
  bookmarks: Article[];
  isLoading: boolean;
  error: string | null;

  // Actions
  addArchive: (archive: ZimArchive) => void;
  removeArchive: (id: string) => void;
  setCurrentArchive: (archive: ZimArchive | null) => void;
  setCurrentArticle: (article: Article | null) => void;
  addToRecent: (article: Article) => void;
  addBookmark: (article: Article) => void;
  removeBookmark: (url: string) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
}

// src/entitlements/store.ts
export interface EntitlementsState {
  gatekeeper: EntitlementsGatekeeper;
  ready: boolean;
  lastUpdated: string;
}
```

---

## Phase 17: Testing Specifications

### 17.1 Unit Test Requirements (60% Coverage Target)

**Service Layer Tests:**

```typescript
// tests/services/zimService.test.ts
describe('ZIMService', () => {
  describe('openArchive', () => {
    it('should open valid ZIM archive', async () => {
      const archive = await zimService.openArchive('test.zim');
      expect(archive.title).toBe('Test Wikipedia');
      expect(archive.articleCount).toBeGreaterThan(0);
    });

    it('should reject non-.zim files', async () => {
      await expect(zimService.openArchive('test.txt'))
        .rejects.toThrow('Invalid file type');
    });

    it('should handle corrupted archives', async () => {
      await expect(zimService.openArchive('corrupt.zim'))
        .rejects.toThrow(ZIMError);
    });

    it('should cache opened archives', async () => {
      const archive1 = await zimService.openArchive('test.zim');
      const archive2 = await zimService.openArchive('test.zim');
      expect(archive1.id).toBe(archive2.id);
    });
  });

  describe('getArticle', () => {
    it('should retrieve article by URL', async () => {
      const archive = await zimService.openArchive('test.zim');
      const article = await zimService.getArticle(archive.id, 'A/Main_Page');
      expect(article.title).toBe('Main Page');
      expect(article.content).toContain('<html');
    });

    it('should throw on invalid article URL', async () => {
      const archive = await zimService.openArchive('test.zim');
      await expect(zimService.getArticle(archive.id, 'A/Invalid_Article'))
        .rejects.toThrow('ARTICLE_NOT_FOUND');
    });

    it('should decompress LZMA content', async () => {
      const archive = await zimService.openArchive('test.zim');
      const article = await zimService.getArticle(archive.id, 'A/Compressed_Article');
      expect(article.content).toBeDefined();
      expect(article.content?.length).toBeGreaterThan(0);
    });
  });

  describe('searchArchive', () => {
    it('should find articles by title', async () => {
      const archive = await zimService.openArchive('test.zim');
      const results = await zimService.searchArchive(archive.id, 'Main');
      expect(results.length).toBeGreaterThan(0);
      expect(results[0].title).toContain('Main');
    });

    it('should return empty array for no matches', async () => {
      const archive = await zimService.openArchive('test.zim');
      const results = await zimService.searchArchive(archive.id, 'XYZ123NonExistent');
      expect(results).toEqual([]);
    });
  });
});

// tests/services/semanticEngine.test.ts
describe('SemanticEngine', () => {
  describe('extractLinks', () => {
    it('should extract internal links', () => {
      const html = '<a href="./Article">Link</a>';
      const links = semanticEngine.extractLinks(html);
      expect(links).toHaveLength(1);
      expect(links[0].url).toBe('Article');
      expect(links[0].isInternal).toBe(true);
    });

    it('should ignore external links', () => {
      const html = '<a href="https://example.com">Link</a>';
      const links = semanticEngine.extractLinks(html);
      expect(links).toHaveLength(0);
    });

    it('should handle malformed HTML', () => {
      const html = '<a href="./Article">Unclosed';
      const links = semanticEngine.extractLinks(html);
      expect(links).toHaveLength(1);
    });
  });

  describe('generateMesh', () => {
    it('should generate graph with max nodes', async () => {
      const graph = await semanticEngine.generateMesh('A/Main_Page', {
        maxNodes: 15
      });
      expect(graph.nodes.length).toBeLessThanOrEqual(15);
      expect(graph.edges.length).toBeGreaterThan(0);
    });

    it('should include focus node', async () => {
      const graph = await semanticEngine.generateMesh('A/Main_Page');
      const focusNode = graph.nodes.find(n => n.isFocused);
      expect(focusNode).toBeDefined();
      expect(focusNode?.articleUrl).toBe('A/Main_Page');
    });
  });
});
```

**Store Tests:**

```typescript
// tests/stores/archiveStore.test.ts
describe('ArchiveStore', () => {
  beforeEach(() => {
    useArchiveStore.getState().archives = [];
  });

  it('should add archive', () => {
    const archive: ZimArchive = {
      id: '1',
      name: 'test.zim',
      title: 'Test',
      path: '/test.zim',
      size: 1000,
      addedAt: new Date(),
      articleCount: 100,
      mediaCount: 50,
      language: 'en',
      description: 'Test archive',
      mainPage: 'A/Main_Page'
    };
    useArchiveStore.getState().addArchive(archive);
    expect(useArchiveStore.getState().archives).toHaveLength(1);
  });

  it('should limit recent articles to 50', () => {
    for (let i = 0; i < 60; i++) {
      useArchiveStore.getState().addToRecent({
        url: `A/Article_${i}`,
        archiveId: '1',
        title: `Article ${i}`,
        namespace: 'A',
        mimeType: 'text/html',
        links: []
      });
    }
    expect(useArchiveStore.getState().recentArticles).toHaveLength(50);
  });

  it('should prevent duplicate bookmarks', () => {
    const article: Article = {
      url: 'A/Test',
      archiveId: '1',
      title: 'Test',
      namespace: 'A',
      mimeType: 'text/html',
      links: []
    };
    useArchiveStore.getState().addBookmark(article);
    useArchiveStore.getState().addBookmark(article);
    expect(useArchiveStore.getState().bookmarks).toHaveLength(1);
  });
});
```

### 17.2 Component Test Requirements

```typescript
// tests/components/library/LibraryView.test.tsx
import { render, screen } from '@testing-library/react';
import { LibraryView } from '@/components/library/LibraryView';

describe('LibraryView', () => {
  it('should render archive cards', () => {
    const archives = [
      { id: '1', title: 'Wikipedia', /* ... */ }
    ];
    render(<LibraryView archives={archives} onArchiveSelect={vi.fn()} />);
    expect(screen.getByText('Wikipedia')).toBeInTheDocument();
  });

  it('should handle empty state', () => {
    render(<LibraryView archives={[]} onArchiveSelect={vi.fn()} />);
    expect(screen.getByText(/no archives/i)).toBeInTheDocument();
  });
});
```

### 17.3 Test Coverage Targets

| Layer | Target Coverage | Priority |
|-------|-----------------|----------|
| Services | 80% | HIGH |
| Stores | 80% | HIGH |
| Components | 50% | MEDIUM |
| Utils | 90% | HIGH |
| Overall | 60% | MVP Requirement |

---

## Phase 18: Security Requirements

### 18.1 XSS Prevention (CRITICAL)

**Requirement:** All article HTML must be sanitized before rendering.

**Implementation Checklist:**

| Task | Status | Acceptance |
|------|--------|------------|
| Add DOMPurify or ammonia | [ ] | Dependency installed |
| Configure whitelist (tags, attributes) | [ ] | Safe tags only |
| Sanitize in ArticleContent component | [ ] | No script execution |
| Test with XSS payload | [ ] | `<script>alert(1)</script>` blocked |
| Add CSP headers to WebView | [ ] | `script-src 'none'` |

**Test Cases:**
```typescript
describe('XSS Prevention', () => {
  it('should block script tags', () => {
    const malicious = '<script>alert(1)</script><p>Safe content</p>';
    const sanitized = DOMPurify.sanitize(malicious);
    expect(sanitized).not.toContain('<script>');
    expect(sanitized).toContain('<p>Safe content</p>');
  });

  it('should block event handlers', () => {
    const malicious = '<img src=x onerror="alert(1)">';
    const sanitized = DOMPurify.sanitize(malicious);
    expect(sanitized).not.toContain('onerror');
  });
});
```

### 18.2 Path Traversal Prevention

**Requirement:** Validate all file paths from user input.

```typescript
// src/utils/validators.ts
import { normalize, isAbsolute } from 'path';

export function validateZimPath(path: string): void {
  const normalized = normalize(path);

  if (!isAbsolute(normalized)) {
    throw new Error('Path must be absolute');
  }

  if (normalized.includes('..')) {
    throw new Error('Path traversal detected');
  }

  if (!normalized.endsWith('.zim')) {
    throw new Error('Invalid file type');
  }
}
```

### 18.3 Secure Token Storage

**Requirement:** Store entitlement tokens securely.

**Desktop (Tauri):**
```typescript
// Use Tauri Store (encrypted)
import { Store } from '@tauri-apps/plugin-store';

const store = new Store('entitlements.dat');
await store.set('token', tokenString);
await store.save();  // Encrypted on disk
```

**Browser:**
```typescript
// Use sessionStorage or secure cookie (not localStorage)
// Tokens expire on session end
```

---

## Architecture Artifact Summary

### PlantUML Diagrams (9 files)

| File | Status |
|------|--------|
| `01_system_context.puml` | ✅ |
| `02_container_diagram.puml` | ✅ |
| `03_component_diagram.puml` | ✅ (includes auth/billing) |
| `04_class_diagram.puml` | ✅ (includes User/Subscription) |
| `05_seq_content_load.puml` | ✅ |
| `06_seq_search.puml` | ✅ |
| `07_seq_semantic_mesh.puml` | ✅ |
| `08_ui_theme_system.puml` | ✅ |
| `09_ui_layout.puml` | ✅ |

### Mermaid Diagrams (6 files)

| File | Status |
|------|--------|
| `architecture_overview.mmd` | ✅ |
| `data_flow.mmd` | ✅ |
| `erd_models.mmd` | ✅ |
| `state_diagram.mmd` | ✅ |
| `deployment_diagram.mmd` | ✅ |
| `ui_component_hierarchy.mmd` | ✅ |

### Documentation (4 files)

| File | Status |
|------|--------|
| `DOCS/ARCHITECTURE/ARCHITECTURE.md` | ✅ |
| `DOCS/ARCHITECTURE/UI_COMPONENTS.md` | ✅ |
| `README.md` | ✅ |
| `CHECKLIST.md` | ✅ |

---

## UI-Samples Reference (Look & Feel Source)

| Prototype File | Theme | Key Visual Elements |
|----------------|-------|---------------------|
| `synaptic_cartography_veil_1.html` | Synaptic | Dark, neon cyan/magenta, glassmorphic, canvas particles |
| `brutalist_archive_monolith_0.html` | Brutalist | Light concrete, stark black, hard shadows, grid |
| `prismatic_swiss_utility_0.html` | Prismatic | White/light, cyan/magenta/gold, clean Swiss design |
| `AnZimmerman-.html` | Spectral | Dark, vibrant highlights, mouse refraction effect |

---

## Session Log

| Session | Date | Focus | Outcome |
|---------|------|-------|---------|
| 1 | 2026-01-16 | Architecture foundation, UI analysis, auth/billing placeholders | 15 diagrams, 4 docs created |
| 2 | 2026-01-17 | Auth/billing UI flows, wireframes, state bindings | 7 UI artifacts, comprehensive checklist |

---

## UI Artifacts Created (Session 2)

| File | Type | Description |
|------|------|-------------|
| `UI/auth_flow.puml` | PlantUML | Email/OAuth login, signup, session restore, logout sequences |
| `UI/billing_flow.puml` | PlantUML | Subscription CRUD, Stripe webhooks, feature sync |
| `UI/feature_gate_flow.puml` | PlantUML | Feature checks, usage limits, gating patterns |
| `UI/auth_state_machine.mmd` | Mermaid | Auth state transitions with variable annotations |
| `UI/ui_wireframes.md` | Markdown | Annotated wireframes with backend variable bindings |
| `UI/state_bindings.md` | Markdown | Zustand stores, hooks, type definitions |
| `UI/api_contracts.md` | Markdown | AuthService, BillingService, FeatureGateService APIs |

---

## Global Rules Compliance Summary

| Requirement | Phase | Status |
|-------------|-------|--------|
| Tauri 2 for Desktop/Mobile | Phase 5 | [X] Done |
| Epoch-based build number (`floor(epoch/60) % 100000`) | Phase 5.2 | [X] Done |
| Executable naming with version | Phase 5.2 | [X] Done |
| Copyright splash on launch | Phase 5.3 | [X] Done |
| `File → About` with version | Phase 5.3 | [X] Done |
| Version in status bar | Phase 5.3 | [X] Done |
| Menu: File \| Edit \| View \| Help | Phase 5.3 | [X] Done |
| GitHub Actions CI/CD | Phase 5.4 | [X] Done |
| Light/Dark/System toggle | Phase 6.2 | [X] Done |
| 11 themes (4 UI-Samples + 7 global) | Phase 6.3-6.4 | [X] Done |
| PWA installable | Phase 11.2 | [ ] Planned |

---

## Phase 16: Dual-Purpose Browser/Desktop Audit (Session — 2026-02-07)

> **Purpose:** Ensure every interactive control works in both Tauri (desktop) and browser (PWA) environments.
> Audit performed 2026-02-07. 28 of ~62 controls were broken/dead and have been fixed.

### 16.1 Platform Utilities

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Create `src/utils/platform.ts` — `isTauri()` helper | Dev | Session | ✅ | Detects Tauri vs browser |
| Add `openExternalUrl()` to `platform.ts` | Dev | Session | ✅ | Uses shell plugin (Tauri) or anchor click (browser) |
| Update `zimService.ts` — dual-mode `openFileDialog` | Dev | Session | ✅ | Hidden `<input type="file">` fallback in browser |
| Update `zimService.ts` — dual-mode `loadArchive` | Dev | Session | ✅ | Accepts `File` (browser) or path `string` (Tauri) |

### 16.2 P0 Critical — Core User Flow

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Fix DropZone **Download Archives** button | Dev | Session | ✅ | Uses `openExternalUrl` (not `window.open`) |
| Wire `AppShell` → `MenuBar` `onOpenFile` callback | Dev | Session | ✅ | File → Open ZIM Archive works |
| Wire `AppShell` → `MenuBar` `onDownloadArchives` callback | Dev | Session | ✅ | File → Download Archives opens Kiwix |
| Wire `AppShell` → `MenuBar` `onSettings` callback | Dev | Session | ✅ | File → Settings navigates to /settings |
| Wire Header search bar (onChange, onKeyDown, onFocus) | Dev | Session | ✅ | Typing navigates to /search |
| Wire Sidebar "Drop ZIM here" zone (drag/drop + click) | Dev | Session | ✅ | Loads archives, shows archive list |
| Wire MainContent "Open ZIM Archive" CTA button | Dev | Session | ✅ | Opens dual-mode file dialog |
| Wire ArchiveCard **Trash** button (grid + list) | Dev | Session | ✅ | Calls `removeArchive(id)` |

### 16.3 P1 Important — Expected Functionality

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Wire ArchiveCard ⋮ context menu (list mode) | Dev | Session | ✅ | Dropdown: Open, Details, Remove |
| Wire ArchiveCard grid hover buttons (Open, Info, Delete) | Dev | Session | ✅ | All three buttons functional |
| Wire Reader **Back/Forward** navigation | Dev | Session | ✅ | History stack with ref-based tracking |
| Wire Reader **Share** button | Dev | Session | ✅ | `navigator.share` / clipboard fallback |
| Wire Reader **Fullscreen** button | Dev | Session | ✅ | Uses Fullscreen API |
| Wire MenuBar File → **Exit** | Dev | Session | ✅ | `window.close()` |
| Wire MenuBar View → **Zoom In/Out/Actual Size** | Dev | Session | ✅ | Adjusts `document.documentElement.style.fontSize` |
| Wire MenuBar View → **Full Screen** | Dev | Session | ✅ | Uses Fullscreen API |
| Wire MenuBar Edit → **Undo/Redo/Cut/Copy/Paste** | Dev | Session | ✅ | `document.execCommand` / Clipboard API |
| Wire HistoryView **Clear History** button | Dev | Session | ✅ | Resets `recentArticles` in store |
| Wire MeshPanel **node click** → navigate to article | Dev | Session | ✅ | Sets `currentArticle` via store |

### 16.4 P2 Nice-to-Have — Stubs & Polish

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Wire MenuBar Help → **Documentation** | Dev | Session | ✅ | Opens GitHub README via `openExternalUrl` |
| Wire MenuBar Help → **Getting Started** | Dev | Session | ✅ | Opens Kiwix library |
| Wire MenuBar Help → **Report Issue** | Dev | Session | ✅ | Opens GitHub Issues |
| Wire MenuBar Help → **GitHub Repository** | Dev | Session | ✅ | Opens GitHub repo |
| Wire Header **User Avatar** → navigate /settings | Dev | Session | ✅ | Placeholder; navigates to settings |
| Wire SettingsView **Language** select | Dev | Session | ✅ | Controlled `<select>` with state |
| Wire SettingsView **TTS Speed** slider | Dev | Session | ✅ | Controlled with displayed value |
| Implement `ArchiveDetails` modal | Dev | Future | [ ] | Full metadata display |
| Implement `TTSControls` component | Dev | Future | [ ] | Play/pause, speed, voice selection |
| Implement `AnnotationToolbar` component | Dev | Future | [ ] | Highlight, note actions |
| Implement `SearchFilters` component | Dev | Future | [ ] | Archive, date filters |
| Implement force-directed graph layout | Dev | Future | [ ] | Visual mesh graph |

### 16.5 Remaining Known Gaps (Post-Audit)

| Gap | Priority | Notes |
|-----|----------|-------|
| MenuBar Edit → Find (no find-in-page yet) | Low | Browser has native Ctrl+F |
| Settings TTS Voice select (only default) | Low | Needs `speechSynthesis.getVoices()` |
| Full-text search indexing (Web Worker) | Medium | Phase 8.2 |
| LZMA/ZSTD decompression | Critical | Phase 15.1 |
| Android/iOS builds | Low | Phase 5.5 |

---

## Phase 17: Fix ZIM Archive Loading in Tauri (Blocker)

> **Purpose:** The Tauri file dialog opens and returns a path, but `loadArchive()` silently fails — the archive never appears in the library. Three root causes identified.
>
> **Delegatable:** Yes — a smaller agent can implement these fixes using the exact file/line references below.

### 17.1 Bug: `Uint8Array.buffer` offset (CRITICAL)

**File:** `src/services/zimService.ts` line 167
**Problem:** `readFile` from `@tauri-apps/plugin-fs` v2 returns a `Uint8Array`. Using `.buffer` directly gives the **underlying shared** `ArrayBuffer` which may have a non-zero `byteOffset`, causing all `DataView` reads to be misaligned and the magic number check to fail.

| Task | Status | Acceptance Criteria |
|------|--------|---------------------|
| Replace `buffer = fileData.buffer as ArrayBuffer;` with `buffer = fileData.buffer.slice(fileData.byteOffset, fileData.byteOffset + fileData.byteLength);` | [ ] | `loadArchive` correctly reads the ZIM header magic `0x4D495A5A` |
| Add `console.log('[ZIM] Tauri: read', fileData.byteLength, 'bytes, offset:', fileData.byteOffset);` after `readFile` call for debugging | [ ] | Logs appear in Tauri devtools console |

### 17.2 Bug: FS Scope Too Narrow (CRITICAL)

**File:** `src-tauri/capabilities/default.json`
**Problem:** The `fs:scope` only permits `$HOME/**`, `$APPDATA/**`, `$DOWNLOAD/**`, `$DOCUMENT/**`. If the user picks a file from `/mnt/`, `/media/`, `/tmp/`, or any path outside `$HOME`, the read is **silently denied** by Tauri's security layer.

| Task | Status | Acceptance Criteria |
|------|--------|---------------------|
| Change the `fs:scope` allow list to `[{"path": "**"}]` to permit reading any file the user explicitly selected via dialog | [ ] | Files from any directory load successfully |
| Alternative (more secure): add `"fs:allow-read-file-from-dialog"` permission if supported by Tauri 2 | [ ] | Only dialog-selected files are readable |

### 17.3 Bug: Silent Error Swallowing (MEDIUM)

**File:** `src/services/zimService.ts` lines 205–208
**Problem:** `loadArchive` catches all errors and returns `null` with only `console.error`. The user sees nothing — the UI just doesn't respond.

| Task | Status | Acceptance Criteria |
|------|--------|---------------------|
| Add `console.log` at each major step: before `stat`, after `readFile`, after header parse, after entry parse | [ ] | Debugging visible in Tauri devtools |
| In the callers (`AppShell.tsx:37-60`, `DropZone.tsx:33-57`, `Sidebar.tsx:42-63`, `MainContent.tsx:16-40`, `LibraryView.tsx:25-49`): show an `alert()` or status message on failure instead of silent catch | [ ] | User sees "Failed to load archive: ..." message |

### 17.4 Enhancement: `stat().size` Field Validation

**File:** `src/services/zimService.ts` line 165
**Problem:** Tauri 2's `stat()` returns an object — verify that `.size` exists and is a number. If the field name changed between Tauri versions, this would silently produce `NaN`.

| Task | Status | Acceptance Criteria |
|------|--------|---------------------|
| Add `console.log('[ZIM] stat result:', JSON.stringify(fileInfo));` before using `fileInfo.size` | [ ] | Confirms field name in devtools |
| Add guard: `if (typeof fileInfo.size !== 'number') throw new Error('stat did not return size');` | [ ] | Clear error if API shape changed |

### 17.5 Verification Steps

1. Kill existing dev server: `fuser -k 1420/tcp`
2. Launch Tauri: `PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 npx tauri dev`
3. Open DevTools (Ctrl+Shift+I in the Tauri window)
4. Click **File → Open ZIM Archive** or the **Browse Files** button
5. Select a `.zim` file
6. Check console for `[ZIM]` log messages
7. Confirm the archive appears in the Library sidebar and grid

---

*Last updated: 2026-02-07T20:17:00-05:00*
