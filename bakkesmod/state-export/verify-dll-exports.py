#!/usr/bin/env python3
"""Validate built state-export DLL artifacts.

Checks that ``state_export.dll`` exports exactly the ``state_export_``
functions declared in ``rust/include/state_export.h`` (the header the C++
plugin binds with GetProcAddress), and that ``StateExportPlugin.dll``
exposes the BakkesMod plugin entry points. Reuses the PE export-table
parser from ``bakkesmod/subtr-actor/verify-rust-dll-exports.py``.
"""

from __future__ import annotations

import argparse
import importlib.util
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
ABI_HEADER = REPO_ROOT / "bakkesmod/state-export/rust/include/state_export.h"
PE_EXPORT_VERIFIER = REPO_ROOT / "bakkesmod/subtr-actor/verify-rust-dll-exports.py"
EXPORT_PREFIX = "state_export_"
REQUIRED_BAKKESMOD_EXPORTS = frozenset({"getPlugin", "deleteMe", "exports"})


def load_pe_parser():
    spec = importlib.util.spec_from_file_location(
        "verify_rust_dll_exports",
        PE_EXPORT_VERIFIER,
    )
    module = importlib.util.module_from_spec(spec)
    # Register before exec so dataclass processing can resolve the module.
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def declared_header_exports() -> set[str]:
    header = ABI_HEADER.read_text()
    return set(re.findall(rf"\b({EXPORT_PREFIX}[A-Za-z0-9_]+)\s*\(", header))


def validate_rust_dll(path: Path, pe_parser, header_exports: set[str]) -> list[str]:
    errors: list[str] = []
    if not path.is_file():
        return [f"{path}: missing DLL artifact"]
    exports = pe_parser.parse_pe_exports(path)
    prefixed = {name for name in exports.names if name.startswith(EXPORT_PREFIX)}
    missing = sorted(header_exports - prefixed)
    if missing:
        errors.append(f"{path}: missing header-declared exports: {missing}")
    undocumented = sorted(prefixed - header_exports)
    if undocumented:
        errors.append(f"{path}: exports undocumented {EXPORT_PREFIX} symbols: {undocumented}")
    if not errors:
        print(f"Verified {len(prefixed)} {EXPORT_PREFIX} exports in {path}")
    return errors


def validate_plugin_dll(path: Path, pe_parser) -> list[str]:
    errors: list[str] = []
    if not path.is_file():
        return [f"{path}: missing DLL artifact"]
    exports = pe_parser.parse_pe_exports(path)
    missing = sorted(REQUIRED_BAKKESMOD_EXPORTS - set(exports.names))
    if missing:
        errors.append(f"{path}: missing BakkesMod plugin exports: {missing}")
    else:
        print(f"Verified BakkesMod plugin exports in {path}")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rust-dll", type=Path, required=True, help="state_export.dll artifact")
    parser.add_argument(
        "--plugin-dll", type=Path, required=True, help="StateExportPlugin.dll artifact"
    )
    args = parser.parse_args()

    pe_parser = load_pe_parser()
    header_exports = declared_header_exports()
    if not header_exports:
        print(f"ERROR: no {EXPORT_PREFIX} declarations found in {ABI_HEADER}", file=sys.stderr)
        return 1

    errors = validate_rust_dll(args.rust_dll, pe_parser, header_exports)
    errors.extend(validate_plugin_dll(args.plugin_dll, pe_parser))
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
