# EnZIME Reader — Build Recipes

## Current operator command — every platform, one build set

```bash
bash build_all.sh
```

Commit intended source before invocation. The script snapshots that commit,
stamps one fresh version, prepares the frontend once, and attempts every platform
with bounded concurrent jobs. Optional `--jobs N` controls concurrency;
`--source-ref REF` selects a committed source revision. There is no platform-only
option. Historical recipes below describe the internal commands and prior evidence.

Open `dist/index.html` for the current attempt, including its Windows rows.
`dist/current-build-manifest.json` contains artifact hashes and lane results.
Failed targets never link older versions. The process exits nonzero for any
missing/failed required output.

**Matrix (2026-11-10, CC11).** One Linux invocation produces eleven outputs:
source, web, AppImage, deb, rpm, Windows NSIS installer, Windows standalone
`.exe` (the same release binary the installer carries), Android release APK,
AAB and native-symbols ZIP, and the SBOM. Release profile only; there is no
debug build and no test or smoke-test stage. MSI/MSIX are Store packaging and
are built when booted into Windows: on any other host the `windows-native` lane
is recorded `unavailable` and its two ids are listed under
`deferred_artifact_ids`, so a Linux set is `complete` at eleven.
The Windows half is `build_all_windows.ps1`, run from a Windows boot after
pulling the commit that holds the Linux set:

```powershell
powershell -ExecutionPolicy Bypass -File build_all_windows.ps1
```

It follows the same version scheme as `build_all.sh`: MINOR is bumped and BUILD
recomputed on every run, so a Windows set bears its own new version. One run
builds the standalone `.exe`, NSIS installer, MSI and MSIX together, writes them
to `dist/` as `enzime-reader-v<version>-windows-x64*.{exe,msi,msix}` with
`-windows-build-manifest.json` and `current-windows-build-manifest.json`, then
commits (stamped version files included) and pushes. Windows Installer and MSIX
cap their internal version fields at 255.255.65535, so those two fields carry
`MAJOR.MINOR.0` / `MAJOR.MINOR.0.0`; filenames and the app carry the full version.
Needs: Rust MSVC toolchain, `cargo-tauri`, pnpm, WiX (Tauri downloads it), the
Windows SDK (`makeappx.exe`), and `ENZIME_MS_STORE_PUBLISHER`. Signing happens when
`ENZIME_WINDOWS_CERTIFICATE_THUMBPRINT` and `ENZIME_WINDOWS_TIMESTAMP_URL` are set.
`scripts/verify-build-set.py --require-full-matrix` passes once the current
Windows manifest and its files are present. Status 2026-11-10: written on Linux, not yet run on Windows.

**Android signing** uses the organization-wide production keystore, read by
pointer from `~/Admin-Manual/CREDENTIALS/PlayStore/keystore.properties`
(`storeFile`, `storePassword`, `keyAlias`, `keyPassword`); override the pointer
with `ENZIME_KEYSTORE_PROPERTIES`. Explicit `ENZIME_KEYSTORE_*` environment
values, then `signing/build-credentials.json` / `ENZIME_SIGNING_CONFIG`, take
precedence when present. The tracked `signing/production.keystore` is the
project keystore that signed v1.0.52684 and v1.6.55832 (certificate
`95ffe60a8badd261…`); it is retained for those artifacts and is no longer the
default. The organization certificate is `56c13674ef22df95…`.

The manifest records `monetization_env` (each `ENZIME_BILLING_*`,
`ENZIME_ENTITLEMENT_*`, `ENZIME_GA_*` name as `set`/`unset`, never its value) and
the build warns when billing is unset, because such artifacts ship with
purchasing disabled. Credentials are excluded from source archives.

Workstation prerequisites beyond the Tauri/Android toolchain: `cargo-cyclonedx`
(`cargo install cargo-cyclonedx --locked`) and a `bundletool` launcher on `PATH`.
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
cd /path/to/EnZIME-v4
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

`bash build_all.sh` ran against committed source `b016ef1`. Preparation
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

## 2026-11-10 — unified build attempt v1.18.05715: Android lane failed (8 of 11)

