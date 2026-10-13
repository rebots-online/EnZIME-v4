# DynDon user requirements — 2026-10-13

Status: agreed requirements, not implemented or verified features. Preserve the existing native acquisition/offline/annotation goal. These decisions supplement the current architecture; detailed entities, design and executable checklist contracts remain required before coding.

## Desktop storage destinations

Allow users to choose a writable folder on another partition or external drive, persist that destination and install there. Existing backend registered destinations are not a substitute for a usable folder chooser. Show the actual selected path and check its filesystem capacity and availability.

## Suggested storage maximum

Replace the arbitrary 1 GiB dialog default with a recommended maximum derived from actual free space on the selected destination, minus an explicit safety reserve. Include metadata and temporary working space in budget accounting. The reserve policy is not yet finalized. Explain the recommendation, allow edits, recompute on destination change and recheck before transfer. A budget is a ceiling, not an instruction to fill the disk. At the 01:18 EDT observation, / had 84 GB free and /home 24 GB; these are historical observations, not constants or a continuing shortage.

## Installer-style content selection

Provide expandable domain/topic checkboxes with mixed parent states. Users can select a few subjects deeply while excluding whole other domains, within a fixed size limit. Depth/coverage must be backed by real source-matched metadata; existing priority-tier limits do not establish topic-specific semantic depth. Compute real shared-cluster and required-resource cost without double counting. Do not fabricate a taxonomy or silently discard explicitly required selections.

## Unavailable article links — simplified interaction

The user's final direction supersedes the proposed multi-state visible icon taxonomy: grey links whose target is unavailable in the local collection, keeping them clickable and accessible. Offline, explain that the page is not downloaded. With connectivity, offer two explicit choices: Open live page, or Download this section for offline use. Open live page uses the actual original web address when it can be resolved reliably and opens it in the system browser; do not invent an original URL from the archive path. If no reliable original address is known, explain that limitation rather than showing a broken live action. For the download choice, identify the section to add, check the real target and calculate incremental required cluster/resource storage, counting retained data once. If it fits within the collection budget and current destination capacity, ask whether to download more. On confirmation, acquire the needed content, restore normal link appearance and open the page. Unknown target/size must be checked before making an availability or size claim. Confirmed missing targets must not offer a fictitious download.

If the addition cannot fit, explain the shortfall and offer to adjust the storage limit or collection. Never silently evict selected/pinned content, exceed the limit or override explicit exclusions. Link availability must update after acquisition, eviction or loss of the storage destination. Keep annotations durable across collection changes.

## Release evidence

DynDon is central product value: demonstrate chosen destination, suggested budget, meaningful selected coverage, bounded acquisition, offline restart/read/annotation and optional expansion through a grey link. These requirements do not mean the current artifact passes that journey. Rebuild all platforms through scripts/build-all.sh; commit and verify GitHub pushes for each completed task. Forgejo remains unavailable by operator instruction.
