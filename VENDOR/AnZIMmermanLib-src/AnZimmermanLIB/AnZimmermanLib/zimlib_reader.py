"""Lazy Python reader over local files or transport-neutral range sources."""
import io
import struct
from collections.abc import Sequence
from dataclasses import replace
from zimlib_format import ZIMHeader, DirectoryEntry, RedirectEntry, SpecialEntry, ZimError, NO_PAGE, MAX_MIME_TYPES, parse_entry
from zimlib_io import RandomAccessSource, FileRangeSource, PagedRangeReader, RangeReadLimits, ClusterDecodeLimits, checked_range
from zimlib_cluster import decode_cluster_to, read_cluster_blob


class _LazySequence(Sequence):
    def __init__(self, length, getter):
        self._length, self._getter = length, getter

    def __len__(self):
        return self._length()

    def __getitem__(self, index):
        if isinstance(index, slice):
            return [self._getter(i) for i in range(*index.indices(len(self))) ]
        if index < 0:
            index += len(self)
        if not 0 <= index < len(self):
            raise IndexError(index)
        return self._getter(index)


class _RangeStream:
    def __init__(self, reader, start, end):
        self.reader, self.position, self.end = reader, start, end

    def read(self, length):
        if not isinstance(length, int) or not 0 <= length <= 65536:
            raise ZimError('AllocationLimit', 'Cluster source reads must be bounded')
        length = min(length, self.end - self.position)
        data = self.reader.read_exact(self.position, length)
        self.position += length
        return data


