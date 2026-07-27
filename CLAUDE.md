# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this repository currently is

A **specification-only reboot pak**. There is no application source, no build system,
no `Cargo.toml`, no `package.json`, and no git repository yet. `codegraph init`
indexes zero files because every file here is Markdown or JSON.

The deliverable described by these documents is **EnZIME Reader** — an offline-first
multilingual ZIM knowledge reader, annotator and cited local-conversation product for
Android, Linux and Windows (Tauri 2 + Rust + React 19/TypeScript/Vite/Zustand, SQLite
with content-addressed object storage).

**Implementation has not been authorized.** See "Gates that block coding" below before
creating any source file.

## Authority order (ARCHITECTURE.md §0)

1. `DOCS/REBOOT-PAK/ARCHITECTURE.md` — the normative implementation contract
2. machine-readable pak files (`DOCS/REBOOT-PAK/STITCH/*.json`, `PAK-MANIFEST.sha256`)
3. `DOCS/REBOOT-PAK/PRD.md` — canonical product intent and acceptance
4. locked vendored source listed in `DOCS/REBOOT-PAK/VENDOR-MANIFEST.md`
5. fresh decisions, recorded as ADRs, that change no pre-committed public entity

If two authorities conflict, **implementation stops** and the conflict is resolved in
the higher authority. No agent may invent a second name for an entity already named in
`ARCHITECTURE.md`.

`CHECKLIST.md`, `README.md` and `DOCS/ARCHITECTURE.md` at other paths are byte-identical
copies of their `DOCS/REBOOT-PAK/` originals. **The pak copies are authoritative**; the
duplicates are convenience only and must not diverge.

## The closed-world rule

Implementation inputs are limited to: this pak, the exact verified vendor tree, and
ordinary package registries. Do **not** inspect, clone, copy or infer from a previous
EnZIME repository, a prior conversation, or operator memory. No absolute workstation
path (`/home/robin`, `~`) may appear in any instruction or source file.

## Gates that block coding

Implementation is blocked until all of these clear, in order:

| Blocker | Where | Current state |
|---|---|---|
| Unavailable source attachments | `DOCS/REBOOT-PAK/SOURCE-GAPS.md` | open |
| Vendor trees absent, digests unattested | `VENDOR-MANIFEST.md` | status is **not** `VERIFIED` |
| Stitch remote evidence not executed | `STITCH/generation-evidence.json` | no screen IDs claimed |
| Cold-start audit not run | `audits/cold-start-audit.md` | unrun (sections A–F) |
| Phase 0 gate | `CHECKLIST.md` `P0-GATE` | `[ ]` |

Vendor trees required at `VENDOR/`: Tauri tag `v2.11.1`, LiteRT-LM commit `1ff29cc`,
and AnZIMmermanLib (revision must be filled from supplied source). A missing revision,
dirty tree, floating branch or unrecorded local patch is a rejection condition.

Do not work around a failed cold-start audit — correct the pak. Do not compensate in
application code.

## Architecture: the parts that require reading several files

### Three rings, one dependency direction

- **Ring 1 — domain and durable data** (`crates/domain`, `crates/ports`,
  `crates/application`, `crates/storage-sqlite`, `crates/object-store`): entities,
  policies and the traits everything else implements. Depends on nothing outward.
- **Ring 2 — replaceable engines** (`zim-anzimmerman`, `catalog-openzim`,
  `transfer-*`, `inference-litert`, `inference-gguf`, `retrieval`, `lore`,
  `entitlement`, `revenuecat-adapter`): provider implementations behind Ring 1 ports.
- **Ring 3 — platform delivery** (`apps/reader/src-tauri`, `platform-android`,
  `platform-desktop`): command/event delivery only, no business logic.

Ports precede providers (invariant A-02). A port crate importing a provider crate is a
contract violation, and `P1-03`/`P1-06` acceptance tests assert it.

### The contract chain — why entity names are immovable

One typed contract flows through four surfaces and is validated mechanically:

