# Unified EnZIME build set

Contract date: 2026-10-12. Owner: root integrator. Status: implementation contract ready for integrator review; execution remains unverified.

## Requirement and observed baseline

Every build invocation attempts the complete platform matrix under one version, source snapshot and command. Windows NSIS is a first-class required output. A successful Linux lane does not complete a build set. No caller selects a platform subset. Manual script execution is the release path; no CI activation.

Bootstrap used deployed sesh architect role, CodeGraph status (261 files, 4,828 nodes), project CLAUDE and Admin-Manual CC4/CC11/CC12/CC14. CodeGraph explicitly reports parser gaps for `scripts/update-version.sh` and `DOCS/BUILD-RECIPES.md`; targeted reads of those files and adjacent package/Tauri configuration establish this contract. Only the version stamper and mirror helper exist in scripts. Recipes record working Linux deb/AppImage, Linux-cross Windows NSIS, signed Android APK/AAB, CycloneDX and source archive. Current host has cargo-xwin, makensis and cargo-cyclonedx. No Windows executor or makeappx/WiX/PowerShell executable was discovered in the named inventory/current PATH. That observation is not a claim that a remote Windows machine is unavailable.

Existing `release.lock` for 1.9.63091 is a historical Linux-only trial. Preserve it under the build log before starting a new set; do not reuse its version for the unified attempt. Existing dist artifacts remain historical and are never input to collection.

## Entities

New-file line anchors are line 1 until implementation freezes exact positions.

| Entity | Target | Role and exact interface |
|---|---|---|
| build_all | `scripts/build-all.sh:1` | Sole public entrypoint: `bash scripts/build-all.sh [--jobs N] [--source-ref REF]`; default REF=HEAD, N=3; forwards validated args to build_set.py. No platform selector, implicit upload, commit or push. |
| BuildSet | `scripts/build_set.py:1` | Standard-library Python orchestrator; `main(argv:list[str])->int`, `prepare_snapshot(repo:Path,source_ref:str)->BuildSet`, `run_lanes(build:BuildSet,jobs:int)->list[LaneResult]`, `finalize(build:BuildSet,results:list[LaneResult])->int`. Fields version, version_code, source_commit, source_archive_sha256, root, snapshot, started_at, artifact_dir. |
| LaneResult | `scripts/build_set.py:1` | Fields id, status (`success`,`failed`,`unavailable`), exit_code, reason, log_path, started_at, finished_at, artifacts. Every required lane produces one record even on preflight failure. |
| BuildArtifact | `scripts/build_set.py:1` | Fields id, platform, kind, filename, size_bytes, sha256, signing (`verified`,`unsigned`,`not-applicable`), locations; locations hold role/transport/host/path/public=false. |
| build_lane | `scripts/build-lane.sh:1` | Internal interface `bash scripts/build-lane.sh LANE SNAPSHOT TARGET_DIR OUTPUT_DIR`; LANE=linux/windows-nsis/android/sbom/windows-native. Validates paths and version; executes documented recipes; never stamps. |
| verify_build_set | `scripts/verify-build-set.py:1` | Read-only `python3 scripts/verify-build-set.py MANIFEST [--require-complete]`; validates schema, all required target records, exact version filenames, size/hash, source digest and complete status. Default accepts an honestly incomplete attempt; require-complete exits nonzero unless every required artifact succeeded. |
| current_build_manifest | `dist/current-build-manifest.json:1` | Generated atomic snapshot of most recent attempt, never a directory scan; full statuses and relative artifact locations. |
| current_complete_build_manifest | `dist/current-complete-build-manifest.json:1` | Generated only after all required artifacts pass; prior complete set may remain, clearly labeled previous complete version on page. |
| build_downloads | `dist/index.html:1` | Generated self-contained download table with latest attempted version, each target status, exact versioned relative links and hashes. Windows NSIS row always present. Failed current rows never link an old artifact. No remote JS/CDN. |
| build_manifest | `dist/enzime-reader-v<VERSION>-build-manifest.json:1` | Schema `mba.robin.enzime.build-set.v1`; immutable completed-attempt record. Fields version, version_code, status (`complete`,`incomplete`), source, started_at, finished_at, lanes, artifacts, required_artifact_ids. |
| build_checksum_manifest | `dist/enzime-reader-v<VERSION>-sha256sums.txt:1` | Hashes only artifacts produced in this attempt, source archive and SBOM; excludes itself and mutable current pointers. |
| AndroidSymbols | `apps/reader/src-tauri/gen/android/app/build.gradle.kts:39` | Release `ndk.debugSymbolLevel = "FULL"`; collection requires actual native-debug-symbols.zip and mapping.txt when shrinking enabled. |
| WindowsNativeExecutor | environment `ENZIME_WINDOWS_BUILD_EXECUTOR` | Optional absolute executable invoked with arguments SOURCE_ARCHIVE OUTPUT_DIR VERSION; receives exact prepared source archive, builds MSI/MSIX with native tools, returns versioned outputs and machine-readable signing verification. Missing executor emits unavailable required targets, never success or silent skip. Executor config is local, not secrets in source. |

