You are a GLM-4.7 coder working IN PLACE in /home/robin/forgejo/EnZIME-v4.
No worktrees, no /tmp, no .tmp, no CI workflows.

# Task APP-01 — Make the Tauri 2 app shell actually build on Linux

Right now `apps/reader/` contains only a `Cargo.toml` and a zero-byte
`lib.rs`. There is no Tauri dependency, no config, no frontend, so nothing can
be built. Your job is to produce a Tauri 2 desktop application that compiles
and bundles on Linux.

## Files you may create or modify (exhaustive)

  apps/reader/package.json
  apps/reader/index.html
  apps/reader/vite.config.ts
  apps/reader/tsconfig.json
  apps/reader/tsconfig.node.json
  apps/reader/src/main.tsx
  apps/reader/src/App.tsx
  apps/reader/src/router.tsx
  apps/reader/src/styles/global.css
  apps/reader/src-tauri/Cargo.toml
  apps/reader/src-tauri/build.rs
  apps/reader/src-tauri/tauri.conf.json
  apps/reader/src-tauri/src/main.rs
  apps/reader/src-tauri/src/lib.rs
  apps/reader/src-tauri/capabilities/default.json
  apps/reader/src-tauri/icons/icon.png
  pnpm-workspace.yaml   (only if `apps/*` is not already a member)

Touching anything outside this list is an escalation. In particular do NOT
touch `crates/`, `VENDOR/`, `DOCS/`, `CHECKLIST.md` or `scripts/`.

## Identity — exact values, do not invent alternatives

  productName        EnZIME Reader
  identifier         mba.robin.enzime
  version            read from the repo-root `version.json` field "version"
                     (currently 1.0.52684) — hardcode that exact string in
                     tauri.conf.json and package.json. Never invent a version.
  copyright          Copyright (C) 2026 Robin L. M. Cheung, MBA. All rights reserved.
  publisher          Robin L. M. Cheung, MBA

## Rust side (`apps/reader/src-tauri`)

`Cargo.toml` — keep the existing `[package]` block with its
`version.workspace = true` / `edition.workspace = true` inheritance. Add:

```toml
[lib]
name = "enzime_reader_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

Keep the existing optional `platform-adapter` / `inference-wllama` /
`revenuecat-adapter` dependencies and the existing single `default` feature
composition EXACTLY as they are. There must be no `android`, `linux` or
`windows` feature — one composition serves every platform.

`build.rs` — the standard `fn main() { tauri_build::build() }`.

`src/lib.rs` — a `pub fn run()` that builds the Tauri app:
```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())   // ONLY if you add that dep; otherwise omit
        .run(tauri::generate_context!())
        .expect("error while running EnZIME Reader");
}
```
Prefer the simplest form with no plugins at all. Do not add a plugin you do
not need.

`src/main.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    enzime_reader_lib::run()
}
```

`capabilities/default.json` — a Tauri 2 capability granting the `main` window
`core:default`. Nothing wider.

`icons/icon.png` — generate a plain 512x512 PNG. Any solid-colour image is
acceptable; it exists so the bundler has an icon.

## Frontend (`apps/reader`)

React 19 + TypeScript + Vite, matching the stack the architecture declares.

`package.json` — name `@enzime/reader`, version `1.0.52684`, scripts
`dev`/`build`/`preview`, deps `react`, `react-dom`, `react-router-dom`,
`zustand`, `@tauri-apps/api`; devDeps `vite`, `@vitejs/plugin-react`,
`typescript`, `@types/react`, `@types/react-dom`.

`vite.config.ts` — react plugin; `server.port = 41847` and
`server.strictPort = true`; `build.outDir = "dist"`; `clearScreen: false`.
Use exactly port **41847** — high and non-patterned, so it cannot collide with
another project's dev server.

`src/router.tsx` — a `createBrowserRouter` declaring exactly these paths from
the architecture's route table, each rendering a placeholder-free named view
component that renders its own heading and nothing else for now:

```
/setup                          SetupRoute
/                               HomeView
/library                        LibraryView
/reader/:packId/:articleKey     ReaderView
/ask                            AskView
/ask/:conversationId            AskView
/notes                          NotesView
/memory                         MemoryView
/settings                       SettingsView
```

Define those nine components inline in `App.tsx` (or a single file each is
fine, but stay within the file list above — put them in `App.tsx`). Each
renders its own name as an `<h1>` plus the app version read from
`__APP_VERSION__` injected by Vite `define`. That is a real, complete
first-render surface, not a placeholder caption: no "coming soon", no "TODO",
no disabled control.

`src/styles/global.css` — dark canvas `#090B13`, text `#F4F6FF`, font stack
starting `Inter`. Render the version string bottom-right, small and
unobtrusive, on every screen (this is a house requirement for every shipped
UI surface).

`index.html` — mounts `#root`, loads `/src/main.tsx`.

## Constraints (binding)

- No placeholder, stub, mock, TODO, FIXME, `todo!()`, "coming soon" anywhere.
- Do not create `.forgejo/` or `.github/` workflows — this project has no CI.
- Do not modify `version.json`, `version.txt` or `scripts/update-version.sh`.
- Do not delete anything.
- Do NOT run `git add` or `git commit` — leave the tree dirty.
- You MAY run `pnpm install` and `cargo check -p reader` to confirm it builds.

## Return

State the files you created and whether `cargo check -p reader` succeeded.
