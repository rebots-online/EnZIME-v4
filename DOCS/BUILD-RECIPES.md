# EnZIME Reader — Build Recipes

## Root-partition environment on msi4090 (2026-10-13)

The build profile is `/srv/enzime-build/env.sh`, automatically loaded through
`~/.config/enzime/build-env.sh`. `/opt/enzime-build` is a convenience symlink:
`/opt` itself is a separate partition, so the physical build directory is under
`/srv` on the Btrfs root partition of the internal 2 TB WD drive.
**Do not use the removable 500 GB KINGSTON drive or its F2FS partition.**

Rust/Cargo, Android SDK/NDKs, Gradle, pnpm and Windows-cross caches, historical
build workspaces and lane targets have root destinations. Old paths are retained
as compatibility symlinks after checksum verification. Node 24.14.1 and pnpm
11.19.0 are installed under this environment; system JDK 21, Clang/LLD 18,
NSIS and native Linux libraries reside on root under `/usr`.

The same whole-platform command below uses root-contained workspaces, caches,
temporary files, logs and collected artifacts. For this profile the current
artifact index is `/srv/enzime-build/artifacts/index.html`, and its manifest is
`/srv/enzime-build/artifacts/current-build-manifest.json`; existing repository
`dist/` files remain historical records. A configured native Windows executor
is still required for MSI/MSIX. Cross-compiling NSIS does not establish native
Windows runtime acceptance.

To inspect placement before a build:

```bash
source ~/.config/enzime/build-env.sh
findmnt -T "$ENZIME_BUILD_ROOT"
pnpm store path
rustup show home
df -h / /home "$ENZIME_BUILD_ROOT"
bash scripts/build-all.sh
```

An explicit `ENZIME_BUILD_ENV=/absolute/path/profile.sh` overrides the local
profile; a missing explicit profile fails immediately. Other hosts without a
profile retain the existing default locations. The path overrides are
`ENZIME_BUILD_WORKSPACE_ROOT`, `ENZIME_BUILD_CACHE_ROOT`,
`ENZIME_BUILD_LOG_ROOT` and `ENZIME_BUILD_ARTIFACT_DIR`. The repository lock,
frozen source, unified version and required platform matrix remain unchanged.

## Current operator command — every platform, one build set

```bash
bash scripts/build-all.sh
```

Commit intended source before invocation. The script snapshots that commit,
stamps one fresh version, prepares the frontend once, and attempts every platform
with bounded concurrent jobs. Optional `--jobs N` controls concurrency;
`--source-ref REF` selects a committed source revision. There is no platform-only
option. Historical recipes below describe the internal commands and prior evidence.

Open `dist/index.html` for the current attempt, including its Windows NSIS row.
`dist/current-build-manifest.json` contains artifact hashes and lane results.
Failed targets never link older versions. The complete-build pointer advances
only when the complete matrix passes; MSI/MSIX require a configured native Windows
executor. The process exits nonzero for any missing/failed required output.

Android signing uses the existing production keystore and
`ENZIME_KEYSTORE_PASSWORD`/`ENZIME_KEY_PASSWORD`, with optional
`ENZIME_KEYSTORE_PATH` and `ENZIME_KEYSTORE_ALIAS`. The native Windows interface is
`ENZIME_WINDOWS_BUILD_EXECUTOR`; see `DOCS/ARCHITECTURE/unified-build.md`.
The optional ignored `signing/build-credentials.json` supplies these four signing
variables when they are absent from the environment; `ENZIME_SIGNING_CONFIG` can
select another private JSON file. Credentials are excluded from source archives.
No upload, git commit, push, or CI invocation occurs inside the build script.

Manual builds only. No CI until public-release operator signoff (§26 AMD-4).
Every recipe below is recorded **after it actually worked**, with the errors
hit on the way, so the next run does not rediscover them.

Version is always stamped by `scripts/update-version.sh` (copied from
`~/Admin-Manual/versioning/`) — never hand-edited. Artifacts are staged into
the tracked `dist/` under CC12 slug-first names:
`enzime-reader-v<MAJOR.MINOR.BUILD>-<qualifier>`.

---

## Linux — `.deb` + `.AppImage` ✅ WORKING

**First succeeded:** 2026-07-27, v1.0.52684, exit 0, ~1m01s release compile.

### Host prerequisites

