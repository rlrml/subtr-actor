"""Train and evaluate telemetry whiff models against qualified labels.

Supervision comes from adjudicated window labels. Each focal-player candidate
whose anchor falls inside a labeled core interval is matched one-to-one to a
labeled interaction; unmatched candidates are `no_observable_attempt`;
candidates matched to `uncertain` interactions are excluded from the loss and
from every metric. Labeled interactions that no candidate reaches are counted
as candidate-recall misses rather than dropped.
"""

from __future__ import annotations

import collections
import json
import pathlib

import numpy as np
from sklearn.ensemble import GradientBoostingClassifier

import features as feat
from adjudicate import ABSENT
from telemetry import ReplayTelemetry, dump_path, load_dump

MODEL_CLASSES = ("clear_whiff", "disrupted_attempt", "successful_contact", "no_observable_attempt")
EVENT_CLASSES = ("clear_whiff", "disrupted_attempt")
MATCH_TOLERANCE_SECONDS = 0.4


def greedy_match(left: list[float], right: list[float], tolerance: float) -> list[tuple[int, int]]:
    """One-to-one matching of times, closest pairs first."""
    pairs = sorted(
        (abs(a - b), i, j)
        for i, a in enumerate(left)
        for j, b in enumerate(right)
        if abs(a - b) <= tolerance
    )
    used_left, used_right, out = set(), set(), []
    for _, i, j in pairs:
        if i in used_left or j in used_right:
            continue
        used_left.add(i)
        used_right.add(j)
        out.append((i, j))
    return out


class ReplayCache:
    def __init__(self, cache: pathlib.Path):
        self.cache = cache
        self.replays: dict[str, ReplayTelemetry] = {}
        self.candidates: dict[str, list[dict]] = {}

    def replay(self, replay_id: str) -> ReplayTelemetry:
        if replay_id not in self.replays:
            self.replays[replay_id] = load_dump(dump_path(self.cache / "dumps", replay_id))
        return self.replays[replay_id]

    def replay_candidates(self, replay_id: str) -> list[dict]:
        if replay_id not in self.candidates:
            self.candidates[replay_id] = feat.replay_candidates(self.replay(replay_id))
        return self.candidates[replay_id]


def build_examples(windows: list[dict], replays: ReplayCache) -> tuple[list[dict], list[dict]]:
    """(candidate examples, labeled interactions with candidate-coverage info)."""
    examples, interactions = [], []
    for w in windows:
        if not w["usable"]:
            continue
        cands = [
            c
            for c in replays.replay_candidates(w["replay_id"])
            if c["player_id"] == w["player_id"]
            and w["core_start_time"] <= c["anchor_time"] <= w["core_end_time"]
        ]
        labeled = w["interactions"]
        pairs = greedy_match(
            [c["anchor_time"] for c in cands],
            [i["t_replay"] for i in labeled],
            MATCH_TOLERANCE_SECONDS,
        )
        matched_c = {ci: li for ci, li in pairs}
        matched_l = {li for _, li in pairs}
        for ci, c in enumerate(cands):
            label = labeled[matched_c[ci]]["label"] if ci in matched_c else ABSENT
            examples.append(
                {
                    **{k: c[k] for k in ("replay_id", "player_id", "anchor_time", "anchor_frame")},
                    "features": c["features"],
                    "label": label,
                    "split": w["split"],
                    "neutral_id": w["neutral_id"],
                    "manifest_stratum": w["manifest_stratum"],
                }
            )
        for li, item in enumerate(labeled):
            interactions.append(
                {
                    **item,
                    "neutral_id": w["neutral_id"],
                    "replay_id": w["replay_id"],
                    "split": w["split"],
                    "manifest_stratum": w["manifest_stratum"],
                    "has_candidate": li in matched_l,
                }
            )
    return examples, interactions


def to_xy(examples: list[dict]):
    keep = [e for e in examples if e["label"] in MODEL_CLASSES]
    x = np.array([e["features"] for e in keep], dtype=np.float64)
    y = np.array([MODEL_CLASSES.index(e["label"]) for e in keep])
    return keep, np.nan_to_num(x, nan=-999.0), y


