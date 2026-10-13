# Root-partition build environment

Operator request 2026-10-13: provision all Linux/Windows-cross/Android build dependencies and heavy writable state on the root partition. Observed / is nvme0n1p7 with84GiB free, /opt is a separate35GiB-free partition, /home is another partition with24GiB free. Actual root /srv/enzime-build; convenience symlink /opt/enzime-build. User additionally authorizes moving build-related storage from /home with compatibility symlinks. Verify copies before removing old data. Preserve historical attempts. Never use the F2FS partition on the removable 500GB KINGSTON nvme1n1 drive; root is Btrfs nvme0n1p7 on the 2TB WD drive.

| Entity | Target | Contract |
|---|---|---|
| build environment profile | /srv/enzime-build/env.sh | Export CARGO_HOME,RUSTUP_HOME,ANDROID_HOME,ANDROID_SDK_ROOT,NDK_HOME,JAVA_HOME,GRADLE_USER_HOME,XDG_CACHE_HOME,TMPDIR,PNPM_HOME,npm_config_cache,PNPM_CONFIG_STORE_DIR,XWIN_CACHE_DIR,ENZIME_BUILD_WORKSPACE_ROOT,ENZIME_BUILD_CACHE_ROOT,ENZIME_BUILD_LOG_ROOT,ENZIME_BUILD_ARTIFACT_DIR; no HOME reassignment. PATH uses root-contained tools/system binaries, excludes linuxbrew on /home. Contains no signing secrets. |
| environment loader | scripts/build-all.sh | Source explicit ENZIME_BUILD_ENV or existing ~/.config/enzime/build-env.sh before orchestration; fail if explicit requested file missing. Standard whole-platform command unchanged. |
| build root overrides | scripts/build_set.py | Resolve optional environment overrides for attempt workspace, per-lane caches, logs and artifact collection; defaults preserve old paths. Keep existing repo-local lock to prevent concurrent stamping. Frozen source and version semantics unchanged. |
| operator recipe | DOCS/BUILD-RECIPES.md | Record profile, root device checks, actual tools, same bash scripts/build-all.sh entrypoint and native Windows MSI/MSIX boundary. |

Provision verified existing Rust tools/caches and Android SDK into the root environment without exposing secrets. Install root-contained Node/pnpm; use system JDK21/Clang18/NSIS/native dependencies on root. SDK references in generated project must use env/current SDK; no hidden /home Gradle path. Verify actual resolved tool/cache/output paths. All-platform execution is separate from setup; do not claim native Windows Store packages can be produced solely by Linux cross compilation.