```bash
# Tauri 2 Linux deps. Ayatana, NOT the legacy libappindicator3-dev —
# apt refuses to install both on Ubuntu >= 22.04.
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev \
  libssl-dev build-essential pkg-config patchelf file

cargo install tauri-cli --version '^2' --locked   # installed 2.11.4
```

### Recipe

```bash
cd /home/robin/forgejo/EnZIME-v4
./scripts/update-version.sh                 # stamps version.txt + version.json
cd apps/reader
cargo tauri build --bundles deb,appimage
```

Outputs land at the **workspace root** target dir, not under `apps/reader/`:

```
target/release/bundle/deb/EnZIME Reader_<VERSION>_amd64.deb
target/release/bundle/appimage/EnZIME Reader_<VERSION>_amd64.AppImage
```

### Stage into dist/

```bash
VER=$(python3 -c "import json;print(json.load(open('version.json'))['version'])")
SLUG=$(python3 -c "import json;print(json.load(open('version.json'))['internalName'])")
cp "target/release/bundle/deb/EnZIME Reader_${VER}_amd64.deb" \
   "dist/${SLUG}-v${VER}-linux-amd64.deb"
cp "target/release/bundle/appimage/EnZIME Reader_${VER}_amd64.AppImage" \
   "dist/${SLUG}-v${VER}-linux-amd64.AppImage"
(cd dist && sha256sum ${SLUG}-v${VER}-linux-amd64.* > SHA256SUMS-v${VER}.txt)
```

### Observed results (v1.0.52684)

| Artifact | Size |
|---|---|
| `enzime-reader-v1.0.52684-linux-amd64.deb` | 2.9 MB |
| `enzime-reader-v1.0.52684-linux-amd64.AppImage` | 75 MB |

### Notes / gotchas hit on the way

- `cargo tauri` is not present by default — `error: no such command: 'tauri'`.
  Install `tauri-cli ^2` (I-23: install the missing tool, never degrade the
  deliverable).
- The bundler emits Tauri's own `productName_version_arch` naming. Those raw
  names are a toolchain default, not a managed surface — the staging step
  above renames to the CC12 slug-first form. Only the `dist/` copies are
  referenced anywhere.
- `bundle.targets` in `tauri.conf.json` lists `deb`, `appimage`, `msi`;
  `--bundles` on the command line narrows a run to a subset.

---

## Windows — `.msi` / `.exe` ⏳ NOT YET ATTEMPTED

Planned via `cargo-xwin` cross-compile from Linux per the default
multiplatform stack. Recipe recorded here once it actually produces an
artifact.

## Android — `.aab` / `.apk` ⏳ NOT YET ATTEMPTED

Planned via `cargo tauri android build`. Requires Android SDK/NDK; per I-23,
missing components are installed with `sdkmanager` rather than skipped.
Keystore per BUILD_CONVENTIONS (`production.keystore`, alias `upload`).
Recipe recorded here once it actually produces an artifact.

## SBOM + source archive ⏳ NOT YET ATTEMPTED

CycloneDX SBOM and the source archive complete the eight-artifact release set.

---

## Windows — `.exe` (NSIS) ✅ WORKING · `.msi`/`.msix` ❌ NEEDS A WINDOWS HOST

**First succeeded:** 2026-07-27, v1.0.52684.

```bash
cd apps/reader
cargo tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc --bundles nsis
# -> target/x86_64-pc-windows-msvc/release/bundle/nsis/EnZIME Reader_<VER>_x64-setup.exe
```

Prereqs: `rustup target add x86_64-pc-windows-msvc`, `cargo install cargo-xwin`,
`makensis` (apt `nsis`).

### Failure hit and fixed

`icons/icon.ico` not found; required for generating a Windows Resource file
during tauri-build. Fixed once for all platforms with:

```bash
cargo tauri icon src-tauri/icons/icon.png   # emits .ico, .icns, Square*Logo, Android mipmaps
```

### `.msi` / `.msix` — not producible on this host

`cargo tauri build --target x86_64-pc-windows-msvc --bundles msi` is rejected:

```
error: invalid value 'msi' for '--bundles [<BUNDLES>...]'
  [possible values: deb, rpm, appimage]
```

`nsis` is accepted and works; `msi` is not, because the WiX toolchain the MSI
bundler drives is Windows-only. This is a **host-platform limitation, not a
missing package** — no `apt install` resolves it. Producing `.msi`/`.msix`
requires building on a Windows host.

