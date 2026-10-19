"""Pilot diagnostics: representation comparison, judge agreement, label economics."""

from __future__ import annotations

import collections
import json
import pathlib
import statistics

import labeling
from adjudicate import ABSENT, match_interactions
from modeling import greedy_match

REPRESENTATION_ARMS = (
    ("replay", "claude-cli"),
    ("trajectory", "claude-cli"),
    ("compact", "claude-cli"),
    ("combined", "claude-cli"),
    ("combined", "codex-cli"),
)


def _latest(records, representation, transport, pass_name="primary"):
    out = {}
    for r in records:
        if (
            r["pass"] == pass_name
            and r["representation"] == representation
            and r["transport"]["transport"] == transport
        ):
            if r["valid"] or r["neutral_id"] not in out:
                out[r["neutral_id"]] = r
    return out


def cohen_kappa(a: list[str], b: list[str]) -> float | None:
    if not a:
        return None
    labels = sorted(set(a) | set(b))
    n = len(a)
    observed = sum(x == y for x, y in zip(a, b)) / n
    ca, cb = collections.Counter(a), collections.Counter(b)
    expected = sum(ca[label] * cb[label] for label in labels) / (n * n)
    return None if expected >= 1 else (observed - expected) / (1 - expected)


def _interaction_agreement(
    judgment: dict, final_row: dict
) -> tuple[int, int, list[tuple[str, str]]]:
    """Agreement with final (non-uncertain) reference interactions, absent = no attempt."""
    reference = [i for i in final_row["interactions"] if i["tier"] not in ("pending_review",)]
    items = judgment["interactions"]
    pairs = greedy_match(
        [i["closest_approach_t"] for i in items], [i["t_window"] for i in reference], 0.4
    )
    matched_items = {a for a, _ in pairs}
    matched_ref = {b for _, b in pairs}
    confusion = []
    for a, b in pairs:
        confusion.append((reference[b]["label"], items[a]["label"]))
    for b, ref in enumerate(reference):
        if b not in matched_ref:
            confusion.append((ref["label"], ABSENT))
    for a, item in enumerate(items):
        if a not in matched_items and item["label"] != ABSENT:
            confusion.append((ABSENT, item["label"]))
    scored = [(ref, got) for ref, got in confusion if ref != "uncertain"]
    agree = sum(ref == got for ref, got in scored)
    return agree, len(scored), confusion


def pilot_report(cache: pathlib.Path, manifest: pathlib.Path) -> dict:
    windows = [json.loads(line) for line in manifest.read_text().splitlines() if line.strip()]
    records = labeling.Ledger(cache / "labels/attempts.jsonl").records()
    final_path = cache / "labels" / f"final_{manifest.stem}.jsonl"
    finals = {}
    if final_path.exists():
        finals = {
            json.loads(line)["neutral_id"]: json.loads(line)
            for line in final_path.read_text().splitlines()
        }
    nids = {w["neutral_id"] for w in windows}
    report = {"windows": len(windows), "arms": {}}
    arm_records = {arm: _latest(records, *arm) for arm in REPRESENTATION_ARMS}
    reference_arm = arm_records[("combined", "codex-cli")]
    for arm, recs in arm_records.items():
        recs = {k: v for k, v in recs.items() if k in nids}
        valid = [r for r in recs.values() if r["valid"]]
        attempts = [
            r
            for r in records
            if r["neutral_id"] in nids
            and r["pass"] == "primary"
            and r["representation"] == arm[0]
            and r["transport"]["transport"] == arm[1]
        ]
        outcomes = collections.Counter(labeling.window_outcome(r["judgment"]) for r in valid)
        interaction_labels = collections.Counter(
            i["label"] for r in valid for i in r["judgment"]["interactions"]
        )
        agree = total = 0
        confusion = collections.Counter()
        for r in valid:
            if r["neutral_id"] in finals:
                a, t, conf = _interaction_agreement(r["judgment"], finals[r["neutral_id"]])
                agree, total = agree + a, total + t
                confusion.update(conf)
        shared = [
            nid
            for nid in recs
            if nid in reference_arm and recs[nid]["valid"] and reference_arm[nid]["valid"]
        ]
        kappa = None
        if arm != ("combined", "codex-cli"):
            kappa = cohen_kappa(
                [labeling.window_outcome(recs[n]["judgment"]) for n in shared],
                [labeling.window_outcome(reference_arm[n]["judgment"]) for n in shared],
            )
        latencies = [r["latency_s"] for r in attempts if r.get("latency_s")]
        cost = sum(r.get("cost_usd") or 0 for r in attempts)
        usable_labels = sum(
            1 for r in valid for i in r["judgment"]["interactions"] if i["label"] != "uncertain"
        ) + sum(1 for r in valid if not r["judgment"]["interactions"])
        report["arms"]["/".join(arm)] = {
            "attempts": len(attempts),
            "valid_windows": len(valid),
            "validity_rate": len(valid) / max(len(recs), 1),
            "consistent_rate": sum(r["consistent"] for r in valid) / max(len(valid), 1),
            "window_outcomes": dict(outcomes),
            "interaction_labels": dict(interaction_labels),
            "uncertain_share": interaction_labels["uncertain"]
            / max(sum(interaction_labels.values()), 1),
            "agreement_with_final": agree / total if total else None,
            "final_scored_interactions": total,
            "kappa_vs_codex_combined_window_outcome": kappa,
            "median_latency_s": statistics.median(latencies) if latencies else None,
            "reported_cost_usd": round(cost, 3),
            "cost_per_usable_label": round(cost / usable_labels, 4)
            if usable_labels and cost
            else None,
            "confusion_final_vs_arm": {f"{a}->{b}": n for (a, b), n in sorted(confusion.items())},
        }
    time_deltas = []
    first = arm_records[("combined", "claude-cli")]
    for nid in nids:
        if (
            nid in first
            and nid in reference_arm
            and first[nid]["valid"]
            and reference_arm[nid]["valid"]
        ):
            for x, y in match_interactions(
                first[nid]["judgment"]["interactions"],
                reference_arm[nid]["judgment"]["interactions"],
            ):
                if x and y:
                    time_deltas.append(abs(x["closest_approach_t"] - y["closest_approach_t"]))
    report["median_cross_judge_time_delta_s"] = (
        statistics.median(time_deltas) if time_deltas else None
    )
    if finals:
        by_stratum = collections.defaultdict(collections.Counter)
        tiers = collections.Counter()
        for w in windows:
            row = finals.get(w["neutral_id"])
            if not row:
                continue
            if not row["usable"]:
                by_stratum[w["manifest_stratum"]]["unusable"] += 1
                continue
            labels = [i["label"] for i in row["interactions"]]
            by_stratum[w["manifest_stratum"]][
                labeling.window_outcome({"interactions": [{"label": x} for x in labels]})
            ] += 1
            tiers.update(i["tier"] for i in row["interactions"])
        report["final_window_outcome_by_stratum"] = {
            k: dict(v) for k, v in sorted(by_stratum.items())
        }
        report["final_label_counts"] = dict(
            collections.Counter(i["label"] for row in finals.values() for i in row["interactions"])
        )
        report["final_tiers"] = dict(tiers)
    return report
