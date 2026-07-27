# Vendored Source Lock — EnZIME Reader v4

Per ARCHITECTURE.md §26 AMD-3: every vendored component is compiled from
source we hold. Each tree was cloned **once, directly at its permanent path**
under `VENDOR/` (INC-15 vendor-clone-at-home — never clone→move, which erases
provenance). Upstream prebuilt binaries may be used only as development-time
scaffolding and are never linked into a release artifact.

Vendored 2026-07-27 by the EnZIME-v4 orchestrator.

## `VENDOR/tauri` — Tauri 2 application shell

```text
vendor_id:      vendor.tauri.v2.11.1
source_url:     https://github.com/tauri-apps/tauri.git
source_tag:     tauri-v2.11.1
source_commit:  e5ae5b93cdd310045191cc0526f253140ad64b87
license_spdx:   Apache-2.0 OR MIT
clean_tree:     true
```

Note: the manifest's "tag `v2.11.1`" resolves in this monorepo to
`tauri-v2.11.1` (the crate-scoped tag). Sibling tags at the same version
(`@tauri-apps/api-v2.11.1`, `tauri-cli-v2.11.1`, `tauri-runtime-v2.11.1`,
`tauri-runtime-wry-v2.11.1`) are the same release train.

## `VENDOR/wllama3` — local inference engine (ALL platforms)

```text
vendor_id:      vendor.wllama.3.5.1
source_url:     https://github.com/ngxson/wllama.git
source_tag:     3.5.1
source_commit:  766d28e03eeac044fe055327d06b83d3f9b84544
license_spdx:   MIT
clean_tree:     true
```

Per AMD-2 this is the **sole** local inference engine on Android, Linux and
Windows. It replaces the retired `inference-litert` (LiteRT-LM) and
`inference-gguf` entries; the crate is `crates/inference-wllama`. The §10.1
`InferenceBackend` / `InferenceSession` traits are unchanged — wllama3 is a
provider behind those provider-agnostic ports.

## `VENDOR/AnZIMmermanLib-src` — multilingual ZIM engine (our own creation)

```text
vendor_id:      vendor.anzimmerman.c73245e
source_url:     /home/robin/github/EnZIME (canonical local holder)
source_commit:  b1d44d4  (master HEAD at vendor time)
key_commit:     c73245e  fix(zimlib): correct directory entry parsing to use
                         uint16 mimetype + parameterLen per ZIM spec
subtree:        AnZimmermanLIB/AnZimmermanLib/
license_spdx:   Proprietary (first-party)
clean_tree:     true
```

**Ref selection (I-24 — semantic, not by timestamp).** Two local holders carry
this source: `~/forgejo/EnZIM` and `~/github/EnZIME`. Both show 92 tracked
files on `master`. The `~/github/EnZIME` line was selected because it carries
`c73245e`, the ZIM directory-entry parsing correctness fix (uint16 mimetype +
parameterLen per the ZIM spec) — a substantive correctness advance that the
other holder's history does not contain. Selection was made by reading what
the commits do, not by comparing dates.

**First real-world use.** This is the first production consumption of
AnZimmermanLib. Any repair made here is recorded — with its precipitating
scenario — in `DOCS/ANZIMMERMANLIB-UPSTREAM-FIXES.md` for propagation back to
its own repository.
