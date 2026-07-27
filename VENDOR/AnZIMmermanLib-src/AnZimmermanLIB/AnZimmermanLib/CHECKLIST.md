Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
All rights reserved.
Unauthorized use without prior written consent is strictly prohibited.

# AnZimmerman Development Checklist

**Copyright (C) 2025 Robin L. M. Cheung, MBA. All rights reserved.**

## Checklist Legend

- `[ ]` Not yet begun
- `[/]` Started, not complete
- `[X]` Completed, not thoroughly tested
- `✅` Tested and complete

---

## Phase 1: Core Library Implementation (Week 1)

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Python zimlib.py reader | Dev | Week 1 | ✅ | Reads standard ZIM files |
| Python zimlib.py writer | Dev | Week 1 | ✅ | Creates valid ZIM files |
| TypeScript zimlib.ts reader | Dev | Week 1 | ✅ | Node.js and browser support |
| TypeScript zimlib.ts writer | Dev | Week 1 | ✅ | Compression bug fixed |
| Go zimlib.go reader | Dev | Week 1 | ✅ | Handles large files |
| Go zimlib.go writer | Dev | Week 1 | ✅ | Compression implemented |
| PHP zimlib.php reader | Dev | Week 1 | ✅ | Stream-based reading |
| PHP zimlib.php writer | Dev | Week 1 | ✅ | Creates valid ZIM files |

## Phase 2: REST APIs (Week 2)

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Python FastAPI implementation | Dev | Week 2 | ✅ | Swagger UI at /docs |
| TypeScript Express implementation | Dev | Week 2 | ✅ | Swagger UI at /api-docs |
| Go net/http implementation | Dev | Week 2 | ✅ | Swagger UI at /swagger |
| PHP native implementation | Dev | Week 2 | ✅ | Swagger UI at /docs |

## Phase 3: Tools & Integration (Week 3)

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| zim-downloader TUI mode | Dev | Week 3 | ✅ | Browse/download ZIM files |
| zim-downloader Tauri GUI | Dev | Week 3 | [/] | System tray integration |
| TUI theming (9 themes) | Dev | Week 3 | ✅ | Kinetic, Brutalist, Retro, etc. |
| Light/Dark/System mode toggle | Dev | Week 3 | ✅ | Key bindings t/m |
| Version/build in status bar | Dev | Week 3 | ✅ | Bottom right corner |
| Copyright on startup | Dev | Week 3 | ✅ | Displayed before init |
| Chrome extension integration docs | Dev | Week 3 | ✅ | Working example |
| Kiwix-scale integration docs | Dev | Week 3 | ✅ | Docker compose example |
| Annotation system (voice/ink/text) | Dev | Week 3 | ✅ | All annotation types working |
| Cross-platform builds | Dev | Week 3 | ✅ | Windows/Linux/macOS/Android/PWA |
| Kiwix-scale integration docs | Dev | Week 3 | ✅ | Docker compose example |

## Phase 4: Documentation & Testing (Week 4)

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Architecture documentation | Dev | Week 4 | ✅ | Per-language docs complete |
| Usage examples (all languages) | Dev | Week 4 | ✅ | Create/read/edit examples |
| Unit tests (Python) | Dev | Week 4 | [X] | >80% coverage |
| Unit tests (TypeScript) | Dev | Week 4 | [X] | >80% coverage |
| Unit tests (Go) | Dev | Week 4 | [X] | >80% coverage |
| Unit tests (PHP) | Dev | Week 4 | [X] | >80% coverage |
| Integration tests | Dev | Week 4 | [/] | Cross-library compatibility |

## Phase 5: Polish & Release (Week 4+)

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Version/build system | Dev | Week 4 | ✅ | Epoch-based build numbers |
| License update (All rights reserved) | Dev | Week 4 | ✅ | Proper copyright |
| README cleanup | Dev | Week 4 | ✅ | Accurate license info |
| ARCHITECTURE.md created | Dev | Week 4 | ✅ | System diagrams |
| PHP/Go installed | Dev | Week 4 | ✅ | PHP 8.3, Go 1.22 |
| Bug fixes (compression) | Dev | Week 4 | [ ] | TS/Go compression fixed |
| Performance optimization | Dev | Week 4 | [ ] | Large file handling |