```
Rust command registry (§14.4)  →  packages/contracts (generated TS, checked in)
        →  React components (§16.6)  →  §24 entity-to-UI binding table
```

`tools/contract-gen` fails the build when a component references an undeclared entity,
a command is missing from either Rust or TypeScript, a table name is absent from the
migration, a store slice is absent, or a component's public name differs from §16.6.
This is why "rename it to something clearer" is never a local change here.

The §24 table binds every primary component to its TypeScript entity, its Tauri
command(s), its SQLite table(s) and its Zustand slice. Read it before touching UI.

### Platform composition is exclusive, not additive

`ARCHITECTURE.md` §5.2: `default = []`. Exactly one of `android`, `linux`, `windows` is
enabled per release build. `android` excludes `inference-gguf`; `linux`/`windows`
exclude native RevenueCat SDK code. Contract tests `CT-PLAT-001` and `CT-PLAT-002`
enforce the exclusions, and CI audits each dependency graph separately
(`check-android-graph`, `check-linux-graph`, `check-windows-graph`).

A default platform feature is how a previous reboot died — see failure 4 below.

### Ten binding invariants (§4)

`A-01` closed-world bootstrap · `A-02` ports before providers · `A-03` platform graph
isolation · `A-04` last-known-good activation · `A-05` identity is not location ·
`A-06` user work outlives sources and engines · `A-07` untrusted bytes never become
active · `A-08` AI is bounded and cited · `A-09` commerce cannot revoke ownership ·
`A-10` UI follows the entity contract.

Practical consequence of `A-04`+`A-07`: a failed or corrupt update leaves the previous
verified pack readable; unverified bytes are quarantined with no "use anyway" path.

### Data posture

Primary user data (annotations, revisions, notebooks, user-authored messages, accepted
lore, entitlement artifacts, trust keys) is never coupled to an evictable archive or
model binary. Indexes, embeddings, chunk projections and caches are rebuildable and may
be discarded freely.

## Working method

Checklist tasks run in order across eleven phases (Phase 0 closed-world gate → Phase 10
release), 75 tasks, all currently `[ ]`. Each task carries **Allowed files, Do, Verify,
Accept** and is executable from its own text plus the pak — never broaden its scope, and
if a task's input is absent, stop rather than fabricate.

Marker workflow (`KICKOFF-PROMPT.md`):

```
[ ]  →  [/] taken  →  [X] implemented, awaiting independent verification  →  ✅ verifier reran Verify
```

Commit source before flipping the marker, one task per commit, never batching unrelated
tasks:

```
<type>(<scope>): <task-id> <task title>
```

The marker update is the final change in the task. An implementation claim is not
verification; every acceptance statement links to command output or a test artifact.

## Ten historical failures this reboot exists to prevent

From `KICKOFF-PROMPT.md` — these killed previous attempts and are the reason for the
rules above:

1. **State contamination** — a copied checklist carrying stale `[X]`/`✅`. Every initial
   marker is `[ ]`; never infer completion from a prior repository or transcript.
2. **False "empty tree" premise** — vendor source is locked input, not generated
   modules. Inventory it before planning; never casually patch it.
3. **Bulk-module resurrection** — declaring dozens of unfinished modules produced a
   terminal compile-error forest. Create only modules the current vertical slice needs;
   never copy a historical `lib.rs`.
4. **Cross-platform dependency collapse** — desktop and billing machinery leaked into
   targets that did not need them. Hence no default Cargo feature.
5. **Architecture-to-checklist information loss** — tasks that said "consult
   architecture" or premised work on another task's unverified output.
6. **Feature substitution** — a convenient but nonexistent Cargo feature or API
   substituted for the specified one. Verify every feature against locked metadata.
7. **Premature gates and attestation** — compile success is not product acceptance; UI
   prompt submission is not a generated screen.
8. **UI vocabulary drift** — §16 owns names, §24 owns bindings, Stitch output is a
   visual draft. Reject a screen that renames a component or invents state.
