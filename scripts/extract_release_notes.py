#!/usr/bin/env python3

import argparse
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CHANGELOG = ROOT / "CHANGELOG.md"


def extract_release_notes(version: str) -> str:
    normalized = version.removeprefix("v")
    text = CHANGELOG.read_text()
    heading_pattern = re.compile(
        rf"^## v{re.escape(normalized)}(?:\s+-\s+.+)?$",
        re.MULTILINE,
    )
    heading_match = heading_pattern.search(text)
    if not heading_match:
        raise ValueError(f"Could not find changelog entry for v{normalized}.")

    next_heading = re.compile(r"^## v[0-9].*$", re.MULTILINE).search(
        text, heading_match.end()
    )
    end = next_heading.start() if next_heading else len(text)
    return text[heading_match.start() : end].strip()


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Extract a release section from CHANGELOG.md."
    )
    parser.add_argument("version", help="Release version with or without the v prefix.")
    args = parser.parse_args()

    try:
        print(extract_release_notes(args.version))
    except ValueError as exc:
        print(str(exc), file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
