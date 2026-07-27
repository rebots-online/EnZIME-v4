You are a GLM-5.2 ARCHITECT working in /home/robin/forgejo/EnZIME-v4.

ARCHITECT layer ONLY. You do not write application source. You do not touch
`crates/`, `apps/`, `schemas/`, `packages/`, `VENDOR/` or `CHECKLIST.md`.
Your only deliverable is a corrected `DOCS/ARCHITECTURE.md`.

# The defect

§17.2 declares the `PlatformAdapter` trait. Its method signatures reference
five types that are **never defined anywhere in the document** — they appear
only at their use sites inside the trait body:

| Type | Uses |
|---|---|
| `PlatformError` | 8 |
| `PlatformCapabilityReport` | 1 |
| `PlatformStorageRoot` | 1 |
| `PowerState` | 1 |
| `NetworkState` | 1 |

A coder transcribing §17.2 faithfully produces code that cannot compile. This
is the same class of defect as the §7.1 `Hash` defect fixed in `e9f407e`, and
it means §25.1's attestation — "every public entity named here resolves to
exactly one definition" — is currently false.

Read the trait and its surrounding section first:

    sed -n '/^## 17.1 Concern matrix/,/^## 17.4/p' DOCS/ARCHITECTURE.md

# Your task

Define all five types, in §17.2, immediately above the `PlatformAdapter`
trait, as complete Rust declarations in the same style as the rest of the
document (exact derives, exact `#[serde(rename_all = "snake_case")]` where the
document uses it elsewhere for enums, full field lists, no `TBD`).

Derive each type's shape from **how the trait already uses it** and from what
the rest of the architecture already establishes — do not invent capability
the product does not have:

- **`PlatformError`** — an error enum in the same style as `ZimError` (§7.1)
  and `AnnotationError` (§9): `#[derive(Clone, Debug, thiserror::Error)]`,
  one variant per genuine platform failure mode the trait's methods can hit
  (permission denied, unavailable capability, storage-root unavailable, I/O,
  cancelled, unsupported-on-this-platform). Every variant carries the context a
  caller needs to act.
- **`PlatformCapabilityReport`** — must satisfy §6's `capability_reports` table
  and the `capability_probe` command in §14.4, and must carry what
  `SetupWizard`/`CapabilityReport` (§24) render: what the device can actually
  do (available RAM, free storage, background-work permission, notification
  permission, microphone permission, whether local inference is viable).
- **`PlatformStorageRoot`** — must satisfy the `storage_roots` SQLite table
  (§6.2) and the `storage_roots_list` command, since `StoragePlanner` (§24)
  renders these: an identifier, a display label, a path or opaque token, total
  and free bytes, removability, and whether it is the default.
- **`PowerState` / `NetworkState`** — small enums covering exactly the states
  §8 transfer policy and §21 performance/thermal behaviour already reason
  about (e.g. charging vs battery with a level; offline / metered / unmetered).
  Keep them minimal and honest; do not add states nothing consumes.

**Cross-check every field against the existing SQL schema (§6.2), the command
registry (§14.4) and the entity-to-UI bindings (§24).** A field you invent
that no table stores, no command returns and no component renders is scope
creep and must not appear. Conversely, if a §24 component or a §6.2 column
requires a field, it must be present.

Also add the five new types as rows to the §16 entity table in the appropriate
subsection, matching the existing row format exactly.

# One more thing to check while you are in there

`crates/ports/src/platform.rs` (already written from §17.2) needs
`tokio::io::AsyncRead`/`AsyncWrite` because the trait body references them.
Confirm §17.2 genuinely intends tokio async I/O in the port. If it does, say
so explicitly in the section prose so the dependency is not a surprise. If the
document elsewhere establishes a different stream abstraction (check §7.2's
`ZimEntryStream`, which uses `futures_core::Stream`), reconcile §17.2 to that
instead and state which you chose and why.

# Binding constraints

- Edit `DOCS/ARCHITECTURE.md` IN PLACE. First copy it to
  `DOCS/ARCHITECTURE.md.bak.<YYYYmmdd_HHMMSS>` beside itself.
- No `TBD`, no "future work", no placeholder, no stub, no signature-only row.
- Do not renumber sections. Do not touch §26.
- After the fix, append to §25.1 a short amendment note recording that the five
  §17.2 types were defined on this date and that the entity-resolution claim
  now holds. Do not rewrite the earlier attestation — append.
- Do not create `.tmp/`, do not write to `/tmp`, do not use `mktemp`.
- Do not delete anything. Do not run `git commit`.

# Return

List each type you defined with its final shape in one line, state the
tokio-vs-futures decision, and name any place where you had to make a judgment
call the document did not settle.
