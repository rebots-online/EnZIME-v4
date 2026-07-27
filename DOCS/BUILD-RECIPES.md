# EnZIME Reader — Build Recipes

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
  -storepass 'Ch4n3l.C3351' -keypass 'Ch4n3l.C3351' \
  -dname 'CN=EnZIME Reader Upload, O=Robin L M Cheung MBA, C=CA'

# APK -> apksigner
$ANDROID_HOME/build-tools/36.1.0/apksigner sign \
  --ks signing/production.keystore --ks-key-alias upload \
  --ks-pass pass:Ch4n3l.C3351 --key-pass pass:Ch4n3l.C3351 \
  --out dist/<slug>-v<VER>-android-universal-release-signed.apk <unsigned.apk>

# AAB -> jarsigner (apksigner does not sign bundles)
jarsigner -keystore signing/production.keystore -storepass 'Ch4n3l.C3351' \
  -keypass 'Ch4n3l.C3351' -sigalg SHA256withRSA -digestalg SHA-256 \
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
