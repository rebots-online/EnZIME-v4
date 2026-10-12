## KI-01 — Canonical absent-title-index compatibility

Status: exact architecture and CODE contract following observed native Kiwix failure; no replacement parser.

Observed Football Database v6.3 header from an exact HTTP206 80-byte read: source112896910 bytes, UUID3f6bebc3c642c6588892774f740d1dac, entries73697, clusters1210, path_ptr_pos112297638, title_ptr_pos18446744073709551615, cluster_ptr_pos112887214, mime_list_pos80, main73693, checksum112896894. Exact captured header hex:
`5a494d04060003003f6bebc3c642c6588892774f740d1dace11f0100ba040000a686b10600000000ffffffffffffffffae85ba06000000005000000000000000dd1f0100ffffffff7eabba0600000000`.

L is `VENDOR/AnZIMmermanLib-src/AnZimmermanLIB/AnZimmermanLib`; R is L/rust.

| Entity | Exact target | Contract |
|---|---|---|
| NO_TITLE_INDEX | R/src/header.rs | Public u64 constant u64::MAX. |
| ZimHeader::title_index_pos | R/src/header.rs | Public fn title_index_pos(&self)->Option<u64>; None for raw0 and for u64::MAX on supported v6; Some(raw) otherwise. Preserve raw header field. |
| validate_header_offsets / cluster_end | R/src/header.rs | Interpret optional title physical position through helper; omit absent title boundary. Mandatory offsets and ordinary invalid title offsets remain strict. |
| RemoteZimIndex::open | R/src/range/index.rs | Use optional title position for table extent arithmetic and MIME boundary; present table checks checked entry_count*4. |
| Rust regression tests | R/tests/range_access/edge_cases.rs | Existing canonical fixture builder/read-only support; v6 zero/allones range and local open/main/article/resource, present-table bounds, captured80byteheader, mandatory-field sentinels rejected, no sentinel fetch. Existing tests unchanged in intent. |
| ZIMHeader.title_index_pos | L/zimlib_format.py | Equivalent optional physical address helper preserving raw integer and serialized bytes. |
| ZIMFile._validate_header / _boundaries | L/zimlib_reader.py | Use helper for optional title table validity and content boundaries. |
| Python parity regression | L/tests/parity/python_parity.py | Same sentinel/ordinary-invalid distinction, raw roundtrip and real fixture payload extraction. |

**Do:** Modify only named Rust/Python sources and regression files; no second parser, archive-wide preload or transport relaxation. Index/CodeGraph assessment already identified exact consumers. Preserve v5 strictness for allones, retain existing zero compatibility. No tests depend on live network. Add equivalent rule to pending portable format implementation: JS/TS bigint before Number; Go uint64; PHP exact eight0xff bytes before signed conversion, never floating point. These later-language contracts remain pending until implemented and tested.

**Verify:** `cargo test --manifest-path R/Cargo.toml --test range_access --test offline_catalog --test conformance` (use exact existing conformance target from Cargo inventory if named differently); `python3 L/tests/parity/python_parity.py`. **Accept:** sentinel fixtures open/read through range and local APIs, captured header validates, regular invalid bounds and mandatory-field sentinels reject, existing tests pass. Report exact commands and counts. Packaged real-Kiwix trial remains separate.

- [ ] KI-01 absent title-index marker in canonical Rust/Python.
