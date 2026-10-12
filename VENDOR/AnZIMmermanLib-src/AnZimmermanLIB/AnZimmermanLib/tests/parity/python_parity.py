"""Standard-library Python acceptance; canonical fixture and bounded operations."""
import hashlib
import io
import json
import lzma
from pathlib import Path
import struct
import sys
import tempfile
import unittest
import zlib

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))
from zimlib import (
    ZIMReader, ZIMWriter, ZIMHeader, DirectoryEntry, RedirectEntry,
    SpecialEntry, ZimError, RangeReadLimits, ClusterDecodeLimits,
    PagedRangeReader, parse_entry, cluster_encoding, decode_cluster_to,
    read_cluster_blob,
)
BASELINE = (ROOT / 'tests/fixtures/rust-consolidation.zim').read_bytes()
EXPECTED = json.loads((ROOT / 'tests/fixtures/rust-consolidation.expected.json').read_text())


class Source:
    def __init__(self, data, length=None):
        self.data = bytes(data)
        self.length = len(data) if length is None else length
        self.requests = []

    def read_range(self, offset, length):
        self.requests.append((offset, length))
        chunk = self.data[offset:offset + length]
        return chunk + b'\0' * (length - len(chunk))


class Sink(io.BytesIO):
    def __init__(self, data=b''):
        super().__init__(data)
        self.peak_write = 0

    def write(self, data):
        self.peak_write = max(self.peak_write, len(data))
        return super().write(data)


