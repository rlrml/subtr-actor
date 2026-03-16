#!/usr/bin/env python3

import json
import re
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


def load_toml(path: str) -> dict:
    return tomllib.loads((ROOT / path).read_text())


def load_json(path: str) -> dict:
    return json.loads((ROOT / path).read_text())


def load_lock_versions() -> dict[str, str]:
    text = (ROOT / "Cargo.lock").read_text()
    packages: dict[str, str] = {}
    for name, version in re.findall(
        r'\[\[package\]\]\nname = "([^"]+)"\nversion = "([^"]+)"',
        text,
    ):
        if name in {"subtr-actor", "subtr-actor-py", "rl-replay-subtr-actor"}:
            packages[name] = version
    return packages


def main() -> int:
    root = load_toml("Cargo.toml")
    expected = root["workspace"]["package"]["version"]

    python_cargo = load_toml("python/Cargo.toml")
    python_pyproject = load_toml("python/pyproject.toml")
    js_cargo = load_toml("js/Cargo.toml")
    js_package = load_json("js/package.json")
    js_player_package = load_json("js/player/package.json")
    lock_versions = load_lock_versions()

    checks = {
        "workspace.package.version": expected,
        "python/pyproject.toml [project.version]": python_pyproject["project"]["version"],
        "python/Cargo.toml dependency on subtr-actor": python_cargo["dependencies"]["subtr-actor"]["version"],
        "js/package.json version": js_package["version"],
        "js/player/package.json version": js_player_package["version"],
        "js/Cargo.toml dependency on subtr-actor": js_cargo["dependencies"]["subtr-actor"]["version"],
        "Cargo.lock package subtr-actor": lock_versions.get("subtr-actor"),
        "Cargo.lock package subtr-actor-py": lock_versions.get("subtr-actor-py"),
        "Cargo.lock package rl-replay-subtr-actor": lock_versions.get("rl-replay-subtr-actor"),
    }

    mismatches = [
        f"{name}: expected {expected}, found {value}"
        for name, value in checks.items()
        if value != expected
    ]

    if mismatches:
        print("Release version metadata is inconsistent.", file=sys.stderr)
        for mismatch in mismatches:
            print(f"- {mismatch}", file=sys.stderr)
        return 1

    print(f"Release version metadata is consistent at {expected}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