class ZIMReader:
    def __init__(self, file_path: str = '', source: RandomAccessSource = None, limits: RangeReadLimits = None, decode_limits: ClusterDecodeLimits = None):
        self.file_path = file_path
        self.source = source
        self.limits = limits or RangeReadLimits()
        self.decode_limits = decode_limits or ClusterDecodeLimits()
        self.header = None
        self.file = None
        self.mime_types = []
        self._reader = None
        self._owned_source = None
        self.directory_entries = _LazySequence(lambda: self.header.entry_count if self.header else 0, self.entry_at)
        self.cluster_offsets = _LazySequence(lambda: self.header.cluster_count if self.header else 0, self._cluster_pointer)

    def open(self):
        self.close()
        self.limits.validate()
        self.decode_limits.validate()
        try:
            source = self.source
            if source is None:
                source = self._owned_source = FileRangeSource(self.file_path)
                self.file = source._file
            self._reader = PagedRangeReader(source, self.limits)
            self.header = ZIMHeader.from_bytes(self._reader.read_exact(0, 80))
            self._validate_header()
            self._read_mime_types()
        except Exception:
            self.close()
            raise

    def _require_open(self):
        if self.header is None or self._reader is None:
            raise ZimError('Io', 'Reader is not open')

    def _validate_header(self):
        h, length = self.header, self._reader.length
        if not 80 <= h.mime_list_pos < length:
            raise ZimError('InvalidFormat', 'MIME list lies outside source')
        for position, count, width in [(h.path_ptr_pos, h.entry_count, 8), (h.cluster_ptr_pos, h.cluster_count, 8), (h.title_ptr_pos, h.entry_count if h.title_ptr_pos else 0, 4)]:
            checked_range(position, count * width, length)
            if position >= length:
                raise ZimError('InvalidFormat', 'Pointer table lies outside source')
        if h.checksum_pos:
            checked_range(h.checksum_pos, 16, length)
        for page in (h.main_page, h.layout_page):
            if page != NO_PAGE and page >= h.entry_count:
                raise ZimError('InvalidFormat', 'Page index outside directory')

    def _boundaries(self):
        h = self.header
        return (h.path_ptr_pos, h.title_ptr_pos, h.cluster_ptr_pos, h.mime_list_pos, h.checksum_pos, self._reader.length)

    def _read_mime_types(self):
        position = self.header.mime_list_pos
        end = min(boundary for boundary in self._boundaries() if boundary > position)
        consumed = 0
        value = bytearray()
        self.mime_types = []
        while position < end:
            size = min(256, end - position, self.limits.max_mime_bytes - consumed)
            if size == 0:
                raise ZimError('AllocationLimit', 'MIME list exceeds configured limit')
            for byte in self._reader.read_exact(position, size):
                position += 1
                consumed += 1
                if byte:
                    value.append(byte)
                elif not value:
                    return
                else:
                    try:
                        self.mime_types.append(value.decode('utf-8'))
                    except UnicodeError as error:
                        raise ZimError('InvalidFormat', 'Invalid MIME UTF-8') from error
                    value.clear()
                    if len(self.mime_types) > MAX_MIME_TYPES:
                        raise ZimError('AllocationLimit', 'Too many MIME types')
        raise ZimError('InvalidFormat', 'Unterminated MIME list')

    def _pointer(self, base, index):
        return struct.unpack('<Q', self._reader.read_exact(base + index * 8, 8))[0]

    def entry_at(self, index):
        self._require_open()
        if not isinstance(index, int) or not 0 <= index < self.header.entry_count:
            raise ZimError('EntryNotFound', 'Entry index outside directory')
        pointer = self._pointer(self.header.path_ptr_pos, index)
        if not 80 <= pointer < self._reader.length:
            raise ZimError('InvalidFormat', 'Directory pointer outside source')
        entry, _ = parse_entry(self._reader.read_exact(pointer, min(self.limits.max_entry_bytes, self._reader.length - pointer)))
        if isinstance(entry, DirectoryEntry) and (entry.mimetype_index >= len(self.mime_types) or entry.cluster_number >= self.header.cluster_count):
            raise ZimError('InvalidFormat', 'Article MIME or cluster index outside table')
        entry._index = index
        return entry

    def iter_entries(self):
        self._require_open()
        for index in range(self.header.entry_count):
            yield self.entry_at(index)

    def get_entry_by_path(self, path):
        self._require_open()
        if len(path) < 2 or path[1] != '/' or ord(path[0]) > 255:
            return None
        try:
            target = ord(path[0]), path[2:].encode('utf-8')
        except UnicodeError as error:
            raise ZimError('InvalidFormat', 'Invalid lookup UTF-8') from error
        low, high = 0, self.header.entry_count
        while low < high:
            middle = (low + high) // 2
            entry = self.entry_at(middle)
            key = entry.namespace, entry.url.encode('utf-8')
            if key < target:
                low = middle + 1
            elif key > target:
                high = middle
            else:
                if isinstance(entry, SpecialEntry):
                    raise ZimError('UnsupportedFeature', 'Unsupported directory variant')
                return entry
        return None

    def resolve_entry(self, entry):
        self._require_open()
        visited = set()
        hops = 0
        while isinstance(entry, RedirectEntry):
            identity = getattr(entry, '_index', (entry.namespace, entry.url))
            if identity in visited:
                raise ZimError('RedirectCycle', 'Redirect cycle')
            if hops >= self.limits.max_redirect_hops:
                raise ZimError('RedirectLimit', 'Redirect hop limit exceeded')
            visited.add(identity)
            entry = self.entry_at(entry.redirect_index)
            hops += 1
        if not isinstance(entry, DirectoryEntry):
            raise ZimError('UnsupportedFeature', 'Entry does not contain article content')
        return entry

    def _cluster_pointer(self, index):
        self._require_open()
        if not isinstance(index, int) or not 0 <= index < self.header.cluster_count:
            raise ZimError('EntryNotFound', 'Cluster index outside table')
        return self._pointer(self.header.cluster_ptr_pos, index)

    def _content_to(self, entry, store, limits):
        entry = self.resolve_entry(entry)
        start = self._cluster_pointer(entry.cluster_number)
        end = self._reader.length
        if not 80 <= start < end:
            raise ZimError('InvalidFormat', 'Cluster pointer outside source')
        if entry.cluster_number + 1 < self.header.cluster_count:
            end = self._cluster_pointer(entry.cluster_number + 1)
            if not start < end <= self._reader.length:
                raise ZimError('InvalidFormat', 'Invalid next cluster pointer')
        for boundary in self._boundaries():
            if boundary > self._reader.length:
                raise ZimError('InvalidFormat', 'Metadata boundary outside source')
            if boundary > start:
                end = min(end, boundary)
        if end <= start:
            raise ZimError('InvalidFormat', 'Empty cluster range')
        decoded = decode_cluster_to(_RangeStream(self._reader, start, end), store, limits)
        return read_cluster_blob(store, decoded, entry.blob_number, min(limits.maximum_blob_bytes, self.limits.max_blob_bytes))

    def get_article_content_to(self, entry, store):
        return self._content_to(entry, store, self.decode_limits)

    def get_article_content(self, entry):
        with io.BytesIO() as store:
            limits = replace(self.decode_limits, maximum_output_bytes=min(self.decode_limits.maximum_output_bytes, 33554432))
            return self._content_to(entry, store, limits)

    def get_main_page(self):
        if self.header is None or self.header.main_page == NO_PAGE:
            return None
        return self.entry_at(self.header.main_page)

    def list_articles(self):
        return [entry for entry in self.iter_entries() if isinstance(entry, DirectoryEntry)]

    def close(self):
        if self._owned_source:
            self._owned_source.close()
        self._owned_source = None
        self.file = self.header = self._reader = None
        self.mime_types = []

    def __enter__(self):
        self.open()
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        self.close()