The NSIS `.exe` is a genuine Windows installer and covers the "`.exe`
installer" row of the §20.4 artifact matrix; the `.msix` row remains open.

### Signing

Tauri skips Authenticode when the host is not Windows:

> Signing, by default, is only supported on Windows hosts … skipping signing

The staged `.exe` is therefore **unsigned**. Authenticode signing is a separate
release step on a Windows host or via a configured `bundler > windows >
sign_command`.

---

## Android — `.apk` + `.aab` ✅ WORKING (release-signed)

**First succeeded:** 2026-07-27, v1.0.52684.

### Prereqs

```
ANDROID_HOME=$HOME/Android/Sdk
NDK_HOME=$HOME/Android/Sdk/ndk/27.0.12077973
JDK 21
rustup targets: aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

### Recipe

```bash
cd apps/reader
export ANDROID_HOME="$HOME/Android/Sdk"
export NDK_HOME="$HOME/Android/Sdk/ndk/27.0.12077973"
cargo tauri android init          # once; generates gen/android
cargo tauri android build --apk --aab
```

Outputs:

```
gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk
gen/android/app/build/outputs/bundle/universalRelease/app-universal-release.aab
```

### Failure hit and fixed

```
Error failed to validate library: libenzime_reader_lib.so does not include
required runtime symbols. This means you are likely missing the
tauri::mobile_entry_point macro usage
```

Fix — in `apps/reader/src-tauri/src/lib.rs`:

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() { ... }
```

The `mobile` cfg is emitted automatically by `tauri-build`; no Cargo feature
is needed.

**Do not run `gen/android/gradlew` directly to debug.** The Rust task talks
back to the Tauri CLI over a websocket that only exists during
`cargo tauri android build`, so a direct Gradle invocation fails with a
misleading `failed to build WebSocket client … ConnectionRefused` that hides
the real error.

### Signing (production keystore)

```bash
# created once, per BUILD_CONVENTIONS
keytool -genkey -v -keystore signing/production.keystore -alias upload \
  -keyalg RSA -keysize 2048 -sigalg SHA256withRSA -validity 25000 \
  -storepass "$ENZIME_KEYSTORE_PASSWORD" -keypass "$ENZIME_KEY_PASSWORD" \
  -dname 'CN=EnZIME Reader Upload, O=Robin L M Cheung MBA, C=CA'

# APK -> apksigner
$ANDROID_HOME/build-tools/36.1.0/apksigner sign \
  --ks signing/production.keystore --ks-key-alias upload \
  --ks-pass env:ENZIME_KEYSTORE_PASSWORD --key-pass env:ENZIME_KEY_PASSWORD \
  --out dist/<slug>-v<VER>-android-universal-release-signed.apk <unsigned.apk>

# AAB -> jarsigner (apksigner does not sign bundles)
jarsigner -keystore signing/production.keystore -storepass "$ENZIME_KEYSTORE_PASSWORD" \
  -keypass "$ENZIME_KEY_PASSWORD" -sigalg SHA256withRSA -digestalg SHA-256 \
  dist/<slug>-v<VER>-android-universal-release-signed.aab upload
```

Verified signer: `CN=EnZIME Reader Upload, O=Robin L M Cheung MBA, C=CA`.

---

## SBOM — CycloneDX ✅ WORKING

```bash
cargo install cargo-cyclonedx --locked
cargo cyclonedx --all --format json      # emits <crate>.cdx.json beside each Cargo.toml
```

The per-crate files are then merged, de-duplicated by (name, version), and
written to `dist/<slug>-v<VER>-sbom.cdx.json` as a single CycloneDX 1.5
document with the application as `metadata.component`. 421 unique components
at v1.0.52684.

## Source archive ✅ WORKING

```bash
git archive --format=tar.gz --prefix="<slug>-v<VER>/" \
  -o "dist/<slug>-v<VER>-source.tar.gz" HEAD
```

Built from **committed HEAD**, so it is reproducible and never picks up
uncommitted working-tree state. ~104 MB because the vendored Tauri, wllama3
and AnZimmermanLib trees are genuinely part of our tree — the archive is
self-contained and rebuilds without reaching upstream.


## 2026-09-13 — v1.7.21541 native stylesheet repair trial

