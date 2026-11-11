# Rubric run — v1.23.06045 AppImage, 2026-11-10

Artifact hash: `artifact-sha256.txt`. Same test profile as the v1.20.05778 runs (two packs installed). Scope: the
five fixes merged since v1.20.05778. Not a full gauntlet; no recording.

| Fix | Result | Evidence |
|---|---|---|
| M2-D4 Home "Continue reading" no longer loads forever | Observed. With no history it shows "No reading in progress" | 01 |
| M2-D7 "Open a ZIM file" removed from Home | Observed. The card is absent | 01, 03 |
| M2-D5a/b opening an article records a visit | Partly. Library card changes from "not opened" to "opened ..." and Home offers "Resume reading", so the visit is recorded. What is displayed is wrong in both places (defects 12, 13) | 02, 03 |
| M2-D2 fractional budget accepted | Not run in this session | — |

Defects seen on this artifact:

12. Library shows the opened time as `11745-4294851302-4294967291T02:30:39Z`: the date text produced for
    `lastOpenedAt` in `apps/reader/src-tauri/src/ipc/library.rs` is malformed.
13. Home "Continue reading" shows only "· opened" with no article or pack title, and "Recent packs" rows read
    "undefined articles · — · not opened" with no pack title. The backend's Home snapshot sends `PackManifest` and
    `ReaderHistoryItem` shapes; the frontend expects `PackSummary` and `ContinueReadingEntry`
    (`apps/reader/src/services/homeService.ts`). The "Recent packs" half of this was already present in
    v1.20.05778 and was not noticed then.
14. Home "Task inbox" still mentions asking a question and running a backup (defect 6, unchanged).
