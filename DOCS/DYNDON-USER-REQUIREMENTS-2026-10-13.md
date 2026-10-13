# DynDon user requirements — 2026-10-13

Status: agreed requirements, not implemented or verified features. **Non-blocking for initial release**, by explicit user decision 2026-10-13. Preserve the existing native acquisition/offline/annotation goal. These decisions supplement the current architecture; detailed entities, design and executable checklist contracts remain required before coding.

## Desktop storage destinations

Allow users to choose a writable folder on another partition or external drive, persist that destination and install there. Existing backend registered destinations are not a substitute for a usable folder chooser. Show the actual selected path and check its filesystem capacity and availability.

## Suggested storage maximum

Operator refinement, 2026-10-13 17:54 EDT: provide a storage-budget slider synchronized with an editable numeric value. Derive its valid range from the destination capacity and the applicable metadata/workspace policy. Display the minimum and overhead before planning; do not accept a value only to reject it after a wait. The existing policy minimum is 16 MiB; lowering that minimum requires a reviewed policy change rather than removing the storage checks.

Replace the arbitrary 1 GiB dialog default with a recommended maximum derived from actual free space on the selected destination, minus an explicit safety reserve. Include metadata and temporary working space in budget accounting. The reserve policy is not yet finalized. Explain the recommendation, allow edits, recompute on destination change and recheck before transfer. A budget is a ceiling, not an instruction to fill the disk. At the 01:18 EDT observation, / had 84 GB free and /home 24 GB; these are historical observations, not constants or a continuing shortage.

## Installer-style content selection

Operator refinement, 2026-10-13 17:55 EDT: use a topic tree selector and, where source metadata supports it, begin background cluster mapping on branch hover or selection so calculation overlaps the user's choices. Support keyboard focus and touch selection as equivalent triggers. Prioritize selected branches, deduplicate shared work, reuse source-identity-bound mappings, and update selected coverage/storage incrementally. Coalesce rapid changes and cancel or discard stale selection results; bound metadata traffic, disk use and concurrency. Show partial versus settled calculations honestly. Preparation may retrieve bounded planning metadata; hovering must not install article payloads or commit a download. Final confirmation rechecks the latest selection, destination capacity and budget. Real topic-to-entry metadata remains a prerequisite; do not fabricate topics from clusters or imply this behavior is implemented.

Provide expandable domain/topic checkboxes with mixed parent states. Users can select a few subjects deeply while excluding whole other domains, within a fixed size limit. Depth/coverage must be backed by real source-matched metadata; existing priority-tier limits do not establish topic-specific semantic depth. Compute real shared-cluster and required-resource cost without double counting. Do not fabricate a taxonomy or silently discard explicitly required selections.

## Unavailable article links — simplified interaction

The user's final direction supersedes the proposed multi-state visible icon taxonomy: grey links whose target is unavailable in the local collection, keeping them clickable and accessible. Offline, explain that the page is not downloaded. With connectivity, offer two explicit choices: Open live page, or Download this section for offline use. Open live page uses the actual original web address when it can be resolved reliably and opens it in the system browser; do not invent an original URL from the archive path. If no reliable original address is known, explain that limitation rather than showing a broken live action. For the download choice, identify the section to add, check the real target and calculate incremental required cluster/resource storage, counting retained data once. If it fits within the collection budget and current destination capacity, ask whether to download more. On confirmation, acquire the needed content, restore normal link appearance and open the page. Unknown target/size must be checked before making an availability or size claim. Confirmed missing targets must not offer a fictitious download.

If the addition cannot fit, explain the shortfall and offer to adjust the storage limit or collection. Never silently evict selected/pinned content, exceed the limit or override explicit exclusions. Link availability must update after acquisition, eviction or loss of the storage destination. Keep annotations durable across collection changes.

## Page-local link availability — explicit implementation constraint

Do not walk or scan the whole ZIM to classify links. During rendering of the current article, inspect its internal links and use the existing local indexed availability data to determine whether each target is readable. Deduplicate repeated targets and use bounded lookups/batches; the work scales with links on the displayed page, not archive entry count. Grey unavailable targets, keeping them clickable. Missing local index coverage means unavailable locally, not confirmed nonexistent upstream. Do not prefetch remote targets or compute section download sizes just to style links. Resolve the live address or calculate incremental acquisition only on the relevant user action. This requirement concerns link demarcation; it does not claim that initial collection planning needs no metadata.

## Enhancement acceptance evidence (not an initial-release gate)

Operator version-identity clarification, 2026-10-13 17:54 EDT: show the artifact version in a consistent standard location across platforms, using the automatically generated build version. Never manually enter or hard-code the displayed artifact version. Propagate the same generated identity through packages and the visible UI.

DynDon is central product value: demonstrate chosen destination, suggested budget, meaningful selected coverage, bounded acquisition, offline restart/read/annotation and optional expansion through a grey link. These requirements do not mean the current artifact passes that journey. Rebuild all platforms through scripts/build-all.sh; commit and verify GitHub pushes for each completed task. Forgejo remains unavailable by operator instruction.


## Incremental acquisition and pruning — core accounting invariant

Operator clarification, 2026-10-13 17:59 EDT: source-identity-bound reusable
cluster mappings are fundamental to both incremental additions and removing/paring
the retained ZIM collection. Track dependencies from retained entries/resources
to shared clusters. Additions count only newly required payload and metadata;
pruning frees a cluster only when no retained or pinned dependency still needs it.
Account for actual decoded workspace and metadata independently of payload.
Update local availability atomically with retained-content changes and preserve
annotations. Never infer a cluster can be deleted merely because one topic was
unselected. A changed archive identity invalidates the old mapping. These are
requirements for the future selection/pruning implementation, not verification
of current behavior.
