"""Train the deployed whiff classifier on features computed by the core detector.

`whiff_candidate_dump` runs the production `WhiffCalculator` with candidate
recording and writes every candidate's feature vector, so training and
inference share one feature implementation (`src/stats/calculators/
whiff_features.rs`). Model selection uses replay-grouped cross-validation over
non-test labels; test label files are scored once with the selected
configuration. The deployed model can then be refit on every label file and
exported as the generated Rust weights the core crate includes.
"""

from __future__ import annotations

import collections
import gzip
import json
import pathlib

import numpy as np
from sklearn.ensemble import GradientBoostingClassifier

import modeling

PARAM_GRID = [
    {
        "n_estimators": n,
        "max_depth": d,
        "learning_rate": lr,
        "subsample": 0.8,
        "min_samples_leaf": leaf,
    }
    for n, d, lr, leaf in [
        (100, 2, 0.1, 8),
        (150, 3, 0.05, 8),
        (100, 3, 0.1, 12),
        (200, 2, 0.05, 8),
    ]
]
THRESHOLD_GRID = (0.0, 0.3, 0.4, 0.5, 0.6)
CV_FOLDS = 5
CV_SEEDS = (3, 11, 19)


def load_core_dump(path: pathlib.Path) -> dict:
    with gzip.open(path, "rt") as handle:
        return json.load(handle)


class CoreReplayCache(modeling.ReplayCache):
    """Candidates and events from `whiff_candidate_dump` output."""

    def __init__(self, cache: pathlib.Path, candidates_dir: pathlib.Path):
        super().__init__(cache)
        self.candidates_dir = candidates_dir
        self.docs: dict[str, dict] = {}

    def doc(self, replay_id: str) -> dict:
        if replay_id not in self.docs:
            self.docs[replay_id] = load_core_dump(
                self.candidates_dir / f"{replay_id}.candidates.json.gz"
            )
        return self.docs[replay_id]

    def feature_names(self, replay_id: str) -> list[str]:
        return self.doc(replay_id)["feature_names"]

    def replay_candidates(self, replay_id: str) -> list[dict]:
        if replay_id not in self.candidates:
            self.candidates[replay_id] = [
                {
                    "replay_id": replay_id,
                    "player_id": c["player_id"],
                    "anchor_time": c["anchor_time"],
                    "anchor_frame": c["anchor_frame"],
                    "features": np.array(
                        [np.nan if v is None else v for v in c["features"]], dtype=np.float64
                    ),
                    "core_probabilities": c["probabilities"],
                }
                for c in self.doc(replay_id)["candidates"]
            ]
        return self.candidates[replay_id]


def core_event_predictions(windows: list[dict], replays: CoreReplayCache) -> list[dict]:
    """Events emitted by the core detector build that produced the dumps."""
    kinds = {"whiff": "clear_whiff", "disrupted_attempt": "disrupted_attempt"}
    preds = []
    for w in windows:
        if not w["usable"]:
            continue
        for e in replays.doc(w["replay_id"])["whiff_events"]:
            label = kinds.get(e.get("kind", "whiff"))
            if (
                label
                and e["player_id"] == w["player_id"]
                and w["core_start_time"] <= e["time"] <= w["core_end_time"]
            ):
                preds.append({"neutral_id": w["neutral_id"], "t": e["time"], "label": label})
    return preds


def _xy(examples: list[dict]):
    keep = [e for e in examples if e["label"] in modeling.MODEL_CLASSES]
    x = np.nan_to_num(np.array([e["features"] for e in keep]), nan=-999.0)
    y = np.array([modeling.MODEL_CLASSES.index(e["label"]) for e in keep])
    return keep, x, y


def replay_folds(replay_ids, seed: int, k: int = CV_FOLDS) -> dict[str, int]:
    ids = sorted(set(replay_ids))
    order = np.random.default_rng(seed).permutation(ids)
    return {replay_id: i % k for i, replay_id in enumerate(order)}


