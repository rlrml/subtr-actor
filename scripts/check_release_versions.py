#!/usr/bin/env python3

import json
import re
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CHANGELOG = ROOT / "CHANGELOG.md"

JS_PACKAGE_PATHS = [
    "js/package.json",
    "js/player/package.json",
    "js/stat-evaluation-player/package.json",
    "js/pages/package.json",
    "js/mechanic-review-player/package.json",
]

JS_PACKAGE_LOCK_PATHS = [
    "js/package-lock.json",
    "js/player/package-lock.json",
    "js/stat-evaluation-player/package-lock.json",
    "js/mechanic-review-player/package-lock.json",
]


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
        if name in {
            "subtr-actor",
            "subtr-actor-py",
            "rl-replay-subtr-actor",
            "subtr-actor-tools",
        }:
            packages[name] = version
    return packages


def changelog_has_release_entry(version: str) -> bool:
    normalized = version.removeprefix("v")
    text = CHANGELOG.read_text()
    heading_pattern = re.compile(
        rf"^## v{re.escape(normalized)}(?:\s+-\s+.+)?$",
        re.MULTILINE,
    )
    return heading_pattern.search(text) is not None


def main() -> int:
    root = load_toml("Cargo.toml")
    expected = root["workspace"]["package"]["version"]

    python_cargo = load_toml("python/Cargo.toml")
    python_pyproject = load_toml("python/pyproject.toml")
    js_cargo = load_toml("js/Cargo.toml")
    lock_versions = load_lock_versions()

    checks = {
        "workspace.package.version": expected,
        "python/pyproject.toml [project.version]": python_pyproject["project"]["version"],
        "python/Cargo.toml dependency on subtr-actor": python_cargo["dependencies"]["subtr-actor"]["version"],
        "js/Cargo.toml dependency on subtr-actor": js_cargo["dependencies"]["subtr-actor"]["version"],
        "Cargo.lock package subtr-actor": lock_versions.get("subtr-actor"),
        "Cargo.lock package subtr-actor-py": lock_versions.get("subtr-actor-py"),
        "Cargo.lock package rl-replay-subtr-actor": lock_versions.get("rl-replay-subtr-actor"),
        "Cargo.lock package subtr-actor-tools": lock_versions.get("subtr-actor-tools"),
    }
    checks.update(
        {
            f"{path} version": load_json(path)["version"]
            for path in JS_PACKAGE_PATHS
        }
    )

    for path in JS_PACKAGE_LOCK_PATHS:
        package_lock = load_json(path)
        checks[f"{path} version"] = package_lock["version"]
        checks[f"{path} packages[''].version"] = package_lock["packages"][""]["version"]

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

    if not changelog_has_release_entry(expected):
        print(
            f"CHANGELOG.md is missing a release section for v{expected}.",
            file=sys.stderr,
        )
        return 1

    print(f"Release version metadata is consistent at {expected}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