## Preparation and concurrency

1. Acquire an exclusive repository build-set lock under `target/build-set/build.lock` before version mutation. Refuse a second invocation cleanly. Resolve source-ref to a commit and archive only that commit; do not include concurrent uncommitted source edits. Print commit and dirty-path count without file contents. User work stays untouched. The integrator commits intended app changes before invocation.
2. Create semantic workspace `target/build-set/<UTC>-<commit12>/`. Preserve preexisting canonical release.lock to `~/Admin-Manual/LOGS/EnZIME-v4/build-sets/<attempt>/previous-release.lock`. Do not delete or overwrite unrelated locks without provenance: the build mutex owns only its lock and the freshly written version freeze.
3. Extract committed source to snapshot. Copy current canonical version.txt into snapshot as the version counter authority, remove only the snapshot release.lock, invoke its existing `scripts/update-version.sh` exactly once, then generate release.lock from resulting version.json using numeric fields (no octal interpretation). Freeze all version fields including buildDate by preventing subsequent stamper invocation. Synchronize the stamper-owned version files and release.lock back to canonical checkout only if their pre-run bytes still match; a concurrent change fails preparation without overwriting it. Paths: version.txt, version.json, package.json, Cargo.toml, apps/reader/src-tauri/Cargo.toml, apps/reader/src-tauri/tauri.conf.json, apps/reader/src-tauri/gen/android/tauri.properties. Never overwrite other source files.
4. Install frontend dependencies with `pnpm install --frozen-lockfile` in snapshot using shared pnpm store. Run `pnpm --dir apps/reader exec tsc --noEmit` and `pnpm --dir apps/reader exec vite build` directly inside snapshot (or equivalent cwd), avoiding package build hook's stamper. Preserve ordinary direct-development package scripts. Run Cargo lock update needed for stamped workspace version once, before lanes, record resulting Cargo.lock in snapshot and sync canonical Cargo.lock with the same compare-before-write rule. Dependency resolution failure is preparation failure, not permission to ignore lock mismatches.
5. Create source tar.gz from exact prepared snapshot excluding generated frontend, node_modules, target, dist, signing credentials and local config. Include release.lock, stamped manifests and resolved lockfiles. Its SHA256 is authoritative build-source identity in addition to parent commit. Never label this archive as plain committed HEAD: stamping changed it. Dist/source archive never contains credentials. Use tracked committed paths plus the explicit stamped/generated lockfile allowlist.
6. Make independent lane workspaces from prepared snapshot using reflink/copy and shared immutable frontend bytes. Each has its own generated Android tree, Cargo target directory and merged Tauri config. Merge `build.beforeBuildCommand` to harmless `echo Using prepared frontend`; set frontendDist to its copied prepared frontend. No lane reruns stamper or Vite. This prevents both version and Android/desktop generated-code races. Concurrency maximum --jobs; launch all lanes, collect all outcomes even if one fails. Limit Cargo build jobs within each lane to avoid multiplying memory pressure unconstrained.
7. Logs stream with lane prefixes and durable per-lane logs under `~/Admin-Manual/LOGS/EnZIME-v4/build-sets/<attempt>/`. Never log environment dumps or signing secrets. Toolchain caches remain shared; build targets/workspaces isolated. On signal terminate process groups, wait, finalize interrupted statuses; never promote partial files.

## Required matrix and concrete recipes

All names start `enzime-reader-v<VERSION>-`. Collection reads only that lane's empty-at-start output and verifies exactly one matching result. Unstamped raw tool output stays inside target workspaces. The existing Tauri product name `EnZIME Reader` yields internal Debian/RPM package identity `en-zime-reader`; validate that exact identity while retaining the established external artifact filename prefix.