def out_of_fold_proba(examples: list[dict], params: dict, folds: dict[str, int]) -> np.ndarray:
    keep, x, y = _xy(examples)
    keep_fold = np.array([folds[e["replay_id"]] for e in keep])
    all_x = np.nan_to_num(np.array([e["features"] for e in examples]), nan=-999.0)
    all_fold = np.array([folds[e["replay_id"]] for e in examples])
    proba = np.zeros((len(examples), len(modeling.MODEL_CLASSES)))
    for fold in range(CV_FOLDS):
        model = modeling.train_boosted(x[keep_fold != fold], y[keep_fold != fold], params)
        selected = all_fold == fold
        part = np.zeros((selected.sum(), len(modeling.MODEL_CLASSES)))
        part[:, model.classes_] = model.predict_proba(all_x[selected])
        proba[selected] = part
    return proba


def thresholded(proba: np.ndarray, thresholds: dict[str, float]) -> list[str]:
    out = []
    for row in proba:
        order = np.argsort(row)[::-1]
        name = modeling.MODEL_CLASSES[order[0]]
        if name in thresholds and row[order[0]] < thresholds[name]:
            name = next(
                modeling.MODEL_CLASSES[k]
                for k in order
                if modeling.MODEL_CLASSES[k] not in thresholds
            )
        out.append(name)
    return out


def example_events(examples: list[dict], labels: list[str]) -> list[dict]:
    return [
        {"neutral_id": e["neutral_id"], "t": e["anchor_time"], "label": label}
        for e, label in zip(examples, labels)
        if label in modeling.EVENT_CLASSES
    ]


def strict_counts(examples: list[dict], labels: list[str]) -> dict:
    """Predictions matched to `uncertain` interactions, reported separately."""
    out = {}
    for cls in modeling.EVENT_CLASSES:
        on = collections.Counter(e["label"] for e, p in zip(examples, labels) if p == cls)
        tp, uncertain = on[cls], on["uncertain"]
        fp = sum(on.values()) - tp - uncertain
        out[cls] = {
            "on_uncertain": uncertain,
            "strict_precision": tp / (tp + fp + uncertain) if tp + fp + uncertain else None,
        }
    return out


def macro_f1(metrics: dict) -> float:
    scores = []
    for cls in modeling.EVENT_CLASSES:
        p, r = metrics[cls]["precision"] or 0.0, metrics[cls]["recall"] or 0.0
        scores.append(2 * p * r / (p + r) if p + r else 0.0)
    return float(np.mean(scores))


def select(examples: list[dict], label_events: list[dict]) -> dict:
    """Pick params and thresholds by mean replay-grouped CV macro event F1."""
    replay_ids = [e["replay_id"] for e in examples]
    results = []
    for params in PARAM_GRID:
        probas = [
            out_of_fold_proba(examples, params, replay_folds(replay_ids, s)) for s in CV_SEEDS
        ]
        for a in THRESHOLD_GRID:
            for b in THRESHOLD_GRID:
                thresholds = {"clear_whiff": a, "disrupted_attempt": b}
                scores, metrics = [], []
                for proba in probas:
                    m = modeling.event_metrics(
                        example_events(examples, thresholded(proba, thresholds)), label_events
                    )
                    scores.append(macro_f1(m))
                    metrics.append(modeling.strip_per_window(m))
                results.append(
                    {
                        "params": params,
                        "thresholds": thresholds,
                        "cv_macro_f1": float(np.mean(scores)),
                        "cv_macro_f1_by_seed": scores,
                        "cv_metrics_first_seed": metrics[0],
                        "cv_strict_first_seed": strict_counts(
                            examples, thresholded(probas[0], thresholds)
                        ),
                    }
                )
    results.sort(key=lambda r: -r["cv_macro_f1"])
    return {"best": results[0], "top": results[:5]}


def fit(examples: list[dict], params: dict) -> GradientBoostingClassifier:
    _, x, y = _xy(examples)
    return modeling.train_boosted(x, y, params)


def predict_windows(model, thresholds, windows, replays) -> list[dict]:
    preds = []
    for w in windows:
        if not w["usable"]:
            continue
        cands = [
            c
            for c in replays.replay_candidates(w["replay_id"])
            if c["player_id"] == w["player_id"]
            and w["core_start_time"] <= c["anchor_time"] <= w["core_end_time"]
        ]
        if not cands:
            continue
        x = np.nan_to_num(np.array([c["features"] for c in cands]), nan=-999.0)
        proba = np.zeros((len(x), len(modeling.MODEL_CLASSES)))
        proba[:, model.classes_] = model.predict_proba(x)
        for c, label in zip(cands, thresholded(proba, thresholds)):
            if label in modeling.EVENT_CLASSES:
                preds.append({"neutral_id": w["neutral_id"], "t": c["anchor_time"], "label": label})
    return preds


