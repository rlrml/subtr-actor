#!/usr/bin/env python3
"""Validate the exported C ABI of a built subtr_actor_bakkesmod.dll.

The BakkesMod plugin loads the Rust analysis engine with LoadLibrary and
GetProcAddress. This verifier reads the PE export table directly so CI proves
the built DLL artifact exposes the ABI declared in the checked-in C header.
"""

from __future__ import annotations

import argparse
import importlib.util
import re
import struct
import sys
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
ABI_HEADER = REPO_ROOT / "crates/subtr-actor-bakkesmod/include/subtr_actor_bakkesmod.h"
PLUGIN_SOURCE_VERIFIER = REPO_ROOT / "bakkesmod/verify-plugin-source.py"
EXPORT_PREFIX = "subtr_actor_bakkesmod_"


@dataclass(frozen=True)
class Section:
    name: str
    virtual_address: int
    virtual_size: int
    raw_size: int
    raw_pointer: int


@dataclass(frozen=True)
class PeExports:
    machine: int
    is_pe32_plus: bool
    names: frozenset[str]


class PeFormatError(ValueError):
    pass


def read_u16(data: bytes, offset: int, label: str) -> int:
    require_range(data, offset, 2, label)
    return struct.unpack_from("<H", data, offset)[0]


def read_u32(data: bytes, offset: int, label: str) -> int:
    require_range(data, offset, 4, label)
    return struct.unpack_from("<I", data, offset)[0]


def require_range(data: bytes, offset: int, size: int, label: str) -> None:
    if offset < 0 or size < 0 or offset + size > len(data):
        raise PeFormatError(f"{label} points outside file bounds")


