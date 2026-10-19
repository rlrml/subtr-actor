#!/usr/bin/env python3
"""Whiff-learning workflow: mine windows, render evidence, label, adjudicate.

All bulk artifacts live under `--cache` (default `~/.cache/subtr-actor-whiff`):

    dumps/                      whiff_dataset_dump output (one file per replay)
    work/opportunities.jsonl    mined proposal, random, and tiled windows
    work/manifests/<name>.jsonl frozen review manifests (refuse to overwrite)
    renders/<neutral_id>/       rendered evidence and render.json
    labels/attempts.jsonl       append-only labeling attempts
"""

from __future__ import annotations

import argparse
import collections
import concurrent.futures
import hashlib
import json
import pathlib
import random
import sys

import numpy as np

import adjudicate
import core_model
import fresh_eval
import labeling
import mining
import modeling
import render
import report
from telemetry import dump_path, load_dump

DEFAULT_CACHE = pathlib.Path.home() / ".cache/subtr-actor-whiff"

PILOT_ALLOCATION = {
    "detector_whiff": 25,
    "near_miss_no_touch": 25,
    "other_touch_nearby": 25,
    "dodge_near_ball": 15,
    "focal_touch": 15,
    "wide_approach": 15,
    "random_live": 30,
}


def read_jsonl(path: pathlib.Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def write_jsonl(path: pathlib.Path, rows: list[dict]):
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(".part")
    with tmp.open("w") as handle:
        for row in rows:
            handle.write(json.dumps(row) + "\n")
    tmp.rename(path)


def sha256_file(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _mine_one(args):
    path, random_per_player, tile_seconds, seed = args
    r = load_dump(path)
    split = mining.replay_split(r.content_id)
    windows = mining.proposals_for_replay(r)
    windows += mining.random_windows(r, random_per_player, seed)
    windows += mining.tiled_windows(r, tile_seconds, seed)
    for window in windows:
        window["split"] = split
        window["stratum"] = mining.stratum(window)
    return r.replay_id, r.content_id, windows


def cmd_mine(args):
    cache = args.cache
    manifest = read_jsonl(args.replay_manifest)
    seen_content = {}
    jobs = []
    for row in manifest:
        path = dump_path(cache / "dumps", row["ballchasing_id"])
        if path.exists():
            jobs.append((path, args.random_per_player, args.tile_seconds, args.seed))
    rows = []
    duplicates = []
    with concurrent.futures.ProcessPoolExecutor(max_workers=args.workers) as pool:
        for replay_id, content_id, windows in pool.map(_mine_one, jobs):
            if content_id in seen_content:
                duplicates.append((replay_id, seen_content[content_id]))
                continue
            seen_content[content_id] = replay_id
            rows.extend(windows)
    out = cache / "work/opportunities.jsonl"
    write_jsonl(out, rows)
    counts = collections.Counter((w["split"], w["stratum"]) for w in rows)
    print(
        json.dumps({"windows": len(rows), "replays": len(seen_content), "duplicates": duplicates})
    )
    for key in sorted(counts):
        print(key, counts[key])


def neutral_id(window_id: str, salt: str) -> str:
    return "w" + hashlib.sha256(f"{salt}:{window_id}".encode()).hexdigest()[:10]


def cmd_freeze(args):
    """Freeze a stratified review manifest from mined opportunities."""
    out = args.cache / "work/manifests" / f"{args.name}.jsonl"
    if out.exists():
        sys.exit(f"{out} exists; manifests are immutable")
    rows = read_jsonl(args.cache / "work/opportunities.jsonl")
    splits = set(args.splits.split(","))
    rng = random.Random(f"{args.seed}:{args.name}")
    allocation = json.loads(args.allocation) if args.allocation else PILOT_ALLOCATION
    used = set()
    for path in args.exclude_manifest or []:
        used |= {row["window_id"] for row in read_jsonl(path)}
    picked = []
    for stratum_name, quota in allocation.items():
        if stratum_name == "tiled_segment":
            segments = sorted(
                {
                    w["segment_id"]
                    for w in rows
                    if w["stratum"] == "other"
                    and "tiled_segment" in w["sources"]
                    and w["split"] in splits
                }
            )
            rng.shuffle(segments)
            chosen = set(segments[:quota])
            members = [w for w in rows if w.get("segment_id") in chosen]
            for w in members:
                picked.append(
                    {
                        **w,
                        "manifest_stratum": "tiled_segment",
                        "sampling_probability": quota / max(len(segments), 1),
                    }
                )
            continue
        pool = [
            w
            for w in rows
            if w["stratum"] == stratum_name
            and w["split"] in splits
            and w["window_id"] not in used
            and w["live_fraction"] >= 0.9
        ]
        by_replay = collections.defaultdict(list)
        for w in pool:
            by_replay[w["replay_id"]].append(w)
        replays = sorted(by_replay)
        rng.shuffle(replays)
        for replay in replays:
            rng.shuffle(by_replay[replay])
        chosen = []
        while len(chosen) < quota and any(by_replay[r] for r in replays):
            for replay in replays:
                if by_replay[replay] and len(chosen) < quota:
                    chosen.append(by_replay[replay].pop())
        for w in chosen:
            used.add(w["window_id"])
            picked.append(
                {
                    **w,
                    "manifest_stratum": stratum_name,
                    "sampling_probability": len(chosen) / max(len(pool), 1),
                }
            )
    salt = f"{args.name}:{args.seed}"
    for w in picked:
        w["neutral_id"] = neutral_id(w["window_id"], salt)
        w["manifest"] = args.name
    rng.shuffle(picked)
    write_jsonl(out, picked)
    meta = {
        "name": args.name,
        "sha256": sha256_file(out),
        "seed": args.seed,
        "splits": sorted(splits),
        "allocation": allocation,
        "windows": len(picked),
        "replays": len({w["replay_id"] for w in picked}),
        "opportunities_sha256": sha256_file(args.cache / "work/opportunities.jsonl"),
    }
    out.with_suffix(".meta.json").write_text(json.dumps(meta, indent=2))
    print(json.dumps(meta))


def _render_group(args):
    cache, replay_id, windows, force = args
    r = load_dump(dump_path(cache / "dumps", replay_id))
    done = 0
    for w in windows:
        target = cache / "renders" / w["neutral_id"]
        if (target / "render.json").exists() and not force:
            continue
        render.render_window(r, w, target)
        done += 1
    return done


def cmd_render(args):
    windows = read_jsonl(args.manifest)
    groups = collections.defaultdict(list)
    for w in windows:
        groups[w["replay_id"]].append(w)
    jobs = [(args.cache, replay, ws, args.force) for replay, ws in groups.items()]
    with concurrent.futures.ProcessPoolExecutor(max_workers=args.workers) as pool:
        total = sum(pool.map(_render_group, jobs))
    print(f"rendered {total} windows")


def label_tasks(
    cache: pathlib.Path, windows: list[dict], representations: list[str], pass_name: str
) -> list[dict]:
    tasks = []
    for w in windows:
        render_dir = cache / "renders" / w["neutral_id"]
        meta = json.loads((render_dir / "render.json").read_text())
        for representation in representations:
            names = (
                meta["images"]["trajectory"] + ["telemetry.jpg"]
                if representation == "compact"
                else meta["images"][representation]
            )
            paths = [render_dir / name for name in names]
            tasks.append(
                {
                    "neutral_id": w["neutral_id"],
                    "pass": pass_name,
                    "representation": representation,
                    "image_paths": [str(p) for p in paths],
                    "image_sha256": [meta["image_sha256"][name] for name in names],
                    "renderer_version": meta["renderer_version"],
                    "core_interval": meta["core_interval"],
                    "prompt": labeling.build_prompt(representation, paths, meta["core_interval"]),
                }
            )
    return tasks


def cmd_label(args):
    windows = read_jsonl(args.manifest)
    if args.limit:
        windows = windows[: args.limit]
    tasks = label_tasks(args.cache, windows, args.representations.split(","), args.pass_name)
    random.Random(args.pass_name).shuffle(tasks)
    transport = labeling.make_transport(args.transport)
    ledger = labeling.Ledger(args.cache / "labels/attempts.jsonl")
    stats = labeling.run_tasks(
        tasks,
        transport,
        ledger,
        args.cache / "labels/work",
        args.max_calls,
        args.max_cost,
        args.workers,
    )
    print(json.dumps(stats))


def latest_valid(records: list[dict], pass_name: str, representation: str, transport: str) -> dict:
    out = {}
    for r in records:
        if (
            r["valid"]
            and r["pass"] == pass_name
            and r["representation"] == representation
            and r["transport"]["transport"] == transport
        ):
            out[r["neutral_id"]] = r
    return out


FIRST_JUDGES = {
    "combined": (("primary", "combined", "claude-cli"), ("primary", "combined", "codex-cli")),
    "compact": (("primary", "compact", "claude-cli"), ("primary", "combined", "codex-cli")),
}
REVIEW_TRANSPORTS = ("claude:claude-opus-5-5", "codex")


def _disputes(r, window, first, second):
    a = first["judgment"]["interactions"]
    b = second["judgment"]["interactions"]
    out = []
    for x, y in adjudicate.match_interactions(a, b):
        t = (x or y)["closest_approach_t"]
        la, lb = adjudicate.label_of(x), adjudicate.label_of(y)
        flags = adjudicate.telemetry_flags(r, window, la, t) if la == lb else []
        out.append(
            {
                "t": t,
                "first": x,
                "second": y,
                "labels": [la, lb],
                "flags": flags,
                "needs_review": la != lb or bool(flags),
            }
        )
    return out


def _review_task(cache, window, meta, dispute, round_index):
    render_dir = cache / "renders" / window["neutral_id"]
    r = load_dump(dump_path(cache / "dumps", window["replay_id"]))
    closeup = render.render_closeup(r, window, dispute["t"], render_dir)
    names = meta["images"]["combined"] + closeup["images"]
    hashes = {**meta["image_sha256"], **closeup["image_sha256"]}
    first, second = dispute["first"], dispute["second"]
    if mining.stable_unit(window["neutral_id"], dispute["t"]) < 0.5:
        first, second = second, first
    extra = adjudicate.review_context(dispute["t"], first, second)
    paths = [render_dir / n for n in names]
    return {
        "neutral_id": window["neutral_id"],
        "pass": f"review{round_index}@{dispute['t']:.2f}",
        "representation": "combined+closeup",
        "image_paths": [str(p) for p in paths],
        "image_sha256": [hashes[n] for n in names],
        "renderer_version": meta["renderer_version"],
        "core_interval": meta["core_interval"],
        "prompt": labeling.build_prompt("combined", paths, meta["core_interval"], extra),
    }


def _review_label(record, t):
    if record is None:
        return None
    items = record["judgment"]["interactions"]
    best = min(items, key=lambda i: abs(i["closest_approach_t"] - t), default=None)
    if best is None or abs(best["closest_approach_t"] - t) > adjudicate.MATCH_TOLERANCE_SECONDS:
        return adjudicate.ABSENT
    return best["label"]


def cmd_adjudicate(args):
    windows = read_jsonl(args.manifest)
    ledger = labeling.Ledger(args.cache / "labels/attempts.jsonl")
    for round_index, spec in enumerate(REVIEW_TRANSPORTS, start=1):
        records = ledger.records()
        firsts = [latest_valid(records, *judge) for judge in FIRST_JUDGES[args.judges]]
        reviews = {}
        for prior in range(1, round_index):
            for rec in records:
                if rec["valid"] and rec["pass"].startswith(f"review{prior}@"):
                    reviews[(rec["neutral_id"], rec["pass"].split("@")[1], prior)] = rec
        tasks = []
        for w in windows:
            if w["neutral_id"] not in firsts[0] or w["neutral_id"] not in firsts[1]:
                continue
            r = load_dump(dump_path(args.cache / "dumps", w["replay_id"]))
            meta = json.loads(
                (args.cache / "renders" / w["neutral_id"] / "render.json").read_text()
            )
            for d in _disputes(r, w, firsts[0][w["neutral_id"]], firsts[1][w["neutral_id"]]):
                if not d["needs_review"]:
                    continue
                prior_labels = [
                    _review_label(reviews.get((w["neutral_id"], f"{d['t']:.2f}", k)), d["t"])
                    for k in range(1, round_index)
                ]
                label, _tier = adjudicate.resolve(*d["labels"], [p for p in prior_labels if p])
                if round_index > 1 and label != "uncertain":
                    continue
                tasks.append(_review_task(args.cache, w, meta, d, round_index))
        transport = labeling.make_transport(spec)
        stats = labeling.run_tasks(
            tasks,
            transport,
            ledger,
            args.cache / "labels/work",
            args.max_calls,
            args.max_cost,
            args.workers,
        )
        print(f"review round {round_index} ({spec}): {len(tasks)} disputes, {json.dumps(stats)}")
    write_final_labels(
        args.cache, args.manifest, windows, ledger.records(), FIRST_JUDGES[args.judges]
    )


def write_final_labels(cache, manifest_path, windows, records, judges):
    firsts = [latest_valid(records, *judge) for judge in judges]
    reviews = {}
    for rec in records:
        if rec["valid"] and rec["pass"].startswith("review"):
            round_index = int(rec["pass"][len("review") :].split("@")[0])
            reviews[(rec["neutral_id"], rec["pass"].split("@")[1], round_index)] = rec
    out = []
    for w in windows:
        nid = w["neutral_id"]
        if nid not in firsts[0] or nid not in firsts[1]:
            continue
        a, b = firsts[0][nid], firsts[1][nid]
        usable = a["judgment"]["window_usable"] or b["judgment"]["window_usable"]
        r = load_dump(dump_path(cache / "dumps", w["replay_id"]))
        finals = []
        for d in _disputes(r, w, a, b):
            key = f"{d['t']:.2f}"
            review_records = [reviews.get((nid, key, k)) for k in (1, 2)]
            review_labels = [_review_label(rec, d["t"]) for rec in review_records if rec]
            if d["needs_review"] and not review_labels:
                label, tier = "uncertain", "pending_review"
            elif d["labels"][0] == d["labels"][1] and d["flags"]:
                label, tier = adjudicate.resolve(
                    d["labels"][0], d["labels"][0] + "?", review_labels
                )
                if tier == "adjudicated":
                    tier = "agreed_confirmed"
                else:
                    label, tier = "uncertain", "telemetry_contradiction"
            else:
                label, tier = adjudicate.resolve(*d["labels"], review_labels)
            if label == adjudicate.ABSENT and all(x is None for x in (d["first"], d["second"])):
                continue
            finals.append(
                {
                    "t_window": d["t"],
                    "t_replay": w["window_start_time"] + d["t"],
                    "label": label,
                    "tier": tier,
                    "first_pass_labels": d["labels"],
                    "review_labels": review_labels,
                    "telemetry_flags": d["flags"],
                    "attempt_ids": [a["attempt_id"], b["attempt_id"]]
                    + [rec["attempt_id"] for rec in review_records if rec],
                }
            )
        out.append(
            {
                "neutral_id": nid,
                "window_id": w["window_id"],
                "replay_id": w["replay_id"],
                "split": w["split"],
                "player_id": w["player_id"],
                "player_index": w["player_index"],
                "manifest_stratum": w["manifest_stratum"],
                "window_start_time": w["window_start_time"],
                "core_start_time": w["core_start_time"],
                "core_end_time": w["core_end_time"],
                "usable": usable,
                "interactions": finals,
                "rubric_version": labeling.PROMPT_VERSION,
                "first_judges": [list(judge) for judge in judges],
            }
        )
    path = cache / "labels" / f"final_{pathlib.Path(manifest_path).stem}.jsonl"
    write_jsonl(path, out)
    counts = collections.Counter(
        (i["label"], i["tier"]) for row in out for i in row["interactions"]
    )
    print(f"wrote {path}")
    for key in sorted(counts):
        print(key, counts[key])


def system_predictions(system, windows, replays, model=None, thresholds=None, rule=None):
    """Per-window predicted events (neutral_id, t, label) for one system."""
    preds = []
    for w in windows:
        if not w["usable"]:
            continue
        if system == "detector":
            r = replays.replay(w["replay_id"])
            for e in r.whiff_events:
                if (
                    e["player_id"] == w["player_id"]
                    and w["core_start_time"] <= e["time"] <= w["core_end_time"]
                ):
                    preds.append(
                        {"neutral_id": w["neutral_id"], "t": e["time"], "label": "clear_whiff"}
                    )
            for e in r.beaten_to_ball_events:
                if (
                    e["player_id"] == w["player_id"]
                    and w["core_start_time"] <= e["time"] <= w["core_end_time"]
                ):
                    preds.append(
                        {
                            "neutral_id": w["neutral_id"],
                            "t": e["time"],
                            "label": "disrupted_attempt",
                        }
                    )
            continue
        cands = [
            c
            for c in replays.replay_candidates(w["replay_id"])
            if c["player_id"] == w["player_id"]
            and w["core_start_time"] <= c["anchor_time"] <= w["core_end_time"]
        ]
        if not cands:
            continue
        if system == "rule":
            labels = [modeling.geometric_rule(c["features"], rule) for c in cands]
        else:
            x = np.nan_to_num(np.array([c["features"] for c in cands]), nan=-999.0)
            labels = modeling.predict_classes(model, x, thresholds)
        for c, label in zip(cands, labels):
            if label in modeling.EVENT_CLASSES:
                preds.append({"neutral_id": w["neutral_id"], "t": c["anchor_time"], "label": label})
    return preds


def label_events(windows):
    return [
        {"neutral_id": w["neutral_id"], "t": i["t_replay"], "label": i["label"]}
        for w in windows
        if w["usable"]
        for i in w["interactions"]
        if i["label"] in modeling.EVENT_CLASSES + ("uncertain",)
    ]


def macro_event_f1(metrics):
    scores = []
    for cls in modeling.EVENT_CLASSES:
        m = metrics[cls]
        p, r = m["precision"] or 0.0, m["recall"] or 0.0
        scores.append(2 * p * r / (p + r) if p + r else 0.0)
    return float(np.mean(scores))


RULE_GRID = [
    {
        "touch_window": tw,
        "min_closing": mc,
        "min_alignment": ma,
        "max_gap": mg,
        "disrupt_window": dw,
    }
    for tw in (0.15, 0.3)
    for mc in (0.0, 300.0)
    for ma in (0.3, 0.6)
    for mg in (150.0, 300.0)
    for dw in (0.4, 0.8)
]
GB_GRID = [
    {
        "n_estimators": n,
        "max_depth": d,
        "learning_rate": lr,
        "subsample": 0.8,
        "min_samples_leaf": leaf,
    }
    for n in (100, 250)
    for d in (2, 3)
    for lr in (0.05, 0.1)
    for leaf in (3, 8)
]
THRESHOLD_GRID = (0.0, 0.35, 0.45, 0.55, 0.65)


def evaluate_system(name, windows, replays, sets, **kwargs):
    out = {}
    for set_name, set_windows in sets.items():
        preds = system_predictions(name, set_windows, replays, **kwargs)
        metrics = modeling.event_metrics(preds, label_events(set_windows))
        window_replay = {w["neutral_id"]: w["replay_id"] for w in set_windows}
        player_minutes = (
            sum((w["core_end_time"] - w["core_start_time"]) for w in set_windows if w["usable"])
            / 60
        )
        summary = modeling.strip_per_window(metrics)
        for cls in modeling.EVENT_CLASSES:
            summary[cls]["precision_ci"] = modeling.bootstrap_ci(
                metrics[cls]["per_window"], window_replay, "precision"
            )
            summary[cls]["recall_ci"] = modeling.bootstrap_ci(
                metrics[cls]["per_window"], window_replay, "recall"
            )
            summary[cls]["fp_per_player_minute"] = (
                summary[cls]["fp"] / player_minutes if player_minutes else None
            )
            labeled = summary[cls]["labeled"]
            summary[cls]["count_ratio"] = summary[cls]["predicted"] / labeled if labeled else None
        summary["macro_event_f1"] = macro_event_f1(metrics)
        summary["player_minutes"] = player_minutes
        out[set_name] = summary
    return out


def cmd_train(args):
    windows = []
    for path in args.labels:
        windows += read_jsonl(path)
    replays = modeling.ReplayCache(args.cache)
    by_set = collections.defaultdict(list)
    for w in windows:
        by_set[w.get("eval_set") or w["split"]].append(w)
    test_sets = {}
    for path in args.test_labels:
        rows = read_jsonl(path)
        test_sets[pathlib.Path(path).stem.removeprefix("final_")] = rows
    examples, interactions = modeling.build_examples(windows, replays)
    train_ex = [e for e in examples if e["split"] == "train"]
    dev_windows = [w for w in windows if w["split"] == "dev"]
    _, x_train, y_train = modeling.to_xy(train_ex)
    print(
        "train class counts", dict(collections.Counter(modeling.MODEL_CLASSES[k] for k in y_train))
    )
    print(
        "dev windows",
        len(dev_windows),
        "excluded uncertain examples",
        sum(e["label"] == "uncertain" for e in examples),
    )

    best = None
    for params in GB_GRID:
        model = modeling.train_boosted(x_train, y_train, params)
        for whiff_t in THRESHOLD_GRID:
            for disrupted_t in THRESHOLD_GRID:
                thresholds = {"clear_whiff": whiff_t, "disrupted_attempt": disrupted_t}
                preds = system_predictions(
                    "model", dev_windows, replays, model=model, thresholds=thresholds
                )
                score = macro_event_f1(modeling.event_metrics(preds, label_events(dev_windows)))
                if best is None or score > best[0]:
                    best = (score, params, thresholds, model)
    dev_score, params, thresholds, model = best
    print("selected", params, thresholds, "dev macro F1", round(dev_score, 3))

    best_rule = max(
        RULE_GRID,
        key=lambda rule: macro_event_f1(
            modeling.event_metrics(
                system_predictions("rule", dev_windows, replays, rule=rule),
                label_events(dev_windows),
            )
        ),
    )

    curve = []
    rng = np.random.default_rng(7)
    train_replays = sorted({e["replay_id"] for e in train_ex})
    for fraction in (0.25, 0.5, 0.75, 1.0):
        chosen = set(rng.permutation(train_replays)[: max(1, int(len(train_replays) * fraction))])
        _, xs, ys = modeling.to_xy([e for e in train_ex if e["replay_id"] in chosen])
        m = modeling.train_boosted(xs, ys, params)
        preds = system_predictions("model", dev_windows, replays, model=m, thresholds=thresholds)
        curve.append(
            {
                "fraction": fraction,
                "examples": len(ys),
                "dev_macro_event_f1": macro_event_f1(
                    modeling.event_metrics(preds, label_events(dev_windows))
                ),
            }
        )

    eval_sets = {"dev": dev_windows, **test_sets}
    results = {
        "model": evaluate_system(
            "model", windows, replays, eval_sets, model=model, thresholds=thresholds
        ),
        "rule": evaluate_system("rule", windows, replays, eval_sets, rule=best_rule),
        "detector": evaluate_system("detector", windows, replays, eval_sets),
    }
    _, test_interactions = modeling.build_examples(
        [w for rows in test_sets.values() for w in rows], replays
    )
    recall = collections.defaultdict(lambda: [0, 0])
    for item in test_interactions + [i for i in interactions if i["split"] == "dev"]:
        if item["label"] in modeling.EVENT_CLASSES:
            recall[item["label"]][0] += item["has_candidate"]
            recall[item["label"]][1] += 1
    importances = sorted(zip(feat_names(), model.feature_importances_), key=lambda kv: -kv[1])[:15]
    out_dir = args.cache / "models" / args.name
    out_dir.mkdir(parents=True, exist_ok=True)
    report_doc = {
        "name": args.name,
        "label_files": [str(p) for p in args.labels],
        "test_label_files": [str(p) for p in args.test_labels],
        "label_file_sha256": {
            str(p): sha256_file(pathlib.Path(p)) for p in list(args.labels) + list(args.test_labels)
        },
        "selected_params": params,
        "thresholds": thresholds,
        "rule_params": best_rule,
        "dev_macro_event_f1": dev_score,
        "learning_curve": curve,
        "candidate_recall": {
            k: {"covered": v[0], "total": v[1], "recall": v[0] / v[1] if v[1] else None}
            for k, v in recall.items()
        },
        "train_class_counts": dict(collections.Counter(modeling.MODEL_CLASSES[k] for k in y_train)),
        "feature_importances": importances,
        "results": results,
    }
    (out_dir / "report.json").write_text(json.dumps(report_doc, indent=2))
    modeling.export_model(
        model,
        out_dir / "model.json",
        {"name": args.name, "thresholds": thresholds},
        check_x=x_train,
    )
    print(json.dumps({k: report_doc[k] for k in ("candidate_recall", "learning_curve")}, indent=1))
    for system, by in results.items():
        for set_name, summary in by.items():
            row = {
                cls: {
                    k: summary[cls][k]
                    for k in ("tp", "fp", "fn", "precision", "recall", "fp_per_player_minute")
                }
                for cls in modeling.EVENT_CLASSES
            }
            print(system, set_name, "macroF1", round(summary["macro_event_f1"], 3), json.dumps(row))


def cmd_select(args):
    """Active-learning batch: windows around unlabeled train-split candidates."""
    out = args.cache / "work/manifests" / f"{args.name}.jsonl"
    if out.exists():
        sys.exit(f"{out} exists; manifests are immutable")
    report_doc = json.loads((args.cache / "models" / args.model / "report.json").read_text())
    windows = []
    for path in report_doc["label_files"]:
        windows += read_jsonl(pathlib.Path(path))
    replays = modeling.ReplayCache(args.cache)
    examples, _ = modeling.build_examples(windows, replays)
    _, x, y = modeling.to_xy([e for e in examples if e["split"] == "train"])
    model = modeling.train_boosted(x, y, report_doc["selected_params"])
    labeled = collections.defaultdict(list)
    for w in windows:
        labeled[(w["replay_id"], w["player_id"])].append(
            (w["core_start_time"] - 1.0, w["core_end_time"] + 1.0)
        )
    opportunities = read_jsonl(args.cache / "work/opportunities.jsonl")
    train_replays = sorted({w["replay_id"] for w in opportunities if w["split"] == "train"})
    pool = []
    for replay_id in train_replays:
        r = replays.replay(replay_id)
        cands = replays.replay_candidates(replay_id)
        if not cands:
            continue
        proba = np.zeros((len(cands), len(modeling.MODEL_CLASSES)))
        xs = np.nan_to_num(np.array([c["features"] for c in cands]), nan=-999.0)
        proba[:, model.classes_] = model.predict_proba(xs)
        detector_times = collections.defaultdict(list)
        for e in r.whiff_events:
            detector_times[e["player_id"]].append(e["time"])
        for c, p in zip(cands, proba):
            if any(lo <= c["anchor_time"] <= hi for lo, hi in labeled[(replay_id, c["player_id"])]):
                continue
            if not r.live[c["anchor_index"]]:
                continue
            miss_p = p[0] + p[1]
            top = np.sort(p)[::-1]
            detector = any(abs(t - c["anchor_time"]) < 0.4 for t in detector_times[c["player_id"]])
            pool.append(
                {
                    "c": c,
                    "miss_p": float(miss_p),
                    "margin": float(top[0] - top[1]),
                    "disagree": detector != (p[0] == p.max()),
                }
            )
    rng = random.Random(f"select:{args.name}")
    chosen, used = [], set()

    def take(items, n, reason):
        for item in items:
            if len([c for c in chosen if c[1] == reason]) >= n:
                break
            key = (
                item["c"]["replay_id"],
                item["c"]["player_id"],
                round(item["c"]["anchor_time"] / 2),
            )
            if key in used:
                continue
            used.add(key)
            chosen.append((item, reason))

    take(sorted(pool, key=lambda i: -i["miss_p"]), args.count * 35 // 100, "likely_miss")
    take(sorted(pool, key=lambda i: i["margin"]), args.count * 25 // 100, "uncertain")
    disagreements = [i for i in pool if i["disagree"]]
    rng.shuffle(disagreements)
    take(disagreements, args.count * 20 // 100, "detector_disagreement")
    rng.shuffle(pool)
    take(pool, args.count - len(chosen), "random_candidate")
    rows = []
    salt = f"{args.name}:select"
    for item, reason in chosen:
        c = item["c"]
        r = replays.replay(c["replay_id"])
        w = mining.make_window(r, c["player_index"], c["anchor_index"], ["active_learning"])
        w.update(
            {
                "split": "train",
                "stratum": "active_learning",
                "manifest_stratum": f"al_{reason}",
                "sampling_probability": None,
                "manifest": args.name,
                "selection_model": args.model,
            }
        )
        w["neutral_id"] = neutral_id(w["window_id"], salt)
        rows.append(w)
    rng.shuffle(rows)
    write_jsonl(out, rows)
    print(
        json.dumps(
            {
                "windows": len(rows),
                "reasons": dict(collections.Counter(r["manifest_stratum"] for r in rows)),
                "replays": len({r["replay_id"] for r in rows}),
                "sha256": sha256_file(out),
            }
        )
    )


def cmd_parity(args):
    """Compare Rust shadow output with Python features and exported-model probabilities."""
    payload = json.loads(args.model.read_text())
    replays = modeling.ReplayCache(args.cache)
    shadow = collections.defaultdict(dict)
    for row in read_jsonl(args.shadow):
        shadow[row["replay_id"]][(row["player_id"], row["anchor_frame"])] = row
    report_rows = []
    for replay_id, rust in shadow.items():
        python = {
            (c["player_id"], c["anchor_frame"]): c for c in replays.replay_candidates(replay_id)
        }
        shared = sorted(set(python) & set(rust))
        x_py = np.array([python[k]["features"] for k in shared])
        x_rs = np.array([[np.nan if v is None else v for v in rust[k]["features"]] for k in shared])
        both_nan = np.isnan(x_py) & np.isnan(x_rs)
        diff = np.where(both_nan, 0.0, np.abs(x_py - x_rs))
        p_py = modeling.exported_proba(payload, x_py)
        p_rs = np.array([rust[k]["proba"] for k in shared])
        classes_rs = [rust[k]["class"] for k in shared]
        report_rows.append(
            {
                "replay_id": replay_id,
                "python_candidates": len(python),
                "rust_candidates": len(rust),
                "shared": len(shared),
                "max_feature_abs_diff": float(np.nanmax(diff)) if diff.size else 0.0,
                "features_with_diff_gt_1e-6": [
                    payload["feature_names"][j] for j in np.flatnonzero((diff > 1e-6).any(axis=0))
                ],
                "max_proba_abs_diff": float(np.abs(p_py - p_rs).max()) if len(shared) else 0.0,
                "class_counts_rust": dict(collections.Counter(classes_rs)),
            }
        )
    print(json.dumps(report_rows, indent=1))


def cmd_parity_fixture(args):
    """Python reference features/probabilities for a Rust parity test."""
    payload = json.loads(args.model.read_text())
    candidates = modeling.ReplayCache(args.cache).replay_candidates(args.replay_id)
    x = np.array([c["features"] for c in candidates])
    proba = modeling.exported_proba(payload, x)
    rows = [
        {
            "player_id": c["player_id"],
            "anchor_frame": c["anchor_frame"],
            "features": [None if np.isnan(v) else float(v) for v in c["features"]],
            "proba": p.tolist(),
        }
        for c, p in zip(candidates, proba)
    ]
    doc = {"model": payload["name"], "replay_id": args.replay_id, "candidates": rows}
    args.out.write_text(json.dumps(doc) + "\n")
    print(f"wrote {len(rows)} candidates to {args.out}")


def cmd_train_core(args):
    """Select, evaluate, and export the deployed classifier from core candidate dumps."""
    replays = core_model.CoreReplayCache(args.cache, args.candidates)
    tune = []
    seen = set()
    for path in args.labels:
        for w in read_jsonl(path):
            if w["neutral_id"] not in seen:
                seen.add(w["neutral_id"])
                tune.append(w)
    tests = {pathlib.Path(p).stem.removeprefix("final_"): read_jsonl(p) for p in args.test_labels}
    examples, interactions = modeling.build_examples(tune, replays)
    feature_names = replays.feature_names(tune[0]["replay_id"])
    print("tune examples", dict(collections.Counter(e["label"] for e in examples)))
    selection = core_model.select(examples, label_events(tune))
    best = selection["best"]
    print(
        "selected", best["params"], best["thresholds"], "cv macro F1", round(best["cv_macro_f1"], 3)
    )
    model = core_model.fit(examples, best["params"])
    results = {}
    for name, windows in tests.items():
        labels = label_events(windows)
        window_replay = {w["neutral_id"]: w["replay_id"] for w in windows}
        systems = {
            "model": core_model.predict_windows(model, best["thresholds"], windows, replays),
            "legacy_detector": system_predictions(
                "detector", windows, modeling.ReplayCache(args.cache)
            ),
        }
        results[name] = {}
        for system, preds in systems.items():
            metrics = modeling.event_metrics(preds, labels)
            summary = modeling.strip_per_window(metrics)
            for cls in modeling.EVENT_CLASSES:
                summary[cls]["precision_ci"] = modeling.bootstrap_ci(
                    metrics[cls]["per_window"], window_replay, "precision"
                )
                summary[cls]["recall_ci"] = modeling.bootstrap_ci(
                    metrics[cls]["per_window"], window_replay, "recall"
                )
            results[name][system] = summary
            print(
                name,
                system,
                {
                    cls: [summary[cls][k] for k in ("tp", "fp", "fn", "precision", "recall")]
                    for cls in modeling.EVENT_CLASSES
                },
            )
    fit_examples = examples
    if args.refit_all:
        test_examples, _ = modeling.build_examples(
            [w for rows in tests.values() for w in rows], replays
        )
        fit_examples = examples + test_examples
        model = core_model.fit(fit_examples, best["params"])
    out_dir = args.cache / "models" / args.name
    out_dir.mkdir(parents=True, exist_ok=True)
    report_doc = {
        "name": args.name,
        "candidates_dir": str(args.candidates),
        "label_files": [str(p) for p in args.labels],
        "test_label_files": [str(p) for p in args.test_labels],
        "label_file_sha256": {
            str(p): sha256_file(pathlib.Path(p)) for p in list(args.labels) + list(args.test_labels)
        },
        "feature_names": feature_names,
        "selection": selection,
        "test_results": results,
        "refit_all": args.refit_all,
        "fit_class_counts": dict(collections.Counter(e["label"] for e in fit_examples)),
        "feature_importances": sorted(
            zip(feature_names, model.feature_importances_.tolist()), key=lambda kv: -kv[1]
        )[:20],
    }
    (out_dir / "report.json").write_text(json.dumps(report_doc, indent=2))
    fit_counts = {
        k: v for k, v in report_doc["fit_class_counts"].items() if k in modeling.MODEL_CLASSES
    }
    provenance = (
        f"{args.name}: {best['params']['n_estimators']} stages, depth "
        f"{best['params']['max_depth']}; fit on {sum(fit_counts.values())} labeled candidates "
        f"({fit_counts.get('clear_whiff', 0)} clean whiffs, "
        f"{fit_counts.get('disrupted_attempt', 0)} disrupted); replay-grouped CV macro F1 "
        f"{best['cv_macro_f1']:.3f}"
    )
    metadata = {
        "name": args.name,
        "feature_version": replays.doc(tune[0]["replay_id"])["feature_version"],
        "thresholds": best["thresholds"],
        "provenance": provenance,
    }
    modeling.export_model(
        model,
        out_dir / "model.json",
        {**metadata, "feature_names": feature_names},
        check_x=None,
    )
    core_model.export_rust(model, args.rust_out, metadata, feature_names)
    core_model.export_rust_parity_fixture(
        model, fit_examples, args.rust_out.with_name("whiff_model_parity_fixture.rs")
    )
    print(f"wrote {out_dir}/report.json and {args.rust_out}")


def cmd_freeze_fresh(args):
    """Freeze a stratified before/after evaluation manifest on fresh replays."""
    out = args.cache / "work/manifests" / f"{args.name}.jsonl"
    if out.exists():
        sys.exit(f"{out} exists; manifests are immutable")
    replay_ids = [row["ballchasing_id"] for row in read_jsonl(args.replay_manifest)]
    quotas = json.loads(args.quotas)
    windows, population = fresh_eval.build_manifest(
        args.cache, args.candidates, replay_ids, quotas, args.name, args.seed
    )
    write_jsonl(out, windows)
    meta = {
        "name": args.name,
        "sha256": sha256_file(out),
        "seed": args.seed,
        "replays": replay_ids,
        "candidates_dir": str(args.candidates),
        "population": population,
        "quotas": quotas,
        "windows": len(windows),
    }
    out.with_suffix(".meta.json").write_text(json.dumps(meta, indent=2))
    print(json.dumps({k: meta[k] for k in ("population", "quotas", "windows")}))


def cmd_score_fresh(args):
    windows = read_jsonl(args.manifest)
    finals = {
        row["neutral_id"]: row
        for row in read_jsonl(args.cache / "labels" / f"final_{args.manifest.stem}.jsonl")
    }
    result = fresh_eval.score(windows, finals)
    out = args.cache / "reports" / f"fresh_{args.manifest.stem}.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(result, indent=2))
    print(json.dumps(result, indent=1))


def feat_names():
    import features

    return features.FEATURE_NAMES


def cmd_report(args):
    result = report.pilot_report(args.cache, args.manifest)
    out = args.cache / "reports" / f"pilot_{args.manifest.stem}.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(result, indent=2))
    print(json.dumps(result, indent=2))


def cmd_status(args):
    records = labeling.Ledger(args.cache / "labels/attempts.jsonl").records()
    table = collections.Counter()
    latency = collections.defaultdict(list)
    cost = collections.defaultdict(float)
    for r in records:
        key = (r["pass"].split("@")[0], r["representation"], r["transport"]["transport"])
        table[key + ("valid" if r["valid"] else "invalid",)] += 1
        if r.get("latency_s"):
            latency[key].append(r["latency_s"])
        cost[key] += r.get("cost_usd") or 0.0
    for key in sorted({k[:3] for k in table}):
        lat = sorted(latency[key])
        median = lat[len(lat) // 2] if lat else None
        print(
            key,
            "valid",
            table[key + ("valid",)],
            "invalid",
            table[key + ("invalid",)],
            "median_latency",
            None if median is None else round(median, 1),
            "cost",
            round(cost[key], 3),
        )


def main(argv=None):
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--cache", type=pathlib.Path, default=DEFAULT_CACHE)
    sub = parser.add_subparsers(dest="command", required=True)

    p = sub.add_parser("mine")
    p.add_argument("--replay-manifest", type=pathlib.Path, default=DEFAULT_CACHE / "manifest.jsonl")
    p.add_argument("--random-per-player", type=int, default=3)
    p.add_argument("--tile-seconds", type=float, default=20.0)
    p.add_argument("--seed", default="whiff-mine-v1")
    p.add_argument("--workers", type=int, default=8)
    p.set_defaults(func=cmd_mine)

    p = sub.add_parser("freeze")
    p.add_argument("--name", required=True)
    p.add_argument("--splits", default="train,dev")
    p.add_argument("--allocation", help="JSON {stratum: count}; default is the pilot allocation")
    p.add_argument("--exclude-manifest", type=pathlib.Path, action="append")
    p.add_argument("--seed", default="whiff-freeze-v1")
    p.set_defaults(func=cmd_freeze)

    p = sub.add_parser("render")
    p.add_argument("--manifest", type=pathlib.Path, required=True)
    p.add_argument("--workers", type=int, default=8)
    p.add_argument("--force", action="store_true")
    p.set_defaults(func=cmd_render)

    p = sub.add_parser("label")
    p.add_argument("--manifest", type=pathlib.Path, required=True)
    p.add_argument("--transport", default="claude:claude-opus-5-5")
    p.add_argument("--representations", default="combined")
    p.add_argument("--pass-name", default="primary")
    p.add_argument("--limit", type=int)
    p.add_argument("--max-calls", type=int, default=50)
    p.add_argument("--max-cost", type=float, default=10.0)
    p.add_argument("--workers", type=int, default=4)
    p.set_defaults(func=cmd_label)

    p = sub.add_parser("adjudicate")
    p.add_argument("--manifest", type=pathlib.Path, required=True)
    p.add_argument("--judges", choices=sorted(FIRST_JUDGES), default="compact")
    p.add_argument("--max-calls", type=int, default=200)
    p.add_argument("--max-cost", type=float, default=30.0)
    p.add_argument("--workers", type=int, default=4)
    p.set_defaults(func=cmd_adjudicate)

    p = sub.add_parser("train")
    p.add_argument("--name", required=True)
    p.add_argument(
        "--labels", type=pathlib.Path, nargs="+", required=True, help="train/dev final label files"
    )
    p.add_argument("--test-labels", type=pathlib.Path, nargs="*", default=[])
    p.set_defaults(func=cmd_train)

    p = sub.add_parser("train-core")
    p.add_argument("--name", required=True)
    p.add_argument(
        "--candidates", type=pathlib.Path, required=True, help="whiff_candidate_dump dir"
    )
    p.add_argument("--labels", type=pathlib.Path, nargs="+", required=True)
    p.add_argument("--test-labels", type=pathlib.Path, nargs="*", default=[])
    p.add_argument("--refit-all", action="store_true", help="refit on tune + test labels")
    p.add_argument(
        "--rust-out",
        type=pathlib.Path,
        default=pathlib.Path(__file__).resolve().parents[2]
        / "src/stats/calculators/whiff_model_weights.rs",
    )
    p.set_defaults(func=cmd_train_core)

    p = sub.add_parser("freeze-fresh")
    p.add_argument("--name", required=True)
    p.add_argument("--replay-manifest", type=pathlib.Path, required=True)
    p.add_argument(
        "--candidates", type=pathlib.Path, required=True, help="whiff_candidate_dump dir"
    )
    p.add_argument("--quotas", required=True, help="JSON {stratum: windows}")
    p.add_argument("--seed", default="whiff-fresh-v1")
    p.set_defaults(func=cmd_freeze_fresh)

    p = sub.add_parser("score-fresh")
    p.add_argument("--manifest", type=pathlib.Path, required=True)
    p.set_defaults(func=cmd_score_fresh)

    p = sub.add_parser("select")
    p.add_argument("--name", required=True)
    p.add_argument(
        "--model", required=True, help="models/<name> whose report.json defines the labels"
    )
    p.add_argument("--count", type=int, default=400)
    p.set_defaults(func=cmd_select)

    p = sub.add_parser("parity")
    p.add_argument("--model", type=pathlib.Path, required=True)
    p.add_argument("--shadow", type=pathlib.Path, required=True)
    p.set_defaults(func=cmd_parity)

    p = sub.add_parser("parity-fixture")
    p.add_argument("--model", type=pathlib.Path, required=True)
    p.add_argument("--replay-id", required=True)
    p.add_argument("--out", type=pathlib.Path, required=True)
    p.set_defaults(func=cmd_parity_fixture)

    p = sub.add_parser("report")
    p.add_argument("--manifest", type=pathlib.Path, required=True)
    p.set_defaults(func=cmd_report)

    p = sub.add_parser("status")
    p.set_defaults(func=cmd_status)

    args = parser.parse_args(argv)
    args.func(args)


if __name__ == "__main__":
    main()
