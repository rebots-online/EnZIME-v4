# Vendored Source Manifest

**Status in this generated pak:** source trees were not available inside the
current Work workspace and therefore their tree digests could not be attested.

Implementation MUST NOT begin until the pak assembler replaces this status with
`VERIFIED` and records all digests below from the actual included vendor tree.
This is a hard input gap, not permission to fetch an arbitrary latest version.

## Required inputs

| Vendor ID | Required path | Locked revision | Role |
|---|---|---|---|
| `vendor.tauri.v2.11.1` | `VENDOR/tauri/` | tag `v2.11.1` | Tauri reference/source |
| `vendor.litert-lm.1ff29cc` | `VENDOR/litert-lm/` | commit prefix `1ff29cc` | LiteRT-LM runtime |
| `vendor.anzimmerman.current` | `VENDOR/AnZIMmermanLib/` | MUST be filled from supplied source | multilingual ZIM engine |

## Required attestation fields

For each vendor:

```text
vendor_id:
source_url:
source_commit:
tree_sha256:
license_spdx:
license_file:
clean_tree: true
attested_by:
attested_at:
```

## Tree digest algorithm

1. exclude `.git/`, build outputs and platform caches;
2. sort paths by UTF-8 byte order;
3. for every regular file hash:
   `path_length || path_bytes || file_length || file_bytes`;
4. hash the concatenated record stream with SHA-256;
5. record lowercase hexadecimal digest.

The digest tool and its source must be included under `tools/release/`.

## Rejection conditions

- missing revision;
- dirty vendor tree;
- unknown licence;
- unrecorded local patch;
- generated build output inside vendor tree;
- source path mismatch;
- “latest” or floating branch;
- absent AnZIMmerman revision/digest.