9. **Placeholder public release** — production contains no TODO/FIXME/stub/mock/
   coming-soon path and no entitlement bypass. A visible control completes its action
   or is absent.
10. **Unverified remote evidence** — claim no Stitch project/screen ID, vendor digest or
    read attachment without tool output.

## Build, test and release (target state — none of this exists yet)

No commands are runnable today. The architecture mandates this pipeline; treat the job
names as the contract that `P1-01` onward must satisfy.

**Toolchain locks** (§20.1): `rust-toolchain.toml`, `Cargo.lock`, `pnpm-lock.yaml`,
`VENDOR/LOCKFILE.md`, Android Gradle wrapper, JDK version file, Node version file.
Release builds reject dirty vendored trees.

**CI jobs** (§20.2):

```
lint-rust                lint-typescript          test-domain
test-storage             test-contracts           test-zim-conformance
test-sidecar-conformance test-entitlement-conformance
test-react               test-e2e-linux
check-android-graph      check-linux-graph        check-windows-graph
build-android-debug      build-linux-debug        build-windows-debug
audit-dependencies       generate-sbom            verify-vendor-lock
verify-entity-table      verify-no-production-placeholders
```

**Placeholder gate** (§20.3) scans production source and user-visible strings for:
`TODO`, `FIXME`, `stub`, `mock`, `placeholder`, `coming soon`, `not implemented`,
`lorem ipsum`, `test only`, `bypass`, `skip entitlement`. Allow-list entries require an
exact file/line pattern, reason, test-only or documentation-only classification, expiry
release and reviewer. **Production behavior cannot be allow-listed.**

**Required contract tests** (§19.2) are the fastest way to check a change did not break
the contract chain: `CT-IPC-001..003` (command-name and serialization parity),
`CT-SQL-001..004`, `CT-ZIM-001`, `CT-SIDE-001..002`, `CT-ENT-001..002`,
`CT-RET-001..002`, `CT-LORE-001..002`, `CT-PLAT-001..002`.

**Seven end-to-end release stories** (§19.4) are the product acceptance, not the unit
suite: `E2E-01-airplane-reader`, `E2E-02-interrupted-download`,
`E2E-03-sidecar-portability`, `E2E-04-local-ask`, `E2E-05-lore-correction`,
`E2E-06-offline-entitlement`, `E2E-07-backup-restore`.

**Artifacts** (§20.4): Android `.aab` + universal `.apk`; Linux `.AppImage` + `.deb`;
Windows `.msix` + `.exe` installer; CycloneDX SBOM and source archive — all covered by a
signed release manifest.

## UI design source

`DOCS/REBOOT-PAK/STITCH/` holds the frozen design complement: `design-system.json`
(Aurora Glass — dark mode, Inter/JetBrains Mono/Literata, glass surfaces),
`screen-manifest.json` (ten screens: `home-dashboard`, `library-pack-manager`, `reader`,
`annotation-workspace`, `ask-ai-conversation`, `notes-browser`, `memory-lore-browser`,
`settings`, `first-launch-setup`, `download-plan-dialog`), and
`STITCH-MCP-RUNBOOK.md` (executable prompts + verification rubric).

`generation-evidence.json` currently claims no project or screen IDs — the runbook has
not been executed. Update it only from successful Stitch tool results.

All ten UI contracts must be implemented for release **whether or not Stitch was used to
draft their visual layout**.

## Known context not in the pak

`DOCS/REBOOT-PAK/CONVERSATION_THREAD_thinkspace-navigator_2026-07-26.md` reconstructs a
lost design conversation about a **Thinkspace 6DoF Semantic Navigator** (ThinkingSphere
+ Knowledge Pysanky) — 6DoF camera physics, force-directed knowledge graph, ZIM articles
sliced into graph nodes, size-budget slider with a guaranteed "PZN essentials" floor and
honest degradation to text-only. **None of this is in `ARCHITECTURE.md` or the entity
table.** It is recovered intent, not contract; folding it in requires an architecture
amendment first, not implementation.
