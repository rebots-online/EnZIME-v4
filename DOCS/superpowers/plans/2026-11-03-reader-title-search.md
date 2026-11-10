# P2-05-TITLE implementation checklist and cold-coder recipe

Architecture: `DOCS/ARCHITECTURE/reader-title-search.md`, published in `cd3e5b4a888bf3c8ce0ea45bc0bfb686908f73e0`. One bounded change; native Windows working checkout; original source baseline `176b047c75a988a68f12f33823290b91a1071113`.

## Global constraints

Use the real production archive, IPC and UI. No invented title/archive/catalog/user data, mock IPC, fake success or temporary customer flow, including tests. Keep existing source/annotation persistence and guards intact. Do not run a platform-only packaging command: the prescribed release entrypoint is `bash build_all.sh`. Source checks are not a release build or product acceptance. Preserve the untracked crash transcript and all unrelated work. Work on the current master direction, stage only owned files, and preserve concurrent remote commits. Root owns source publication and its exact remote readback for this iteration.

## Review focus

1. A real retained archive searched after restart must not need upstream connectivity or return unretained-only hits.
2. A delayed response for another pack/query/page must not replace the current search or revive after unmount.
3. Result navigation must preserve literal percent/slash/non-ASCII keys and trigger the existing dirty-draft guard.
4. A missing pack/archive must be an error, never a successful empty list; empty query and actual zero hits remain distinct usable states.
5. Page boundaries, max limit and actual lookahead must not fabricate totals, skip/duplicate results or silently lose the final next page.

## Tasks

### TITLE-01 — Native bounded title search

State: NOT_STARTED.

Files: modify `apps/reader/src-tauri/src/ipc/reader.rs`, `apps/reader/src-tauri/src/ipc/dto.rs`, and `apps/reader/src-tauri/src/lib.rs`. Query CodeGraph for the actual current command/DTO organization before edits.

Consume: `AppState.archives` and `ports::zim::{TitleSearchQuery, TitleSearchHit, ZimArchive}`. Produce a registered `reader_search_titles` command with JS arguments `{packId, query, offset?, limit?}` and camelCase DTO fields exactly as the architecture specifies. Native DTO names are `ReaderTitleSearchItem` and `ReaderTitleSearchPage`; item `{articleKey,title,score}`, page `{packId,query,offset,limit,hasMore,items}`. Use default limit 25, clamp 1..100, default offset 0, and at most 256 Unicode scalars after trimming. Obtain an actual extra hit for lookahead; handle adapter page caps explicitly. Reuse the installed archive; no mutex across await. Inspect the real durable archive search path to confirm retained availability semantics; if deficient, report it to root rather than fabricate results.

Completion: exact registration/serde contract is reachable, bounded and semantically consistent with actual archive behavior. Archive absence produces a real error. Supply source-level and real-data verification evidence, not a fake archive test.

Current-source ruling: the canonical archive caps offset at 10,000. Native rejects larger offsets rather than silently repeating the capped page. Truthful lookahead remains; the UI asks the user to refine the query when the next actual page would exceed that boundary. Do not modify the canonical archive for this task.

### TITLE-02 — Reader search UI

State: NOT_STARTED.

Files: modify `apps/reader/src/services/readerService.ts`, `apps/reader/src/routes/reader/ReaderView.tsx`; create `apps/reader/src/routes/reader/ReaderTitleSearch.tsx`; modify existing Reader styles only if required. No broad store/router/media rewrite.

Consume: TITLE-01 wire contract, existing `navigate`/`useBlocker` in ReaderView and existing shared styles/error presentation. Produce typed `readerService.readerSearchTitles(packId, query, offset?, limit?)`, the exact item/page types, and a production form/results panel for the current route pack.

The component owns real query/page/loading/error/results state, keyboard form submission, explicit previous/next, stale-response generation/cancellation handling, pack-change reset and unmount invalidation. It calls the production service; no test/demo data branch. Result navigation uses the existing encoded article route and the parent's normal router navigation, preserving the unsaved annotation guard. Never show fabricated total hits. Initial instructions are useful production copy, not example result content.

Completion: errors/loading/empty/success are honest; only the current request can update results/status; every result opens its actual article key; existing Reader capture/save/navigation behavior remains intact.

### TITLE-03 — Verify and self-review actual paths

State: NOT_STARTED.

Run the existing frontend typecheck and native source checks in the documented toolchain; report exact command/exit/log. New regression verification must use production code and real resource data, not a mocked service, synthetic archive or invented customer content. If a real resource is not yet locally available, state that fact and leave the corresponding runtime observation unproven while root prepares the real acquisition path. Never label structural inspection as end-to-end success.

Trace native registration -> handler -> existing archive search -> DTO -> real service -> UI -> encoded route -> existing draft guard. Inspect returned scopes and page lookahead. Root independently reviews each task and returns deficiencies to this same coder. Keep fixes within this iteration, with independently published correction commits.

### TITLE-04 — Supervising semantic review/correction

State: NOT_STARTED.

Root checks each architecture paragraph, task completion claim, pagination/empty/error behavior, lifecycle guards, actual retained catalog behavior, and user navigation. Missing wiring or reliance on synthetic success goes back to the same coder. Passing typecheck cannot substitute for semantic review.

### TITLE-05 — Prescribed build and real continuous Windows demonstration

State: NOT_STARTED.

After source publication and adequate review, root runs the prescribed all-platform coordinator, resolves declared configuration through current pointer rules, and records actual lane outcomes without stale substitution. Record the actual packaged Windows product continuously against `DOCS/TEST_RUBRIC.md` section P2-05-TITLE. Bind source/artifact/resource identities to timecodes. Any discovered defect returns to this same coder and requires correction plus re-recording. Only the iteration is completed; parent P2-05/P2-07 and release gates stay incomplete until their other clauses pass.

## Delivery ledger

- Architecture boundary: `cd3e5b4a888bf3c8ce0ea45bc0bfb686908f73e0`, remote verified.
- Checklist/rubric boundary: recorded by root's next published commit.
- Implementation/review/build/recording: not performed when this plan was authored.
