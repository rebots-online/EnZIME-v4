# P2-05-TITLE — Installed-pack title search

Date: 2026-11-03. Scope: one independent delivery iteration contributing to P2-05/P2-07; it does not close their other requirements or any release gate.

## Current evidence and intended behavior

The existing production `ZimArchive::search_titles(TitleSearchQuery)` supports bounded title search, including the durable retained catalog. The reader shell currently exposes only article/open commands. Expose that existing capability through real native IPC and an accessible Reader search panel. A user can find and open an installed/retained article without knowing its key, including after an offline restart. Do not fabricate results, add a substitute catalog, or require Firebase/model/network access.

This implements the existing PRD FR-ZIM-003 title-search portion. Full-text search, bookmarks/history, contract generation, and other checklist features remain independently accountable. Current operator authority: KStore production-only rule revision 2; independent-delivery revision 1; annotated-environment/credential-pointer revision 2, read 2026-11-03. No test/demo placeholder exception applies.

## Wire contract

Register `reader_search_titles` in the actual Tauri command handler. Its JS arguments are `{ packId: string, query: string, offset?: number, limit?: number }`. Rust validates the selected installed/open archive, normalizes surrounding query whitespace, rejects more than 256 Unicode scalar values, uses offset 0 by default, and clamps requested page size to 1..100 with default 25. Empty normalized query returns an empty page for the validated archive. Unknown/unavailable pack returns an actionable error, not an empty successful search.

Return a camelCase `ReaderTitleSearchPage` with `packId`, normalized `query`, `offset`, `limit`, `hasMore`, and `items`. Each item has its real `articleKey`, `title`, and finite archive-supplied `score`. Determine `hasMore` from an actual extra result at the requested offset, never an invented total. Respect the archive adapter's actual bounds; if it prevents an extra hit at limit 100, use a separate bounded lookahead request rather than silently reporting no next page.

The handler clones the archive handle under the state lock and releases the lock before awaiting archive work. Do not hold SQLite/archive mutexes across network or asynchronous work. The retained offline archive determines available hits; do not advertise unavailable upstream titles as locally readable results. Existing archive title ranking/order is preserved.

## Frontend behavior

- Add the exact DTOs and method to `readerService.ts`; there is one authoritative shape for this new command.
- Add a focused `ReaderTitleSearch` component in the Reader. It uses the selected route pack and the real service method, with an explicit search form, clear label, loading/error/empty/results states, and bounded Previous/Next pagination. Do not display a fabricated result count or example resource.
- A pack change resets query/result state and invalidates in-flight responses. A newer search/page request invalidates an older response. Unmounting never applies delayed results. Loading/error state belongs to the current request, not a previously selected pack.
- A result opens `/reader/${encodeURIComponent(packId)}/${encodeURIComponent(articleKey)}` through the existing router. Do not double-decode the route key. Preserve the existing unsaved annotation/capture guard; search navigation may not silently discard a draft or interrupt a save.
- Keep production styles/accessibility consistent with the existing Reader. Buttons are keyboard reachable, result links have meaningful names, and status changes are announced without moving input focus unexpectedly.

## Affected boundaries

Native: `apps/reader/src-tauri/src/ipc/reader.rs`, `ipc/dto.rs`, and `src/lib.rs` registration. Frontend: `apps/reader/src/services/readerService.ts`, new `routes/reader/ReaderTitleSearch.tsx`, existing `ReaderView.tsx` and scoped Reader styles if needed. Reuse the current archive implementations; do not rewrite parser, transfer, storage-fit policy, media capture, entitlement, account or mesh code.

## Acceptance

Production code typechecks/compiles on its supported compositions. Actual article titles from a real verified resource appear, pagination has no invented count, errors do not impersonate zero results, stale responses cannot replace a newer request, and opening a result respects the existing draft guard. The Windows packaged demonstration starts from a real retained pack, searches for an actual retained article not already open, opens and reads it, then repeats after quitting/restarting with internet unavailable. Record artifact/source/fixture identities and timecodes. Android runtime execution is not required for this Windows shared-function iteration; other platform build outcomes remain separately reported.

No synthetic archive, mocked IPC service, invented catalog/customer data, or substitute success may be introduced for acceptance. Use real existing resource data and production archive/IPC paths. Source/build success alone does not complete the iteration. The preauthored checklist/rubric, coder implementation, supervising semantic correction, and real continuous recording precede the COMPLETE mark and the next change's architecture amendment.
