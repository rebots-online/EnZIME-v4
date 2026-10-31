# Proposed Windows EXE screencast gauntlet and specification coverage

Date: 2026-10-25. Status: DESIGN DRAFT; NOT RUN. No requirement has been marked passed by this document.

## Purpose, scope, and authority boundary

The user selected the built Windows EXE as the present demonstration target for general/shared codebase functionality. Android execution is not needed for this gate. Tests must show real end-user paths through the packaged application, not a Vite dev server, an isolated library test, static screenshots, or simulated successful service responses.

The existing `DOCS/TEST_RUBRIC.md` is a July DyNDon M1/AppImage rubric. Preserve it as historical scope/evidence. This draft proposes the expanded Windows acceptance contract, covering the 27 functional requirements in `DOCS/REBOOT-PAK/PRD.md`, its required surfaces/non-functional requirements, the baseline architecture contracts/budgets, and the companion topology's proposed capabilities.

Current KStore Admin-Manual/reminder retrieval failed (404/500/timeouts). KStore remains the sole authority. The older local rubric's SC4/TC11 citations do not establish the current global rules, current archival destination, or current publication policy. Those must be reconciled before this draft is finalized or executed. The user's explicit Windows/no-Android-test instruction controls the present platform scope.

The scope is Windows shared-behavior acceptance. It does not claim Android lifecycle/permissions/keystore correctness, Linux/AppImage packaging, PWA installation/browser storage behavior, or real radio propagation from a Windows-only observation.

## Requirement-to-evidence ledger

Before execution, enumerate every binding requirement into a coverage ledger. No normative clause may disappear just because no UI path or test exists. Include:

1. All 27 `FR-*` rows below.
2. Every PRD required surface and principle, reliability/security/privacy/accessibility clause, and release story.
3. Architecture invariants, IPC/storage/format contracts, every primary-view state from section 19.3, and every applicable desktop performance budget in section 21.2.
4. Effective baseline amendments, and every approved new auth/sync/topology requirement.

Each ledger entry has requirement ID/source anchor, observable claim, scope, implementation reachability, preconditions, fixture IDs/digests, EXE/commit hash, Windows profile IDs, exact actions, expected results and failure triggers, test IDs, required supporting probes, screencast file/hash/timecodes, actual observations, status, defect IDs, and reviewer/operator attestation. Named source anchors remain stable even if a document's line numbers change.

The preparation gate requires 100% of in-scope requirements mapped to concrete scenarios, no uncovered normative clauses, no missing evidence paths, and no unexplained scope exclusions. A placeholder scenario title is not coverage. New requirements extend the ledger and invalidate a previously complete coverage count until mapped.

## Current implementation boundary

The current EXE build has a successful compile and brief startup observation. It has not run this gauntlet. Source inspection shows only the M1 command set registered in the shell. A crate, UI control, or documented DTO without a reachable production path is UNIMPLEMENTED/UNPROVEN for end-to-end acceptance.

The future runner must probe the actual command registry and reachable UI actions before starting a full run. It must record missing implementation as a gap, not fabricate responses, weaken assertions, or quietly change the fixture. A partial debugging run is never promoted to an all-specification verdict.

## Windows scenario rubric

Every scenario below starts NOT RUN. Existing or missing source implementations do not change that into PASS.