## Phase 6: ZIM Header Spec Compliance Fix (2026-02-08)

**Severity:** Critical — all ZIM files failed to load due to wrong header layout.

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Fix magic number to `0x044D495A` (all langs) | Dev | 2026-02-08 | [X] TS only | Matches ZIM spec (72173914 decimal) |
| Add 16-byte UUID field at offset 8 | Dev | 2026-02-08 | [X] TS only | Header reads correct field values |
| Remove fabricated fields (`articleCount`, `redirectCount`, `clusterCountPos`) | Dev | 2026-02-08 | [X] TS only | Interface matches spec exactly |
| Rename fields to spec names (`pathPtrPos`, `titlePtrPos`, `mimeListPos`, `mainPage`, `layoutPage`) | Dev | 2026-02-08 | [X] TS only | Consistent with openzim.org spec |
| Fix `readDirectory()` to use `header.pathPtrPos` | Dev | 2026-02-08 | [X] TS only | No hardcoded offsets |
| Fix `ZIMWriter.create()` and `finalize()` header layout | Dev | 2026-02-08 | [X] TS only | Writes spec-compliant 80-byte header |
| Fix `ZIMReaderBrowser` header parsing | Dev | 2026-02-08 | [X] TS only | Matches ZIMReader fixes |
| Fix `getMainPage()` to check `0xFFFFFFFF` sentinel | Dev | 2026-02-08 | [X] TS only | Handles "no main page" correctly |
| Update ARCHITECTURE.md header table | Dev | 2026-02-08 | [X] | Correct spec table with changelog |
| Apply same fix to `zimlib.py` | Dev | TBD | [ ] | Python reads real ZIM files |
| Apply same fix to `zimlib.go` | Dev | TBD | [ ] | Go reads real ZIM files |
| Apply same fix to `zimlib.php` | Dev | TBD | [ ] | PHP reads real ZIM files |
| Runtime test with real ZIM file | Dev | TBD | [ ] | freecodecamp_en_all_2025-11.zim loads |

## Phase 7: CI/CD Cross-Platform Build Pipeline (2026-02-08)

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Add `master` branch to workflow triggers | Dev | 2026-02-08 | [X] | Workflow runs on push to master |
| Fix Rust action (`rust-action` → `rust-toolchain`) | Dev | 2026-02-08 | [X] | Action resolves correctly |
| Fix tsc `noUnusedLocals` errors | Dev | 2026-02-08 | [X] | `tsc --noEmit` passes clean |
| Remove stale `pnpm-lock.yaml` | Dev | 2026-02-08 | [X] | No pnpm auto-detection |
| Force `tauriScript: npx tauri` in tauri-action | Dev | 2026-02-08 | [X] | Uses npm, not pnpm |
| Fix tauri crate/CLI version mismatch in CI | Dev | 2026-02-08 | [/] | All 4 targets build green |
| Push to GitHub mirror (`rebots-online/EnZIME`) | Dev | 2026-02-08 | [X] | GitHub Actions runners available |
| Linux build (ubuntu-22.04, x86_64-linux-gnu) | Dev | 2026-02-08 | [ ] | `.deb` + `.AppImage` artifacts |
| Windows build (windows-latest, x86_64-msvc) | Dev | 2026-02-08 | [ ] | `.msi` + `.exe` artifacts |
| macOS x64 build (macos-latest, x86_64-darwin) | Dev | 2026-02-08 | [ ] | `.dmg` artifact |
| macOS ARM64 build (macos-latest, aarch64-darwin) | Dev | 2026-02-08 | [ ] | `.dmg` artifact |
| Android build (ubuntu-latest, aarch64/armv7/x86_64) | Dev | 2026-02-08 | [ ] | `.apk` + `.aab` artifacts |
| iOS build (macos-latest, aarch64-apple-ios) | Dev | TBD | [ ] | `.ipa` artifact (needs Apple cert) |

