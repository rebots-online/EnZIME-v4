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