| Scenario | Requirement coverage | Observable success and decisive failure triggers |
| --- | --- | --- |
| W-BOOT | Home/Setup, readiness, build identity | Packaged EXE starts from a clean protected profile, shows the correct version, supports completion without a model/account, and renders useful empty/error states. Blank UI or a fabricated readiness claim fails. |
| W-CATALOG | FR-PACK-001; Library/catalog surfaces | Real catalog search/filter and installed packs/storage roots render through production IPC/storage. Known fixture entries match the catalog, with source and version visible. Empty-real state is tested separately. |
| W-DYNDON | FR-PACK-002; PRD download plan; M1 R-PLAN | A real resource exceeds the device's declared budget. The UI shows full size, selected subset, final/staging/index costs, source and destination. Selected bytes fit the effective reservation and are less than the full resource. A small full-pack download does not satisfy this row. |
| W-TRANSFER | FR-PACK-003, FR-PACK-004, FR-PACK-005; E2E-02 | Start a real selected-cluster transfer, show acknowledged bytes, pause/resume/cancel, terminate the process, restart and resume verified chunks. Corrupt staging bytes: rejection preserves the last active pack. Retry verifies and atomically activates. No optimistic timer may impersonate byte progress. |
| W-READER | FR-ZIM-001, FR-ZIM-002, FR-ZIM-003; E2E-01 | Import/acquire a verified real ZIM, render known text/resources, navigate links/back/forward and title/available full-text search, restore reading position. Disable all network routes, restart and read cached selections. Missing uncached content is labeled unavailable. Malicious HTML cannot access the native bridge. |
| W-ANNOTATION | FR-ANN-001, FR-ANN-002, FR-ANN-003; Reader/Notes | Create a highlight, comment, ink and voice asset, notebook and tags; restart offline and verify content/source anchoring. Relocation fixtures produce resolved, ambiguous, unavailable and orphaned states accurately. Evicting source bytes preserves personal work. |
| W-SIDECAR | FR-SIDE-001, FR-SIDE-002; E2E-03 | Export a deterministic signed sidecar, inspect signer, plan conflicts, import into clean profile B, and re-import without duplicates. Wrong key/tampered content is rejected/quarantined. Imported layer visibility does not alter local notes. |
| W-LOCAL-AI | FR-AI-001, FR-AI-002, FR-AI-003, FR-AI-004, FR-CITE-001; E2E-04 | Install a verified compatible local model; exercise each declared Ask scope, bounded retrieval/context, local streaming and cancellation without internet. Every citation opens its exact supporting passage. Unknown citations are rejected. No-model and incompatible-model states remain useful; removing a model does not destroy conversations/citations. |
| W-LORE | FR-LORE-001, FR-LORE-002; E2E-05 | Produce an evidence-linked candidate without auto-accepting it; accept/reject/correct/pin/delete. Verify current revision in retrieval and absence of deleted text in current retrieval/diagnostic export; stale replica replay does not resurrect deletion. |
| W-ENTITLEMENT | FR-ENT-001, FR-ENT-002, FR-ENT-003; E2E-06 | Exercise normalized provider events through the real configured integration or an explicitly scoped contract fixture; label fixture-only proof honestly. Verify signed perpetual ownership after offline restart, term expiration scope, invalid/duplicate coupons and eventual reconciliation. User data/export remain accessible. Live purchase-provider acceptance cannot be inferred from a signed-grant fixture. |
| W-BACKUP | FR-BACK-001, FR-BACK-002; E2E-07 | Create/verify an encrypted backup, show conflict plan, restore into a genuinely clean protected profile offline. Recover notes/assets/notebooks/conversations/lore/settings/bindings. Missing resource bytes remain unavailable. Wrong password, corruption and interrupted restore cannot destroy the previous state. |
| W-DIAGNOSTICS | FR-DIAG-001; privacy | Preview/export diagnostics with useful failure evidence and inspect exclusions. No account secrets, private note text, token, raw voice, or private key may appear in the public evidence bundle. User-facing state remains understandable. |
| W-ACCESSIBILITY | FR-A11Y-001; accessibility/UI state matrix | Complete every core flow with keyboard and screen reader; demonstrate focus and accessible live updates, high contrast, 200% text, reduced motion, and status beyond color. Exercise loading/empty/populated/recoverable/fatal/offline/denied states per primary view; responsive desktop resizing is not Android acceptance. |
| W-IDENTITY | Proposed offline identity/account linking | Use a new local profile without Firebase; link an account online, restart offline, continue authorized local use after the cloud token's normal lifetime, and distinguish local unlock from fresh cloud auth. Two independently populated devices signing into one account retain separate spaces until explicit enrollment/import. Existing encrypted-space access requires trusted-member approval or a recovery key. Preview/cancel import without changing the original data. New cloud login/privilege changes fail honestly when offline. Wrong-profile/private data never leaks. |
| W-INDEPENDENT-FIT | Proposed per-device DyNDon | Isolated Windows profiles A/B use the same manifest but different explicit budgets and different installed subsets. Shared reading/annotation metadata converges; a quota change/eviction on A does not evict B's data or the home master. Show the actual resident-byte accounting, not two mock budget labels. |
| W-LAN-MASTER | Proposed one-full-copy resource topology | On a real LAN with internet unavailable, Windows clients use one verified home resource source, fetch verified ranges and read uncached data. Disconnect the source: cached content works, uncached content becomes unavailable. A single-host loopback test is protocol evidence only, not physical LAN acceptance. |
| W-METADATA-SYNC | Proposed transport-independent sync | Independent Windows profiles exchange real bookmark/note/reading changes; stop/restart mid-sync, reorder/duplicate deliveries and create concurrent edits. Show converged projections or explicit conflicts with no lost work. Verify crash-safe outbox/inbox, per-peer acknowledgment, and replay protection. |
| W-CLOUD-REJOIN | Proposed Firebase relay | With LAN absent, authorized Windows profiles exchange real metadata through the configured Firebase backend, then edit offline and reconnect. Show durable pending state, server rejection handling, deduplication when an operation traveled by both LAN and cloud, and account separation. Backend unavailability is not a PASS. |
| W-PAIRING | Proposed trust/revocation | Offline owner-approved pairing verifies fingerprints, scoped grants, and challenge-response. Revoke a device with an owner-signed history checkpoint and rotate keys: informed replicas deny future epoch keys and quarantine operations outside that checkpoint regardless of timestamp or whether a partitioned replica previously projected them. Deliver revocation/edit in opposite orders and include an authorized dependent edit; show a common explicit branch conflict, owner-reviewed recovery/rebase as new authorized operations, and eventual equal canonical projections. Retain recoverable work without claiming remote erasure of old data. Uninformed peers enforce the update only after reconciliation. |
| W-RADIO | Proposed optional constrained transports | A bounded recorded/synthetic link can test codec, limits, queueing and replay behavior, clearly labeled protocol-only. Real Meshtastic/LoRa/Helium acceptance requires compatible real radios/gateways, e.g. attached to Windows hosts. Show compact allowed messages arriving and bulk content refusing this transport. Windows synthetic playback alone is not physical-radio acceptance. |

