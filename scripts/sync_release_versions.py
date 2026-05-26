#!/usr/bin/env python3

import argparse
import json
import re
import subprocess
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent

PACKAGE_JSON_PATHS = [
    "js/package.json",
    "js/player/package.json",
    "js/stat-evaluation-player/package.json",
    "js/pages/package.json",
]

PACKAGE_LOCK_PATHS = [
    "js/package-lock.json",
    "js/player/package-lock.json",
    "js/stat-evaluation-player/package-lock.json",
]


def read_workspace_version() -> str:
    data = tomllib.loads((ROOT / "Cargo.toml").read_text())
    return data["workspace"]["package"]["version"]


def replace_once(path: str, pattern: str, replacement: str) -> None:
    target = ROOT / path
    text = target.read_text()
    next_text, count = re.subn(pattern, replacement, text, count=1, flags=re.MULTILINE | re.DOTALL)
    if count != 1:
        raise RuntimeError(f"Expected exactly one version match in {path}, found {count}.")
    target.write_text(next_text)


def set_workspace_version(version: str) -> None:
    replace_once(
        "Cargo.toml",
        r'(\[workspace\.package\][^\[]*?^version = ")[^"]+(")',
        rf"\g<1>{version}\2",
    )


def set_project_version(path: str, version: str) -> None:
    replace_once(
        path,
        r'(\[project\][^\[]*?^version = ")[^"]+(")',
        rf"\g<1>{version}\2",
    )


def set_subtr_actor_dependency_version(path: str, version: str) -> None:
    replace_once(
        path,
        r'(\[dependencies\.subtr-actor\][^\[]*?^version = ")[^"]+(")',
        rf"\g<1>{version}\2",
    )


def update_json_version(path: str, version: str) -> None:
    target = ROOT / path
    data = json.loads(target.read_text())
    data["version"] = version
    target.write_text(json.dumps(data, indent=2) + "\n")


def update_package_lock_version(path: str, version: str) -> None:
    target = ROOT / path
    data = json.loads(target.read_text())
    data["version"] = version
    data["packages"][""]["version"] = version
    target.write_text(json.dumps(data, indent=2) + "\n")


def run_cargo_metadata() -> None:
    subprocess.run(
        ["cargo", "metadata", "--format-version", "1"],
        cwd=ROOT,
        check=True,
        stdout=subprocess.DEVNULL,
    )


def sync_versions(version: str) -> None:
    set_workspace_version(version)
    set_project_version("python/pyproject.toml", version)
    set_subtr_actor_dependency_version("python/Cargo.toml", version)
    set_subtr_actor_dependency_version("js/Cargo.toml", version)

    for path in PACKAGE_JSON_PATHS:
        update_json_version(path, version)
    for path in PACKAGE_LOCK_PATHS:
        update_package_lock_version(path, version)

    run_cargo_metadata()


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Sync release metadata from the Cargo workspace version.",
    )
    parser.add_argument(
        "version",
        nargs="?",
        help="Version to set before syncing. Defaults to Cargo.toml workspace.package.version.",
    )
    args = parser.parse_args()

    version = args.version or read_workspace_version()
    if not re.fullmatch(r"\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?", version):
        print(f"Invalid release version: {version}", file=sys.stderr)
        return 2

    sync_versions(version)
    print(f"Synced release metadata to {version}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