User rejected the v1.6.55832 Library appearance. P8-03-STYLE restored pure route
stylesheet rendering and shell ownership of shared styles/tokens in four TSX files.
Backups are in the Admin-Manual core-sesh-recovery staging area.

- Host: msi4090, Linux; cargo 1.94.1, tauri-cli 2.11.4, Node 24.14.1,
  pnpm 11.13.0. Existing toolchains/caches used; no CI.
- Preserved old release.lock (1/6/55832), ran `bash scripts/update-version.sh`
  once, and froze 1.7.21541 before Tauri's beforeBuildCommand ran the stamper.
- Command: `cargo tauri build --bundles appimage` from `apps/reader`.
  TypeScript passed; Vite built 92 modules; Rust release completed in 29.92s;
  AppImage bundling exited 0. Existing transfer-http unused-import warning remains.
- Artifact: `dist/enzime-reader-v1.7.21541-linux-amd64.AppImage`, 83,212,792 bytes;
  SHA-256 `6a95b8f2a69e8d1e027f72367fd72884751615b4bb5f03ecffcc2a88d427f366`.
- Trial: isolated data/cache/runtime paths under Admin-Manual staging; the launched
  process omitted inherited LD_LIBRARY_PATH. Initial native launch-log snapshot was empty; system
  gst-inspect finds appsink. This is not a media playback verification.
- Observed: desktop Library styling survives settled loading, wikipedia/en input,
  and Home→Library return. Search returns no matches from the empty catalog.
- FAILED at 720 CSS pixel minimum: pre-existing shell media/cascade conflict
  displaces the navigation rail and hides route content. See P8-03-NARROW;
  preserve this artifact and screenshot, then stamp the next iteration.
- Capture note: ffmpeg x11grab recorded the fixed-size native window; shrinking
  that window terminated capture with an X11 image-size error. The narrow failure
  is also preserved as a window screenshot. It is not a catalog/offline gauntlet.
- Evidence: `dist/rubric-runs/v1.7.21541-linux-styles-20260913T094211Z/`.


## 2026-09-13 — v1.8.21560 native width correction and verified UI trial

P8-03-NARROW corrects the observed shell grid/rail conflict, preserving the
existing navigation rail, using a shrinkable content track and containing the
search label. Independent review passed both source tasks; no broader redesign.

- Preserved the 1.7.21541 freeze in its evidence folder; invoked the canonical
  stamper once and froze 1.8.21560. Same manual command:
  `cargo tauri build --bundles appimage` from `apps/reader`.
- Build: TypeScript exits 0; Vite 92 modules; Rust optimized release 19.36s;
  AppImage bundler exits 0. Existing unused-import warning is unchanged.
- Source: `b574ed8504a4298bab1e326ec201bb0e82e0be2f`, committed from the compiled
  source after the successful build, before native verification; no application
  source edits followed. The commit includes the original build freeze.
- Artifact: `dist/enzime-reader-v1.8.21560-linux-amd64.AppImage`, 83,200,504 bytes;
  SHA-256 `a96dd71e871cdf4e39752c7e5b4b52af3b707b428112b8d2a147bfddda28a72d`.
- Native run: isolated data/cache/runtime on msi4090; inherited LD_LIBRARY_PATH
  omitted for this launch. The later log reports `GStreamer element appsink not
  found`; system gst-inspect finds the plugin, so packaged-runtime diagnosis
  and media verification remain outstanding.
- PASS bounded protocol: settled desktop Library, wikipedia/en input with no
  matches, Home→Library remount, 720 CSS pixel minimum-width layout, scrolling
  to Downloads, then desktop restoration. Root viewed narrow/scrolled/final
  stills; independent reviewer viewed query/Home/Library-return stills.
- Recorded two native clips (12.13s desktop, 5.93s narrow) with screenshots and
  timecode/seek-link index. Stop capture before resizing; this avoids the
  fixed-size X11 capture error from v1.7. The corrected app remains open on Library.
- Evidence: `dist/rubric-runs/v1.8.21560-linux-styles-20260913T100046Z/RESULTS.md`;
  `dist/enzime-reader-v1.8.21560-linux-ui-evidence.zip`. Artifact and ZIP hashes
  are in `dist/SHA256SUMS-v1.8.21560.txt`.
- Archived release.lock in the evidence folder and released the active freeze
  after this completed capture cycle, per M1-STATE. For reproducing this exact
  version, restore the archived freeze before running the same recipe.
