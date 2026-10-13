# EnZIME / DynDon remaining work

User instruction, 2026-10-13: display the current colour-coded remaining-work list at the top of every EnZIME turn, including status/discussion turns, before other commentary or tools. Follow it with a short status statement. Colours classify work, not completion: white project, purple planning, orange implementation, blue build/publication, green verification. Keep pending items visible; remove or revise them only from observed evidence. Distinguish implemented source, tested code, built artifacts, published artifacts and verified native behavior.

⚪ **EnZIME core release work and non-blocking DynDon enhancements**

- 🟠 Add a desktop folder/drive chooser and persist the selected destination.
- 🟠 Suggest an editable storage maximum from actual destination free space, accounting for reserve, metadata and working space; recheck before transfer.
- 🟠 Add expandable topic checkboxes and per-topic depth choices within the budget, using real source-matched content metadata.
- 🟠 Check internal links on the page being rendered against the local availability index; grey unavailable targets, keeping them clickable. No whole-ZIM walk or remote prefetch for link styling.
- 🟠 On a grey-link click: explain unavailability offline; online offer Open live page or Download this section. Resolve real targets, calculate incremental storage when requested, and confirm before acquisition. Do not silently evict content or exceed the budget.
- 🟠 Replace misleading placeholder statuses and disconnected controls with working behavior or explicit unavailable states.
- 🟠 Resolve packaged media support and outstanding Android-symbol / native Windows MSI-MSIX build requirements.
- 🔵 Rebuild all platforms together through scripts/build-all.sh and publish clearly identified artifacts. Commit and verify each completed task's GitHub push; Forgejo remains unavailable.
- 🟢 Verify real Kiwix partial download → read → offline restart → on-canvas drawing, typing, voice, photo and video annotation persistence, with incremental section-download verification deferred alongside that non-blocking enhancement. Voice recording and typing remain separate; automatic transcription is later work.

## Release scope

The first five implementation items (destinations, suggested budgets, topic selection, grey links and optional expansion) are still to do but **do not block release**, per the user on 2026-10-13. Keep them visible as non-blocking roadmap enhancements. Existing core correctness and release checks remain separate. See ROADMAP.md.

## Current evidence boundary

As of 2026-10-13 01:24 EDT, the new destination, suggested-budget, topic-selection and grey-link requirements are documented, not implemented. Native 1.13.63942 demonstrated live Kiwix partial acquisition, but failed the installed-pack handoff. The source repair passed 80 frontend tests and TypeScript checking and is pushed; rebuilt native acceptance remains pending. Unified 1.13 collected 7/12 required artifacts. Its Android symbols step failed on historical insufficient workspace storage; a later observation found / with 84 GB and /home with 24 GB free, so re-probe rather than assume that shortage persists. New native binaries are local, not published GitHub downloads.

Detailed product decisions: DYNDON-USER-REQUIREMENTS-2026-10-13.md. Executable coding contracts: ../CHECKLIST.md. This remaining-work view does not replace those contracts or mark unfinished features complete.
