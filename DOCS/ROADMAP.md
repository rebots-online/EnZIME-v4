# EnZIME roadmap

## Planned DynDon enhancements — non-blocking for release

These user-requested enhancements are **still to do, not implemented, and do not block the initial release** (operator decision 2026-10-13):

- Desktop folder/drive selection and persistent download destinations.
- An editable default maximum based on actual destination free space, with safety reserve, metadata and working-space accounting.
- Expandable topic checkboxes and per-topic depth choices within a size budget, backed by real content metadata.
- Grey unavailable internal links by checking links on the current rendered page against the local index; no whole-ZIM walk.
- When online, clicking a grey link offers **Open live page** or **Download this section for offline use**. Check incremental storage and request confirmation before downloading; do not silently evict content or exceed the limit. Offline, explain that the page is not downloaded.

Existing core functionality and release verification remain required; this deferral does not waive working acquisition, reading, offline persistence, annotations or truthful UI claims. See [detailed requirements](DYNDON-USER-REQUIREMENTS-2026-10-13.md) and the [remaining-work list](DYNDON-REMAINING-WORK.md).

## Release verification work

Continue the existing executable contracts in [CHECKLIST.md](../CHECKLIST.md) and [native test rubric](TEST_RUBRIC.md). The non-blocking enhancements above remain visible in the every-turn colour-coded list; their absence alone must not delay release.
