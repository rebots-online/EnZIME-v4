You are a GLM-4.7 coder working IN PLACE in /home/robin/forgejo/EnZIME-v4.
No /tmp, no .tmp, no CI workflows, no git commits.

# Task FIX-03 — Make `crates/transfer-http` compile

Files you may modify (exhaustive):
  crates/transfer-http/src/parse.rs
  crates/transfer-http/src/lib.rs
  crates/transfer-http/Cargo.toml

Three causes, 6 errors. Fix exactly these; change nothing else.

## Cause 1 — `str` indexed by `usize` (parse.rs:20 and parse.rs:28)

    let total_part = value.get(slash + 1)?;          // line 20
    let end: u64 = range_part.get(dash + 1)?.parse().ok()?;   // line 28

`str::get` takes a RANGE, not an index. `s.get(n)` is invalid; `s.get(n..)` is
the tail from n. Both sites want "everything after this delimiter", so both
need a range: `value.get(slash + 1..)` and `range_part.get(dash + 1..)`.

Read the surrounding function first and make sure the resulting slice is what
the parse logic actually wants — these are parsing a Content-Range header, so
byte offsets must stay exact. Do not change the parsing semantics, only make
the slicing valid.

## Cause 2 — `reqwest::Error::is_resolver()` does not exist (lib.rs:301)

    if error.is_connect() || error.is_request() || error.is_resolver() {

`reqwest::Error` has no `is_resolver` method. The intent is clearly "this is a
retryable network-layer failure". Use the methods that genuinely exist on
`reqwest::Error`: `is_connect()`, `is_timeout()`, `is_request()`. DNS
resolution failures surface through `is_connect()`. Replace `is_resolver()`
with `is_timeout()` so the retry predicate covers connect + request + timeout.

Do not add a dependency to detect resolver errors. Do not weaken the predicate
to always-true.

## Constraints

- No placeholder, stub, TODO, `todo!()`, `unimplemented!()`.
- Do not delete anything. Do NOT run `git add`/`git commit`.
- You MAY run `cargo check -p transfer-http`.
- Errors in other crates are not yours.

## Return
State the fixes and the final `cargo check -p transfer-http` result.