## Phase 8: UI/UX Bug Fixes & Internal Download Manager (2026-02-08)

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Fix archive open → navigate to Reader | Dev | 2026-02-08 | [X] | Clicking archive card opens reader view |
| Fix Download Archives button always visible | Dev | 2026-02-08 | [X] | Button in library header, not just empty state |
| Move delete to selection bar (prevent accidental delete) | Dev | 2026-02-08 | [X] | Trash only in bottom bar when items selected |
| Internal download manager (Kiwix OPDS catalog browser) | Dev | 2026-02-08 | [X] | In-app catalog with search, filters, pagination |
| Download to app storage with progress tracking | Dev | 2026-02-08 | [X] | Progress bar, auto-import to library on complete |
| Article browser in Reader view | Dev | TBD | [ ] | Browse/list articles in loaded ZIM, not just search |

## Phase 9: Entitlements Matrix & RevenueCat (2026-02-08)

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| 3-tier entitlement matrix (Free/Annotator/Creator) | Dev | 2026-02-08 | [X] | types.ts, defaults.ts with all capability keys |
| RevenueCat provider (`revenueCatProvider.ts`) | Dev | 2026-02-08 | [X] | Maps RC entitlements → app capabilities |
| Gatekeeper tests for all 3 tiers | Dev | 2026-02-08 | [X] | 3 tests passing (free, annotator, creator) |
| Micropayment/virtual currency types | Dev | 2026-02-08 | [X] | Interfaces for BTC Lightning / credit packs |
| RevenueCat dashboard setup (products/offerings) | Dev | TBD | [ ] | RC project configured with annotator + creator |
| Annotation quota tracking in UI | Dev | TBD | [ ] | Trial counter, upsell when 25 exhausted |
| BTC Lightning micropayment provider | Dev | TBD | [ ] | Purchase annotation/highlight packs with sats |

## Phase 10: CI Improvements & Versioned Artifacts (2026-02-08)

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Version in artifact filenames (`enzim-v0.1.0-09629-linux`) | Dev | 2026-02-08 | [X] | Disambiguates cycling 5-digit build numbers |
| Fix Android CI build (cargo lib target) | Dev | TBD | [ ] | Android job green alongside 4 desktop builds |

## Phase 11: "Talk to your ZIMs" — Voice Conversation (Roadmap)

**Prerequisite**: Phase 9 (RevenueCat entitlements) must be complete first.

### Architecture: Modular Provider Abstraction

All LLM, STT, and TTS providers behind interfaces — config-driven, no code changes when models/providers change.

```
src/services/speech/
  speechProvider.ts       — interface { startListening, speak, stop }
  parakeetProvider.ts     — on-device STT via Sherpa-ONNX WASM (Parakeet-TDT)
  kokoroProvider.ts       — on-device TTS via Sherpa-ONNX WASM (Kokoro/Piper)
  webSpeechProvider.ts    — fallback: browser Web Speech API

src/services/llm/
  llmProvider.ts          — interface { complete(prompt, context): string }
  zhipuProvider.ts        — z.ai GLM-4.7-Flash (free) / GLM-4.7 (paid)
  geminiProvider.ts       — Google Gemini 2.0 Flash / 2.5 Pro
  providerRouter.ts       — config-driven selection + automatic failover
```

### LLM Provider Pricing & Failover

| Priority | Provider | Model | Cost / 1M tok (in/out) | Tier |
|----------|----------|-------|------------------------|------|
| Primary | Zhipu (z.ai) | GLM-4.7-Flash | Free (rate-limited) | Free |
| Failover 1 | Google | Gemini 2.0 Flash | $0.10 / $0.40 | Free (failover) |
| Failover 2 | Zhipu | GLM-4.7 | $0.60 / $2.20 | Annotator |
| Premium | Google | Gemini 2.5 Pro | $2.00 / $12.00 | Creator |

Model names, endpoints, and API keys all in env vars / config — no hardcoded model strings.

