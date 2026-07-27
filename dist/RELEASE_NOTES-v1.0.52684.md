# EnZIME Reader v1.0.52684

**Copyright (C) 2026 Robin L. M. Cheung, MBA. All rights reserved.**

First multiplatform build of the EnZIME Reader v4 reboot. Built manually on a
Linux host; no CI (release-signoff gate not yet given).

## What this build is

A Tauri 2 application shell that installs and launches on Android, Linux and
Windows from **one codebase with one feature composition** — no platform gets a
reduced capability set. It renders the nine application routes and the version
chip.

## What this build is NOT

The fifteen backend crates (ZIM engine, object store, catalog, transfer,
annotations, sidecar, inference, retrieval, lore, entitlement, backup,
diagnostics, storage) **compile but are not yet wired to the UI** — the Tauri
command registry is not implemented. No ZIM archive opens, nothing is
annotated, nothing is asked. This is a packaging and platform milestone, not a
functional reader.

## Platform posture

| | |
|---|---|
| Local inference engine | wllama3 — **sole engine, all platforms** |
| Vendored + frozen in-tree | Tauri 2.11.1, wllama 3.5.1, AnZimmermanLib @c73245e |
| Android signing | production.keystore, alias `upload`, RSA-2048/SHA256withRSA |
| Windows signing | **unsigned** — Authenticode requires a Windows host |
| Android versionCode | 100000 |

## Artifacts

| Artifact | Size | Notes |
|---|---|---|
| `android-universal-release-signed.apk` | 37 MB | release-signed, verified |
| `android-universal-release-signed.aab` | 14 MB | release-signed, Play-uploadable |
| `android-universal-debug.apk` | 421 MB | debug-signed |
| `android-native-debug-symbols.zip` | 10 MB | 4 ABIs |
| `linux-amd64.deb` | 2.9 MB | |
| `linux-amd64.AppImage` | 75 MB | |
| `windows-x64-setup.exe` | 2.0 MB | NSIS, unsigned |
| `web-pwa.zip` | 83 KB | Vite production build |
| `sbom.cdx.json` | — | CycloneDX 1.5, 421 components |
| `source.tar.gz` | 104 MB | from committed HEAD, self-contained |

Every build recipe, including the three failures hit and how each was fixed,
is recorded in `DOCS/BUILD-RECIPES.md`.
