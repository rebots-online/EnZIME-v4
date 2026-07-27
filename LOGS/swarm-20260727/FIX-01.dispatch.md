You are a GLM-4.7 coder working IN PLACE in /home/robin/forgejo/EnZIME-v4.
No worktrees, no /tmp, no .tmp.

# Task FIX-01 — Move the annotation traits from `domain` to `ports`

## The compile error you are fixing

    error[E0425]: cannot find type `ArticleDocument` in this scope
      --> crates/domain/src/annotation.rs:126:19

`crates/domain/src/annotation.rs` declares two traits — `AnnotationResolver`
and `AnnotationService` — and `AnnotationResolver::resolve` takes
`&ArticleDocument`. `ArticleDocument` is defined in `crates/ports/src/zim.rs`.
`domain` cannot see `ports`: the dependency runs ports -> domain, never the
reverse.

## Why they move (this is settled by the frozen architecture, do not re-decide)

DOCS/ARCHITECTURE.md §5.1 assigns:
  `crates/domain` = entities, value objects, policies, state enums
  `crates/ports`  = traits used by application services

`AnnotationResolver` and `AnnotationService` are traits. They belong in
`crates/ports`. Every data type in `annotation.rs` (structs, enums, the
selector model, `AnnotationError`) is an entity and STAYS in `domain`.

## Files you may create or modify (exhaustive)

  crates/domain/src/annotation.rs   (remove ONLY the two trait declarations)
  crates/ports/src/annotation.rs    (new — the two traits land here)
  crates/ports/src/lib.rs           (add `pub mod annotation;` to the existing list)
  crates/ports/Cargo.toml           (add deps if the traits need them)

## Do

1. Move the `AnnotationResolver` and `AnnotationService` trait declarations
   verbatim out of `crates/domain/src/annotation.rs` into a new
   `crates/ports/src/annotation.rs`. Signatures, method names, parameter names,
   `#[async_trait]` attributes and return types must be byte-identical to what
   is there now — this is a relocation, not a redesign.
2. In the new file, import what the traits reference:
   - from `domain::annotation::{...}` — the selector/annotation entity types
     and `AnnotationError`
   - from `crate::zim::ArticleDocument` — the article type
3. Add `pub mod annotation;` to `crates/ports/src/lib.rs`, keeping the existing
   module list intact and alphabetical.
4. Leave every struct, enum and error type in `crates/domain/src/annotation.rs`
   exactly where it is. Do not move them. Do not rename anything anywhere.

## Constraints (binding)

- No placeholder, stub, mock, TODO, FIXME, `todo!()` or `unimplemented!()`.
- Do not delete any file. Do not create a CI workflow.
- Do NOT run `git add` or `git commit` — leave the tree dirty.
- You MAY run `cargo check --workspace` to confirm the error is gone.
- Do not pick up any other work, do not "improve" neighbouring code.

## Return

State what you moved and the final `cargo check --workspace` result.
