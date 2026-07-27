You are a GLM-4.7 coder working IN PLACE in /home/robin/forgejo/EnZIME-v4.
No /tmp, no .tmp, no CI workflows, no git commits.

# Task FIX-04 — Make `crates/entitlement` compile

Files you may modify (exhaustive):
  crates/entitlement/src/policy.rs
  crates/entitlement/src/*.rs (only the file that declares CapabilityDecisionSource)

One error:

    error[E0599]: no variant or associated item named `Gracepolicy` found for
    enum `CapabilityDecisionSource`
      --> crates/entitlement/src/policy.rs:236:59

The use site writes `CapabilityDecisionSource::Gracepolicy` but the enum
declares no such variant. Find where `CapabilityDecisionSource` is declared in
this crate, read its actual variant list, and correct the USE SITE to the
variant that genuinely expresses "this capability is allowed because a grace
period applies".

If — and only if — the enum genuinely has no variant for that meaning, add one
with the same naming convention as its siblings (Rust enum variants are
UpperCamelCase, so `GracePolicy`, never `Gracepolicy`) and give it the same
derives and serde attribute the enum already carries.

Do not rename existing variants. Do not change the decision logic around line
236 — only make it name a real variant.

## Constraints

- No placeholder, stub, TODO, `todo!()`, `unimplemented!()`.
- Do not delete anything. Do NOT run `git add`/`git commit`.
- You MAY run `cargo check -p entitlement`.
- Errors in other crates are not yours.

## Return
State the fix and the final `cargo check -p entitlement` result.