- This is an existing-design repair and a Linux trial, not a commercial release
  or full platform build. Catalog acquisition, downloads, restart/offline reading,
  media, inference, entitlements and the full gauntlet remain unverified.

## 2026-10-12 — v1.9.63091 diagnostic Linux checkpoint

Manual `cargo tauri build --bundles deb,appimage` completed: TypeScript and Vite
(102 modules) passed, Rust release compile completed, both bundles emitted.
The AppImage opened via tauri-driver/WebKitWebDriver and returned the native Home
DOM. This does not verify media recording, live installation, or offline restart.

- AppImage: `dist/enzime-reader-v1.9.63091-linux-amd64.AppImage`, 83,859,960 bytes;
  SHA256 `c43e2c525b1f260fea06d22a37573831fa17572eb4b5ec53ca79ac43adb33272`.
- Debian: `dist/enzime-reader-v1.9.63091-linux-amd64.deb`, 8,457,860 bytes;
  SHA256 `348111ef6c5a08719fd0a2165eb97e568a990fa951317e69106d05042be337cd`.
- Application source checkpoint: `ec4f83f`; stamped version files accompany this
  evidence. Build log: Admin-Manual/LOGS/EnZIME-v4/native-trial-v1.9.63091-build.log.
- Not an all-platform set or release. Offline selected-entry metadata gap remains.

**Operator correction:** all subsequent builds use one unified script, one source
snapshot and version, all platform lanes, and a current-build manifest. Separate
one-off Linux/Windows/Android builds are not the normal delivery path.

## 2026-10-12 — unified build attempt v1.10.63169

`bash scripts/build-all.sh` ran against committed source `b016ef1`. Preparation
produced web/source artifacts; concurrent Linux/NSIS/Android lanes all rejected
a Rust/npm Tauri minor-version mismatch. Cause: the orchestration fallback used
`cargo generate-lockfile`, upgrading locked third-party dependencies. The corrected
preparation runs full offline Cargo metadata against the existing lock, asserts
third-party name/version/source/checksum tuples unchanged, and forwards --locked
to all platform cargo invocations. The failed generated lock remains in the log.

SBOM generation emitted23 documents; collection erroneously tested absolute paths
for a target component even though every lane lives below target/. Filtering now
uses paths relative to the snapshot. Native MSI/MSIX executor remains unconfigured.
Android chooses the existing known recipe NDK27 only when NDK_HOME is unset.

The script exited1 and published an honest incomplete current manifest with2/12
artifacts. Independent verifier accepts its hashes/state; --require-complete
correctly fails. No old installer is linked in any failed row. Logs are under
Admin-Manual/LOGS/EnZIME-v4/build-sets/20261012T072934.973569Z-b016ef12a393/.


## 2026-10-13 — operator-selected nvme0n1p8 build storage

The operator changed the requested build destination to
`/media/robin/pinokio/pinokio/enzime-build` on `/dev/nvme0n1p8`.
`findmnt` verified that device at the named mount, with 141 GiB free.
An actual executable and relative-symlink probe passed on its fuseblk filesystem.
Keep existing root toolchains and historical attempts; copy reusable caches
without removing their originals. Use the existing explicit environment override:

```bash
ENZIME_BUILD_ENV=/media/robin/pinokio/pinokio/enzime-build/env.sh bash scripts/build-all.sh
```

The profile puts new workspaces, lane compilation caches, Gradle/package caches,
temporary files, logs and artifact collection on nvme0n1p8. Installed Rust,
Cargo registry and Android SDK toolchains remain under /srv/enzime-build;
this is a build-storage selection, not a completed migration of every tool.
No HOME reassignment or Pinokio application-data changes. Keep source checkout
and repository lock in their existing locations. Version stamping remains
automatic through the unified entrypoint; never type an artifact version label.
The manifest remains the authority for actual per-platform results; Linux
cross-compilation does not establish native MSI/MSIX acceptance.


Operator clarification, 17:59 EDT: the purpose is to reclaim /home space.
Copies are temporary protection only. After the destination works and copies
pass checksum verification, remove transferred build data from the old physical
filesystem and retain compatibility symlinks. Measure actual /home free-space
change; moving /srv data alone cannot count as home-space recovery. Current
inspection finds about 25 GiB in this checkout's target/debug and additional
release/Android targets physically on /home. Preserve their contents at the new
destination before removal; retain application data and unrelated projects.
