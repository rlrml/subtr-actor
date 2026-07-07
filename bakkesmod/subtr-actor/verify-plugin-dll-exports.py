#!/usr/bin/env python3
"""Validate BakkesMod-facing exports of a built plugin DLL artifact."""

from __future__ import annotations

import argparse
import importlib.util
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PE_EXPORT_VERIFIER = REPO_ROOT / "bakkesmod/subtr-actor/verify-rust-dll-exports.py"
REQUIRED_BAKKESMOD_EXPORTS = frozenset({"getPlugin", "deleteMe", "exports"})


def load_pe_parser():
    spec = importlib.util.spec_from_file_location(
        "verify_rust_dll_exports",
        PE_EXPORT_VERIFIER,
    )
    if spec is None or spec.loader is None:
        raise RuntimeError(f"could not import {PE_EXPORT_VERIFIER}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def validate_dll(path: Path, pe_parser) -> list[str]:
    errors: list[str] = []
    try:
        pe_exports = pe_parser.parse_pe_exports(path)
    except (OSError, UnicodeDecodeError, pe_parser.PeFormatError) as exc:
        return [f"{path}: {exc}"]

    missing = sorted(REQUIRED_BAKKESMOD_EXPORTS - pe_exports.names)
    if missing:
        errors.append(f"{path}: missing required BakkesMod plugin exports: {missing}")
    if pe_exports.machine != 0x8664:
        errors.append(f"{path}: expected AMD64 PE machine 0x8664, got 0x{pe_exports.machine:04x}")
    if not pe_exports.is_pe32_plus:
        errors.append(f"{path}: expected PE32+ 64-bit DLL")

    if not errors:
        print(
            "Verified BakkesMod plugin exports "
            f"{sorted(REQUIRED_BAKKESMOD_EXPORTS)} in {path}"
        )
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("dll", nargs="+", type=Path, help="SubtrActorPlugin.dll artifact")
    args = parser.parse_args()

    pe_parser = load_pe_parser()
    errors: list[str] = []
    for dll in args.dll:
        errors.extend(validate_dll(dll, pe_parser))

    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
