"""Bounded random access and caller-owned source adapters."""
import os
import threading
from collections import OrderedDict
from dataclasses import dataclass
from typing import Protocol
from zimlib_format import ZimError

U64_MAX = (1 << 64) - 1


def checked_range(offset, length, source_length):
    if any(not isinstance(value, int) or isinstance(value, bool) or value < 0 or value > U64_MAX for value in (offset, length, source_length)) or offset + length > min(source_length, U64_MAX):
        raise ZimError('InvalidFormat', 'Range outside source')


@dataclass(frozen=True)
class RangeReadLimits:
    page_bytes: int = 65536
    cached_pages: int = 8
    max_entry_bytes: int = 65536
    max_mime_bytes: int = 1048576
    max_blob_bytes: int = 33554432
    max_redirect_hops: int = 16

    def validate(self):
        for field, maximum in [('page_bytes', 65536), ('cached_pages', 16), ('max_entry_bytes', 65536), ('max_mime_bytes', 1048576), ('max_blob_bytes', 33554432), ('max_redirect_hops', 32)]:
            value = getattr(self, field)
            if not isinstance(value, int) or isinstance(value, bool) or not 0 < value <= maximum:
                raise ZimError('AllocationLimit', f'Invalid {field}')


@dataclass(frozen=True)
class ClusterDecodeLimits:
    maximum_output_bytes: int = 2147483648
    maximum_blob_bytes: int = 33554432
    maximum_decoder_window_bytes: int = 67108864

    def validate(self):
        for field, maximum in [('maximum_output_bytes', 2147483648), ('maximum_blob_bytes', 33554432), ('maximum_decoder_window_bytes', 67108864)]:
            value = getattr(self, field)
            if not isinstance(value, int) or isinstance(value, bool) or not 0 < value <= maximum:
                raise ZimError('AllocationLimit', f'Invalid {field}')


class RandomAccessSource(Protocol):
    @property
    def length(self) -> int: ...
    def read_range(self, offset: int, length: int) -> bytes: ...


class FileRangeSource:
    def __init__(self, path: str):
        self._lock = threading.RLock()
        try:
            self._file = open(path, 'rb')
            self._length = os.fstat(self._file.fileno()).st_size
        except OSError as error:
            raise ZimError('Io', str(error)) from error

    @property
    def length(self):
        return self._length

    def read_range(self, offset: int, length: int) -> bytes:
        checked_range(offset, length, self.length)
        if length > 1048576:
            raise ZimError('AllocationLimit', 'Range exceeds 1 MiB')
        with self._lock:
            try:
                self._file.seek(offset)
                result = bytearray()
                while len(result) < length:
                    part = self._file.read(length - len(result))
                    if not part:
                        raise ZimError('Io', 'Short file read')
                    result.extend(part)
                return bytes(result)
            except (OSError, ValueError) as error:
                if isinstance(error, ZimError):
                    raise
                raise ZimError('Io', str(error)) from error

    def close(self):
        with self._lock:
            self._file.close()


class PagedRangeReader:
    def __init__(self, source: RandomAccessSource, limits: RangeReadLimits = None):
        self.source = source
        self.limits = limits or RangeReadLimits()
        self.limits.validate()
        checked_range(0, 0, source.length)
        self.length = source.length
        self._pages = OrderedDict()
        self._peak = 0
        self._lock = threading.RLock()

    def cache_counts(self) -> tuple[int, int]:
        with self._lock:
            return len(self._pages), self._peak

    def read_exact(self, offset: int, length: int) -> bytes:
        checked_range(offset, length, self.length)
        if length > 1048576:
            raise ZimError('AllocationLimit', 'Read exceeds 1 MiB')
        result = bytearray()
        with self._lock:
            while len(result) < length:
                cursor = offset + len(result)
                page = cursor // self.limits.page_bytes
                start = page * self.limits.page_bytes
                data = self._pages.pop(page, None)
                if data is None:
                    if len(self._pages) == self.limits.cached_pages:
                        self._pages.popitem(last=False)
                    size = min(self.limits.page_bytes, self.length - start)
                    try:
                        data = self.source.read_range(start, size)
                    except ZimError:
                        raise
                    except Exception as error:
                        raise ZimError('Io', str(error)) from error
                    if not isinstance(data, bytes) or len(data) != size:
                        raise ZimError('Io', 'Short or excess source response')
                self._pages[page] = data
                self._peak = max(self._peak, len(self._pages))
                within = cursor - start
                result.extend(data[within:within + min(length - len(result), len(data) - within)])
        return bytes(result)
