You are a GLM-4.7 coder working IN PLACE in /home/robin/forgejo/EnZIME-v4.
No worktrees, no /tmp, no .tmp, no CI workflows.

# Task FIX-02 — Make `crates/ports` compile

`cargo check -p ports` currently reports 8 errors with three distinct causes.
Fix all three. Do not refactor anything else.

## Files you may create or modify (exhaustive)

  crates/ports/src/platform.rs      (new)
  crates/ports/src/signature.rs     (fix one import)
  crates/ports/src/inference.rs     (add two missing enum definitions)
  crates/ports/Cargo.toml           (only if a dependency is genuinely missing)

Do NOT touch `crates/domain/`, `crates/ports/src/lib.rs`, or any other crate.

## Cause 1 — `error[E0583]: file not found for module 'platform'`

`crates/ports/src/lib.rs` line 15 declares `pub mod platform;` but
`crates/ports/src/platform.rs` does not exist. Create it, containing the
platform port from the frozen architecture. Read exactly this and transcribe
the trait and its types verbatim:

    sed -n '/^## 17.2 Platform port/,/^## 17.3/p' DOCS/ARCHITECTURE.md

Note: per §26 AMD-1 there is ONE `crates/platform-adapter` implementation
crate whose per-target behaviour is selected with `#[cfg(target_os = "...")]`.
`crates/ports/src/platform.rs` holds only the TRAIT and its associated types —
no implementation, no `#[cfg]`, no platform-specific code.

## Cause 2 — `error[E0432]: unresolved import 'crate::trust'`

`crates/ports/src/signature.rs` line 1 imports `crate::trust`. There is no
`trust` module in `ports`. The trust types live in the `domain` crate at
`crates/domain/src/trust.rs`. Change the import to `domain::trust::{...}`,
keeping the exact same imported item names.

## Cause 3 — `BackendKind` / `ModelFormat` not found in `inference.rs`

`crates/ports/src/inference.rs` references `BackendKind` (lines 10, 47) and
`ModelFormat` (lines 12, 42) but neither is defined or imported. They are
declared in the frozen architecture §10.1. Read it and transcribe both enums
verbatim into `crates/ports/src/inference.rs`, above their first use:

    sed -n '/^## 10.1 Exact traits/,/^## 10.2/p' DOCS/ARCHITECTURE.md

The variant sets are binding and were deliberately narrowed by §26 AMD-2:
`BackendKind` has exactly one variant `LocalLm`, and `ModelFormat` has exactly
one variant `Gguf`. Do not add `LiteRtLm`, do not add a `Gguf` *engine*
variant, do not "restore" retired variants — wllama3 is the sole engine and
GGUF is merely the model file format it consumes. Keep the derives and serde
attributes exactly as the architecture gives them.

## Constraints (binding)

- No placeholder, stub, mock, TODO, FIXME, `todo!()`, `unimplemented!()`.
- Do not delete anything. Do not create a CI workflow.
- Do NOT run `git add` or `git commit` — leave the tree dirty.
- You MAY run `cargo check -p ports` to confirm the errors are gone.
- Fix only these three causes. If a NEW unrelated error appears in another
  crate, leave it — it is not yours.

## Return

State what you changed and the final `cargo check -p ports` result.
