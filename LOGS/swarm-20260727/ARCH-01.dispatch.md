You are a GLM-5.2 ARCHITECT working in /home/robin/forgejo/EnZIME-v4.

You are operating at the ARCHITECT layer ONLY. You do not write application
source code, you do not touch `crates/`, `apps/`, `schemas/` or `CHECKLIST.md`.
Your single deliverable is a corrected, internally-consistent
`DOCS/ARCHITECTURE.md`.

# The problem

`DOCS/ARCHITECTURE.md` (6,416 lines, §0–§26) currently **contradicts itself**.
A `§26 Amendments` block was appended carrying four operator-directed changes
(AMD-1..AMD-4), but the §1–§25 baseline was never reconciled to them. Coders
reading the baseline and coders reading §26 build different products.

Read the amendments first:

    sed -n '/^# 26. Amendments/,$p' DOCS/ARCHITECTURE.md

# Your task — three corrections, one re-attestation

## Correction 1 — §7.1 does not compile (a genuine architectural defect)

Read it:

    sed -n '1338,1372p' DOCS/ARCHITECTURE.md

`ContentDigest` carries `#[derive(..., Hash, ...)]`, and one of its fields is
`algorithm: DigestAlgorithm`. `DigestAlgorithm`'s own derive list has no
`Hash`. A struct cannot derive `Hash` when a field type does not implement it,
so the specification as written **cannot compile**.

Fix it in the §7.1 code block by adding `Hash` to `DigestAlgorithm`'s derive
list. Do not remove `Hash` from `ContentDigest` — content digests are used as
map keys, which is why the derive is there.

## Correction 2 — reconcile the baseline to AMD-1 (one codebase, all platforms)

Operator-directed and binding: Android, Linux and Windows ship **the same
Tauri 2 codebase with the complete feature set**. No Cargo feature may remove
a user-visible capability from any platform. Platform difference is expressed
ONLY by `#[cfg(target_os = "...")]` inside a single `crates/platform-adapter`.

Every one of these baseline locations still describes the retired
per-platform-feature model and must be rewritten in place to match:

- **§1.1 platform posture table** — the per-platform rows that imply different
  capability sets.
- **§5 repository structure tree** — replace `crates/platform-android/` and
  `crates/platform-desktop/` with a single `crates/platform-adapter/`.
- **§5.1 file purposes table** — same substitution.
- **§5.2 Cargo feature compositions** — replace the `android`/`linux`/`windows`
  feature blocks with the single `default` composition, and replace the
  "exactly one of android/linux/windows is enabled" assertion with: *every
  platform's dependency graph is identical; a graph difference is a build
  failure*.
- **§16.1 Rust crates and modules** — the crate rows.
- **§17 Platform Adapters** (§17.1 concern matrix, §17.2 platform port,
  §17.3/§17.4/§17.5) — these become `#[cfg]` modules of one crate, not
  separate crates.
- **§19.2 `CT-PLAT-001` / `CT-PLAT-002`** — restate as: the Android, Linux and
  Windows dependency graphs each contain the full crate set.

## Correction 3 — reconcile the baseline to AMD-2 (wllama3 is the only engine)

Operator-directed and absolute: **wllama3 is the sole local inference engine
on ALL platforms**, crate `crates/inference-wllama`. LiteRT-LM and the GGUF
adapter are retired entirely.

Rewrite in place:

- **§1.1** "Local inference" row → wllama3 on all three platforms.
- **§5 / §5.1 / §5.2** — `crates/inference-litert` and `crates/inference-gguf`
  become the single `crates/inference-wllama`.
- **§10.3 Release backends** — wllama3 only.
- **§10.4 Selection policy** — resolves to one backend; remove the
  multi-backend arbitration that existed to choose between LiteRT and GGUF.
- **§16.1** — the crate rows.

Leave §10.1 `InferenceBackend`, §10.2 `EmbeddingModel` and the
`InferenceSession` trait **unchanged** — they are provider-agnostic ports and
wllama3 is simply a provider behind them. Do not introduce any wllama-specific
type, field or variant into a port.

After the corrections land, no LiteRT-LM or GGUF identifier may remain anywhere
in the document except inside the §26 AMD-2 amendment record itself, which
stays as the historical account of the change.

## Also fold in AMD-3 and AMD-4 where the baseline contradicts them

- **AMD-3** — vendored components are compiled from our own source under
  `VENDOR/`, already vendored and frozen (`VENDOR/VENDOR-LOCK.md`). §22.2's
  initial vendor inputs must name: `tauri` (tag `tauri-v2.11.1`), `wllama3`
  (tag `3.5.1`), `AnZIMmermanLib-src` (commit `c73245e`). LiteRT-LM is no
  longer a vendor input.
- **AMD-4** — no CI until public-release operator signoff. §20.2's CI job list
  becomes local scripts under `tools/release/` invoked manually; §20.6's "all
  CI jobs green" becomes "all local release scripts pass on the release
  commit". Do not describe any `.forgejo/` or `.github/` workflow.

## Re-attestation (§25)

§25 is the completeness attestation. After every correction is made, append to
§25 an attestation block stating: the document is complete, stub-free and
end-state; the ISO-8601 timestamp; and the authoring agent (model family,
version, harness). The document must be attested **only when whole** — so do
this last, after Corrections 1–3 are all in.

# Binding constraints

- **Edit `DOCS/ARCHITECTURE.md` IN PLACE.** First copy it to
  `DOCS/ARCHITECTURE.md.bak.<YYYYmmdd_HHMMSS>` beside itself, then edit the
  original so every path pointing at it keeps working.
- **§26 stays.** It is the append-only historical record of what changed and
  why. You are reconciling the baseline TO it, not deleting it.
- **No `TBD`, no "future work", no "open issues", no placeholder, no stub.**
  If a section cannot be made complete, that is an escalation, not a note in
  the file.
- Do not renumber sections. Do not reorder the entity table. Do not invent an
  entity that no section requires.
- Do not touch `CHECKLIST.md` — regenerating it is a separate, later layer.
- Do not touch `crates/`, `apps/`, `schemas/`, `packages/` or `VENDOR/`.
- Do not create `.tmp/`, do not write to `/tmp`, do not use `mktemp`.
- Do not delete anything. Do not run `git commit`.

# Return

List every section you changed and what you changed in it, then stop.
