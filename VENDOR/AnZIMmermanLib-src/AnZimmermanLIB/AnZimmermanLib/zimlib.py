# Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
# All rights reserved.
# Unauthorized use without prior written consent is strictly prohibited.
"""Public Python facade for the canonical AnZimmerman library edition."""
from zimlib_format import (
    CompressionType, Namespace, ZimError, ZIMHeader, DirectoryEntry,
    RedirectEntry, SpecialEntry, Entry, ZIM_MAGIC, NO_PAGE, parse_entry,
    cluster_encoding,
)
from zimlib_io import RandomAccessSource, FileRangeSource, PagedRangeReader, RangeReadLimits, ClusterDecodeLimits
from zimlib_cluster import decode_cluster_to, read_cluster_blob
from zimlib_reader import ZIMReader
from zimlib_writer import ZIMWriter


def read_zim_file(file_path: str) -> ZIMReader:
    return ZIMReader(file_path)


def create_zim_file(file_path: str) -> ZIMWriter:
    return ZIMWriter(file_path)


__all__ = [
    'CompressionType', 'Namespace', 'ZimError', 'ZIMHeader', 'DirectoryEntry',
    'RedirectEntry', 'SpecialEntry', 'Entry', 'ZIM_MAGIC', 'NO_PAGE',
    'parse_entry', 'cluster_encoding', 'RandomAccessSource', 'FileRangeSource',
    'PagedRangeReader', 'RangeReadLimits', 'ClusterDecodeLimits',
    'decode_cluster_to', 'read_cluster_blob', 'ZIMReader', 'ZIMWriter',
    'read_zim_file', 'create_zim_file',
]
