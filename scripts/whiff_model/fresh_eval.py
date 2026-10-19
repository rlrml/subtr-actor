"""Stratified before/after evaluation of whiff detectors on unlabeled replays.

Population: every core candidate closest approach (`whiff_candidate_dump`)
plus every event of the legacy detector (`whiff_dataset_dump` built from the
pre-model code) and the new detector, in replays never used for training.
Interactions are grouped per player and time (0.4 s), then stratified by which
system flagged them:

    both, new_only, legacy_only   flagged by at least one system
    near_unflagged                neither system, contact gap <= 300 uu

Each stratum is sampled with a known probability, and every sampled
interaction becomes a blinded review window anchored on it. Precision and
recall per system and class are Horvitz-Thompson estimates over the labeled
anchor interactions; `uncertain` anchors are reported separately and
excluded from the lenient metrics. Recall is relative to interactions inside
this population: misses wider than 600 uu that no system proposes are not
covered.
"""

from __future__ import annotations

import collections
import hashlib
import pathlib
import random

import numpy as np

import mining
from core_model import load_core_dump
from modeling import MATCH_TOLERANCE_SECONDS
from telemetry import dump_path, load_dump

NEAR_GAP = 300.0
NEW_KINDS = {"whiff": "clear_whiff", "disrupted_attempt": "disrupted_attempt"}
LEGACY_KINDS = {"whiff": "clear_whiff", "beaten_to_ball": "disrupted_attempt"}
EVENT_CLASSES = ("clear_whiff", "disrupted_attempt")


def interactions_for_replay(cache: pathlib.Path, core_dir: pathlib.Path, replay_id: str):
    r = load_dump(dump_path(cache / "dumps", replay_id))
    core = load_core_dump(core_dir / f"{replay_id}.candidates.json.gz")
    rows = []

    def attach(player_id, time, frame, field, value, gap=None):
        for row in rows:
            if row["player_id"] == player_id and abs(row["time"] - time) <= MATCH_TOLERANCE_SECONDS:
                if row.get(field) is None:
                    row[field] = value
                if gap is not None and row["gap"] is None:
                    row["gap"] = gap
                return
        rows.append(
            {
                "replay_id": replay_id,
                "player_id": player_id,
                "time": time,
                "frame": frame,
                "new": None,
                "legacy": None,
                "gap": gap,
                field: value,
            }
        )

    for e in core["whiff_events"]:
        attach(e["player_id"], e["time"], e["frame"], "new", NEW_KINDS[e["kind"]])
    for e in r.whiff_events:
        attach(e["player_id"], e["time"], e["frame"], "legacy", LEGACY_KINDS["whiff"])
    for e in r.beaten_to_ball_events:
        attach(e["player_id"], e["time"], e["frame"], "legacy", LEGACY_KINDS["beaten_to_ball"])
    for c in core["candidates"]:
        gap = c["features"][core["feature_names"].index("gap_at_anchor")]
        if gap is not None and gap <= NEAR_GAP:
            attach(c["player_id"], c["anchor_time"], c["anchor_frame"], "candidate", True, gap)
    for row in rows:
        row["stratum"] = (
            "both"
            if row["new"] and row["legacy"]
            else "new_only"
            if row["new"]
            else "legacy_only"
            if row["legacy"]
            else "near_unflagged"
        )
    return r, rows


