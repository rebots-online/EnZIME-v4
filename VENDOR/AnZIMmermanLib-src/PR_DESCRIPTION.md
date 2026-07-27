# Pull Request: Critical ZIM Library Fixes

## Title
fix: Correct ZIM magic number and directory reading logic

## Description
This PR fixes critical issues in the AnZimmermanLib ZIM library that prevented it from correctly reading ZIM archives.

### Critical Fixes

#### 1. Magic Number Correction (BREAKING FIX)
**Problem**: The library was using the wrong magic number `0x4D495A5A` ("ZZIM") instead of the correct `0x004D495A` ("ZIM\0" in little-endian).

**Impact**: All ZIM files failed to open with "Invalid ZIM file format" errors.

**Solution**: Updated `ZIM_MAGIC_NUMBER` constant to the correct value.

```typescript
// Before (INCORRECT):
const ZIM_MAGIC_NUMBER = 0x4D495A5A; // "ZZIM"

// After (CORRECT):
const ZIM_MAGIC_NUMBER = 0x004D495A; // "ZIM\0" in little-endian
```

#### 2. Added Missing Header Field
**Problem**: The `urlPtrPos` field was missing from the `ZIMHeader` interface.

**Impact**: Directory entries could not be properly located.

**Solution**: Added `urlPtrPos: bigint` to the interface and updated `readHeader()` to read it.

#### 3. Fixed Directory Reading Logic
**Problem**: The library was reading directory entries from a hardcoded offset (80) instead of using the `urlPtrPos` field from the header.

**Impact**: Directory entries were read from wrong locations, causing crashes or missing entries.

**Solution**: Updated `readDirectory()` to use `urlPtrPos` from the header.

```typescript
// Before (INCORRECT):
const indexPtrStart = 80;
const reader = new BinaryReader(this.fileBuffer.subarray(indexPtrStart));

// After (CORRECT):
const urlPtrPos = Number(this.header.urlPtrPos);
const reader = new BinaryReader(this.fileBuffer.subarray(urlPtrPos));
```

#### 4. Added Special Entry Type Handling
**Problem**: The library didn't handle special mimetype indices (link targets at 0xFFFE, deleted entries at 0xFFFD).

**Solution**: Added constants and handling for special entry types.

```typescript
export const ZIM_REDIRECT_MIMETYPE = 0xFFFF;
export const ZIM_LINKTARGEST_MIMETYPE = 0xFFFE;
export const ZIM_DELETED_MIMETYPE = 0xFFFD;
```

### Files Changed

- `AnZimmermanLIB/AnZimmermanLib/zimlib.ts` - Core library fixes
  - Added format constants
  - Fixed magic number
  - Added urlPtrPos to header
  - Fixed directory reading logic
  - Added special entry handling

### Testing

- [ ] Test with Wikipedia ZIM dumps
- [ ] Test with Gutenberg archives
- [ ] Test with StackExchange dumps
- [ ] Verify magic number detection works
- [ ] Verify all directory entries load

### Backwards Compatibility

- API remains compatible
- All existing interfaces preserved
- Improved error messages

### References

- [openZIM ZIM File Format Specification](https://openzim.org/wiki/ZIM_file_format)

---

## Commit Message

```
fix: correct ZIM magic number and directory reading

This commit fixes critical issues that prevented the library from
reading ZIM files:

1. Magic number: Changed from 0x4D495A5A ("ZZIM") to correct
   0x004D495A ("ZIM\0" in little-endian)

2. Header: Added missing urlPtrPos field required to locate
   directory entries

3. Directory reading: Fixed to use urlPtrPos from header instead
   of hardcoded offset 80

4. Special entries: Added handling for link targets (0xFFFE) and
   deleted entries (0xFFFD)

Fixes: All ZIM files failing to open with "Invalid format" error
```

## Checklist

- [x] Magic number corrected
- [x] Header interface updated
- [x] Directory reading logic fixed
- [x] Special entry types handled
- [x] Documentation updated
- [x] Error messages improved
- [x] Format constants added

---

**Note**: This is a critical fix that must be merged before the library can successfully read any ZIM files.
