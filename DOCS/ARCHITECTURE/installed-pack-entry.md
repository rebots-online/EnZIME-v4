# Installed pack entry repair — 2026-10-12

Observed in packaged 1.13.63942: live Kiwix Football Database downloads 7.4 MB of a 108 MB archive, installs 549 articles, but PackGrid navigates to fabricated key A. Reader reports catalog_transfer_allowance after looking for that nonexistent key. Installation already decoded the actual main page. Existing index/CodeGraph identified PackGrid and readerService; targeted reads establish the exact handoff.

| Entity | Target | Contract |
|---|---|---|
| PackGrid.openInstalledPack | apps/reader/src/routes/library/PackGrid.tsx:25 | async (packId:string)=>Promise<void>; call existing readerService.readerOpen(packId), require matching packId and nonempty mainPageKey, then navigate with encodeURIComponent on both identities. One pending request disables all tiles; a ref guards same-tick duplicate clicks; mounted guard rejects late navigation. Error stays in Library with retry by clicking tile. Never invent A or another default. |
| OpenPackResponse.mainPageKey | apps/reader/src/services/readerService.ts:19 | string or null, matching Rust Option. Existing store open must reject missing key before fetching an article. |
| PackGrid entry regression | apps/reader/src/routes/library/pack-entry.test.tsx:1 | Real router and component, mocked native invoke only. Prove C/path with slashes, spaces and percent characters reaches exact decoded route; no invented A; pending double click is one request; error/missing key retry; unmounted completion does not navigate. |
| Reader error palette | apps/reader/src/routes/reader/ReaderStyles.tsx:180 | Error/empty containers use opaque dark chrome background and explicit foreground; preserve light article canvas. Runtime screenshot required separately. |

This repairs existing interactions, not a novel UI. Native trial and all-platform rebuild remain separate from component acceptance. No cache allowance relaxation or alternative ZIM implementation.
