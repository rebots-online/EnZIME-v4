# EnZIME Reader — Product Requirements (v5.0.0)

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
