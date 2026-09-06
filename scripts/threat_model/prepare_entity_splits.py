"""Freeze common replay partitions and audit player overlap before fitting either model."""

import argparse
import hashlib
import json
from pathlib import Path

import pandas as pd

from train_entity_model import sha256, split_replays


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("manifest", type=Path)
    parser.add_argument("audit", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    audit = [json.loads(line) for line in args.audit.read_text().splitlines()]
    usable = {row["replay_id"]: row for row in audit if "error" not in row}
    rows = [json.loads(line) for line in args.manifest.read_text().splitlines()]
    seen, duplicates = {}, []
    for row in rows:
        replay_id = row["ballchasing_id"]
        if replay_id not in usable:
            continue
        digest = sha256(row["path"])
        if digest in seen:
            duplicates.append({"replay_id": replay_id, "duplicate_of": seen[digest]})
            del usable[replay_id]
        else:
            seen[digest] = replay_id
    frame = pd.DataFrame(
        [
            {"replay_id": row["ballchasing_id"], "date": row["date"]}
            for row in rows
            if row["ballchasing_id"] in usable
        ]
    )
    parts = split_replays(frame)
    players = {
        name: {
            json.dumps(player, sort_keys=True)
            for replay in part["replay_ids"]
            for player in usable[replay]["player_ids"]
        }
        for name, part in parts.items()
    }
    # This is a historical replay holdout; repeated accounts are reported, not treated as unseen players.
    overlap = {
        name: {"players": len(ids), "shared_with_train": len(ids & players["train"])}
        for name, ids in players.items()
        if name != "train"
    }
    output = {
        "partitions": {name: part["replay_ids"] for name, part in parts.items()},
        "player_overlap": overlap,
        "duplicate_replays_excluded": duplicates,
        "skipped_replays": [row for row in audit if "error" in row],
        "manifest_sha256": hashlib.sha256(args.manifest.read_bytes()).hexdigest(),
        "audit_sha256": hashlib.sha256(args.audit.read_bytes()).hexdigest(),
    }
    args.output.write_text(json.dumps(output, indent=2) + "\n")
    print(overlap)


if __name__ == "__main__":
    main()
