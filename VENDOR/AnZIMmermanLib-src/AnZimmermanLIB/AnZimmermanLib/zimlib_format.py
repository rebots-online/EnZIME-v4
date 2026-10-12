# Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
"""Canonical Python ZIM format primitives; shared edition, no transport policy."""
import struct
from dataclasses import dataclass
from enum import IntEnum

ZIM_MAGIC = 0x044D495A
NO_PAGE = 0xFFFFFFFF
MAX_URL_BYTES = 2048
MAX_TITLE_BYTES = 2048
MAX_MIME_TYPES = 65530
HEADER = struct.Struct('<IHH16sIIQQQQIIQ')


class ZimError(ValueError):
    def __init__(self, code: str, message: str):
        self.code = code
        super().__init__(message)


class CompressionType(IntEnum):
    DEFAULT = 0
    NONE = 1
    ZLIB = 2
    BZIP2 = 3
    LZMA = 4
    ZSTD = 5


class Namespace(IntEnum):
    MAIN_ARTICLE = ord('A')
    IMAGE = ord('I')
    METADATA = ord('M')
    RAW_DATA = ord('-')
    STYLE = ord('S')
    SCRIPT = ord('J')
    FONT = ord('T')
    TRANSLATION = ord('U')
    VIDEO = ord('V')
    AUDIO = ord('W')


@dataclass
class ZIMHeader:
    magic_number: int
    major_version: int
    minor_version: int
    uuid: bytes
    entry_count: int
    cluster_count: int
    path_ptr_pos: int
    title_ptr_pos: int
    cluster_ptr_pos: int
    mime_list_pos: int
    main_page: int
    layout_page: int
    checksum_pos: int

    @classmethod
    def from_bytes(cls, data: bytes):
        if len(data) < HEADER.size:
            raise ZimError('InvalidFormat', 'Truncated 80-byte header')
        header = cls(*HEADER.unpack_from(data))
        if header.magic_number != ZIM_MAGIC:
            raise ZimError('InvalidFormat', 'Invalid ZIM magic')
        if header.major_version not in (5, 6):
            raise ZimError('UnsupportedFeature', 'Unsupported ZIM major version')
        return header

    def to_bytes(self) -> bytes:
        if len(self.uuid) != 16:
            raise ZimError('InvalidFormat', 'UUID must contain 16 bytes')
        try:
            data = HEADER.pack(*(getattr(self, name) for name in self.__dataclass_fields__))
        except (struct.error, TypeError) as error:
            raise ZimError('InvalidFormat', 'Invalid header field') from error
        self.from_bytes(data)
        return data

    mime_type_list_pos = property(lambda self: self.mime_list_pos)
    @property
    def title_index_pos(self):
        """Optional physical title-index address; raw bytes remain unchanged."""
        if self.title_ptr_pos == 0 or (self.major_version == 6 and self.title_ptr_pos == (1 << 64) - 1):
            return None
        return self.title_ptr_pos
    main_page_index = property(lambda self: self.main_page)
    layout_page_index = property(lambda self: self.layout_page)


class EntrySerialization:
    @classmethod
    def from_bytes(cls, data: bytes):
        entry, _ = parse_entry(data)
        if not isinstance(entry, cls):
            raise ZimError('InvalidFormat', 'Wrong directory variant')
        return entry

    def to_bytes(self) -> bytes:
        if not isinstance(self.parameters, bytes) or len(self.parameters) > 255:
            raise ZimError('AllocationLimit', 'Directory parameters exceed 255 bytes')
        try:
            prefix = struct.pack('<HBBI', self.mimetype_index, len(self.parameters), self.namespace, self.revision)
            if isinstance(self, DirectoryEntry):
                if self.mimetype_index >= 0xFFFD:
                    raise ZimError('InvalidFormat', 'Article uses reserved MIME discriminator')
                prefix += struct.pack('<II', self.cluster_number, self.blob_number)
            elif isinstance(self, RedirectEntry):
                if self.mimetype_index != 0xFFFF:
                    raise ZimError('InvalidFormat', 'Invalid redirect discriminator')
                prefix += struct.pack('<I', self.redirect_index)
            elif self.mimetype_index not in (0xFFFD, 0xFFFE):
                raise ZimError('InvalidFormat', 'Invalid special discriminator')
            strings = []
            for value, maximum in ((self.url, MAX_URL_BYTES), (self.title, MAX_TITLE_BYTES)):
                if '\0' in value:
                    raise ZimError('InvalidFormat', 'Embedded directory terminator')
                encoded = value.encode('utf-8')
                if len(encoded) > maximum:
                    raise ZimError('AllocationLimit', 'Directory string exceeds limit')
                strings.append(encoded + b'\0')
        except (struct.error, UnicodeError, TypeError) as error:
            raise ZimError('InvalidFormat', 'Invalid directory field') from error
        result = prefix + b''.join(strings) + self.parameters
        if len(result) > 65536:
            raise ZimError('AllocationLimit', 'Directory entry exceeds 64 KiB')
        return result


@dataclass
class DirectoryEntry(EntrySerialization):
    mimetype_index: int
    namespace: int
    revision: int
    cluster_number: int
    blob_number: int
    url: str
    title: str
    parameters: bytes = b''


@dataclass
class RedirectEntry(EntrySerialization):
    mimetype_index: int
    namespace: int
    revision: int
    redirect_index: int
    url: str
    title: str
    parameters: bytes = b''


@dataclass
class SpecialEntry(EntrySerialization):
    mimetype_index: int
    namespace: int
    revision: int
    url: str
    title: str
    parameters: bytes = b''

    @property
    def kind(self):
        return 'linktarget' if self.mimetype_index == 0xFFFE else 'deleted'


Entry = DirectoryEntry | RedirectEntry | SpecialEntry


def parse_entry(data: bytes) -> tuple[Entry, int]:
    if len(data) < 8:
        raise ZimError('InvalidFormat', 'Truncated directory prefix')
    mime, parameter_length, namespace, revision = struct.unpack_from('<HBBI', data)
    position = 12 if mime == 0xFFFF else 8 if mime in (0xFFFE, 0xFFFD) else 16
    if len(data) < position:
        raise ZimError('InvalidFormat', 'Truncated directory variant')
    extra = struct.unpack_from('<I', data, 8) if mime == 0xFFFF else () if mime in (0xFFFE, 0xFFFD) else struct.unpack_from('<II', data, 8)
    strings = []
    for maximum in (MAX_URL_BYTES, MAX_TITLE_BYTES):
        end = data.find(b'\0', position)
        if end < 0:
            raise ZimError('InvalidFormat', 'Unterminated directory string')
        if end - position > maximum:
            raise ZimError('AllocationLimit', 'Directory string exceeds limit')
        try:
            strings.append(data[position:end].decode('utf-8'))
        except UnicodeError as error:
            raise ZimError('InvalidFormat', 'Invalid directory UTF-8') from error
        position = end + 1
    end = position + parameter_length
    if end > len(data):
        raise ZimError('InvalidFormat', 'Truncated directory parameters')
    constructor = RedirectEntry if mime == 0xFFFF else SpecialEntry if mime in (0xFFFE, 0xFFFD) else DirectoryEntry
    return constructor(mime, namespace, revision, *extra, *strings, data[position:end]), end


def cluster_encoding(tag: int) -> tuple[int, int]:
    if not isinstance(tag, int) or tag < 0 or tag > 255 or tag & 0xE0:
        raise ZimError('InvalidFormat', 'Invalid cluster flags')
    return tag & 15, 8 if tag & 0x10 else 4
