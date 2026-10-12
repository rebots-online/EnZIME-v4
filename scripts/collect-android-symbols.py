#!/usr/bin/env python3
"""Collect verified Android DWARF from the current unified build, without rebuilding."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import struct
import subprocess
import sys
import uuid
import zipfile

BLOCK = 65536
ABIS = {
    "arm64-v8a": "aarch64-linux-android",
    "armeabi-v7a": "armv7-linux-androideabi",
    "x86": "i686-linux-android",
    "x86_64": "x86_64-linux-android",
}
LIBRARY = "libenzime_reader_lib.so"


def digest(path):
    result = hashlib.sha256()
    with path.open("rb") as source:
        while block := source.read(BLOCK):
            result.update(block)
    return result.hexdigest()


class Elf:
    """Read only section metadata; hash section contents in bounded chunks."""

    def __init__(self, path):
        self.path = path
        self.length = path.stat().st_size
        with path.open("rb") as source:
            ident = self.read(source, 0, 16)
            if ident[:4] != b"\x7fELF" or ident[4] not in (1, 2) or ident[5] not in (1, 2):
                raise ValueError(f"Invalid ELF: {path}")
            self.bits = ident[4]
            endian = "<" if ident[5] == 1 else ">"
            header_format = endian + ("16sHHIIIIIHHHHHH" if self.bits == 1 else "16sHHIQQQIHHHHHH")
            header = struct.unpack(header_format, self.read(source, 0, struct.calcsize(header_format)))
            self.machine = header[2]
            self.endian = ident[5]
            offset, width, count, names_index = header[6], header[11], header[12], header[13]
            section_format = endian + ("IIIIIIIIII" if self.bits == 1 else "IIQQQQIIQQ")
            expected = struct.calcsize(section_format)
            if width != expected or not offset:
                raise ValueError(f"Invalid ELF section table: {path}")
            first = struct.unpack(section_format, self.read(source, offset, width))
            if count == 0:
                count = first[5]
            if names_index == 0xFFFF:
                names_index = first[6]
            if not 0 < count <= 65536 or not 0 <= names_index < count:
                raise ValueError(f"Unbounded/invalid ELF section count: {path}")
            if offset + width * count > self.length:
                raise ValueError(f"Truncated ELF section table: {path}")
            rows = [struct.unpack(section_format, self.read(source, offset + i * width, width))
                    for i in range(count)]
            name_row = rows[names_index]
            if name_row[5] > 16 * 1024 * 1024:
                raise ValueError(f"ELF section names exceed 16 MiB: {path}")
            names = self.read(source, name_row[4], name_row[5])
            self.sections = []
            for row in rows:
                end = names.find(b"\0", row[0])
                if row[0] >= len(names) or end < 0:
                    raise ValueError(f"Invalid ELF section name: {path}")
                name = names[row[0]:end]
                section = (name, row[1], row[2], row[3], row[5], row[4])
                if row[1] != 8 and row[4] + row[5] > self.length:
                    raise ValueError(f"ELF section extends outside file: {path}")
                self.sections.append(section)

    def read(self, source, offset, length):
        if offset < 0 or length < 0 or offset + length > self.length:
            raise ValueError(f"ELF read outside file: {self.path}")
        source.seek(offset)
        data = source.read(length)
        if len(data) != length:
            raise ValueError(f"Short ELF read: {self.path}")
        return data

    def require_dwarf(self):
        if not any(s[0] == b".debug_info" and s[1] == 1 and s[4] > 0 for s in self.sections):
            raise ValueError(f"Full DWARF .debug_info missing: {self.path}")

    def allocated(self):
        result = {}
        with self.path.open("rb") as source:
            for name, kind, flags, address, size, offset in self.sections:
                if not flags & 2:
                    continue
                metadata = (name, kind, flags, address, size)
                if metadata in result:
                    raise ValueError(f"Ambiguous duplicate allocated ELF section: {self.path}")
                value = None
                if kind != 8:  # SHT_NOBITS has no bytes in the file.
                    source.seek(offset)
                    remaining = size
                    hasher = hashlib.sha256()
                    while remaining:
                        data = source.read(min(BLOCK, remaining))
                        if not data:
                            raise ValueError(f"Truncated allocated ELF section: {self.path}")
                        hasher.update(data)
                        remaining -= len(data)
                    value = hasher.hexdigest()
                result[metadata] = value
        if not result:
            raise ValueError(f"ELF has no allocated sections: {self.path}")
        return result


def corresponding(compiled, packaged, compiled_hash, packaged_hash):
    if compiled_hash == packaged_hash:
        return "exact-sha256"
    if (compiled.bits, compiled.endian, compiled.machine) != (packaged.bits, packaged.endian, packaged.machine):
        raise ValueError(f"Packaged ELF architecture mismatch: {packaged.path}")
    if compiled.allocated() != packaged.allocated():
        raise ValueError(f"Packaged ELF allocated sections differ: {packaged.path}")
    return "all-shf-alloc-sections-sha256"


def member(archive, name):
    matches = [entry for entry in archive.infolist() if entry.filename == name]
    if len(matches) != 1 or matches[0].file_size <= 0:
        raise ValueError(f"Expected one nonempty package library: {name}")
    return matches[0]


def extract(archive, info, destination):
    total = 0
    with archive.open(info) as source, destination.open("xb") as output:
        while block := source.read(BLOCK):
            total += len(block)
            if total > info.file_size:
                raise ValueError(f"Package member exceeds declared size: {info.filename}")
            output.write(block)
    if total != info.file_size:
        raise ValueError(f"Truncated package member: {info.filename}")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("target-dir", "android-output-dir", "apk", "aab", "output", "ndk"):
        parser.add_argument("--" + name, required=True, type=Path)
    parser.add_argument("--version", required=True)
    args = parser.parse_args(argv)
    if not re.fullmatch(r"\d+\.\d+\.\d+", args.version):
        raise ValueError("Invalid frozen version")
    expected_name = f"enzime-reader-v{args.version}-android-native-debug-symbols.zip"
    if args.output.name != expected_name:
        raise ValueError("Symbols filename must match the frozen version")
    if args.output.exists():
        raise ValueError("Refusing to overwrite an existing symbols artifact")
    tool = args.ndk / "toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-objcopy"
    if not tool.is_file() or not os.access(tool, os.X_OK):
        raise ValueError(f"NDK llvm-objcopy missing: {tool}")
    mapping = args.android_output_dir / "mapping/universalRelease/mapping.txt"
    if not mapping.is_file() or not mapping.stat().st_size:
        raise ValueError(f"Nonempty universalRelease mapping required: {mapping}")
    inputs = {abi: args.target_dir / triple / "release" / LIBRARY for abi, triple in ABIS.items()}
    elfs = {abi: Elf(path) for abi, path in inputs.items()}
    # Reject absent DWARF before creating staging directories or output artifacts.
    architectures = {"arm64-v8a": (2, 183), "armeabi-v7a": (1, 40), "x86": (1, 3), "x86_64": (2, 62)}
    for abi, elf in elfs.items():
        if (elf.bits, elf.machine) != architectures[abi] or elf.endian != 1:
            raise ValueError(f"Compiled ELF architecture does not match {abi}: {elf.path}")
        elf.require_dwarf()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(args.apk) as apk, zipfile.ZipFile(args.aab) as aab:
        members = {abi: (member(apk, f"lib/{abi}/{LIBRARY}"), member(aab, f"base/lib/{abi}/{LIBRARY}")) for abi in ABIS}
        # Debug files cannot require more stored section bytes than their source ELF.
        # Reserve staging plus worst-case ZIP bytes, two current packaged copies, and margin.
        reserve = (2 * sum(p.stat().st_size for p in inputs.values())
                   + max(a.file_size + b.file_size for a, b in members.values())
                   + 2 * mapping.stat().st_size + 256 * 1024 * 1024)
        if shutil.disk_usage(args.output.parent).free < reserve:
            raise ValueError(f"Insufficient symbols workspace storage: need {reserve} free bytes")
        work = args.output.parent / "symbols-work" / f"v{args.version}-{uuid.uuid4().hex}"
        work.mkdir(parents=True)
        tool_version = subprocess.check_output([str(tool), "--version"], text=True).strip()
        provenance = {"version": args.version, "method": "ndk-llvm-objcopy",
                      "tool": {"path": str(tool.resolve()), "version": tool_version},
                      "mapping_sha256": digest(mapping), "abis": {}}
        try:
            for abi, source in inputs.items():
                folder = work / abi
                folder.mkdir()
                apk_copy, aab_copy = folder / "packaged-apk.so", folder / "packaged-aab.so"
                extract(apk, members[abi][0], apk_copy)
                extract(aab, members[abi][1], aab_copy)
                source_hash = digest(source)
                apk_hash, aab_hash = digest(apk_copy), digest(aab_copy)
                apk_match = corresponding(elfs[abi], Elf(apk_copy), source_hash, apk_hash)
                aab_match = corresponding(elfs[abi], Elf(aab_copy), source_hash, aab_hash)
                symbols = folder / (LIBRARY + ".dbg")
                subprocess.run([str(tool), "--only-keep-debug", str(source), str(symbols)], check=True)
                Elf(symbols).require_dwarf()
                if digest(source) != source_hash:
                    raise ValueError(f"Compiled ELF changed during collection: {source}")
                provenance["abis"][abi] = {"target": ABIS[abi], "input_sha256": source_hash,
                    "apk_sha256": apk_hash, "aab_sha256": aab_hash,
                    "symbols_sha256": digest(symbols), "apk_correspondence": apk_match,
                    "aab_correspondence": aab_match}
                apk_copy.unlink()
                aab_copy.unlink()
            provenance_path = work / "provenance.json"
            provenance_path.write_text(json.dumps(provenance, indent=2) + "\n")
            archive_path = work / args.output.name
            with zipfile.ZipFile(archive_path, "x", compression=zipfile.ZIP_DEFLATED, compresslevel=6) as archive:
                for abi in ABIS:
                    archive.write(work / abi / (LIBRARY + ".dbg"), abi + "/" + LIBRARY + ".dbg")
                archive.write(mapping, "mapping.txt")
                archive.write(provenance_path, "provenance.json")
            # Same-filesystem atomic creation; never overwrite a competing/historical result.
            os.link(archive_path, args.output)
            shutil.rmtree(work)
            print(f"Collected verified full symbols: {args.output}")
        except Exception:
            print(f"Symbols evidence retained in {work}", file=sys.stderr)
            raise
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, struct.error, zipfile.BadZipFile, subprocess.CalledProcessError) as exc:
        print(f"Android symbols collection failed: {exc}", file=sys.stderr)
        raise SystemExit(1)