| Artifact ID | Qualifier | Recipe / verification |
|---|---|---|
| linux-appimage | linux-amd64.AppImage | `cargo tauri build --bundles deb,appimage,rpm --config <prepared-config>` in Linux lane apps/reader; executable file nonempty; hash. |
| linux-deb | linux-amd64.deb | Same invocation; `dpkg-deb --field` product/version/architecture matches prepared version. |
| linux-rpm | linux-amd64.rpm | Same invocation; `rpm -qp` package version/architecture; missing tool installed through existing host provisioning, never drop row. |
| windows-nsis | windows-x64-setup.exe | `cargo tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc --bundles nsis --config <prepared-config>`; verify PE installer and nonempty. Record unsigned unless actual signature verification succeeds. |
| windows-msi | windows-x64.msi | WindowsNativeExecutor required; verify native package product/version and hash. No relabeling NSIS as MSI. |
| windows-msix | windows-x64.msix | WindowsNativeExecutor required; verify package manifest identity/version and actual signature. No fabricated manifest identity or certificate; configured publisher must match existing product signing authority. |
| android-apk | android-universal-release-signed.apk | `cargo tauri android build --apk --aab --config <prepared-config>` all four ABI targets; apksigner with existing production keystore, `--ks-pass env:ENZIME_KEYSTORE_PASSWORD --key-pass env:ENZIME_KEY_PASSWORD`; verify signature and manifest versionCode/versionName. |
| android-aab | android-universal-release-signed.aab | Same build, jarsigner uses `-storepass:env ENZIME_KEYSTORE_PASSWORD -keypass:env ENZIME_KEY_PASSWORD`; verify signature, bundle manifest version and all ABI contents. Do not create/rotate keystore. |
| android-symbols | android-native-debug-symbols.zip | Require actual AGP native-debug-symbols.zip containing all four ABIs; optionally include mapping.txt in named archive; inspect ZIP entries. Empty symbols cannot pass. |
| web | web.tar.gz | Archive single prepared frontend; inspect index.html and JS/CSS assets. This is packaged web output, not proof browser-native backend features work. |
| source | source.tar.gz | Prepared allowlisted source archive above; record archive digest and parent commit. |
| sbom | sbom.cdx.json | `cargo cyclonedx --all --format json` in isolated sbom lane; merge dedup by component identity preserving relationships and app version; include frontend production dependency inventory from lockfile, mark dependency scope. Valid CycloneDX document, not renamed npm JSON. |

Keystore location is `ENZIME_KEYSTORE_PATH`, fallback existing repository `signing/production.keystore`; alias `ENZIME_KEYSTORE_ALIAS` default upload. Read password environment or existing local credential source through integrator provisioning, never hardcode donor recipe literals. Do not silently sign with a debug key. Windows Authenticode remains explicit verified/unsigned state; no claim of signed commercial readiness from unsigned NSIS.

MSI/MSIX are required matrix members. Their missing executor does not prevent the current Linux/NSIS/Android/web build attempts, but final exit is nonzero and current-complete pointer cannot advance. An unconfigured native executor is an actionable target failure, not permission to call all platforms complete. The executor implementation/host must be verified before claiming full-complement completion; this interface intentionally exposes that remaining fact rather than inventing a reachable machine.

## Finalization and delivery

Each successful lane copies into canonical tracked dist only after hash/version validation, using temporary files inside the build workspace and atomic replacement on same filesystem. Filename collision with different bytes is an error; do not overwrite historical artifact. Manifest records actual current attempt only. Set current-build-manifest and dist/index atomically even on failure, with failed/unavailable rows clearly visible; do not mix earlier Windows installers into current results. Promote current-complete only if all required IDs succeeded. Process exit 0 means full matrix built; any failed/unavailable/interrupted required target yields nonzero.

Source pushes remain integrator-owned GitHub code publication under outage instruction. No script network publication, Forgejo access, GitHub LFS upload or automatic git commit. Locally staged artifact availability and externally published availability are separate manifest locations and claims.

## Self-contained checklist contract

### UB01 — Implement one build-set entrypoint

**Do:** Add build_all/BuildSet/LaneResult/BuildArtifact/build_lane/verify_build_set and generated manifest/page entities at exact paths above. Implement preparation, immutable snapshot, single stamping, bounded parallel lanes, all-target outcome aggregation, source provenance, signing environment handling and atomic no-stale promotion exactly as this document. Modify only AndroidSymbols release block plus DOCS/BUILD-RECIPES.md to make `bash scripts/build-all.sh` the sole operator build command and retain old recipes as internal provenance. Do not alter app feature source, user credentials, remotes or CI. Files under dist are generated by the script, not hand-authored release claims.

**Verify:** `bash -n scripts/build-all.sh scripts/build-lane.sh`; `python3 -m py_compile scripts/build_set.py scripts/verify-build-set.py`. Run actual `bash scripts/build-all.sh` once after intended source is committed; this is production execution, not an added smoke suite. Independently run `python3 scripts/verify-build-set.py dist/current-build-manifest.json` and inspect lane log outcomes. Then `python3 scripts/verify-build-set.py dist/current-build-manifest.json --require-complete`.