class PythonParity(unittest.TestCase):
    def assert_code(self, code, action):
        with self.assertRaises(ZimError) as caught:
            action()
        self.assertEqual(caught.exception.code, code, str(caught.exception))

    def reader(self, data=BASELINE, **kwargs):
        reader = ZIMReader(source=Source(data), **kwargs)
        reader.open()
        self.addCleanup(reader.close)
        return reader

    def cluster(self, payload=b'payload', codec=1, width=4):
        body = struct.pack('<II' if width == 4 else '<QQ', width * 2, width * 2 + len(payload)) + payload
        if codec == 2:
            body = zlib.compress(body)
        elif codec == 4:
            body = lzma.compress(body)
        elif codec == 12:  # test selector for raw DEFLATE on codec 2
            encoder = zlib.compressobj(wbits=-15)
            body = encoder.compress(body) + encoder.flush()
            codec = 2
        return bytes([codec | (16 if width == 8 else 0)]) + body

    def test_shared_fixture_payload_identity_unicode_main_and_mime(self):
        self.assertEqual(hashlib.sha256(BASELINE).hexdigest(), EXPECTED['sha256'])
        reader = self.reader()
        self.assertEqual(reader.header.uuid.hex(), EXPECTED['uuid'])
        self.assertEqual(reader.header.to_bytes(), BASELINE[:80])
        self.assertEqual(reader.get_main_page().url, 'Home')
        self.assertIsInstance(reader.get_main_page(), RedirectEntry)
        self.assertEqual(reader.get_main_page().redirect_index, 0)
        for expected in EXPECTED['entries']:
            entry = reader.get_entry_by_path(expected['key'])
            self.assertEqual(entry.title, expected['title'])
            if 'payload' in expected:
                self.assertEqual(reader.mime_types[entry.mimetype_index], expected['mime'])
                self.assertEqual(reader.get_article_content(entry), expected['payload'].encode())
        self.assertEqual(reader.get_article_content(reader.get_main_page()), b'<p>Library fixture</p>')
        self.assertEqual(len(reader.directory_entries), 3)
        self.assertFalse(isinstance(reader.directory_entries, list))
        self.assertEqual(len(reader.list_articles()), 2)
        self.assertEqual(reader.header.mime_type_list_pos, reader.header.mime_list_pos)

    def test_absent_v6_title_pointer_preserves_bytes_and_reads_payload(self):
        for marker in (0, (1 << 64) - 1):
            data = bytearray(BASELINE)
            struct.pack_into('<H', data, 6, 3)
            struct.pack_into('<Q', data, 40, marker)
            reader = self.reader(data)
            self.assertEqual(reader.header.title_ptr_pos, marker)
            self.assertIsNone(reader.header.title_index_pos)
            self.assertEqual(reader.header.to_bytes(), data[:80])
            self.assertEqual(reader.get_article_content(reader.get_main_page()), b'<p>Library fixture</p>')
            self.assertEqual(reader.get_article_content(reader.get_entry_by_path('I/pixel.txt')), b'pixel')
        for field in (32, 48, 56):
            data = bytearray(BASELINE); struct.pack_into('<Q', data, field, (1 << 64) - 1)
            self.assert_code('InvalidFormat', lambda data=data: self.reader(data))
        for position in (len(BASELINE), len(BASELINE) + 1, len(BASELINE) - 1, (1 << 64) - 2):
            data = bytearray(BASELINE); struct.pack_into('<Q', data, 40, position)
            self.assert_code('InvalidFormat', lambda data=data: self.reader(data))
        data = bytearray(BASELINE)
        struct.pack_into('<H', data, 4, 5)
        struct.pack_into('<Q', data, 40, (1 << 64) - 1)
        self.assert_code('InvalidFormat', lambda: self.reader(data))
        data = bytearray(BASELINE)
        position = len(data)
        data.extend(struct.pack('<III', 0, 1, 2))
        struct.pack_into('<Q', data, 40, position)
        self.assertEqual(self.reader(data).header.title_index_pos, position)

    def test_header_directory_and_pointer_failures(self):
        self.assert_code('InvalidFormat', lambda: self.reader(BASELINE[:79]))
        bad = bytearray(BASELINE); bad[0] = 0
        self.assert_code('InvalidFormat', lambda: self.reader(bad))
        bad = bytearray(BASELINE); struct.pack_into('<H', bad, 4, 9)
        self.assert_code('UnsupportedFeature', lambda: self.reader(bad))
        bad = bytearray(BASELINE); struct.pack_into('<Q', bad, 32, (1 << 64) - 4)
        self.assert_code('InvalidFormat', lambda: self.reader(bad))
        header = ZIMHeader.from_bytes(BASELINE)
        bad = bytearray(BASELINE); struct.pack_into('<Q', bad, header.path_ptr_pos, len(BASELINE) + 8)
        reader = self.reader(bad)
        self.assert_code('InvalidFormat', lambda: reader.entry_at(0))
        for data in [b'\0' * 7, struct.pack('<HBBI', 0, 0, 65, 0), struct.pack('<HBBIII', 0, 0, 65, 0, 0, 0) + b'url', struct.pack('<HBBIII', 0, 0, 65, 0, 0, 0) + b'\xff\0title\0']:
            self.assert_code('InvalidFormat', lambda data=data: parse_entry(data))

    def test_parameters_special_variants_and_original_indices(self):
        entry = DirectoryEntry(0, 65, 2, 0, 1, 'Café', 'Title', b'\x01\x00\xff')
        decoded, consumed = parse_entry(entry.to_bytes() + b'unconsumed')
        self.assertEqual(decoded, entry); self.assertEqual(consumed, len(entry.to_bytes()))
        self.assert_code('InvalidFormat', lambda: parse_entry(entry.to_bytes()[:-1]))
        h = ZIMHeader.from_bytes(BASELINE)
        data = bytearray(BASELINE); at = struct.unpack_from('<Q', data, h.path_ptr_pos)[0]
        special = SpecialEntry(0xFFFE, 65, 0, 'Café', 'Café').to_bytes()
        data[at:at + len(special)] = special
        reader = self.reader(data)
        self.assertIsInstance(reader.entry_at(0), SpecialEntry)
        self.assertEqual(reader.entry_at(1).url, 'Home')
        self.assert_code('UnsupportedFeature', lambda: reader.get_article_content(reader.get_main_page()))
        self.assert_code('UnsupportedFeature', lambda: reader.get_entry_by_path('A/Café'))

    def test_redirect_cycle_target_and_limit(self):
        h = ZIMHeader.from_bytes(BASELINE)
        at = struct.unpack_from('<Q', BASELINE, h.path_ptr_pos + 8)[0]
        data = bytearray(BASELINE); struct.pack_into('<I', data, at + 8, 1)
        reader = self.reader(data)
        self.assert_code('RedirectCycle', lambda: reader.resolve_entry(reader.get_main_page()))
        data = bytearray(BASELINE); struct.pack_into('<I', data, at + 8, 99)
        reader = self.reader(data)
        self.assert_code('EntryNotFound', lambda: reader.resolve_entry(reader.get_main_page()))
        reader = self.reader(limits=RangeReadLimits(max_redirect_hops=1))
        reader.entry_at = lambda index: RedirectEntry(0xFFFF, 65, 0, index + 1, str(index), str(index))
        self.assert_code('RedirectLimit', lambda: reader.resolve_entry(reader.entry_at(0)))

    def test_streamed_codecs_widths_empty_single_frame_and_bounds(self):
        for codec in (0, 1, 2, 4, 12):
            for width in (4, 8):
                for payload in (b'', b'x' * 300000):
                    with self.subTest(codec=codec, width=width, length=len(payload)):
                        encoded = self.cluster(payload, codec, width)
                        output = Sink(b'stale data')
                        decoded = decode_cluster_to(io.BytesIO(encoded), output, ClusterDecodeLimits())
                        self.assertEqual(read_cluster_blob(output, decoded, 0, 33554432), payload)
                        self.assertLessEqual(output.peak_write, 65536)
                        self.assertFalse(output.closed)
                        if codec in (2, 4, 12):
                            second = Sink()
                            result = decode_cluster_to(io.BytesIO(encoded + b'ignored trailing frame'), second, ClusterDecodeLimits())
                            self.assertEqual(result, decoded)
        for codec in (2, 4, 12):
            output = Sink(b'stale')
            self.assert_code('InvalidFormat', lambda codec=codec: decode_cluster_to(io.BytesIO(self.cluster(codec=codec)[:-2]), output, ClusterDecodeLimits()))
            self.assertEqual(output.getvalue(), b'')
        for codec in (3, 5, 15):
            self.assert_code('UnsupportedCompression', lambda codec=codec: decode_cluster_to(io.BytesIO(bytes([codec])), Sink(), ClusterDecodeLimits()))
        self.assert_code('InvalidFormat', lambda: cluster_encoding(0x21))
        output = Sink(b'stale')
        self.assert_code('DecompressionLimit', lambda: decode_cluster_to(io.BytesIO(self.cluster(b'x' * 200000, 2)), output, ClusterDecodeLimits(maximum_output_bytes=100)))
        self.assertEqual(output.getvalue(), b'')
        self.assert_code('DecompressionLimit', lambda: decode_cluster_to(io.BytesIO(self.cluster(codec=2)), Sink(), ClusterDecodeLimits(maximum_decoder_window_bytes=1024)))
        self.assert_code('DecompressionLimit', lambda: decode_cluster_to(io.BytesIO(self.cluster(codec=4)), Sink(), ClusterDecodeLimits(maximum_decoder_window_bytes=1024)))
        dictionary = zlib.compressobj(zdict=b'dictionary')
        self.assert_code('UnsupportedFeature', lambda: decode_cluster_to(io.BytesIO(b'\x02' + dictionary.compress(b'dictionary') + dictionary.flush()), Sink(), ClusterDecodeLimits()))

    def test_malformed_blob_tables_and_allocation_limits(self):
        for table in [struct.pack('<II', 7, 8), struct.pack('<II', 0xFFFFFFFC, 8), struct.pack('<III', 12, 15, 14) + b'abc', struct.pack('<II', 8, 99)]:
            blob = 1 if len(table) == 15 else 0
            self.assert_code('InvalidFormat', lambda table=table, blob=blob: read_cluster_blob(io.BytesIO(table), (len(table), 4), blob, 1024))
        output = Sink(); decoded = decode_cluster_to(io.BytesIO(self.cluster(b'four')), output, ClusterDecodeLimits())
        self.assert_code('AllocationLimit', lambda: read_cluster_blob(output, decoded, 0, 3))
        self.assert_code('EntryNotFound', lambda: read_cluster_blob(output, decoded, 1, 1024))

    def test_virtual_tib_source_cache_and_short_reads(self):
        source = Source(BASELINE, 1 << 40)
        reader = ZIMReader(source=source); reader.open()
        self.assertEqual(reader.get_article_content(reader.get_entry_by_path('A/Café')), b'<p>Library fixture</p>')
        self.assertEqual(reader.resolve_entry(reader.get_main_page()).url, 'Café')
        self.assertLess(sum(length for _, length in source.requests), 1024 * 1024)
        self.assertLessEqual(max(length for _, length in source.requests), 65536)
        for index in range(20):
            reader._reader.read_exact(index * 65536, 1)
        self.assertEqual(reader._reader.cache_counts(), (8, 8))
        self.assert_code('AllocationLimit', lambda: reader._reader.read_exact(0, 1048577))
        self.assert_code('InvalidFormat', lambda: reader._reader.read_exact((1 << 64) - 1, 8))
        bad = Source(BASELINE); bad.read_range = lambda offset, length: b''
        self.assert_code('Io', lambda: PagedRangeReader(bad).read_exact(0, 1))
        self.assert_code('AllocationLimit', lambda: PagedRangeReader(source, RangeReadLimits(cached_pages=17)))
        self.assert_code('AllocationLimit', lambda: self.reader(limits=RangeReadLimits(max_mime_bytes=1)))
        self.assert_code('InvalidFormat', lambda: self.reader(limits=RangeReadLimits(max_entry_bytes=8)).entry_at(0))

    def test_last_cluster_stops_before_metadata_not_checksum(self):
        h = ZIMHeader.from_bytes(BASELINE)
        pointer_bytes = BASELINE[h.cluster_ptr_pos:h.cluster_ptr_pos + h.cluster_count * 8]
        data = bytearray(BASELINE[:h.checksum_pos])
        h.cluster_ptr_pos = len(data)
        data.extend(pointer_bytes)
        h.checksum_pos = len(data)
        data.extend(b'\0' * 16)
        data[:80] = h.to_bytes()
        reader = self.reader(data)
        self.assertEqual(reader.get_article_content(reader.get_entry_by_path('I/pixel.txt')), b'pixel')
        store = Sink(); reader.get_article_content_to(reader.get_entry_by_path('I/pixel.txt'), store)
        self.assertEqual(len(store.getvalue()), 13)  # offset table plus five bytes, no metadata

    def test_canonical_directory_string_bounds(self):
        entry = DirectoryEntry(0, 65, 0, 0, 0, 'é' * 1024, 't' * 2048)
        self.assertEqual(parse_entry(entry.to_bytes())[0], entry)
        entry.url += 'x'
        self.assert_code('AllocationLimit', entry.to_bytes)
        malformed = struct.pack('<HBBIII', 0, 0, 65, 0, 0, 0) + b'x' * 2049 + b'\0title\0'
        self.assert_code('AllocationLimit', lambda: parse_entry(malformed))

    def test_million_entry_open_does_not_read_pointer_table(self):
        header = ZIMHeader.from_bytes(BASELINE)
        header.entry_count = 1000000
        header.path_ptr_pos = 1 << 30
        source = Source(header.to_bytes() + BASELINE[80:], 1 << 40)
        reader = ZIMReader(source=source)
        reader.open()
        self.addCleanup(reader.close)
        self.assertEqual(len(reader.directory_entries), 1000000)
        self.assertLessEqual(sum(length for _, length in source.requests), 65536)
        self.assertFalse(any(offset >= header.path_ptr_pos for offset, _ in source.requests))
        self.assertEqual(reader._reader.cache_counts(), (1, 1))

    def test_large_cluster_uses_caller_store_and_respects_decode_limit(self):
        # A small first blob shares a compressed cluster with 33 MiB of data.
        first = b'tiny'
        total = 33 * 1024 * 1024
        encoder = zlib.compressobj()
        chunks = [encoder.compress(struct.pack('<III', 12, 16, 16 + total) + first)]
        for _ in range(total // 65536):
            chunks.append(encoder.compress(b'z' * 65536))
        chunks.append(encoder.flush())
        encoded = b'\x02' + b''.join(chunks)
        header = ZIMHeader.from_bytes(BASELINE)
        start = struct.unpack_from('<Q', BASELINE, header.cluster_ptr_pos)[0]
        data = bytearray(BASELINE[:start])
        header.cluster_count = 1
        header.checksum_pos = start + len(encoded)
        data.extend(encoded + b'\0' * 16)
        data[:80] = header.to_bytes()
        reader = self.reader(data)
        entry = reader.entry_at(0)
        self.assert_code('DecompressionLimit', lambda: reader.get_article_content(entry))
        staging = ROOT / 'test-output/language-parity'
        staging.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryFile(dir=staging) as store:
            self.assertEqual(reader.get_article_content_to(entry, store), first)
            self.assertEqual(store.seek(0, 2), total + 16)
            limited = self.reader(data, decode_limits=ClusterDecodeLimits(maximum_output_bytes=1024))
            self.assert_code('DecompressionLimit', lambda: limited.get_article_content_to(limited.entry_at(0), store))
            self.assertEqual(store.seek(0, 2), 0)

    def test_writer_remaps_sorted_indices_digest_unicode_empty_and_two_clusters(self):
        staging = ROOT / 'test-output/language-parity'; staging.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=staging) as directory:
            path = Path(directory) / 'python.zim'
            with ZIMWriter(str(path), uuid=bytes(range(16))) as writer:
                writer.add_article(65, 'é', 'Accent', b'first')
                writer.add_article(65, 'Z', 'Last ASCII', b'')
                writer.add_redirect(65, 'Home', 'Main', 0)
                writer.directory_entries[0].parameters = b'\x01\x00'
                writer.set_main_page(2)
            raw = path.read_bytes(); header = ZIMHeader.from_bytes(raw)
            self.assertEqual(raw[-16:], hashlib.md5(raw[:header.checksum_pos]).digest())
            self.assertEqual(header.title_ptr_pos, 0); self.assertEqual(header.uuid, bytes(range(16)))
            self.assertEqual(header.cluster_count, 2)
            self.assertEqual(struct.unpack_from('<H', raw, 4)[0], 6)
            pointers = struct.unpack_from('<QQQ', raw, header.path_ptr_pos)
            self.assertEqual(struct.unpack_from('<I', raw, pointers[0] + 8)[0], 2)
            self.assertEqual(header.main_page, 0)
            with ZIMReader(str(path)) as reader:
                self.assertEqual([e.url for e in reader.iter_entries()], ['Home', 'Z', 'é'])
                self.assertEqual(reader.get_article_content(reader.get_main_page()), b'first')
                self.assertEqual(reader.get_article_content(reader.entry_at(1)), b'')
                self.assertEqual(reader.entry_at(2).parameters, b'\x01\x00')
            writer = ZIMWriter(str(path), maximum_bytes=128)
            self.assert_code('AllocationLimit', lambda: writer.add_article(65, 'x', 'x', b'x' * 100))
            writer = ZIMWriter(str(path)); writer.create(); writer.add_article(65, 'x', 'x', b'x')
            self.assert_code('InvalidFormat', lambda: writer.add_article(65, 'x', 'duplicate', b'x'))
            writer.add_redirect(65, 'bad', 'bad', 999)
            self.assert_code('InvalidFormat', writer.finalize)
            self.assertEqual(path.read_bytes(), raw)  # rejected output never replaced valid archive


if __name__ == '__main__':
    unittest.main()
