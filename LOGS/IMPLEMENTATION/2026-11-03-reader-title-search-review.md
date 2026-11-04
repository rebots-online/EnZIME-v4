# P2-05-TITLE — source implementation and semantic review

Date: 2026-11-03. Scope: TITLE-01 through TITLE-04 source work only. NOT a packaged/runtime acceptance or completed parent checklist gate.

Architecture commit: `cd3e5b4a888bf3c8ce0ea45bc0bfb686908f73e0`.
Preauthored checklist/rubric commit: `38b8e4363d63344263f9f2643512bac8f5a8c851`.
Same-iteration pagination clarification: `572c729179b94a16837ef85c3ceadce4d15781d1`.
Coder: `/root/title_search_coder`. Supervising semantic reviewer: root. Independent source reviewer: `/root/checklist_audit`.

## Actual implementation

Seven production files provide native command/DTO/handler registration, the typed frontend service, a focused Reader panel/styles, and integration through the existing router navigator. Search calls the actual installed archive. No mocked IPC/archive, invented resource data, fake result branch or temporary customer flow was added.

## Semantic checks

- Registration, Rust camelCase serialization and JS arguments agree. The native handler validates the archive even for an empty query, and releases the archive-state lock before awaiting work.
- Query normalization/256-scalar bound, default/clamped page sizes, actual lookahead and the canonical offset ceiling are explicit. Offset requests are not silently clamped. Real `hasMore` is preserved at the refine-query boundary; no total count is fabricated.
- The actual retained range composition uses `DiskEntryCatalog` entries joined to owner clusters with state `complete`; search does not enumerate remote/unretained-only titles.
- Input edits, subsequent requests, route-pack remount and layout-effect cleanup invalidate old responses. Genuine errors are separate from actual empty results. An empty later page retains Previous and does not imply the whole query has no matches.
- Result links encode complete pack/article keys and call the parent's normal `navigate`, preserving the existing unsaved annotation/save blocker and avoiding duplicate decoding.
- Root reviewed the actual diff/new component. The independent reviewer found no must-fix defect in this current production composition. A possible future adapter with both a 100-hit cap and a 10,000-offset cap would need its own explicit terminal lookahead policy; that is not the currently composed retained range adapter, whose limit is 200.

## Executed source verification

- `pnpm exec tsc --noEmit`, cwd `apps/reader`: exit 0; rerun after final empty-page correction also exit 0. Log: `E:\DevTools\caches\build-temp\enzime-title-typecheck.log`.
- Dot-source `E:\DevTools\pearl-build-env.ps1`; `cargo check -p reader`: exit 0, native dev check completed in 3m39s. Log: `E:\DevTools\caches\build-temp\enzime-title-native-check.log`.
- `git diff --check`: passed for the actual source delta.

Those commands are source checks, not substitute release builds. The sole release build invocation remains `bash scripts/build-all.sh`.

## Still unproven

TITLE-05 is not complete: real packaged Windows title results, distinct page transitions, navigation/draft preservation, offline restart, and the required continuous recording have not yet been observed. The real Kiwix acquisition and build-tool route are being prepared. Parent P2-05/P2-07 and release gates retain their incomplete states. Any runtime defect returns to the same coder for correction and a newly published source commit before re-recording.