Every functional requirement is explicitly covered above; optional-provider fixture evidence and future topology acceptance retain their scope labels. Required features that are not implemented remain gaps in the complete-product verdict.

## Non-functional and structural evidence

A screencast establishes visible semantic behavior. It cannot by itself prove every property of cryptography, sandbox isolation, crash atomicity, dependency parity, or timing distribution. Each such requirement gets both its visible story and the declared supporting evidence; neither silently substitutes for the other.

- Reliability: show real process termination and persistence recovery; inspect transaction/checkpoint/activation evidence. Exercise migration rollback/backup, foreign-key violations and invalid job transitions with controlled fixtures.
- Security: invalid signatures/digests, traversal, oversized/decompression limits, untrusted HTML/native-bridge isolation, key protection, membership/replay failures, and quarantine need bounded adversarial fixtures and observed refusal. Do not weaken controls to make the video pass.
- Privacy: inspect exported evidence and diagnostics, explicit network policy, profile isolation, deletion state and user export. Recording must not expose passwords, recovery material or private tokens.
- Performance: map each applicable desktop budget in architecture section 21.2. Record actual reference hardware/OS, EXE/commit and fixture/model digests, cold/warm state, sample count, p50/p95/max, and profiler evidence. Desktop shell <=2.5 s, warm article <=250 ms, title search <=150 ms, annotation acknowledgment <=100 ms, and the remaining stated budgets retain their actual thresholds. A single fast video frame is not a distribution. Hardware that does not match the declared reference does not certify that reference's budget.
- Contracts: map all section 19.2 CT IDs to supporting contract evidence and their corresponding visible flows. Compile checks and dependency inclusion are prerequisites, not proof that the EXE exposes a capability. Android graph/build checks can be separately recorded when authorized; Android runtime execution is excluded from this current gate.
- Surfaces: Home, Setup, Library, Reader, annotation workspace, Ask, Notes, Memory, Settings and Download Plan each need the applicable section 19.3 UI-state matrix, not just one successful screenshot.
- Platform delivery: Linux/AppImage and browser/PWA install/update/storage tests remain separate platform evidence. They are neither silently passed nor required Android runs for the present Windows shared-function gate.