def train_boosted(x, y, params: dict, seed: int = 7) -> GradientBoostingClassifier:
    counts = np.bincount(y, minlength=len(MODEL_CLASSES)).astype(float)
    weights = np.where(counts[y] > 0, len(y) / (len(MODEL_CLASSES) * np.maximum(counts[y], 1)), 0.0)
    weights = np.sqrt(weights)
    model = GradientBoostingClassifier(random_state=seed, **params)
    model.fit(x, y, sample_weight=weights)
    return model


def predict_classes(model, x, thresholds: dict[str, float]) -> list[str]:
    """Argmax, except event classes require their probability threshold."""
    proba = np.zeros((len(x), len(MODEL_CLASSES)))
    proba[:, model.classes_] = model.predict_proba(x)
    out = []
    for row in proba:
        best = int(np.argmax(row))
        name = MODEL_CLASSES[best]
        if name in thresholds and row[best] < thresholds[name]:
            ordered = np.argsort(row)[::-1]
            name = next(MODEL_CLASSES[k] for k in ordered if MODEL_CLASSES[k] not in thresholds)
        out.append(name)
    return out


def geometric_rule(f: np.ndarray, params: dict) -> str:
    """Hand-built telemetry rule baseline."""
    names = feat.FEATURE_NAMES
    v = dict(zip(names, f))
    if abs(v["focal_touch_dt"]) <= params["touch_window"]:
        return "successful_contact"
    committed = (
        v["closing_rate_0_3"] >= params["min_closing"]
        and v["velocity_alignment_m0_3"] >= params["min_alignment"]
    )
    if not committed or v["gap_at_anchor"] > params["max_gap"]:
        return ABSENT
    other = min(abs(v["teammate_touch_dt"]), abs(v["opponent_touch_dt"]))
    if other <= params["disrupt_window"]:
        return "disrupted_attempt"
    return "clear_whiff"


def event_metrics(
    predictions: list[dict], labels: list[dict], tolerance: float = MATCH_TOLERANCE_SECONDS
) -> dict:
    """Per-class event precision/recall from per-window predicted and labeled events.

    `predictions` / `labels`: dicts with neutral_id, t (replay time), label.
    Uncertain labels neither reward nor penalize matching predictions.
    """
    out = {}
    by_window_pred = collections.defaultdict(list)
    by_window_label = collections.defaultdict(list)
    for p in predictions:
        by_window_pred[p["neutral_id"]].append(p)
    for item in labels:
        by_window_label[item["neutral_id"]].append(item)
    for cls in EVENT_CLASSES:
        tp = fp = fn = 0
        timing = []
        per_window = {}
        for nid in set(by_window_pred) | set(by_window_label):
            preds = [p for p in by_window_pred[nid] if p["label"] == cls]
            labs = [x for x in by_window_label[nid] if x["label"] == cls]
            uncertain = [x for x in by_window_label[nid] if x["label"] == "uncertain"]
            pairs = greedy_match([p["t"] for p in preds], [x["t"] for x in labs], tolerance)
            matched_p = {i for i, _ in pairs}
            w_tp = len(pairs)
            w_fn = len(labs) - w_tp
            w_fp = 0
            for i, p in enumerate(preds):
                if i in matched_p:
                    continue
                if any(abs(p["t"] - u["t"]) <= tolerance for u in uncertain):
                    continue
                w_fp += 1
            timing += [preds[i]["t"] - labs[j]["t"] for i, j in pairs]
            tp, fp, fn = tp + w_tp, fp + w_fp, fn + w_fn
            per_window[nid] = (w_tp, w_fp, w_fn)
        out[cls] = {
            "tp": tp,
            "fp": fp,
            "fn": fn,
            "precision": tp / (tp + fp) if tp + fp else None,
            "recall": tp / (tp + fn) if tp + fn else None,
            "predicted": tp + fp,
            "labeled": tp + fn,
            "median_abs_timing_error": float(np.median(np.abs(timing))) if timing else None,
            "per_window": per_window,
        }
    return out


