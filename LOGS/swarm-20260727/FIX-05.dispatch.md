You are a GLM-4.7 coder working IN PLACE in /home/robin/forgejo/EnZIME-v4.
No /tmp, no .tmp, no CI workflows, no git commits.

# Task FIX-05 — Add the Tauri mobile entry point so the Android build links

## The error

    Error failed to validate library: Library from
    target/aarch64-linux-android/release/libenzime_reader_lib.so does not
    include required runtime symbols. This means you are likely missing the
    tauri::mobile_entry_point macro usage

## Files you may modify (exhaustive)

  apps/reader/src-tauri/src/lib.rs

Nothing else. Do not touch main.rs, Cargo.toml, tauri.conf.json or any crate.

## Do

`apps/reader/src-tauri/src/lib.rs` currently declares `pub fn run()` which
builds and runs the Tauri app. On Android the shared library must export a
mobile entry point symbol, which Tauri 2 generates from an attribute macro.

Add the attribute directly above `pub fn run()`:

    #[cfg_attr(mobile, tauri::mobile_entry_point)]
    pub fn run() {
        ...unchanged body...
    }

Leave the function body exactly as it is. Change nothing else in the file.

Note: `mobile` is a cfg that `tauri-build` emits automatically (you can see
`cargo:rustc-check-cfg=cfg(mobile)` in the build output), so no Cargo feature
or manual cfg declaration is needed.

## Constraints

- No placeholder, stub, TODO, `todo!()`, `unimplemented!()`.
- Do not delete anything. Do NOT run `git add`/`git commit`.
- You MAY run `cargo check -p reader` to confirm the desktop build still
  compiles. Do not attempt the Android build yourself.

## Return
State the change and the `cargo check -p reader` result.
