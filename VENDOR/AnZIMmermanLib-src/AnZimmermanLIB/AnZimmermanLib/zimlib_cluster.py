"""Single-frame bounded decoding into caller-owned seekable storage."""
import lzma
import zlib
from typing import BinaryIO
from zimlib_format import ZimError, cluster_encoding
from zimlib_io import ClusterDecodeLimits

CHUNK = 65536


def _read(input, length):
    value = input.read(length)
    if not isinstance(value, bytes) or len(value) > length:
        raise ZimError('Io', 'Invalid stream read response')
    return value


def _exact(input, length):
    result = bytearray()
    while len(result) < length:
        part = _read(input, length - len(result))
        if not part:
            raise ZimError('InvalidFormat', 'Truncated stream')
        result.extend(part)
    return bytes(result)


def _write(output, data):
    position = 0
    while position < len(data):
        written = output.write(memoryview(data)[position:])
        if not isinstance(written, int) or not 0 < written <= len(data) - position:
            raise ZimError('Io', 'Short stream write')
        position += written


def decode_cluster_to(input: BinaryIO, output: BinaryIO, limits: ClusterDecodeLimits) -> tuple[int, int]:
    """Truncate before/after failure, retain ownership, stop after first frame."""
    try:
        output.seek(0)
        output.truncate(0)
        limits.validate()
        codec, width = cluster_encoding(_exact(input, 1)[0])
        count = 0

        def emit(data):
            nonlocal count
            if len(data) > CHUNK or count + len(data) > limits.maximum_output_bytes:
                raise ZimError('DecompressionLimit', 'Decoded cluster exceeds output limit')
            _write(output, data)
            count += len(data)

        if codec in (0, 1):
            while data := _read(input, CHUNK):
                emit(data)
        elif codec == 2:
            prefix = _exact(input, 2)
            wrapped = prefix[0] & 15 == 8 and prefix[0] >> 4 <= 7 and int.from_bytes(prefix, 'big') % 31 == 0
            if wrapped and prefix[1] & 32:
                raise ZimError('UnsupportedFeature', 'Preset DEFLATE dictionary is unsupported')
            if limits.maximum_decoder_window_bytes < 32768:
                raise ZimError('DecompressionLimit', 'DEFLATE window exceeds decoder limit')
            decoder = zlib.decompressobj(15 if wrapped else -15)
            pending = prefix
            while not decoder.eof:
                if not pending:
                    pending = _read(input, CHUNK)
                    if not pending:
                        raise ZimError('InvalidFormat', 'Truncated DEFLATE frame')
                data = decoder.decompress(pending, min(CHUNK, limits.maximum_output_bytes - count + 1))
                tail = decoder.unconsumed_tail
                if not data and tail == pending and not decoder.eof:
                    raise ZimError('InvalidFormat', 'Invalid DEFLATE frame')
                emit(data)
                pending = tail
        elif codec == 4:
            decoder = lzma.LZMADecompressor(format=lzma.FORMAT_XZ, memlimit=limits.maximum_decoder_window_bytes)
            while not decoder.eof:
                data = _read(input, CHUNK) if decoder.needs_input else b''
                if decoder.needs_input and not data:
                    raise ZimError('InvalidFormat', 'Truncated XZ frame')
                emit(decoder.decompress(data, max_length=min(CHUNK, limits.maximum_output_bytes - count + 1)))
        else:
            raise ZimError('UnsupportedCompression', f'Unsupported compression codec {codec}')
        return count, width
    except Exception as error:
        try:
            output.seek(0)
            output.truncate(0)
        except Exception as truncate_error:
            raise ZimError('Io', 'Could not truncate failed decode output') from truncate_error
        if isinstance(error, ZimError):
            raise
        if isinstance(error, (zlib.error, lzma.LZMAError)):
            code = 'DecompressionLimit' if 'Memory usage limit' in str(error) else 'InvalidFormat'
            raise ZimError(code, str(error)) from error
        raise ZimError('Io', str(error)) from error


def read_cluster_blob(store: BinaryIO, decoded: tuple[int, int], blob_index: int, maximum_bytes: int) -> bytes:
    length, width = decoded
    if width not in (4, 8) or not isinstance(length, int) or length < width or not isinstance(blob_index, int) or blob_index < 0:
        raise ZimError('InvalidFormat', 'Invalid decoded cluster or blob index')
    if not isinstance(maximum_bytes, int) or not 0 < maximum_bytes <= 33554432:
        raise ZimError('AllocationLimit', 'Invalid blob allocation limit')
    try:
        def offset(position):
            store.seek(position)
            return int.from_bytes(_exact(store, width), 'little')
        table = offset(0)
        if table < 2 * width or table % width or table > length:
            raise ZimError('InvalidFormat', 'Invalid blob offset table')
        if blob_index >= table // width - 1:
            raise ZimError('EntryNotFound', 'Blob index outside offset table')
        start, end = offset(blob_index * width), offset((blob_index + 1) * width)
        if start < table or end < start or end > length:
            raise ZimError('InvalidFormat', 'Invalid blob endpoints')
        if end - start > maximum_bytes:
            raise ZimError('AllocationLimit', 'Requested blob exceeds allocation limit')
        store.seek(start)
        # _exact may read at most a returned-blob ceiling, not a whole cluster.
        result = bytearray()
        while len(result) < end - start:
            result.extend(_exact(store, min(CHUNK, end - start - len(result))))
        return bytes(result)
    except ZimError:
        raise
    except Exception as error:
        raise ZimError('Io', str(error)) from error
