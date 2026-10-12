"""Bounded buffered writer for the existing Python edition."""
import hashlib
import lzma
import struct
import uuid as uuid_module
import zlib
from dataclasses import replace
from zimlib_format import CompressionType, DirectoryEntry, RedirectEntry, ZIMHeader, ZIM_MAGIC, NO_PAGE, MAX_MIME_TYPES, ZimError


class ZIMWriter:
    def __init__(self, file_path: str, maximum_bytes: int = 134217728, uuid: bytes = None):
        if not isinstance(maximum_bytes, int) or not 96 <= maximum_bytes <= 134217728:
            raise ZimError('AllocationLimit', 'Invalid writer output limit')
        self.file_path, self.maximum_bytes = file_path, maximum_bytes
        self.uuid = uuid_module.uuid4().bytes if uuid is None else uuid
        if not isinstance(self.uuid, bytes) or len(self.uuid) != 16:
            raise ZimError('InvalidFormat', 'UUID must contain 16 bytes')
        self.file = None
        self.mime_types, self.directory_entries, self.clusters = [], [], []
        self.main_page_index = NO_PAGE
        self._created = False
        self._retained = 98  # header, empty MIME terminator and checksum
        self._keys = set()

    def _reserve(self, count):
        if count < 0 or self._retained + count > self.maximum_bytes:
            raise ZimError('AllocationLimit', 'Buffered writer exceeds output limit')

    def create(self):
        self._created = True

    def add_mime_type(self, mime_type):
        if '\0' in mime_type or not mime_type:
            raise ZimError('InvalidFormat', 'Invalid MIME string')
        if mime_type not in self.mime_types:
            raw = mime_type.encode('utf-8')
            self._reserve(len(raw) + 1)
            if len(self.mime_types) >= MAX_MIME_TYPES:
                raise ZimError('AllocationLimit', 'Too many MIME types')
            self.mime_types.append(mime_type)
            self._retained += len(raw) + 1
        return self.mime_types.index(mime_type)

    def _entry(self, entry, cluster=None):
        key = entry.namespace, entry.url.encode('utf-8')
        if key in self._keys:
            raise ZimError('InvalidFormat', 'Duplicate namespace/path')
        serialized = entry.to_bytes()
        growth = len(serialized) + 8 + (len(cluster) + 8 if cluster is not None else 0)
        self._reserve(growth)
        self._keys.add(key)
        self.directory_entries.append(entry)
        if cluster is not None:
            self.clusters.append(cluster)
        self._retained += growth

    def add_article(self, namespace, url, title, content, mime_type='text/html'):
        if not isinstance(content, bytes):
            raise ZimError('InvalidFormat', 'Article content must be bytes')
        # Check payload growth before building/copying an encoded cluster.
        self._reserve(len(content) + 9 + 24)
        mime = self.add_mime_type(mime_type)
        entry = DirectoryEntry(mime, namespace, 0, len(self.clusters), 0, url, title)
        self._entry(entry, self._create_cluster([content], CompressionType.NONE))

    def add_redirect(self, namespace, url, title, redirect_index):
        self._entry(RedirectEntry(0xFFFF, namespace, 0, redirect_index, url, title))

    def set_main_page(self, index):
        if not isinstance(index, int) or not 0 <= index < len(self.directory_entries):
            raise ZimError('EntryNotFound', 'Main page insertion index does not exist')
        self.main_page_index = index

    def _create_cluster(self, blobs, compression):
        size = (len(blobs) + 1) * 4 + sum(len(blob) for blob in blobs)
        self._reserve(size + 1)
        if size > 0xFFFFFFFF:
            raise ZimError('AllocationLimit', '32-bit cluster overflow')
        offsets = [(len(blobs) + 1) * 4]
        for blob in blobs:
            offsets.append(offsets[-1] + len(blob))
        body = b''.join(struct.pack('<I', value) for value in offsets) + b''.join(blobs)
        if compression in (0, 1):
            encoded = body
        elif compression in (2, 4):
            compressor = zlib.compressobj() if compression == 2 else lzma.LZMACompressor()
            chunks, length = [], 0
            for position in range(0, len(body), 65536):
                chunk = compressor.compress(body[position:position + 65536])
                length += len(chunk)
                self._reserve(length + 1)
                chunks.append(chunk)
            tail = compressor.flush()
            self._reserve(length + len(tail) + 1)
            encoded = b''.join(chunks) + tail
        else:
            raise ZimError('UnsupportedCompression', 'Unsupported writer compression')
        return bytes([compression]) + encoded

    def finalize(self):
        if not self._created:
            raise ZimError('Io', 'Writer has not been created')
        indexed = sorted(enumerate(self.directory_entries), key=lambda pair: (pair[1].namespace, pair[1].url.encode('utf-8')))
        mapping = {original: sorted_index for sorted_index, (original, _) in enumerate(indexed)}
        keys = [(entry.namespace, entry.url.encode('utf-8')) for _, entry in indexed]
        if len(set(keys)) != len(keys):
            raise ZimError('InvalidFormat', 'Duplicate namespace/path')
        entries = []
        for _, entry in indexed:
            if isinstance(entry, RedirectEntry):
                if entry.redirect_index not in mapping:
                    raise ZimError('InvalidFormat', 'Dangling redirect insertion index')
                entry = replace(entry, redirect_index=mapping[entry.redirect_index])
            entries.append(entry.to_bytes())
        if self.main_page_index != NO_PAGE and self.main_page_index not in mapping:
            raise ZimError('InvalidFormat', 'Dangling main page insertion index')
        mime = b'\0'.join(value.encode('utf-8') for value in self.mime_types) + b'\0\0'
        position = 80 + len(mime)
        pointers = []
        for data in entries:
            pointers.append(position)
            position += len(data)
        path_pos = position
        cluster_pos = path_pos + len(entries) * 8
        position = cluster_pos + len(self.clusters) * 8
        cluster_pointers = []
        for cluster in self.clusters:
            cluster_pointers.append(position)
            position += len(cluster)
        if position + 16 > self.maximum_bytes:
            raise ZimError('AllocationLimit', 'Buffered writer exceeds output limit')
        header = ZIMHeader(ZIM_MAGIC, 6, 0, self.uuid, len(entries), len(self.clusters), path_pos, 0, cluster_pos, 80, mapping.get(self.main_page_index, NO_PAGE), NO_PAGE, position)
        output = bytearray(header.to_bytes())
        output.extend(mime)
        for part in entries:
            output.extend(part)
        for pointer in pointers + cluster_pointers:
            output.extend(struct.pack('<Q', pointer))
        for cluster in self.clusters:
            output.extend(cluster)
        output.extend(hashlib.md5(output).digest())
        try:
            # Publish only after validating all remapped indices and output size.
            with open(self.file_path, 'wb') as file:
                file.write(output)
        except OSError as error:
            raise ZimError('Io', str(error)) from error

    def close(self):
        if self.file:
            self.file.close()
            self.file = None
        self._created = False

    def __enter__(self):
        self.create()
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        try:
            if exc_type is None:
                self.finalize()
        finally:
            self.close()