def _rust_float(value: float) -> str:
    text = repr(float(value))
    if not np.isfinite(value):
        raise ValueError(f"non-finite model value {value}")
    return text if any(ch in text for ch in ".e") else text + ".0"


def export_rust(model, path: pathlib.Path, metadata: dict, feature_names: list[str]):
    """Write the generated `whiff_model_weights.rs` included by the core crate."""
    classes = [modeling.MODEL_CLASSES[c] for c in model.classes_]
    if classes != list(modeling.MODEL_CLASSES):
        raise ValueError(f"model is missing classes: {classes}")
    prior = np.log(np.maximum(model.init_.class_prior_, 1e-12))
    lines = [
        "// @generated by scripts/whiff_model (`whifflab.py train-core`). Do not edit.",
        f"// {metadata['provenance']}",
        f"pub const WHIFF_MODEL_VERSION: &str = {json.dumps(metadata['name'])};",
        f"pub const WHIFF_MODEL_FEATURE_VERSION: &str = {json.dumps(metadata['feature_version'])};",
        f"const _: () = assert!(WHIFF_FEATURE_COUNT == {len(feature_names)});",
        "const WHIFF_MODEL_MISSING_VALUE: f64 = -999.0;",
        f"const WHIFF_MODEL_LEARNING_RATE: f64 = {_rust_float(model.learning_rate)};",
        "const WHIFF_MODEL_INIT_LOG_PRIOR: [f64; 4] = ["
        + ", ".join(_rust_float(v) for v in prior)
        + "];",
        "const WHIFF_MODEL_CLEAN_WHIFF_THRESHOLD: f64 = "
        + _rust_float(metadata["thresholds"]["clear_whiff"])
        + ";",
        "const WHIFF_MODEL_DISRUPTED_THRESHOLD: f64 = "
        + _rust_float(metadata["thresholds"]["disrupted_attempt"])
        + ";",
        "#[rustfmt::skip]",
        "const WHIFF_MODEL_STAGES: &[[&[WhiffTreeNode]; 4]] = &[",
    ]
    for stage in model.estimators_:
        trees = []
        for tree in stage:
            t = tree.tree_
            nodes = ", ".join(
                f"n({int(t.feature[i]) if t.children_left[i] >= 0 else -2}, "
                f"{_rust_float(t.threshold[i])}, {int(t.children_left[i])}, "
                f"{int(t.children_right[i])}, {_rust_float(t.value[i, 0, 0])})"
                for i in range(t.node_count)
            )
            trees.append(f"&[{nodes}]")
        lines.append("    [" + ", ".join(trees) + "],")
    lines.append("];")
    path.write_text("\n".join(lines) + "\n")


def export_rust_parity_fixture(model, examples: list[dict], path: pathlib.Path, per_class: int = 6):
    """Feature rows with scikit-learn probabilities for the Rust parity test."""
    x = np.array([e["features"] for e in examples])
    proba = np.zeros((len(x), len(modeling.MODEL_CLASSES)))
    proba[:, model.classes_] = model.predict_proba(np.nan_to_num(x, nan=-999.0))
    chosen = []
    for k in range(len(modeling.MODEL_CLASSES)):
        order = np.argsort(-proba[:, k])
        chosen += [int(i) for i in order[:per_class] if int(i) not in chosen]

    def literal(value: float) -> str:
        return "f64::NAN" if np.isnan(value) else _rust_float(value)

    lines = ["// @generated by scripts/whiff_model (`whifflab.py train-core`). Do not edit.", "&["]
    for i in chosen:
        features = ", ".join(literal(v) for v in x[i])
        probabilities = ", ".join(_rust_float(v) for v in proba[i])
        lines.append(f"    (&[{features}], [{probabilities}]),")
    lines.append("]")
    path.write_text("\n".join(lines) + "\n")