**Accept:** Entry scripts parse; read-only verifier proves each manifest artifact exists with recorded digest and version; required artifact IDs match matrix exactly; no failed current target resolves to an older artifact. Full-complement acceptance requires `--require-complete` exit0. A nonzero build is a documented partial implementation/execution outcome, never a checked complete gate. Native UI/device runtime verification remains the separate reader rubric and is not inferred from package construction.

**Operator verification protocol:** Open dist/index.html and choose current Windows NSIS without searching historical files; confirm visible version equals Linux/Android/web/source manifest. Install/run each built platform under DOCS/TEST_RUBRIC.md. Preserve screencasts and named runtime limitations separately from build hashes.

UB01 integration refinements: private `signing/build-credentials.json` may provide the four named signing variables when absent from process environment; `.gitignore` excludes that exact file and source snapshots exclude signing directories. `ENZIME_SIGNING_CONFIG` overrides its path. Completion promotion invokes the independent verifier before advancing the complete pointer. Cargo.lock reader version is asserted before snapshot freeze. Failed lane directory creation/process errors are converted to terminal lane records. Repeated pre-stamp failures use uniquely named attempt manifests alongside artifacts rather than overwriting historical records.


## UB02 — Android frozen version and full symbols

Evidence from unified1.11.63172: Tauri generated app/tauri.properties versionCode1074172 while frozenversion.json is100011. Installed tauri-utils2.9.3 AndroidConfig supports bundle.android.versionCode and autoIncrementVersionCode. Root gen/android/tauri.properties is not the generated app-level input. All4Cargo libenzime_reader_lib.so and APK/AAB counterparts match exact hashes but have .symtab only, no .debug_info or BuildID. AGP FULL cannot create missing Rust DWARF; mapping is outputs/mapping/universalRelease/mapping.txt. This refines UB01's AGP-only symbols collection.

| Entity | File | Exact contract |
|---|---|---|
| Prepared Android overrides | scripts/build-lane.sh | Prepared merged Tauri config includes bundle.android.versionCode=frozenversion.json.versionCode, autoIncrementVersionCode=false; never derive from BUILD. Android lane sets CARGO_PROFILE_RELEASE_DEBUG=2 and CARGO_PROFILE_RELEASE_STRIP=none before cargo. Existing APK/AAB manifest verification remains strict. |
| collect_android_symbols | scripts/collect-android-symbols.py (new) | Standard-library Python CLI `--target-dir PATH --android-output-dir PATH --apk PATH --aab PATH --output PATH --ndk PATH --version VERSION`. Exact4ABI target mapping above; require DWARF .debug_info in each current Cargo ELF, extract actualsymbols with NDKllvm-objcopy --only-keep-debug into <abi>/libenzime_reader_lib.so.dbg. Include nonempty exactuniversalRelease/mapping.txt and provenance.json (version, method=ndk-llvm-objcopy, tool path/version, input/packaged/symbol SHA256). No synthetic symbols or filename-only validation. |
| Symbol correspondence | same helper | Require APK and AAB ABI libraries correspond to compiled library: exact SHA256 when unchanged; if packaging strips, compare every SHF_ALLOC section's name,type,flags,address,size and bytes including relocations; NOBITS compares metadata. Reject missing/extra/changed allocated sections. Validate extracted debug ELF has .debug_info. Use bounded streaming file hashing/extraction, staging beneath output parent's semantic symbols-work directory, no /tmp. |
| Symbol collection invocation | scripts/build-lane.sh | Call helper after APK/AAB signature+identity/version verification; input target/android buildoutput exact currentlane. Verify4ABI ZIP, then emit symbols artifact. Existing AGP output may be inspected but fallback provenance must truthfully identify extraction. |

**Do:** Root owns build-lane.sh config/env/call and contract updates; delegated worker owns only new collect-android-symbols.py helper, no overlapping edits. Preserve other workers. **Verify:** bash parsing, Python compile; helper rejects existing1.11 libraries without DWARF (use semantic stagingoutput, never overwrite historicalartifact); run actual complete `bash scripts/build-all.sh` after commit. **Accept:** all4currentpackageELFs match symbolinputs, extracteddebugsections+mappingnonempty, APK/AAB embeddedversionCode equalsfrozenvalue, symbolsZIPiscollectedandhashed. Whole-build completion remains dependent on everyrequiredtarget; no one-offbuild.

- [ ] UB02 Android frozen version and full native symbols.
