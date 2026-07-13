#!/usr/bin/env python3
"""Fetch a rank-stratified replay corpus from a rocket-sense instance.

Phase 1: paginate /api/v1/replays (processed) and cache the listing.
Phase 2: stratified sample by (playlist, median rank tier), download files.
Writes manifest.jsonl compatible with threat_dataset_dump.

Configuration (command-line flags, with environment-variable defaults):
  ROCKET_SENSE_API_TOKEN         bearer token for the API
  ROCKET_SENSE_TOKEN_COMMAND     shell command printing the token on stdout
                                 (default: `pass show rocket-sense/token`)
  ROCKET_SENSE_BASE_URL          API base (default the production instance)
  THREAT_CORPUS_CACHE            cache directory
                                 (default ~/.cache/subtr-actor-threat-corpus)
  PER_STRATUM                    replays per (playlist, tier) stratum (150)
  THREAT_CORPUS_SEED             deterministic sampling seed (7)
  THREAT_CORPUS_PLAYLISTS        comma-separated playlists
                                 (ranked-doubles)
"""

import argparse
import concurrent.futures
import hashlib
import json
import os
import pathlib
import random
import statistics
import subprocess
import sys
import urllib.request

BASE = os.environ.get("ROCKET_SENSE_BASE_URL", "https://rocket-sense.duckdns.org/api/v1")
CACHE = pathlib.Path(
    os.environ.get("THREAT_CORPUS_CACHE", pathlib.Path.home() / ".cache/subtr-actor-threat-corpus")
)
LISTING = CACHE / "listing.jsonl"
MANIFEST = CACHE / "manifest.jsonl"
REPLAYS = CACHE / "replays"
DEFAULT_PLAYLISTS = "ranked-doubles"
ALLOWED_PLAYLISTS = {"ranked-doubles"}


def resolve_token() -> str:
    token = os.environ.get("ROCKET_SENSE_API_TOKEN")
    if token:
        return token.strip()
    command = os.environ.get("ROCKET_SENSE_TOKEN_COMMAND", "pass show rocket-sense/token")
    result = subprocess.run(command, shell=True, capture_output=True, text=True, check=True)
    token = result.stdout.splitlines()[0].strip() if result.stdout else ""
    if not token:
        raise SystemExit(
            "no API token: set ROCKET_SENSE_API_TOKEN or make ROCKET_SENSE_TOKEN_COMMAND print one"
        )
    return token


TOKEN: str | None = None


def api(path):
    req = urllib.request.Request(BASE + path, headers={"Authorization": f"Bearer {TOKEN}"})
    with urllib.request.urlopen(req, timeout=120) as resp:
        return json.load(resp)


def fetch_listing():
    if LISTING.exists():
        rows = [json.loads(line) for line in LISTING.read_text().splitlines()]
        print(f"listing cache: {len(rows)} rows", file=sys.stderr)
        return rows
    rows = []
    offset = 0
    while True:
        page = api(f"/replays?status=processed&count=200&offset={offset}")
        for r in page["replays"]:
            tiers = [
                p.get("rank_tier") for p in r.get("players") or [] if p.get("rank_tier") is not None
            ]
            rows.append(
                {
                    "id": r["id"],
                    "sha256": r.get("file_sha256"),
                    "playlist": r.get("playlist"),
                    "replay_date": r.get("replay_date"),
                    "team_size": (r.get("playlist_metadata") or {}).get("team_size"),
                    "min_rank_tier": min(tiers) if tiers else None,
                    "max_rank_tier": max(tiers) if tiers else None,
                    "median_rank_tier": statistics.median(tiers) if tiers else None,
                }
            )
        offset = page.get("next_offset")
        print(f"listed {len(rows)}/{page['total']}", file=sys.stderr)
        if offset is None or len(rows) >= page["total"]:
            break
    with open(LISTING, "w") as f:
        for r in rows:
            f.write(json.dumps(r) + "\n")
    return rows


