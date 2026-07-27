# AnZimmermanLib ZIM Library — Critical Header Spec Fix

## Date: February 8, 2026 (Second Fix — Corrects the First Broken Fix)

### Summary

The ZIM header parser in all language implementations was fundamentally wrong:
the 16-byte UUID field at offset 8 was missing, and the magic number was incorrect.
This caused every subsequent header field to be read from the wrong offset, making
all real ZIM files fail to load.

### Correct ZIM Header Layout (80 bytes)

Per https://wiki.openzim.org/wiki/ZIM_file_format:

| Offset | Size | Field          | Notes |
|--------|------|----------------|-------|
| 0      | 4    | magicNumber    | `0x044D495A` (72173914 decimal) |
| 4      | 2    | majorVersion   | 5 or 6 |
| 6      | 2    | minorVersion   | |
| 8      | 16   | uuid           | **WAS MISSING — root cause of all bugs** |
| 24     | 4    | entryCount     | |
| 28     | 4    | clusterCount   | |
| 32     | 8    | pathPtrPos     | Was incorrectly named `urlPtrPos` |
| 40     | 8    | titlePtrPos    | Was incorrectly named `titleIndexPos` |
| 48     | 8    | clusterPtrPos  | |
| 56     | 8    | mimeListPos    | Was incorrectly named `mimeTypeListPos` |
| 64     | 4    | mainPage       | `0xFFFFFFFF` if none. Was `mainPageIndex` |
| 68     | 4    | layoutPage     | Deprecated. Was `layoutPageIndex` |
| 72     | 8    | checksumPos    | |

### Magic Number History

1. **Original**: `0x4D495A5A` ("ZZIM") — wrong
2. **First fix (broken)**: `0x004D495A` — still wrong
3. **Correct value**: `0x044D495A` (72173914) — verified via hex dump and spec

### Fields That Do NOT Exist in the ZIM Spec

These were fabricated and have been removed:
- `articleCount`
- `redirectCount`
- `clusterCountPos`

### Files Changed

- `AnZimmermanLIB/AnZimmermanLib/zimlib.ts` — ZIMReader, ZIMWriter, ZIMReaderBrowser
- `src/services/zimService.ts` — EnZIM app service (same fixes)
- `AnZimmermanLIB/AnZimmermanLib/ARCHITECTURE.md` — Header table corrected, changelog added
- `AnZimmermanLIB/AnZimmermanLib/CHECKLIST.md` — Phase 6 added
- `AnZimmermanLIB/AnZimmermanLib/README.md` — Changelog section added

### Remaining Work

Python (`zimlib.py`), Go (`zimlib.go`), and PHP (`zimlib.php`) still have the
wrong header layout and need the same fix applied.