### On-Device Models (STT + TTS — free tier, no cloud)

| Function | Model | Size | Runtime |
|----------|-------|------|---------|
| STT | Parakeet-TDT-0.6B (NVIDIA) | ~120MB | Sherpa-ONNX WASM |
| TTS | Kokoro (Hexgrad) | ~80MB | Sherpa-ONNX WASM |
| Fallback | Web Speech API | 0MB | Browser built-in |

Models downloaded on first use (~200MB), stored in appDataDir/models/.

### Entitlement Gating

| Feature | Free | Annotator ($) | Creator ($$) |
|---------|------|---------------|--------------|
| On-device STT/TTS | ✅ | ✅ | ✅ |
| ZIM search answers (snippet extraction) | ✅ | ✅ | ✅ |
| Cloud AI conversation (GLM Flash → Gemini failover) | ✅ (rate-limited) | Pay-as-you-go credits | Included (quota) |
| Premium LLM (GLM-4.7 / Gemini Pro) | ❌ | ✅ | ✅ |
| Future on-device LLM (Gemma 3n / LFM 2.5) | ❌ | ❌ | ✅ When ready |

### UX

- **Mobile**: Mic FAB → speak → ZIM search + LLM answer → TTS reads aloud
- **Slide-in transcript tray**: swipe-up for full chat history + text input
- **Desktop**: Docked panel, keyboard-first, optional voice

### Tasks

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| `speechProvider.ts` interface + Web Speech fallback | Dev | TBD | [ ] | STT/TTS works in browser immediately |
| `llmProvider.ts` interface + provider router | Dev | TBD | [ ] | Config-driven, no hardcoded model names |
| `zhipuProvider.ts` (GLM-4.7-Flash + GLM-4.7) | Dev | TBD | [ ] | Free + paid tier inference |
| `geminiProvider.ts` (Gemini 2.0 Flash + 2.5 Pro) | Dev | TBD | [ ] | Failover + premium tier |
| `conversationEngine.ts` — ZIM search → answer extraction | Dev | TBD | [ ] | Query ZIM, extract snippets, format answer |
| `conversationStore.ts` — Zustand message store | Dev | TBD | [ ] | Messages, listening/speaking state |
| `VoiceFab.tsx` — floating mic button (mobile) | Dev | TBD | [ ] | Tap to speak, waveform animation |
| `TranscriptTray.tsx` — slide-in chat + text input | Dev | TBD | [ ] | Swipe-up drawer, chat history, type queries |
| Sherpa-ONNX WASM integration (Parakeet STT) | Dev | TBD | [ ] | On-device STT replacing Web Speech |
| Sherpa-ONNX WASM integration (Kokoro TTS) | Dev | TBD | [ ] | On-device TTS replacing Web Speech |
| Model download manager for speech models | Dev | TBD | [ ] | First-use download ~200MB, progress UI |
| Credit/quota tracking for cloud AI usage | Dev | TBD | [ ] | Pay-as-you-go + subscription quota |

---

## Completed Features

### ZIM Downloader
- **TUI Mode**: Full terminal interface with browse, search, download management
- **Tauri GUI**: React-based UI with system tray, dark/light themes
- **Annotations**: Voice notes, ink drawing, typed post-it notes
- **Cross-platform**: Windows (msi/exe), Linux (deb/appimage), macOS, Android, PWA

### Zimmer Chrome Extension
- Save individual pages, selections, links, or entire sites
- Offline viewing with built-in viewer
- Version comparison between archives
- Annotation system (highlight, ink, notes)
- Export/import archives

### Library Fixes
- TypeScript: Fixed `inflateSync` → `deflateSync` compression bug
- Go: Implemented ZLIB compression using `zlib.NewWriter`

## Phase 12: Directory Entry Parsing Fix — Mimetype uint16 + parameterLen (2026-02-12)

**Severity:** Critical — all ZIM files showed "0 articles" despite valid content.

