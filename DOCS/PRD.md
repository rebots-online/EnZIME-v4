# EnZIME Reader — Product Requirements (v5.1)

**Current direction, 2026-11-13:** The product contract in §14 supersedes conflicting draft wording below. The complete Stitch prototype is being assembled in project `17076380137565911053`; it is not frozen until its screen/state manifest and exports are complete. The earlier draft remains as decision provenance, not permission to omit existing functionality. Implementation follows the completed prototype and its architecture, without discretionary redesign.

**Date:** 2026-11-10
**Status:** Draft for operator review. Abstracted from `DOCS/REBOOT-PAK/PRD.md` v4.0.0, the operator's
2026-11-10 decisions (`DOCS/TODO-LEDGER.md`, `DOCS/RELEASE-STANDARD.md`) and what build 1.20.05778 was observed
to do. Items marked **OPEN** are not decided; nothing in this document decides them.
**Design source:** Google Stitch project named in §13; screens there are the visual vocabulary for architecture.

## 0. Lineage — which attempt this is

Counted from the GitHub repository lists on 2026-11-10, by repository name (the repositories were not opened).

| Line | Repositories before this document | This PRD is attempt |
|---|---|---|
| EnZIM / EnZIME, in `rebots-online` | 7: EnZIM, EnZIMe-Archive-4feb2026, EnZIM1, EnZIM-Stitch, EnZIMErgent-26apr2026-14h, EnZIME, EnZIME-v4 | 8th |
| EnZIM / EnZIME, all organizations | 11: the 7 above plus, in `Robin-s-AI-World`, EnZIMErgent, emergent-EnZIME-Mobile, EnZIME.archive-2026-05-14, EnZIME | 12th |
| Every ZIM-reader line of the operator's own in `rebots-online` | 18: the 7 above plus robinpedia-android, robinpedia-flutter, wikiitake, wikiitake-kotlin, kiwix-robinpedia, robinpedia, AndZimmerMan, AnZimmerman, ZIMnotebolt, BoltZIMnote-8mar2026, ZIMnote | 19th |
| Every ZIM-reader line, all organizations | 23: the 18 above, the 4 in `Robin-s-AI-World`, and Knowledge-Pysanky | 24th |

First repository in the line: November 2024. Inside EnZIME-v4 alone this is the fifth requirements document
("Reboot IV" produced v4.0.0). Eight further repositories in `rebots-online` are Kiwix/openZIM-named
(kiwix-desktop, kiwix-build, kiwix-tools, android-kiwix, kiwix-android-custom, libzim, kiwix-js-pwa,
kiwix-zim-updater) and are not counted here. In two years, none of these twenty-three has reached a public release.

The cost of that history is the first requirement of this one: **work that runs is kept and corrected in place;
it is not discarded and begun again.** Build 1.20.05778 of EnZIME-v4 starts, lists the live catalog, plans and
downloads a storage-fit subset of a real pack. That is the base this document builds on.

## 1. Product

EnZIME Reader is an offline-first reader and annotator for ZIM knowledge packs (Wikipedia and other Kiwix/openZIM
content) on Android, Linux and Windows. A person chooses what knowledge to keep, sees exactly what it costs in
storage before anything downloads, reads and annotates it with no network, and explores it spatially to find
connections they did not know to look for.

## 2. Promise

> The knowledge you acquire and the work you add to it remain usable when the network, model, vendor, source
> location or prior application state is absent.

## 3. The two unique selling points

Both must be visible in the product within the first minute of use and in every store screenshot set.

### 3.1 Dynamic Downloader (DynDon)

Download only what fits. Given a pack and a storage budget the user sets, the app plans a subset of the pack that
fits that budget, shows the plan before any transfer, downloads only that subset by byte range, and reads it offline.

- The user sets a budget in ordinary units (any positive number of MB or GB is accepted).
- The plan states: full pack size, bytes to download, bytes kept, temporary space, free space before and after,
  number of articles and resources that will be readable, source, and what is and is not verified.
- A plan appears, or visible progress with a count appears, within 3 s (5 s on Android); planning never shows an
  indefinite spinner.
- Download is resumable, cancellable, and survives quitting the app.
- A partial pack is always labelled partial; it is never presented as the full archive.
- The plan can be revised later (larger or smaller budget) without discarding what is already kept.

### 3.2 Knowledge Pysanky — 6-DOF semantic navigator

The second main way to view pack content. The pack's articles appear as a constellation of concept nodes in
three-dimensional space, placed so that related concepts are near each other; the user flies through it and
discovers neighbours, bridges and clusters they would not have searched for.