`bash build_all.sh` (then still `scripts/build-all.sh`) ran against committed source
`408c7ed`. Preparation, Linux, Windows NSIS/exe and SBOM lanes succeeded; the
Android lane exited 1; MSI/MSIX deferred to Windows as designed. Manifest:
`dist/enzime-reader-v1.18.05715-build-manifest.json` (status `incomplete`). Log:
`Admin-Manual/LOGS/EnZIME-v4/build-sets/20261110T203552.773738Z-408c7ed7ae4a/android.log`.

**Observed error** (Gradle task `:app:compileUniversalReleaseKotlin`):

```
MainActivity.kt:6:22 Unresolved reference: TauriActivity
```

**Observed cause.** Each attempt builds Android in a fresh lane directory
(`target/build-set/<attempt>/lanes/android`) while Cargo reuses the shared cache
`target/build-set-cache/android`. `gen/android/app/src/main/java/mba/robin/enzime/generated/`
is written by Cargo build scripts during the Rust compile. In this attempt that
directory holds the nine files `wry` writes (`WryActivity.kt`, `RustWebView.kt`, …)
but not `TauriActivity.kt`, which the `tauri` crate's build script writes. In the
1.17.05542 attempt (cold cache, everything compiled) all ten files are present.
So with a warm cache the `tauri` build script was not rerun for the new lane
directory, and Gradle compiled `MainActivity.kt` without its base class.
1.17.05542 was the first Android build of this pipeline and ran cold; 1.18.05715
was the first warm one. The Gradle `keepDebugSymbols` removal made between the
two builds is unrelated: the failure is in Kotlin compilation, before native
libraries are merged or stripped. Whether that removal brings the APK back from
1.2 GB is therefore still unobserved.

**Remedy to undertake** (coder task, `CHECKLIST.md` M0-A1). In the Android lane of
`scripts/build-lane.sh`, before `cargo tauri android build`, force the build
scripts that generate Kotlin to rerun for the current lane directory: run
`cargo clean --release -p tauri -p wry --target <triple>` for each of the four
Android triples against the lane's `CARGO_TARGET_DIR`. After the Tauri build
returns, and before signing, fail with a clear message if
`generated/TauriActivity.kt` or `generated/WryActivity.kt` is absent. The next
`build_all.sh` run is the verification; it bears a new version (1.19 or later).

## 2026-11-10 — unified build attempt v1.21.06024: Android lane failed (8 of 11)

Source commit `d9ec060`. Linux (AppImage, deb, rpm), web, source, SBOM and Windows lanes produced their artifacts;
APK, AAB and symbols ZIP were not produced.

**Failure:** while compiling the app library for `i686-linux-android`:
`error: failed to build archive at .../i686-linux-android/release/deps/libenzime_reader_lib.a: Bad address (os error 14)`.
All workspace crates, including the five changed in this round, had compiled for that target; the failure is in
writing the static library, not in the source.

**Related observation the same hour:** a `cargo check -p reader` on the host crashed inside the compiler on the
dependency `syn` (kernel log 20:42:36: `rustc: segfault ... in librustc_driver`), and the identical rerun passed.
Two memory faults in compiler processes on unrelated inputs point at the machine (memory or kernel), not the code.
Not diagnosed further.

**Remedy:** run `./build_all.sh` again (new version for every platform). If a compiler process faults again, the
machine needs a memory test before further builds are trusted.

## 2026-11-10 — unified build attempt v1.22.06035: Android lane failed again (8 of 11)

Source commit `3f13d83`. Same failure at the same step as v1.21.06024:
`failed to build archive at .../i686-linux-android/release/deps/libenzime_reader_lib.a: Bad address (os error 14)`.
No new compiler segfault in the kernel log during this attempt. A failure that repeats at the same step is not a
random memory fault, so the "machine" explanation recorded for v1.21.06024 is withdrawn as the primary cause.

Observed: the 32-bit x86 cache (`target/build-set-cache/android/i686-linux-android`, 2.3 GB) has no empty files; its
third-party libraries date from the 13:05 build that succeeded (v1.20.05778). Cause not established.

**Remedy to try:** delete only the 32-bit x86 Android cache directory (a regenerable build cache) and run
`./build_all.sh` again, so that target is compiled from nothing. If the same error returns with a fresh cache, the
cause is in the toolchain or in the five source changes since v1.20.05778, and the next step is a coder task.