def stratify(rows, playlists, per_stratum, seed):
    seen_sha = set()
    strata = {}
    for r in rows:
        if r["playlist"] not in playlists or r["median_rank_tier"] is None:
            continue
        replay_identity = r["sha256"] or f"id:{r['id']}"
        if replay_identity in seen_sha:
            continue
        seen_sha.add(replay_identity)
        key = (r["playlist"], int(round(r["median_rank_tier"])))
        strata.setdefault(key, []).append(r)
    picked = []
    for key in sorted(strata):
        bucket = sorted(strata[key], key=lambda r: r["id"])
        seed_material = f"{seed}:{key[0]}:{key[1]}".encode()
        stratum_seed = int.from_bytes(hashlib.sha256(seed_material).digest()[:8], "big")
        rng = random.Random(stratum_seed)
        rng.shuffle(bucket)
        take = bucket[:per_stratum]
        picked.extend(take)
        print(f"stratum {key}: {len(take)}/{len(bucket)}", file=sys.stderr)
    return picked


def download(r):
    dest = REPLAYS / f"{r['id']}.replay"
    if dest.exists() and dest.stat().st_size > 0:
        return "cached"
    try:
        req = urllib.request.Request(
            f"{BASE}/replays/{r['id']}/file",
            headers={"Authorization": f"Bearer {TOKEN}"},
        )
        with urllib.request.urlopen(req, timeout=300) as resp:
            data = resp.read()
        tmp = dest.with_suffix(".part")
        tmp.write_bytes(data)
        tmp.rename(dest)
        return "ok"
    except Exception as e:  # noqa: BLE001
        return f"fail: {e}"


def parse_args():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--playlist",
        action="append",
        dest="playlists",
        choices=sorted(ALLOWED_PLAYLISTS),
        help="playlist to include (fixed to ranked doubles)",
    )
    parser.add_argument(
        "--per-stratum", type=int, default=int(os.environ.get("PER_STRATUM", "150"))
    )
    parser.add_argument("--seed", type=int, default=int(os.environ.get("THREAT_CORPUS_SEED", "7")))
    args = parser.parse_args()
    if args.playlists is None:
        args.playlists = [
            value.strip()
            for value in os.environ.get("THREAT_CORPUS_PLAYLISTS", DEFAULT_PLAYLISTS).split(",")
            if value.strip()
        ]
    unknown = set(args.playlists) - ALLOWED_PLAYLISTS
    if unknown:
        parser.error(f"unknown playlists: {', '.join(sorted(unknown))}")
    if not args.playlists:
        parser.error("select at least one playlist")
    if args.per_stratum <= 0:
        parser.error("--per-stratum must be positive")
    return args


def file_sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main():
    global TOKEN
    args = parse_args()
    TOKEN = resolve_token()
    REPLAYS.mkdir(parents=True, exist_ok=True)
    rows = fetch_listing()
    picked = stratify(rows, set(args.playlists), args.per_stratum, args.seed)
    print(f"selected {len(picked)} replays", file=sys.stderr)

    results = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as ex:
        for r, res in zip(picked, ex.map(download, picked)):
            results[r["id"]] = res
    fails = {k: v for k, v in results.items() if v.startswith("fail")}
    print(
        f"downloaded ok/cached={len(results) - len(fails)} failed={len(fails)}",
        file=sys.stderr,
    )
    for k, v in list(fails.items())[:10]:
        print(f"  {k}: {v}", file=sys.stderr)

    with open(MANIFEST, "w") as f:
        for r in picked:
            if results.get(r["id"], "").startswith("fail"):
                continue
            f.write(
                json.dumps(
                    {
                        "path": str(REPLAYS / f"{r['id']}.replay"),
                        "ballchasing_id": r["id"],
                        "playlist": r["playlist"],
                        "team_size": r["team_size"],
                        "min_rank_tier": r["min_rank_tier"],
                        "max_rank_tier": r["max_rank_tier"],
                        "median_rank_tier": r["median_rank_tier"],
                        "date": r["replay_date"],
                    }
                )
                + "\n"
            )
    provenance = {
        "base_url": BASE,
        "listing": str(LISTING),
        "listing_sha256": file_sha256(LISTING),
        "manifest": str(MANIFEST),
        "manifest_sha256": file_sha256(MANIFEST),
        "seed": args.seed,
        "sampling_algorithm": "sha256-derived per-stratum Python random shuffle v1",
        "per_stratum": args.per_stratum,
        "playlists": sorted(set(args.playlists)),
        "selected_replays": sum(
            not results.get(row["id"], "").startswith("fail") for row in picked
        ),
    }
    provenance_path = MANIFEST.with_suffix(".provenance.json")
    provenance_path.write_text(json.dumps(provenance, indent=2) + "\n")
    print(f"manifest: {MANIFEST}", file=sys.stderr)
    print(f"provenance: {provenance_path}", file=sys.stderr)


if __name__ == "__main__":
    main()