- **Two viewer modes, one toggle.** Installed content is viewed either as the traditional article reader or as the
  navigator. A toggle switches between them. The navigator is shown together with the reader, not hidden behind
  it: alongside the reader, or as an inset windowlet that can be flown in place and swapped with the main pane
  (reference: the natal-chart inset in Natally-ai-studio's atelier view). **OPEN:** side-by-side or inset as the
  default, or both; inset behaviour on narrow and Android screens; whether the toggle is remembered per pack.
- **Flight.** Movement is first-person spacecraft flight through 3-space with six degrees of freedom. The scroll
  wheel is the forward thruster. **OPEN:** inputs for pitch, yaw, roll and lateral thrust; whether velocity
  drifts or damps; touch controls on Android.
- **Nodes and edges.** A node is an article or concept; colour encodes its kind; an edge is a relation with a
  confidence. Selecting a node shows its title and summary; "dive" opens that article in the reader mode and the
  navigator keeps its place.
- **Placement reflects the content.** Node positions are derived from the downloaded ZIM content's semantics, not
  from a fixed demo graph. **OPEN:** the method (vector embedding computed gradually in the background,
  hypergraph of links/categories, or another), where it is computed and stored, and how a partial pack is shown.
- **Journey map.** The path the user has flown and the articles opened along it can be reviewed and revisited.
- **Pysanky launcher (easter egg).** Holding E shows the targeting reticle and charges a launch; releasing throws
  a decorated egg at the aimed node, which shatters into shards of that node's colour.
  - The reticle and the overcharge glitch are reproduced exactly as in
    `VENDOR/knowledge-pysanky/knowledgepysanky-v102.tsx` (lines 887-906): dashed aim line, ring that grows and
    shifts green to red, centre dot, four ticks, eight-segment battery; quiver past the charge limit, then the
    glitch (random-hue ring, flicker, noise bars, alternating "⚠ OVERCHARGE" / "T4RG3T_L0$T").
  - Launch energy rises with hold time, with no drop anywhere along the hold. **OPEN:** the exact curve and
    whether a tap stays a homing shot.
  - About one egg in ten is a Fabergé egg. **OPEN:** its appearance (not present in the v102 artifact).
- The navigator works offline and never blocks reading: if placement data is not ready, the reader mode is fully
  usable and the navigator says what it is still preparing and how far along it is.

## 4. Platforms

Android (current Play target API, four ABIs), Linux desktop (AppImage, deb, rpm), Windows 10/11 x64 (exe, NSIS,
MSI, MSIX). One domain and IPC contract across all. macOS and iOS are not in this release.

## 5. Personas

| Persona | Primary job |
|---|---|
| Offline reader | Find and read reference knowledge without connectivity |
| Storage-constrained user | Fit a deliberate reserve of knowledge into the space they have |
| Explorer | Wander a body of knowledge and surface connections they would not have searched for |
| Knowledge worker | Annotate, connect, retrieve and cite across packs |
| Preparedness user | Know that reading, notes and purchases work with no network |
| Accessibility user | Complete every core flow with assistive technology |

## 6. Principles

1. Reading, annotation, local search, export and purchase verification need no account and no internet.
2. User work survives pack removal and model replacement.
3. Nothing unverified becomes active; the last good version stays active until its replacement verifies.
4. Every capability named in the store listing or the interface exists and can succeed. A feature is complete or
   it is absent; there are no placeholders and no controls that cannot work.
5. Commerce never blocks access to, or export of, what the user created.
6. Every screen shows the version and build in the bottom-right corner.
7. The interface follows the design source and the architecture entity table; nobody invents a second version.

## 7. Surfaces

### 7.1 Setup (first launch)
Capability check, choice of storage location and default budget, a first pack suggestion with its storage-fit
plan, and arrival at a readable article without documentation.

### 7.2 Home
Continue reading (or a clear empty state), recent packs, active downloads, system readiness. Nothing refers to
features this build does not contain.

### 7.3 Library
Installed packs (kept size, article count, partial or full, last opened), live catalog search with language
filter and an offline-cache state, download queue with pause, resume and cancel, pack removal.

### 7.4 Download plan (DynDon)
As §3.1. Destination, budget, the fit summary with a size bar, the detailed plan, and Start download.

### 7.5 Reader
Sanitised article rendering with its images, breadcrumb and back/forward, internal links, title search in the
pack, annotation tools, reading position remembered, the viewer-mode toggle and the navigator inset. A page that
cannot be shown says why.

### 7.6 Navigator (Knowledge Pysanky)
As §3.2: flight view, node detail, dive to article, journey map, legend, preparation progress, launcher.

### 7.7 Annotation workspace
Highlight, typed comment, freehand ink, voice recording, photo, short video clip; each saved at its place in the
article and replayable offline; unsaved work is never lost on navigation.

### 7.8 Notes
All annotations across packs, notebooks, tags, filters, jump to source, signed sidecar export and import.

### 7.9 Settings
Storage and network policy, accessibility, purchase status and restore, backup and restore, privacy and
analytics consent, diagnostics export, about and licences.

### 7.10 Activation (paywall) — four steps

One flow, the same on every platform. Component names are fixed here and used unchanged in the design source,
the architecture and the code. Every screen keeps the app frame: `TopBar` (app mark "Z EnZIME Reader", a Close
control) and the version label bottom-right.

| Step | Component | Shows | Primary action |
|---|---|---|---|
| 1 | `ActivationStepValue` | Headline "Keep everything. Offline. Forever."; `CapabilityList` of the four paid capabilities, each with an icon and one line: unlimited packs; photo, voice, video and ink notes; encrypted backup and restore; signed note export | "Continue" |
| 2 | `ActivationStepOffer` | `OfferCard`: "One-time activation", the price from the store (`{{ product.price }}`), and three `PromiseRow`s: "Pay once — no subscription, no renewal"; "Works with no network after activation"; "Your notes stay yours whether or not you activate" | "Activate for {{ product.price }}" |
| 3 | `ActivationStepPay` | `PaymentMethodPanel`: Google Play on Android, Google Pay on Linux and Windows; order line "EnZIME Reader — Lifetime"; total `{{ product.price }}`; `RestoreLink` "Already activated? Restore" | The store's own pay button |
| 4 | `ActivationStepDone` | `ActivationReceipt`: "Activated on this device", the date, "Stored on this device — no network needed again", and the restore key with a Copy control | "Start reading" |

`StepIndicator` (four dots, current one filled) sits under the `TopBar` on all four steps. A failed or cancelled
payment returns to step 3 with the reason and "Try again"; nothing is activated and nothing is lost.
The app never shows a message that purchasing is unavailable.

### 7.11 Later release (not in the first)
Ask (cited questions to a local model) and Memory (reviewed long-term lore). They do not appear in the first
release's interface.

## 8. Functional requirements

| ID | Requirement |
|---|---|
| `FR-ZIM-001` | Open and validate real ZIM content through the range-capable engine |
| `FR-ZIM-002` | Render safe HTML, images and internal links; explain any page that cannot render |
| `FR-ZIM-003` | Search titles offline, within 100 ms per keystroke on desktop |
| `FR-DYN-001` | Accept any positive budget; plan a subset that fits it |
| `FR-DYN-002` | Show the plan, or counted progress, within 3 s |
| `FR-DYN-003` | Download by range, resumable after process death, cancellable |
| `FR-DYN-004` | Revise a plan without discarding kept content |
| `FR-DYN-005` | Label partial packs as partial everywhere |
| `FR-NAV-001` | Toggle between reader and navigator; show the navigator with the reader (side-by-side or swappable inset) |
| `FR-NAV-002` | Six-degree-of-freedom flight; wheel is forward thrust |
| `FR-NAV-003` | Place nodes from the pack's own semantics; show preparation progress |
| `FR-NAV-004` | Select, summarise and dive from a node to its article and back |
| `FR-NAV-005` | Journey map of the flown path |
| `FR-NAV-006` | Pysanky launcher: exact v102 reticle and overcharge glitch; energy rises with hold time; about 10 % Fabergé |
| `FR-ANN-001` | Text, ink, voice, photo and video annotations stored with the user's data, anchored to the article |
| `FR-ANN-002` | Annotations survive restart, pack removal and reinstall |
| `FR-SIDE-001` | Export and import signed annotation sidecars, idempotently |
| `FR-ENT-001` | Purchases through the billing service; signed entitlement verified offline for 30 days and beyond for perpetual grants |
| `FR-ENT-002` | Paid capabilities enforced where the work happens, not only hidden in the interface |
| `FR-BACK-001` | Encrypted backup and lossless restore of all user data |
| `FR-DIAG-001` | Inspectable, redacted diagnostics export with consent |
| `FR-A11Y-001` | Every core flow by keyboard and screen reader |
| `FR-VER-001` | Version and build visible bottom-right on every screen and in every artifact filename |

## 9. Non-functional requirements

- **Data safety:** every write path leaves the last good state readable if the process is killed; upgrade from
  1.0, 1.6 and 1.14 keeps notes and packs.
- **Performance (measured on a Wikipedia-class pack, ≥ 50 GB source):** cold start ≤ 2 s desktop / 3 s Android
  regardless of installed size; article open p95 ≤ 400 ms / 1 s; navigator holds 60 fps desktop / 30 fps Android
  with the visible node set; idle memory ≤ 350 MB / 250 MB.
- **Accessibility:** WCAG 2.2 AA for the reading, library and settings flows.
- **Privacy:** no content telemetry; analytics opt-in; diagnostics exclude content by default.

## 10. Commercial boundary

Free: read, search, DynDon downloads, highlights and typed comments, human-readable export.
Paid, by one-time activation (operator decision 2026-11-10: no subscription, no renewal, no network needed after
the purchase): rich-media annotations, signed sidecar export, unlimited packs, encrypted backup. The price is set
by the operator in the store and read from it; it is never written into the app. Payment is Google Play in-app
billing on Android and Google Pay on Linux and Windows. **OPEN:** whether the navigator is free, paid, or free with a paid depth; whether a pack-count limit
applies to the free tier. Payment status never blocks access to user-created data.

## 11. Release acceptance

One continuous recording of the built, versioned artifact on each platform showing: first-launch setup to a
readable article; a storage-fit plan for a pack larger than the budget, its download and offline reading after
restart; the navigator placed from that pack's content, flight, dive to an article and back, the launcher with
reticle and overcharge; all five annotation kinds saved and replayed offline; purchase, restore and offline use;
backup and restore on a clean install.

## 12. Non-goals

Social features; required cloud account; hidden cloud inference; arbitrary web browsing in the article view;
claims that a partial pack is a valid full ZIM; Creator Studio; browser extension.

## 13. Design source

Google Stitch project "EnZIME Reader v5 — DynDon + Knowledge Pysanky" (ID recorded here when created).
Project ID: `13978455034360167310`.
Design system: EnZIME Aurora Glass (`DOCS/REBOOT-PAK/STITCH/design-system.json`) with two of its rules removed by
the operator on 2026-11-10: the reduced-motion rule is not a goal, and the reader surface is not opaque — glass
surfaces apply to the reader as to the rest of the app.

## 14. Complete product contract — 2026-11-13

This section is the operative revision. Authority: the operator's instruction to review the existing Stitch work, express the complete intended product, freeze its actual screens, then wire the architecture exactly. It preserves the implemented product and the two established differentiators rather than commissioning another restart. Historical draft statements excluding Ask/Memory, displaying healthy readiness, choosing private storage, adding subscriptions or requiring a new test programme are superseded.

### 14.1 The experience worth recommending

EnZIME helps a person keep useful knowledge and their own work available without depending on a connection. Its first useful result is a readable article, not a configuration report. Its distinctive demonstration is selecting a smaller, deliberate download from a larger archive, then moving between reading and exploring connections without losing one's place. Its lasting value is notes that remain the person's own and large files that are reused across their apps rather than copied repeatedly.

The first-release positioning is preparedness, as explicitly recorded in `DOCS/TODO-LEDGER.md`: dependable knowledge without a connection, without military/security theatre. The audience includes people who deliberately keep reference material: students, independent researchers, travellers, field workers and readers with limited storage or unreliable connectivity. They need understandable choices, predictable storage costs and confidence that their work remains accessible. Technical expertise is not an entry requirement. Spatial exploration and local cited questions deepen that use; neither is a prerequisite to ordinary reading.

The recommendation hypothesis is specific: a user can show someone how they kept the knowledge they needed in the space they had, found an unexpected connection, and retained their own notes. There is no referral prompt, engagement counter, urgency promotion or manufactured social proof. This is a design rationale, not a claim of market validation or guaranteed recommendations.

### 14.2 Scope and precedence

Retain setup, Home, Library, live catalog, DynDon, reader, Knowledge Pysanky navigator, text/highlight/ink/voice/photo/video annotation, notebooks/tags, sidecar import/export, scoped local Ask with citations, reviewed Memory, search, shared resource storage, accessibility, privacy, backup/restore, one-time activation and help/about. Android, Linux and Windows have the same functions. The web artifact remains an artifact of the same UI; it must not simulate native results where a native bridge is required.

The latest catalog-budget and mandatory-shared-storage amendments in `ARCHITECTURE.md` supply exact arithmetic, deadlines, source selection and storage policy. Their behaviour is unchanged by this presentation work. Foreground apps open common model files; no background model runner is introduced. Existing inference ports and their effective architecture are preserved rather than changing engines as a side effect of design.

No new social network, account requirement, private resource namespace selector, mesh service, cloud inference, subscription, terminal, hardware dashboard, fake article, unsupported format or security claim is introduced. Retain user work and last-good content. Removing an archive never silently removes notes. Healthy internal checks, milestone labels such as M1 and implementation identifiers do not appear in normal product UI.

### 14.3 Visual and interaction grammar

Use EnZIME Aurora Glass: midnight canvas `#090B13`, translucent blue-black glass `rgba(23,28,44,.72)`, subtle pale borders, readable near-white text, periwinkle primary actions and teal accents. Broad low-contrast aurora fields establish the existing identity without competing with text. Inter is the interface face; Literata is the article face; monospaced numerals are limited to quantities where alignment helps comparison. Reader glass is retained as explicitly directed; increase its tonal backing as needed for text contrast, without replacing it with a different opaque visual theme. No new decorative telemetry or animation system.

Desktop navigation is a compact left rail with icon and text for Home, Library, Ask, Notes, Memory and Settings. These names are familiar destinations, not internal component names. Reader and Navigator are contextual modes of a selected pack, not another copy of the library. At narrow widths use a bottom bar for Home, Library, Notes and More; More exposes Ask, Memory and Settings with the same labels and no lost function. Reader chrome can collapse to protect reading width. The title and Back remain accessible; controls do not cover the selected text or keyboard.

The top bar carries the Z mark/name and Search. It does not carry an unconditional Ready badge, fabricated account avatar or entitlement tier. Search has an accessible label and shortcut hint on keyboard devices. The current version sits quietly at bottom right in every surface, including dialogs, without covering an action; it identifies the artifact without asking users to understand build machinery.

Use a single filled primary action for the immediate decision, outlined secondary actions and text links for reversible navigation. Destructive actions are separated and name what will be removed. All icon actions have accessible names; visible text accompanies unfamiliar functions. Targets are at least 44 CSS pixels where touched, focus is visible, text wraps under scaling, and overlays have scrollable bodies with reachable actions. Color always has an accompanying label or shape. Responsive reflow changes arrangement, not functionality or data semantics.

States are real interface designs: initial/empty, loading with a named operation, ready, recoverable failure, and completion where meaningful. No skeletons standing in indefinitely for content, mock counts, interpolation tokens or invented receipts. Unknown quantities are labelled as unknown, not zero. A failure explains the affected action and offers the actual recovery; unrelated reading remains available. Successful routine background work is quiet. Toasts confirm completed user actions; they do not replace persistent recovery controls for failures.

### 14.4 Screen and element contracts

The following tables are an element budget: an additional visible element requires a specific user purpose in this PRD before freeze. Controls inherit the common visual grammar; the final Stitch exports determine their exact visual composition. Every data-bearing region binds actual domain data; its empty state is a complete design, not a shipping placeholder.

#### S01 — Setup

Purpose: reach useful knowledge without making technical choices on behalf of the application.

| Element / function | Placement and appearance | Why it belongs / behaviour |
|---|---|---|
| Welcome and one-sentence offline promise | Compact heading above two clear actions | Explains the benefit without a marketing tour or account wall. |
| Open a ZIM / Find a download | Central primary and secondary actions | Serves people with existing files and new readers equally; invokes the native picker or Library catalog. File validation reports the actual result. |
| Shared storage summary | Below acquisition actions, ordinary Internal storage / SD card labels | Explains space reuse. A choice exists only for actual available media; namespace and sharing policy are fixed by the build. |
| Allow shared storage access | Inline only when permission is required | Opens the actual system permission flow; returning refreshes media. Browsing remains available. |
| Continue to library | Quiet navigation below | Allows setup without acquiring data or a model. No benchmark or mandatory AI setup. |

Permission failure, no usable medium, cancelled picker and invalid archive have local recovery. A first successful import leads to the selected pack's reader. Completion is persisted so setup does not recur unnecessarily.

#### S02 — Home

Purpose: resume the person's work, or provide the first useful action.

| Element / function | Placement and appearance | Why it belongs / behaviour |
|---|---|---|
| Continue reading | First and most prominent content card; real article title, pack and Resume | Reduces the work of returning. Without history show Open a ZIM and Find a download with one sentence, not fake recent content. |
| Recent packs | Compact row/grid below, real titles and partial/full labels | Offers a short route to another book without duplicating the entire catalog. No invented covers or counts. |
| Active downloads | Compact progress rows after reading content, present only while relevant | Lets the person return to an acquisition; actual bytes/state and pause/resume/cancel. It does not displace reading. |
| Items needing attention | Conditional inline card with affected item and remedy | Carries real failed restoration/download actions or Memory proposals. No generic readiness panel, milestone pass, healthy check button or alarm without an issue. |

An empty Home remains useful and calm. It contains neither a wall of empty dashboards nor an upsell competing with the first article.

#### S03 — Library and available downloads

Purpose: manage what is kept and acquire what is missing.

| Element / function | Placement and appearance | Why it belongs / behaviour |
|---|---|---|
| Installed / Available downloads / Downloads | Local tabs immediately below Library title | Separates three concrete jobs while preserving their state on switching. Download count comes from actual jobs. |
| Open a ZIM | Header action | Makes an existing archive as easy to use as a catalog item. |
| Search and language | Above the result set, labelled fields, common language names/codes accepted | Narrows real catalog results; language guidance belongs to the field, not an intimidating block of protocol instructions. |
| Pack cards or rows | Main content area, title first; source/language, size and partial/full context below | Provides enough information to choose, without pretending thumbnails or data exist. Installed entries Open; catalog entries Plan download. |
| Catalog source state | Small text next to results | Live catalog or Cached results states provenance without branding EnZIME as a provider. Real publisher attribution stays in records. |
| Storage summary | Near acquisition controls, not a constant global dashboard | Shows real available space and sharing benefit where it affects a decision. |
| Pack actions | Item menu: details, revise download, remove | Keeps secondary/destructive actions away from Open. Removal explains that user notes remain and requires confirmation. |
| Retry / clear filters / pagination | At the affected results region | Deadlines settle into explicit failure, real empty result or results. Retain previous real results while paging; no endless skeleton. |

#### S04 — Download plan (DynDon)

Purpose: decide what to retain and its peak space cost before starting transfer.

| Element / function | Placement and appearance | Why it belongs / behaviour |
|---|---|---|
| Pack title and source size | Dialog header, close at opposite corner | Maintains selection context. Unknown source size is explicit. |
| Physical medium | First field; summary for one medium, choice for several | Capacity determines the valid budget. Always shared across apps; no namespace choice. SD guidance is shown when relevant, including model performance implications. |
| Preferred storage size slider and precise size input | Dominant horizontal control, bound values next to it | Enables quick fitting and exact control. Native whole-byte limits, source-size cap and reserve formulas are the frozen architecture, not visual approximations. |
| Spread cuts evenly / Prioritize a domain | Two labelled choices under size | Makes the retention tradeoff explicit. Priority reveals Preferred domain or topic; explain title/URL/domain matching without claiming a taxonomy or semantic algorithm. |
| Compute plan and progress | Before results, named operation and elapsed/count progress where supplied | Makes the calculation real; no preview numbers presented as an already computed plan. |
| Plan summary and size bar | After controls, aligned quantities with separate content, index reserve and workspace reserve | Distinguishes bytes transferred from reserved maximum and peak. Show full source size, readable article/resource counts when known, free before/after and partial/full. Color supports labels rather than replacing them. |
| Start download / Cancel | Persistent dialog footer after the summary | Start is enabled only for the exact current valid plan, writable shared medium and successful capacity state. Any changed input invalidates the prior plan. |

Maximum storage and maximum content payload are separate labels. Reserve is never called kept content. No-match priority, insufficient space, permission failure and planning deadline have actual recovery. No fake verification tick. The dialog is viewport anchored and scrollable on mobile. A completed start navigates to the actual download job; cancellation does not erase unrelated installed content.

#### S05 — Download queue and pack details

Purpose: understand acquisition progress and safely manage retained content.

| Element / function | Placement and appearance | Why it belongs / behaviour |
|---|---|---|
| Job title, status, transferred/total bytes | One row/card per real job, progress under title | Makes interruption and progress intelligible. Speed/remaining time appear only if computed from actual observations. |
| Pause / Resume / Cancel / Retry | Adjacent to the job, state-appropriate | Gives control without exposing state-machine codes. Cancel confirms its effect on this acquisition. |
| Open | Completion action | Offered only after a readable edition is actually activated. |
| Pack details | Focused sheet: source, edition, language, full/partial, retained size, readable counts, integrity result | Explains what is kept and whether it can be trusted. Technical hashes belong in expandable details where useful. |
| Revise download / Remove | Details actions, separated from Open | Reuses retained data as architecture permits; removal states precisely what is deleted and preserves notes. |

Empty queue copy simply says there are no downloads and links to available downloads. Failed jobs remain discoverable with their reason and recovery.

#### S06 — Reader

Purpose: read comfortably, return to one's place and act on a passage without losing context.

| Element / function | Placement and appearance | Why it belongs / behaviour |
|---|---|---|
| Back/forward, pack and article title | Compact top context row | Supports real navigation history and identifies source. Avoids decorative coordinates or reader counts. |
| Reader / Navigator toggle | Top context row, persistent selected state | Makes both established viewing modes discoverable while preserving article and flight positions. |
| Table of contents / in-pack title search | Left pane on wide layouts, labelled drawer on narrow layouts | Helps navigate long content without narrowing mobile article text. Results represent retained readable scope. |
| Article | Central glass reading surface, Literata and comfortable line length | The content is the product. Images and links resolve through the archive; never substitute generated article prose. |
| Text settings | Small toolbar action opening font/size/spacing controls | Personalizes reading without filling the article view with controls. Changes persist. |
| Highlight / Note / Ink / Voice / Photo / Video | Contextual toolbar and annotation drawer | Actions operate on the actual selection/anchor. Familiar text labels accompany symbols. Rich media invokes real capture/import and local persistence. |
| Related reading / Ask about this | Companion drawer actions scoped to current content | Supports investigation without hiding the article or silently broadening AI scope. |
| Navigator inset / expand-swap | Small resizable contextual pane on wide layouts; switchable sheet on narrow | Shows spatial context without forcing it into the reading column. The mode switch never discards unsaved work. |

Unavailable partial-pack links explain that the article was not retained and offer the existing revise-download path; they do not report an archive-wide failure. Rendering errors identify the affected page. Loading never fabricates article content. External links require an explicit external-opening action under the existing policy.

#### S07 — Knowledge Pysanky navigator

Purpose: explore relationships in the actual retained pack and move naturally into reading.

| Element / function | Placement and appearance | Why it belongs / behaviour |
|---|---|---|
| Constellation and edges | Main canvas, node kinds differentiated by shape/color with readable labels | Spatial proximity represents actual derived relationships; not a fixed demo graph. Confidence belongs to a selected relation, not an unexplained global score. |
| Selected-node detail | Right glass panel, bottom sheet on narrow screens | Title, source summary and relation context support a choice; Dive into article is primary, Add to journey secondary. |
| Reader inset and mode switch | Lower corner on desktop, labelled switch on mobile | Maintains article context and swaps views without resetting either. |
| Journey | Collapsible strip/panel of visited/opened articles | Lets the person retrace exploration instead of getting lost. Entries derive from their actual path. |
| Legend and movement help | Quiet bottom controls and an explicit Help action | Explains unfamiliar interaction when requested, rather than surrounding the view with technical instruments. |
| Preparation progress | Local banner only while deriving graph data | Names the real operation and available count; reader remains usable. No fake percentage or background-runner promise. |
| Pysanky launcher | Existing exact v1.107 reticle/overcharge treatment during deliberate activation | Retains the established playful identity. The later operator direction in `DOCS/TODO-LEDGER.md` supersedes the old draft's v102 reference. Charge appears only while used; no permanent diagnostic HUD. Energy increases with hold, with approximately 10 percent Fabergé eggs as required. |

Default desktop composition is the selected mode with a swappable inset; a user may expand to side-by-side where space permits. Narrow screens switch the companion sheet without shrinking article text. Remember mode per pack. Flight requires six degrees of freedom and wheel thrust; the known orbit/recentre defects in `VENDOR/knowledge-pysanky/thinkspace-navigatorv1107.tsx` are not intended behaviour. Architecture must bind explicit craft-relative inputs and monotonic charge before freeze, rather than instructing a coder to preserve those defects or choose new physics. Touch controls provide equivalent translation/rotation and a deliberate launcher hold; they do not obstruct node selection. Placement must derive from retained content; the unfinished choice recorded in the ledger must be resolved in architecture before implementation rather than pretending an existing algorithm is already specified. The prototype does not authorize a decorative fixed graph.

#### S08 — Annotation workspace

Purpose: add personal work at its source and keep it safe.

| Element / function | Placement and appearance | Why it belongs / behaviour |
|---|---|---|
| Anchor excerpt and source | Top of adjacent panel/sheet | Makes it clear where the note will live and offers return to source. |
| Text editor and notebook/tags | Main panel with explicit labels | Records the person's thought and organizes retrieval. No synthetic starting note. |
| Ink tools | Compact toolbar adjacent to canvas: pen, color, width, erase, undo/redo | Each acts on real strokes; current tool is named and indicated without color alone. |
| Voice capture/playback | Record/Stop, real duration, playback and remove | Clear recording status and explicit microphone permission. Existing dictation can insert actual recognized text without pretending a transcription exists. |
| Photo/video capture/import and preview | Media area, actual local attachment | Supports documented rich notes with real permissions and save state, not stock images. |
| Save / Cancel / delete confirmation | Reachable panel footer | Save reports actual persistence. Navigation with unsaved work offers save/discard/stay; failure preserves the draft. |

Activation may explain a paid creation action, but viewing/exporting existing personal work stays accessible. Remove-media and delete-note actions are distinct and explain their scope.

#### S09 — Notes, notebooks and sharing

Purpose: retrieve personal work across packs and move it without losing provenance.

| Element / function | Placement and appearance | Why it belongs / behaviour |
|---|---|---|
| Notebook list / tags | Left on desktop, filter drawer on mobile | Groups user-created work without forcing a folder hierarchy. Create/rename acts on real notebooks. |
| Search, type and source filters | Above note list | Helps find known work. Counts come from actual results; no decorative activity chart. |
| Note list with excerpt, source and date | Central list, selected detail beside it | Makes scanning efficient and preserves source context; attachments use actual previews. |
| Open source / edit / delete | Selected-note detail | Connects knowledge back to the article while retaining notes when a pack is missing. Missing source offers locate/import rather than deleting the note. |
| Export / Import | Toolbar, scoped to selected notes or notebook | Opens the sidecar flow and makes the export scope explicit. Human-readable export stays available. |
| Sidecar review | Focused dialog with file, author/signature status, real counts/conflicts, destination notebook and import decision | Establishes provenance without inventing trust. Idempotent import reports actual added/skipped/conflicting items. Signing proves authorship/integrity, not truth of claims. |

Empty Notes explains how to add a note from a passage. Import failure keeps the original file and current notes untouched. A selected item is never silently overwritten by an import.

#### S10 — Ask

Purpose: ask about selected local knowledge and inspect the sources of the response.

| Element / function | Placement and appearance | Why it belongs / behaviour |
|---|---|---|
| Conversation history / new conversation | Left collapsible pane | Returns to prior work without crowding the answer. Empty history contains no invented conversation. |
| Scope selector | Immediately above thread | States which packs/notes the question uses before sending. Selection is explicit and remembered appropriately. |
| Thread with citations | Main reading column | Distinguishes user questions from generated responses and links each citation to actual evidence. Unsupported answers are not dressed as sourced facts. |
| Composer, Send / Stop | Bottom of thread above keyboard | Keeps the next action reachable and supports cancellation of foreground generation. |
| Model state and model choice | Compact task-relevant control | Names the selected real model; if absent, offers choose/download a shared model. No fake speed, GPU score or claims that the app must run a background server. |

Model acquisition uses the common storage policy and actual resource size. Reading does not require AI. Generation failure preserves the question and partial response with an honest state; Retry does not silently change scope or use cloud inference.

#### S11 — Memory

Purpose: review proposed retained knowledge with its evidence and correct it.

| Element / function | Placement and appearance | Why it belongs / behaviour |
|---|---|---|
| Proposal inbox | First panel, real pending count | Makes retention a user decision, not hidden accumulation. |
| Evidence and proposed fact | Selected detail, source links adjacent | Lets a person judge the claim before accepting it. No fabricated lore or unexplained certainty. |
| Accept / Reject / Correct | Next to selected proposal | Every action corresponds to the stored proposal; editing preserves revision provenance. |
| Graph / timeline | Secondary workspace tabs | Provides relationship and revision context after there is data. Empty states explain what will appear from accepted work. |
| Pin / delete / source navigation | Detail actions with explicit delete confirmation | Supports deliberate retention and correction. Deletion does not pretend to erase original source material. |

No automatic acceptance, hidden persona profile or global neural-health status is shown.

#### S12 — Search

Purpose: find a pack, article or personal note without guessing a destination.

| Element / function | Placement and appearance | Why it belongs / behaviour |
|---|---|---|
| Query field and clear/close | Top of overlay/full mobile sheet, focused on open | Minimizes steps; keyboard and touch behave consistently. |
| Scope filters | Under query | Distinguishes installed readable content, notes and catalog search. No implicit network request for a local-only scope. |
| Grouped results with source excerpts | Main list, keyboard selection | Identifies what will open. Results are real; partial-pack limitations are explicit. |
| Empty / failure / indexing state | Within results | Explains the actual state with clear query or retry action. No invented result count or elapsed time. |

Opening a result preserves prior reading position and pending draft guards. Global search delegates to existing domain capabilities; the UI does not claim full-text coverage where only title search exists.

#### S13 — Settings and help

Purpose: adjust policies and protect work without exposing application internals as routine tasks.

| Group / elements | Placement and appearance | Why it belongs / behaviour |
|---|---|---|
| Storage: actual media and capacity, model placement guidance | First settings group with human labels | Controls physical location while common sharing is fixed. No raw package namespace or sharing switch. |
| Downloads/network: existing network policy controls | Separate labelled group | Makes allowed transfers predictable; explains effects in plain language. |
| Reading/accessibility: font/size/spacing and existing assistive preferences | Reading group with immediate representation | Supports comfortable reading. No new animation system is required. |
| Local AI: actual model selection/resource state | Dedicated optional group | Keeps setup available without making it a prerequisite. Common-file reuse is explained where helpful. |
| Activation and Restore | Purchase group with actual entitlement status | One-time purchase only, actual store information, no account/enterprise tier. |
| Backup / Restore | Data group with file/passphrase, scope and restore preview | Protects personal work. Show real progress and conflicts; preserve last-good state on failure. Never hard-code encryption or success claims absent from the service. |
| Privacy / analytics consent | Explicit opt-in controls and concise data explanation | Lets people control actual collection, never a fake privacy badge. |
| Help / shortcuts | Task-based guidance: acquire, fit, read, annotate, explore, export, restore | Explains unfamiliar user actions. It does not ask readers to understand M1, internal readiness or namespaces. |
| About / licences / diagnostics export | Secondary group | Meets attribution/support needs without cluttering Home. Diagnostics preview/redaction and explicit export preserve consent. Version is actual. |

Settings saves indicate persistence or an actionable failure. Help content is available offline. Destructive restore/reset actions explain scope and require confirmation. No routine Re-run check exists solely to validate internal development milestones.

#### S14–S17 — One-time activation

Retain the four fixed component names in §7.10. One consistent Z mark, Inter headings, four dots, Close and quiet actual version appear on all steps. This replaces the inconsistent marks, fonts and step bars in the old exports.

| Screen / elements | Placement and appearance | Purpose and real-state contract |
|---|---|---|
| ActivationStepValue: promise, four capability rows, Continue | Focused central panel, single primary action | Explains the paid benefits at a voluntary purchase boundary. Close returns to the prior task and preserves work. No subscription alternatives or urgency. |
| ActivationStepOffer: One-time activation, actual localized store price, no-renewal/offline/ownership promises, Activate, Back | Price and terms together, action immediately below | Makes cost and ownership understandable before payment. While fetching show a named operation; failure offers retry and exit. Never render a price token, invented amount or enabled purchase without a real offer. |
| ActivationStepPay: actual order/total, platform payment method, Restore, Back | One focused order card and the platform's payment action | Google Play on Android and the configured desktop Google Pay flow; no manual card form. Cancellation preserves state. Restore invokes the real service and reports its outcome. |
| ActivationStepDone: actual verified entitlement date, offline grant, real recovery mechanism, Copy where applicable, Start reading | Receipt then one return action | Appears only after a verified grant. No fabricated restore key/date or false success from clicking a prototype button. The stored perpetual entitlement permits offline verification; existing notes remain accessible irrespective of purchase. |

Do not impose a new pack limit or navigator paywall to fill the old draft's OPEN commercial questions. Preserve the effective entitled capability contract until an explicit commercial amendment. Store-configured offers are the price authority. No static sample price/receipt is a valid implementation substitute.

### 14.5 Shared states and interaction completion

Every surface includes its actual empty state, named progress, ready layout and local failure/recovery composition. Confirmations cover pack removal, note deletion, cancellation where data is discarded, unsaved annotation navigation and destructive restore. Permission requests occur at the relevant capture/storage action. File pickers, store payment and OS permission screens are native boundaries; the prototype shows the entry and return states without forging those external screens.

Primary journeys: setup → import/catalog → plan → queue → reader; reader ↔ navigator → journey → reader; reader → annotation → notes → source; notes → export/import; reader → scoped Ask → cited source; Memory proposal → evidence → accept/correct/reject; Settings → backup/restore; paid creation action → activation → original action. Closing a dialog returns to its invoking context. Back never silently discards a draft or resets the selected pack.

### 14.6 Freeze and delivery contract

The reviewed source inventory is `DOCS/STITCH/product-review/screen-inventory.json`; reconciliation is `REVIEW.md` beside it. New complete-prototype project: `17076380137565911053`. Actual screen IDs, exported HTML/images, per-screen state coverage and links belong in the freeze manifest before implementation. A blank Stitch listing is not proof of complete review; the known v5 enumeration gap must remain explicit until resolved.

Freeze only after the actual prototype contains the above functions and state designs. The freeze identifies immutable exported assets and hashes. Architecture maps each named control to exact entities, data and actions; the checklist carries those bindings to the coder. Implementation may adapt static markup into framework components, replace data slots with real state, bundle fonts/assets and use native boundary adapters. It may not substitute a hand-recreated design, change interaction semantics, remove a difficult function, add another product, or silently alter algorithms because an implementation is inconvenient.

Delivery means artifacts for the defined platform set from the unified BuildAll entry, built from published merged source, with the existing version increments plus the required success increment retained. Source review compares implementation with the frozen contract. The operator's suspension of product tests and behaviour checks remains effective; packaging is not runtime acceptance. The operator will open a real ZIM and exercise the product. Matching PRD/architecture/prototype establishes implementation conformance, not product-market success. Report exactly which artifacts exist and which behaviour has not been observed.