def build_manifest(cache, core_dir, replay_ids, quotas, name, seed):
    rng = random.Random(f"{seed}:{name}")
    by_stratum = collections.defaultdict(list)
    telemetry = {}
    for replay_id in replay_ids:
        r, rows = interactions_for_replay(cache, core_dir, replay_id)
        telemetry[replay_id] = r
        for row in rows:
            by_stratum[row["stratum"]].append(row)
    windows = []
    population = {}
    for stratum, rows in sorted(by_stratum.items()):
        rows.sort(key=lambda row: (row["replay_id"], row["player_id"], row["time"]))
        population[stratum] = len(rows)
        take = rng.sample(rows, min(quotas.get(stratum, 0), len(rows)))
        probability = len(take) / len(rows)
        for row in take:
            r = telemetry[row["replay_id"]]
            player_index = next(
                i for i, t in enumerate(r.players) if t.player_id == row["player_id"]
            )
            anchor = r.index_for_frame(row["frame"])
            if anchor is None:
                anchor = r.index_at_time(row["time"])
            window = mining.make_window(r, player_index, anchor, [f"fresh_{stratum}"])
            window.update(
                {
                    "split": "fresh",
                    "stratum": stratum,
                    "manifest_stratum": stratum,
                    "sampling_probability": probability,
                    "interaction": {k: row[k] for k in ("time", "new", "legacy", "gap")},
                    "manifest": name,
                }
            )
            window["neutral_id"] = (
                "w"
                + hashlib.sha256(f"{name}:{seed}:{window['window_id']}".encode()).hexdigest()[:10]
            )
            windows.append(window)
    rng.shuffle(windows)
    return windows, population


def anchor_label(final_row: dict, anchor_time: float) -> str:
    """Final label of the interaction at the anchor; none there = no attempt."""
    if not final_row["usable"]:
        return "unusable"
    near = [
        i
        for i in final_row["interactions"]
        if abs(i["t_replay"] - anchor_time) <= MATCH_TOLERANCE_SECONDS
    ]
    if not near:
        return "no_observable_attempt"
    return min(near, key=lambda i: abs(i["t_replay"] - anchor_time))["label"]


def score(
    windows: list[dict],
    finals: dict[str, dict],
    n_boot: int = 2000,
    seed: int = 7,
    classes: tuple[str, ...] = EVENT_CLASSES,
) -> dict:
    rows = []
    for w in windows:
        final = finals.get(w["neutral_id"])
        if final is None:
            continue
        rows.append(
            {
                "replay_id": w["replay_id"],
                "weight": 1.0 / w["sampling_probability"],
                "stratum": w["manifest_stratum"],
                "new": w["interaction"]["new"],
                "legacy": w["interaction"]["legacy"],
                "label": anchor_label(final, w["anchor_time"]),
            }
        )

    def estimate(sample):
        out = {}
        for system in ("new", "legacy"):
            for cls in classes:
                flagged = [r for r in sample if r[system] == cls]
                tp = sum(r["weight"] for r in flagged if r["label"] == cls)
                known = sum(
                    r["weight"] for r in flagged if r["label"] not in ("uncertain", "unusable")
                )
                uncertain = sum(r["weight"] for r in flagged if r["label"] == "uncertain")
                positives = sum(r["weight"] for r in sample if r["label"] == cls)
                out[f"{system}/{cls}"] = {
                    "precision": tp / known if known else None,
                    "strict_precision": tp / (known + uncertain) if known + uncertain else None,
                    "recall": tp / positives if positives else None,
                    "est_events": sum(r["weight"] for r in flagged),
                    "est_true": tp,
                }
        for cls in classes:
            out[f"population/{cls}"] = sum(r["weight"] for r in sample if r["label"] == cls)
        return out

    point = estimate(rows)
    replays = sorted({r["replay_id"] for r in rows})
    by_replay = collections.defaultdict(list)
    for r in rows:
        by_replay[r["replay_id"]].append(r)
    rng = np.random.default_rng(seed)
    boots = collections.defaultdict(list)
    for _ in range(n_boot):
        sample = [
            r for k in rng.integers(0, len(replays), len(replays)) for r in by_replay[replays[k]]
        ]
        for key, value in estimate(sample).items():
            if isinstance(value, dict):
                for metric in ("precision", "strict_precision", "recall"):
                    if value[metric] is not None:
                        boots[(key, metric)].append(value[metric])
    for (key, metric), values in boots.items():
        point[key][f"{metric}_ci"] = [
            float(np.percentile(values, 2.5)),
            float(np.percentile(values, 97.5)),
        ]
    labels = collections.Counter((r["stratum"], r["new"], r["legacy"], r["label"]) for r in rows)
    point["labeled_windows"] = len(rows)
    point["anchor_labels"] = {"|".join(map(str, k)): v for k, v in sorted(labels.items())}
    return point