## Controlled environments and fixtures

Profiles A/B have independent database roots, object stores, device identities and storage budgets. A future implementation must offer a safe supported profile-selection/test-isolation seam; no command-line profile override is claimed to exist today. Tests must never rename/delete the operator's real data directories or copy live authentication secrets.

Use redistributable real ZIM/model fixtures with known text, links, resources, versions and SHA-256 digests; signed and intentionally damaged variants; a pack whose complete footprint exceeds both test budgets; known concurrent note edits; stale operation/tombstone fixtures; clean migration/backup states; 10,000-annotation performance data; and reference-hardware declarations. Fixtures must carry expected answers so a real article cannot be confused with placeholder content.

Storage limits are enforced by the supported production policy and independently measured storage usage, not fabricated free-space responses. If the architecture adds a test-only quota fixture, it must be explicitly labeled and cannot replace a real low-space/ENOSPC acceptance case.

The network matrix distinguishes: (a) internet+LAN, (b) LAN only with internet egress blocked, (c) no network, (d) home source unavailable, (e) interrupted/reordered/lossy transport, and (f) constrained radio. Record the actual controls and route/probe evidence. A browser's generic online flag is not proof that Firebase or the LAN node is reachable.

Cloud-sync acceptance uses the configured real service; emulator/fixture tests are separate preparatory evidence. LAN acceptance needs a real second Windows host or equivalent independently networked Windows guest with the stated limitations. Neither Android nor a phone is required. Radio cases use Windows-connected real radios when hardware acceptance is requested; otherwise remain protocol-only/unproven for RF behavior.

## Recording and evidence bundle

Record the packaged Windows application and the controlled fixture actions. Each scenario includes preconditions, start state, actions, observable expected result, deliberate failure/recovery when specified, and final durable state after restart. Editing cannot splice away an error while representing a continuous successful run. Timecodes tie the requirement ledger to the actual recording.

The bundle contains EXE SHA-256/version/commit, source/architecture/rubric revisions, fixture digests, hardware/OS, redacted effective network/storage configuration, scenario ledger, video file/hash/timecodes, screenshots as supporting material, supporting contract/profiler/verification output, defect records, and operator attestation. Evidence is reviewed for secrets before any publication. The final archival/publication destination is pending current KStore authority reconciliation; do not create a competing local Admin-Manual copy.

## Status and verdict rules

Per row: NOT RUN, UNIMPLEMENTED, BLOCKED, FAIL, or PASS. A platform-specific item outside the user-selected Windows scope is explicitly OUT OF SCOPE and cannot count toward a global multiplatform claim. A missing shared capability is never labeled out of scope merely to achieve a green result.

- A row passes only when its specified evidence and semantic observation exist for the exact artifact/fixtures.
- Any in-scope observed failure makes the Windows gate DEFECTIVE.
- Any in-scope UNIMPLEMENTED/NOT RUN/BLOCKED row prevents a complete pass and leaves the gate INCOMPLETE; no percentage score or waiver converts it to PASS.
- All in-scope rows passed with coverage checked, but no operator review: INCOMPLETE, pending operator.
- All in-scope rows passed and operator attested: WINDOWS SHARED-FUNCTION GAUNTLET PASSED for the recorded artifact and scope. This is not an all-platform release verdict.
- M1-only completion is reported as M1-only evidence; it never certifies the full PRD or the new sync architecture.

An append-only run record preserves failures and later successful reruns. Every rerun identifies the changed artifact/fixture or explicitly states the unchanged ones. No previous build/startup success is retroactively promoted to a gauntlet result.

## Current deliverable boundary

This is an architecture/rubric proposal only. No scenarios were executed, no screencast was recorded, no Firebase setup or radio integration occurred, and no application code or binary was built during this design work. User design review and current KStore reconciliation precede an implementation plan or test execution.
