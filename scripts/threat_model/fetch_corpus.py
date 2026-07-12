#!/usr/bin/env python3
"""Fetch a rank-stratified replay corpus from a rocket-sense instance.

Phase 1: paginate /api/v1/replays (processed) and cache the listing.
Phase 2: stratified sample by (playlist, median rank tier), download files.
Writes manifest.jsonl compatible with threat_dataset_dump.

Configuration (environment variables):
  ROCKET_SENSE_API_TOKEN         bearer token for the API
  ROCKET_SENSE_TOKEN_COMMAND     shell command printing the token on stdout
                                 (default: `pass show rocket-sense/token`)
  ROCKET_SENSE_BASE_URL          API base (default the production instance)
  THREAT_CORPUS_CACHE            cache directory
                                 (default ~/.cache/subtr-actor-threat-corpus)
  PER_STRATUM                    replays per (playlist, tier) stratum (150)
"""

import concurrent.futures
import json
import os
import pathlib
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
PER_STRATUM = int(os.environ.get("PER_STRATUM", "150"))  # per (playlist, tier)
PLAYLISTS = {"ranked-doubles", "ranked-duels", "ranked-standard"}


def resolve_token() -> str:
    token = os.environ.get("ROCKET_SENSE_API_TOKEN")
    if token:
        return token.strip()
    command = os.environ.get("ROCKET_SENSE_TOKEN_COMMAND", "pass show rocket-sense/token")
    result = subprocess.run(command, shell=True, capture_output=True, text=True, check=True)
    token = result.stdout.splitlines()[0].strip() if result.stdout else ""
    if not token:
        raise SystemExit(
            "no API token: set ROCKET_SENSE_API_TOKEN or make "
            "ROCKET_SENSE_TOKEN_COMMAND print one"
        )
    return token


TOKEN: str | None = None


def api(path):
    req = urllib.request.Request(BASE + path, headers={"Authorization": f"Bearer {TOKEN}"})
    with urllib.request.urlopen(req, timeout=120) as resp:
        return json.load(resp)


def fetch_listing():
    if LISTING.exists():
        rows = [json.loads(l) for l in LISTING.read_text().splitlines()]
        print(f"listing cache: {len(rows)} rows", file=sys.stderr)
        return rows
    rows = []
    offset = 0
    while True:
        page = api(f"/replays?status=processed&count=200&offset={offset}")
        for r in page["replays"]:
            tiers = [p.get("rank_tier") for p in r.get("players") or [] if p.get("rank_tier") is not None]
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


def stratify(rows):
    seen_sha = set()
    strata = {}
    for r in rows:
        if r["playlist"] not in PLAYLISTS or r["median_rank_tier"] is None:
            continue
        if r["sha256"] in seen_sha:
            continue
        seen_sha.add(r["sha256"])
        key = (r["playlist"], int(round(r["median_rank_tier"])))
        strata.setdefault(key, []).append(r)
    picked = []
    for key in sorted(strata):
        bucket = strata[key]
        bucket.sort(key=lambda r: r["id"])  # deterministic
        take = bucket[:PER_STRATUM]
        picked.extend(take)
        print(f"stratum {key}: {len(take)}/{len(bucket)}", file=sys.stderr)
    return picked


def download(r):
    dest = REPLAYS / f"{r['id']}.replay"
    if dest.exists() and dest.stat().st_size > 0:
        return "cached"
    try:
        req = urllib.request.Request(
            f"{BASE}/replays/{r['id']}/file", headers={"Authorization": f"Bearer {TOKEN}"}
        )
        with urllib.request.urlopen(req, timeout=300) as resp:
            data = resp.read()
        tmp = dest.with_suffix(".part")
        tmp.write_bytes(data)
        tmp.rename(dest)
        return "ok"
    except Exception as e:  # noqa: BLE001
        return f"fail: {e}"


def main():
    global TOKEN
    TOKEN = resolve_token()
    REPLAYS.mkdir(parents=True, exist_ok=True)
    rows = fetch_listing()
    picked = stratify(rows)
    print(f"selected {len(picked)} replays", file=sys.stderr)

    results = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as ex:
        for r, res in zip(picked, ex.map(download, picked)):
            results[r["id"]] = res
    fails = {k: v for k, v in results.items() if v.startswith("fail")}
    print(f"downloaded ok/cached={len(results) - len(fails)} failed={len(fails)}", file=sys.stderr)
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
    print(f"manifest: {MANIFEST}", file=sys.stderr)


if __name__ == "__main__":
    main()
