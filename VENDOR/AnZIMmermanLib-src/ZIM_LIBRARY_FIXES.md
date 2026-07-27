# AnZimmermanLib ZIM Library - Critical Fixes

## Summary

This document details critical fixes applied to the AnZimmermanLib ZIM library to ensure proper ZIM file format compliance. These fixes address fundamental issues that prevented the library from correctly reading ZIM archives.

---

## Critical Fixes Applied

### 1. Magic Number Correction (HIGH PRIORITY)

**Issue**: The library was using the wrong magic number `0x4D495A5A` ("ZZIM" in little-endian) instead of the correct `0x004D495A` ("ZIM\0" in little-endian).

**Impact**: All ZIM files failed to open with "Invalid ZIM file format" errors.

**Fix Applied**:
```typescript
// BEFORE (INCORRECT):
const ZIM_MAGIC_NUMBER = 0x4D495A5A; // "ZZIM"
if (header.magicNumber !== 0x4D495A5A) {
    throw new Error('Invalid ZIM file format');
}

// AFTER (CORRECT):
const ZIM_MAGIC_NUMBER = 0x004D495A; // "ZIM\0" in little-endian
if (header.magicNumber !== ZIM_MAGIC_NUMBER) {
    throw new Error(`Invalid ZIM file format: expected magic number 0x${ZIM_MAGIC_NUMBER.toString(16).toUpperCase()}, got 0x${header.magicNumber.toString(16).toUpperCase()}`);
}
```

**Files Modified**:
- `AnZimmermanLIB/AnZimmermanLib/zimlib.ts`
- `src/services/zimService.ts`

---

### 2. ZIM Header Field Addition

**Issue**: The `urlPtrPos` field was missing from the ZIMHeader interface, which is required to locate the directory entries.

**Impact**: Directory entries could not be properly located without this field.

**Fix Applied**:
```typescript
export interface ZIMHeader {
    magicNumber: number;
    majorVersion: number;
    minorVersion: number;
    entryCount: number;
    articleCount: number;
    clusterCount: number;
    redirectCount: number;
    mimeTypeListPos: bigint;
    urlPtrPos: bigint; // NEW: URL pointer list position (at offset 32)
    titleIndexPos: bigint;
    clusterPtrPos: bigint;
    mainPageIndex: number;
}
```

---

### 3. Directory Reading Logic Fix

**Issue**: The library was reading directory entries from a hardcoded offset (80) instead of using the `urlPtrPos` field from the header.

**Impact**: Directory entries were read from wrong locations, causing crashes or missing entries.

**Fix Applied**:
```typescript
// BEFORE (INCORRECT):
const indexPtrStart = 80;
const reader = new BinaryReader(this.fileBuffer.subarray(indexPtrStart));

// AFTER (CORRECT):
// Read URL pointer list from the position specified in header
const urlPtrPos = Number(this.header.urlPtrPos);
const reader = new BinaryReader(this.fileBuffer.subarray(urlPtrPos));
```

---

### 4. Special Entry Type Handling

**Issue**: The library didn't handle special mimetype indices (link targets, deleted entries).

**Impact**: Potential crashes when encountering these special entry types.

**Fix Applied**:
```typescript
export const ZIM_REDIRECT_MIMETYPE = 0xFFFF;      // Redirect entries
export const ZIM_LINKTARGEST_MIMETYPE = 0xFFFE;   // Link target entries
export const ZIM_DELETED_MIMETYPE = 0xFFFD;       // Deleted entries

// In readDirectory():
if (mimetype === ZIM_REDIRECT_MIMETYPE) {
    // Handle redirect
} else if (mimetype === ZIM_LINKTARGEST_MIMETYPE || mimetype === ZIM_DELETED_MIMETYPE) {
    console.warn(`Skipping special entry type: 0x${mimetype.toString(16)}`);
    continue;
} else {
    // Handle regular entry
}
```

---

## Version Updates

- **ZIM Format Version**: Updated to 6.0 (current openZIM standard)
- **Library Version**: Maintained compatibility with existing API

---

## Files Modified

### Core Library (`AnZimmermanLIB/AnZimmermanLib/`)

1. **zimlib.ts**:
   - Added `ZIM_MAGIC_NUMBER` constant (0x004D495A)
   - Added format version constants (6.0)
   - Added special mimetype constants
   - Added `urlPtrPos` to `ZIMHeader` interface
   - Updated `CompressionType` enum with documentation
   - Fixed `readHeader()` to read `urlPtrPos`
   - Fixed `readDirectory()` to use `urlPtrPos` instead of hardcoded offset
   - Fixed magic number check in `ZIMReader.open()`
   - Fixed magic number in `ZIMWriter.create()`

### Application Service (`src/services/`)

2. **zimService.ts**:
   - Added `ZIM_MAGIC_NUMBER` constant
   - Added compression type constants
   - Updated `ZIMHeader` interface with `urlPtrPos`
   - Fixed `readHeader()` to properly read all header fields
   - Updated magic number check with detailed error message

---

## Testing Recommendations

1. **Test with various ZIM files**:
   - Wikipedia dumps (different languages)
   - Gutenberg archives
   - StackExchange dumps
   - Custom ZIM files

2. **Verify magic number detection**:
   - Files with correct magic number should open
   - Files with wrong magic number should fail with descriptive error

3. **Test directory reading**:
   - Verify all entries are loaded
   - Check that redirect entries work
   - Verify article content can be extracted

---

## Backwards Compatibility

These changes maintain API compatibility:
- All existing interfaces remain unchanged (except necessary additions)
- Method signatures remain the same
- Error messages are improved but don't break existing error handling

---

## References

- [openZIM ZIM File Format Specification](https://openzim.org/wiki/ZIM_file_format)
- [ZIM File Format Wiki](https://wiki.openzim.org/wiki/ZIM_file_format)

---

## PR Checklist

- [x] Magic number corrected
- [x] Header interface updated with urlPtrPos
- [x] Directory reading logic fixed
- [x] Special entry types handled
- [x] Documentation added
- [x] Error messages improved
- [x] Format constants added
- [x] Both library and service files updated

---

## Migration Guide

For existing users of the library:

1. **No code changes required** - The API remains compatible
2. **Update imports** - If you were importing the old magic number constant, update to use the new `ZIM_MAGIC_NUMBER`
3. **Test existing code** - Verify your ZIM files still open correctly

---

*Document generated: February 8, 2026*
*Library: AnZimmermanLib*
*Author: Robin L. M. Cheung, MBA*