**Root Cause:** The ZIM spec defines directory entries as:
```
offset 0: uint16  mimetype        (2 bytes)
offset 2: uint8   parameter_len   (1 byte)
offset 3: char    namespace       (1 byte)
offset 4: uint32  revision        (4 bytes)
...
```
The library read `mimetype` as `uint32` (4 bytes) and skipped `parameter_len` entirely,
causing a 2-byte offset shift on every subsequent field. The `namespace` byte read garbage
from the revision field, so article filtering by namespace `'A'` (0x41) returned empty arrays.
The redirect peek (`0xFFFF`) also never matched because it compared a 4-byte value.

**Fix applied to `zimlib.ts`:**
- `ZIMReader.readDirectory()`: peek mimetype as `readUInt16LE()`
- `ZIMReader.readDirectoryEntry()`: mimetype `readUInt32LE()` → `readUInt16LE()`, add `readUInt8()` for parameterLen
- `ZIMReader.readRedirectEntry()`: same fix
- `ZIMReaderBrowser.readDirectory()`: peek mimetype as `readUInt16LE()`
- `ZIMReaderBrowser.readDirectoryEntry()`: same fix as ZIMReader
- `ZIMReaderBrowser.readRedirectEntry()`: same fix as ZIMReader
- `ZIMWriter.serializeEntry()`: `writeUInt32LE()` → `writeUInt16LE()` for mimetype, add `writeUInt8(0)` for parameterLen

| Task | Owner | Due | Status | Acceptance Criteria |
|------|-------|-----|--------|---------------------|
| Fix mimetype read from uint32 to uint16 (ZIMReader) | Dev | 2026-02-12 | [X] | Directory entries parse correctly |
| Add parameterLen uint8 read (ZIMReader) | Dev | 2026-02-12 | [X] | Namespace field aligns to spec |
| Fix mimetype read (ZIMReaderBrowser) | Dev | 2026-02-12 | [X] | Browser reader matches fix |
| Add parameterLen uint8 read (ZIMReaderBrowser) | Dev | 2026-02-12 | [X] | Browser reader matches fix |
| Fix mimetype write (ZIMWriter.serializeEntry) | Dev | 2026-02-12 | [X] | Written ZIM files spec-compliant |
| Add parameterLen uint8 write (ZIMWriter) | Dev | 2026-02-12 | [X] | Written entries include paramLen byte |
| Apply same fix to `zimlib.py` | Dev | TBD | [ ] | Python directory parsing correct |
| Apply same fix to `zimlib.go` | Dev | TBD | [ ] | Go directory parsing correct |
| Apply same fix to `zimlib.php` | Dev | TBD | [ ] | PHP directory parsing correct |
| Runtime test with real ZIM file | Dev | TBD | [ ] | freecodecamp_en_all_2025-11.zim loads articles |

---

## Next Actions

1. **[CRITICAL]** Apply directory entry mimetype/parameterLen fix to `zimlib.py`, `zimlib.go`, `zimlib.php` (see Phase 12)
2. **[CRITICAL]** Apply ZIM header spec compliance fix to `zimlib.py`, `zimlib.go`, `zimlib.php` (see Phase 6)
3. Runtime-test all languages against `freecodecamp_en_all_2025-11.zim`
4. Fix TypeScript compression bug (zimlib.ts: use deflateSync not inflateSync)
5. Complete Go compression implementation (zimlib.go)
6. Add unit test suites for all languages
7. Complete Tauri GUI for zim-downloader

## Themes Available

| Theme | Description |
|-------|-------------|
| **Kinetic** | Colorful, dynamic, Gumroad-inspired design |
| **Brutalist** | Raw, honest, monospace aesthetic |
| **Retro** | CRT terminal vibes with scanlines |
| **Neumorphism** | Soft shadows, extruded surfaces |
| **Glassmorphism** | Frosted glass with depth |
| **Y2K** | Early 2000s web maximalism |
| **Cyberpunk** | Neon-soaked dystopian future |
| **Minimal** | Clean Swiss design |
| **Nordic** | Cool, calm, Scandinavian palette |

---

*Last updated: 2026-01-04T18:05:00Z*
