# EnZIME Reader Reboot IV — Implementation Kickoff Prompt

Copy the prompt below into the coding agent from the extracted project root.

---

You are implementing EnZIME Reader from a closed-world reboot pak.

## Mission

Ship the complete Android, Linux and Windows EnZIME Reader described by:

1. `DOCS/REBOOT-PAK/PRD.md`;
2. `DOCS/REBOOT-PAK/ARCHITECTURE.md`;
3. `DOCS/REBOOT-PAK/CHECKLIST.md`;
4. `DOCS/REBOOT-PAK/VENDOR-MANIFEST.md`;
5. machine-readable files under `DOCS/REBOOT-PAK/STITCH/`.

The only implementation inputs are this pak, its explicitly included vendored
source and ordinary package registries.

Do not inspect, clone, copy or infer from a previous EnZIME repository.
Do not rely on a prior conversation or operator memory.

## First response

Before changing source:

1. print the current repository root;
2. list the reboot-pak files;
3. list the vendor directories;
4. report whether `VENDOR-MANIFEST.md` status is `VERIFIED`;
5. run every mechanically checkable row in
   `DOCS/REBOOT-PAK/audits/cold-start-audit.md`;
6. report PASS or FAIL with exact failed rows;
7. stop on FAIL.

Do not “work around” a failed cold-start audit.
Correct the pak.

## Authority

Use the authority order in `ARCHITECTURE.md` §0.

When a task, entity or schema is ambiguous:

- stop;
- identify the exact ambiguity;
- correct `ARCHITECTURE.md`;
- regenerate affected contract/checklist material;
- resume only after the cold-start/contract audit passes.

Do not invent a duplicate entity or compatibility alias.

## Historical terminal failures you must actively prevent

### 1. State contamination

Earlier reboots copied a historical checklist carrying `[X]`/`✅` state and
stale assumptions.

Rules:

- every initial task marker must be `[ ]`;
- never infer completion from a prior repository or transcript;
- advance only the current task after current evidence;
- an implementation claim is not verification.

### 2. False “empty tree” premise

The tree is not empty when vendor source is present.

Rules:

- inventory vendor inputs before planning;
- treat vendor code as locked source, not generated application modules;
- never overwrite or casually patch vendor code;
- record any unavoidable vendor patch as a manifest entry and ADR.

### 3. Bulk-module resurrection

An earlier attempt made dozens of unfinished modules compulsory through module
declarations, producing a terminal compile-error forest.

Rules:

- create only modules needed by the current vertical slice;
- compile exact platform graphs after every slice;
- never copy a historical `lib.rs`;
- no unused feature module is declared “for later”;
- no release module is a stub.

### 4. Cross-platform dependency collapse

Desktop and billing machinery previously leaked into targets that did not need
them.

Rules:

- no default Cargo platform feature;
- exactly one platform feature per release build;
- Android excludes GGUF desktop adapter;
- Linux/Windows exclude native RevenueCat SDK;
- CI audits each dependency graph.

### 5. Architecture-to-checklist information loss

Earlier tasks told coders to “consult architecture,” left inputs implicit, or
premised work on another task’s unverified output.

Rules:

- a dispatched task includes exact allowed files, entities, Do, Verify and
  Accept;
- never broaden task scope;
- if its input is absent, stop rather than fabricate;
- maintain a complete task-to-entity map.

### 6. Feature substitution

Earlier work substituted a convenient but nonexistent or wrong feature for the
specified capability.

Rules:

- verify every dependency feature against the locked source/package metadata;
- never guess a Cargo feature name;
- never weaken acceptance to fit an available API;
- record an ADR when a verified API forces a contract correction.

### 7. Premature gates and attestation

Earlier work assigned gates and reported success before repository state
supported the claim.

Rules:

- gates observe artifacts and tests;
- no agent attests its own unsupported claim;
- every acceptance statement links to command output or a test artifact;
- compile success does not equal product acceptance;
- UI prompt submission does not equal a generated screen.

### 8. UI vocabulary drift

Generated screens can invent nouns and backend semantics.

Rules:

- `ARCHITECTURE.md` §16 owns names;
- §24 owns UI bindings;
- Stitch output is a visual draft;
- reject a screen that renames a component or invents state;
- do not alter backend contracts to fit a visual draft.

### 9. Placeholder public release

Earlier “complete” builds retained stubs, bypass gates and captions unsuitable
for users.

Rules:

- production contains no TODO/FIXME/stub/mock/coming-soon path;
- a visible control completes its action or is absent;
- entitlement has no bypass;
- privacy, third-party notices, diagnostics, backup and restore are release
  work, not polish;
- run the placeholder gate on every release candidate.

### 10. Unverified remote evidence

Rules:

- do not claim Stitch project or screen IDs without tool output;
- do not claim an attachment was read unless it was available;
- do not claim a vendor revision/tree digest without verification;
- preserve `SOURCE-GAPS.md` until every gap is actually closed.

## Working method

Implement vertical slices in checklist order.

For each task:

1. change marker `[ ]` to `[/]`;
2. confirm allowed files;
3. implement only the task;
4. run its Verify commands;
5. compare observed output to Accept;
6. if accepted, commit source with:

   ```text
   <type>(<scope>): <task-id> <task title>
   ```

7. make the marker update the final change;
8. mark `[X]` for implemented evidence awaiting independent verification;
9. an independent verifier reruns Verify and marks `✅`.

Never batch unrelated checklist tasks into one commit.

## Initial implementation sequence

1. cold-start and vendor verification;
2. repository/toolchain skeleton;
3. domain/port/SQLite/TypeScript contracts;
4. separate green Android/Linux/Windows shells;
5. real ZIM import/read/search/annotation vertical slice;
6. library/catalog/plan/resumable transfer;
7. Notes and signed sidecars;
8. local model installation and cited Ask;
9. lore memory;
10. entitlement, backup, restore, diagnostics;
11. complete UI bindings;
12. performance/security/accessibility/release evidence.

## Non-negotiable product behaviors

- Reader works without AI.
- User work survives source/model removal.
- Unverified bytes never activate.
- Failed update leaves last known good readable.
- Grounded answers cite exact stored evidence.
- Model output never silently becomes memory.
- Perpetual capability remains locally verifiable offline.
- User data remains readable/exportable after entitlement change.
- Every primary UI component maps through §24.

## Definition of done

Done means all:

- checklist tasks independently verified;
- Android AAB/APK builds;
- Linux AppImage/DEB builds;
- Windows MSIX/installer builds;
- seven E2E stories pass;
- entity-table validation passes;
- no forbidden production placeholder;
- capability evidence says `passed`;
- accessibility, privacy and threat reviews complete;
- signed release manifest and SBOM produced;
- clean-install backup restore succeeds offline;
- package contains no secret;
- `SOURCE-GAPS.md` is empty or removed only because every listed item was
  genuinely resolved.

Begin with the cold-start audit.

---