def bootstrap_ci(per_window: dict, window_replay: dict, metric: str, n: int = 2000, seed: int = 7):
    """Replay-level bootstrap CI for precision or recall."""
    by_replay = collections.defaultdict(lambda: np.zeros(3))
    for nid, counts in per_window.items():
        by_replay[window_replay[nid]] += np.array(counts)
    replays = list(by_replay)
    if not replays:
        return None
    rng = np.random.default_rng(seed)
    values = []
    stacked = np.array([by_replay[r] for r in replays])
    for _ in range(n):
        sample = stacked[rng.integers(0, len(replays), len(replays))].sum(axis=0)
        tp, fp, fn = sample
        denominator = tp + fp if metric == "precision" else tp + fn
        if denominator:
            values.append(tp / denominator)
    if not values:
        return None
    return [float(np.percentile(values, 2.5)), float(np.percentile(values, 97.5))]


def strip_per_window(metrics: dict) -> dict:
    return {cls: {k: v for k, v in m.items() if k != "per_window"} for cls, m in metrics.items()}


def export_model(
    model: GradientBoostingClassifier, path: pathlib.Path, metadata: dict, check_x=None
):
    """Serialize trees as flat arrays (feature, threshold, children, leaf value) per class."""
    stages = []
    for stage in model.estimators_:
        trees = []
        for tree in stage:
            t = tree.tree_
            trees.append(
                {
                    "feature": t.feature.tolist(),
                    "threshold": t.threshold.tolist(),
                    "left": t.children_left.tolist(),
                    "right": t.children_right.tolist(),
                    "value": t.value[:, 0, 0].tolist(),
                }
            )
        stages.append(trees)
    init = model.init_
    prior = getattr(init, "class_prior_", None)
    payload = {
        **metadata,
        "classes": [MODEL_CLASSES[c] for c in model.classes_],
        "feature_names": metadata.get("feature_names", feat.FEATURE_NAMES),
        "feature_version": metadata.get("feature_version", feat.FEATURE_VERSION),
        "missing_value": -999.0,
        "learning_rate": model.learning_rate,
        "init_log_prior": None if prior is None else np.log(np.maximum(prior, 1e-12)).tolist(),
        "stages": stages,
    }
    if check_x is not None:
        expected = np.zeros((len(check_x), len(MODEL_CLASSES)))
        expected[:, model.classes_] = model.predict_proba(check_x)
        got = exported_proba(payload, check_x)
        reordered = np.zeros_like(expected)
        reordered[:, model.classes_] = got
        error = float(np.abs(reordered - expected).max())
        if error > 1e-9:
            raise AssertionError(f"exported model disagrees with scikit-learn by {error}")
        payload["export_check_max_abs_error"] = error
    path.write_text(json.dumps(payload))


def exported_proba(payload: dict, features: np.ndarray) -> np.ndarray:
    """Evaluate an exported model exactly as the Rust shadow inference does."""
    x = np.where(np.isnan(features), payload["missing_value"], features)
    raw = np.tile(np.array(payload["init_log_prior"], dtype=np.float64), (len(x), 1))
    for stage in payload["stages"]:
        for k, tree in enumerate(stage):
            node = np.zeros(len(x), dtype=np.int64)
            left = np.array(tree["left"])
            right = np.array(tree["right"])
            feature = np.array(tree["feature"])
            threshold = np.array(tree["threshold"])
            active = left[node] >= 0
            while active.any():
                idx = np.flatnonzero(active)
                go_left = x[idx, feature[node[idx]]] <= threshold[node[idx]]
                node[idx] = np.where(go_left, left[node[idx]], right[node[idx]])
                active = left[node] >= 0
            raw[:, k] += payload["learning_rate"] * np.array(tree["value"])[node]
    raw -= raw.max(axis=1, keepdims=True)
    exp = np.exp(raw)
    return exp / exp.sum(axis=1, keepdims=True)