def load_source_verifier_required_exports() -> set[str]:
    spec = importlib.util.spec_from_file_location(
        "verify_plugin_source",
        PLUGIN_SOURCE_VERIFIER,
    )
    if spec is None or spec.loader is None:
        raise RuntimeError(f"could not import {PLUGIN_SOURCE_VERIFIER}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return {symbol for symbol, _ in module.REQUIRED_PLUGIN_ABI_EXPORTS}


def declared_header_exports() -> set[str]:
    header = ABI_HEADER.read_text(encoding="utf-8")
    return set(re.findall(rf"\b({EXPORT_PREFIX}[A-Za-z0-9_]+)\s*\(", header))


def parse_pe_exports(path: Path) -> PeExports:
    data = path.read_bytes()
    if len(data) < 0x40 or data[:2] != b"MZ":
        raise PeFormatError("missing DOS MZ header")

    pe_offset = read_u32(data, 0x3C, "PE header offset")
    require_range(data, pe_offset, 4 + 20, "PE header")
    if data[pe_offset : pe_offset + 4] != b"PE\0\0":
        raise PeFormatError("missing PE signature")

    coff_offset = pe_offset + 4
    machine = read_u16(data, coff_offset, "COFF machine")
    section_count = read_u16(data, coff_offset + 2, "COFF section count")
    optional_header_size = read_u16(data, coff_offset + 16, "COFF optional header size")

    optional_offset = coff_offset + 20
    require_range(data, optional_offset, optional_header_size, "optional header")
    magic = read_u16(data, optional_offset, "optional header magic")
    if magic == 0x10B:
        data_directories_offset = optional_offset + 96
        is_pe32_plus = False
    elif magic == 0x20B:
        data_directories_offset = optional_offset + 112
        is_pe32_plus = True
    else:
        raise PeFormatError(f"unsupported optional header magic 0x{magic:04x}")
    require_range(data, data_directories_offset, 8, "export data directory")
    export_rva = read_u32(data, data_directories_offset, "export directory RVA")
    export_size = read_u32(data, data_directories_offset + 4, "export directory size")
    if export_rva == 0 or export_size == 0:
        raise PeFormatError("PE file has no export directory")

    sections_offset = optional_offset + optional_header_size
    require_range(data, sections_offset, section_count * 40, "section table")
    sections: list[Section] = []
    for index in range(section_count):
        offset = sections_offset + index * 40
        name_bytes = data[offset : offset + 8].split(b"\0", 1)[0]
        name = name_bytes.decode("ascii", errors="replace")
        virtual_size = read_u32(data, offset + 8, f"{name} virtual size")
        virtual_address = read_u32(data, offset + 12, f"{name} virtual address")
        raw_size = read_u32(data, offset + 16, f"{name} raw size")
        raw_pointer = read_u32(data, offset + 20, f"{name} raw pointer")
        sections.append(
            Section(
                name=name,
                virtual_address=virtual_address,
                virtual_size=virtual_size,
                raw_size=raw_size,
                raw_pointer=raw_pointer,
            )
        )

    def rva_to_offset(rva: int, label: str) -> int:
        for section in sections:
            section_size = max(section.virtual_size, section.raw_size)
            if section.virtual_address <= rva < section.virtual_address + section_size:
                file_offset = section.raw_pointer + (rva - section.virtual_address)
                require_range(data, file_offset, 1, label)
                return file_offset
        raise PeFormatError(f"{label} RVA 0x{rva:x} does not map to a section")

    def read_c_string(rva: int, label: str) -> str:
        offset = rva_to_offset(rva, label)
        end = data.find(b"\0", offset)
        if end == -1:
            raise PeFormatError(f"{label} is not NUL-terminated")
        return data[offset:end].decode("ascii")

    export_offset = rva_to_offset(export_rva, "export directory")
    require_range(data, export_offset, 40, "IMAGE_EXPORT_DIRECTORY")
    name_count = read_u32(data, export_offset + 24, "export name count")
    names_rva = read_u32(data, export_offset + 32, "export name table")
    names_offset = rva_to_offset(names_rva, "export name table")
    require_range(data, names_offset, name_count * 4, "export name table")

    names: set[str] = set()
    for index in range(name_count):
        name_rva = read_u32(data, names_offset + index * 4, f"export name {index}")
        names.add(read_c_string(name_rva, f"export name {index}"))

    return PeExports(machine=machine, is_pe32_plus=is_pe32_plus, names=frozenset(names))


def validate_dll(path: Path, required_exports: set[str], header_exports: set[str]) -> list[str]:
    errors: list[str] = []
    try:
        pe_exports = parse_pe_exports(path)
    except (OSError, UnicodeDecodeError, PeFormatError) as exc:
        return [f"{path}: {exc}"]

    prefixed_exports = {name for name in pe_exports.names if name.startswith(EXPORT_PREFIX)}
    missing_required = sorted(required_exports - pe_exports.names)
    missing_header = sorted(header_exports - pe_exports.names)
    undocumented = sorted(prefixed_exports - header_exports)

    if missing_required:
        errors.append(f"{path}: missing C++-loaded Rust ABI exports: {missing_required}")
    if missing_header:
        errors.append(f"{path}: missing checked-in header exports: {missing_header}")
    if undocumented:
        errors.append(f"{path}: exports undocumented {EXPORT_PREFIX} symbols: {undocumented}")
    if pe_exports.machine != 0x8664:
        errors.append(f"{path}: expected AMD64 PE machine 0x8664, got 0x{pe_exports.machine:04x}")
    if not pe_exports.is_pe32_plus:
        errors.append(f"{path}: expected PE32+ 64-bit DLL")

    if not errors:
        print(f"Verified {len(prefixed_exports)} {EXPORT_PREFIX} exports in {path}")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("dll", nargs="+", type=Path, help="subtr_actor_bakkesmod.dll artifact")
    args = parser.parse_args()

    required_exports = load_source_verifier_required_exports()
    header_exports = declared_header_exports()
    missing_from_header = sorted(required_exports - header_exports)
    if missing_from_header:
        print(
            f"ERROR: required plugin ABI exports missing from checked-in header: {missing_from_header}",
            file=sys.stderr,
        )
        return 1

    errors: list[str] = []
    for dll in args.dll:
        errors.extend(validate_dll(dll, required_exports, header_exports))

    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
