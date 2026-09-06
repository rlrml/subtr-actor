#!/usr/bin/env python3
"""Train goal models on disjoint replay-date train/tune/calibration/test partitions."""

import argparse
import copy
import hashlib
import json
import platform
import sklearn
from pathlib import Path

import numpy as np
import pandas as pd
import torch
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import brier_score_loss, log_loss, roc_auc_score

from entity_model import FEATURE_COUNT, GLOBAL_COUNT, GoalModel, labels, normalization

META = [
    "replay_id",
    "playlist",
    "date",
    "min_rank_tier",
    "max_rank_tier",
    "median_rank_tier",
    "team_size",
    "is_team0",
    "time",
    "time_to_next_goal_for",
    "time_to_next_goal_against",
    "time_to_replay_end",
    "time_to_next_touch",
    "time_to_live_end",
    "live_end_resolved",
]


def sha256(path):
    digest = hashlib.sha256()
    with open(path, "rb") as source:
        for block in iter(lambda: source.read(2**20), b""):
            digest.update(block)
    return digest.hexdigest()


def load_dataset(path, cache, task, horizon):
    cache.mkdir(parents=True, exist_ok=True)
    signature = {
        "sha256": sha256(path),
        "task": task,
        "horizon": horizon,
        "label_source_sha256": sha256(Path(__file__).with_name("entity_model.py")),
    }
    stamp = cache / "source.json"
    if not stamp.exists() or json.loads(stamp.read_text()) != signature:
        metas, count = [], 0
        with (cache / "features.bin").open("wb") as target:
            for frame in pd.read_csv(path, chunksize=100_000):
                columns = [name for name in frame.columns if name not in META]
                if len(columns) != FEATURE_COUNT:
                    raise ValueError(f"Expected {FEATURE_COUNT} features, found {len(columns)}")
                if not (frame.playlist.eq("ranked-doubles") & frame.team_size.eq(2)).all():
                    raise ValueError("Only ranked doubles are supported")
                y, keep = labels(frame, task, horizon)
                values = frame.loc[keep, columns].to_numpy(dtype=np.float32)
                if not np.isfinite(values).all():
                    raise ValueError("Nonfinite model inputs")
                values.tofile(target)
                meta = frame.loc[keep, [name for name in META if name in frame]].copy()
                meta["label"] = y[keep]
                metas.append(meta)
                count += len(values)
                print(f"loaded {count} labeled rows", flush=True)
        pd.concat(metas, ignore_index=True).to_pickle(cache / "metadata.pkl")
        (cache / "columns.json").write_text(json.dumps(columns))
        stamp.write_text(json.dumps(signature))
    frame = pd.read_pickle(cache / "metadata.pkl")
    values = np.memmap(
        cache / "features.bin", mode="r", dtype=np.float32, shape=(len(frame), FEATURE_COUNT)
    )
    return values, frame, json.loads((cache / "columns.json").read_text()), signature


def split_replays(frame):
    dates = pd.to_datetime(frame.date, utc=True, errors="raise")
    ordered = (
        frame.assign(parsed_date=dates)
        .groupby("replay_id")
        .parsed_date.min()
        .reset_index()
        .sort_values(["parsed_date", "replay_id"])
    )
    ids = ordered.replay_id.to_numpy()
    boundaries = [0, int(len(ids) * 0.60), int(len(ids) * 0.75), int(len(ids) * 0.85), len(ids)]
    if any(a == b for a, b in zip(boundaries, boundaries[1:])):
        raise ValueError("Need enough distinct replays for four disjoint partitions")
    partitions = {}
    for name, lo, hi in zip(("train", "tune", "calibration", "test"), boundaries, boundaries[1:]):
        selected = ids[lo:hi].tolist()
        partitions[name] = {
            "replay_ids": selected,
            "indices": np.flatnonzero(frame.replay_id.isin(selected)),
        }
    return partitions


def predict(model, values, indices, batch_size=8192):
    result = []
    model.eval()
    with torch.no_grad():
        for start in range(0, len(indices), batch_size):
            result.append(
                model(
                    torch.from_numpy(np.array(values[indices[start : start + batch_size]]))
                ).numpy()
            )
    return np.concatenate(result)


def probabilities(logits):
    return 1.0 / (1.0 + np.exp(-np.clip(logits, -40, 40)))


def metrics(y, p):
    result = {
        "rows": len(y),
        "log_loss": float(log_loss(y, p, labels=[0, 1])),
        "brier": float(brier_score_loss(y, p)),
        "auc": float(roc_auc_score(y, p)) if len(np.unique(y)) == 2 else None,
        "mean_prediction": float(p.mean()),
        "observed": float(y.mean()),
    }
    bands, ece = [], 0.0
    edges = [0, 0.01, 0.02, 0.05, 0.1, 0.15, 0.25, 0.5, 0.75, 1.000001]
    for lo, hi in zip(edges, edges[1:]):
        keep = (p >= lo) & (p < hi)
        if keep.any():
            expected, observed = float(p[keep].mean()), float(y[keep].mean())
            bands.append(
                {
                    "low": lo,
                    "high": min(hi, 1),
                    "n": int(keep.sum()),
                    "predicted": expected,
                    "observed": observed,
                }
            )
            ece += abs(expected - observed) * keep.mean()
    result.update(calibration=bands, ece=float(ece))
    return result


def fit(values, y, parts, mean, scale, entity, args, reflection):
    torch.manual_seed(args.seed)
    model = GoalModel(mean, scale, entity, reflection=reflection)
    optimizer = torch.optim.AdamW(model.parameters(), lr=0.001, weight_decay=0.0001)
    rng = np.random.default_rng(args.seed)
    best, best_loss, stale, records = None, np.inf, 0, []
    for epoch in range(args.epochs):
        model.train()
        order = rng.permutation(parts["train"]["indices"])
        total = 0.0
        for start in range(0, len(order), args.batch_size):
            indices = order[start : start + args.batch_size]
            x = torch.from_numpy(np.array(values[indices]))
            target = torch.from_numpy(y[indices])
            optimizer.zero_grad()
            loss = torch.nn.functional.binary_cross_entropy_with_logits(model(x), target)
            loss.backward()
            optimizer.step()
            total += float(loss.detach()) * len(indices)
        tune = parts["tune"]["indices"]
        validation = log_loss(y[tune], probabilities(predict(model, values, tune)), labels=[0, 1])
        record = {"epoch": epoch + 1, "training_loss": total / len(order), "tune_loss": validation}
        records.append(record)
        print(f"{'entity' if entity else 'legacy'} {record}", flush=True)
        if validation < best_loss - 0.00001:
            best, best_loss, stale = copy.deepcopy(model.state_dict()), validation, 0
        else:
            stale += 1
        if stale >= 4:
            break
    model.load_state_dict(best)
    calibration = parts["calibration"]["indices"]
    calibrator = LogisticRegression(C=1e6, solver="lbfgs", max_iter=1000)
    calibrator.fit(predict(model, values, calibration).reshape(-1, 1), y[calibration])
    return model, float(calibrator.coef_[0, 0]), float(calibrator.intercept_[0]), records


def rust_array(value):
    if isinstance(value, list):
        return "[" + ", ".join(rust_array(item) for item in value) + "]"
    text = str(np.float32(value))
    return text if any(char in text for char in ".e") else text + ".0"


def export(model, slope, intercept, columns, values, indices, out, horizon):
    state = {name: value.detach().numpy().tolist() for name, value in model.state_dict().items()}
    artifact = {
        "entity": model.entity,
        "horizon_seconds": horizon,
        "weights": state,
        "calibration_slope": slope,
        "calibration_intercept": intercept,
        "feature_names": columns,
    }
    (out / "model.json").write_text(json.dumps(artifact))
    lines = ["// Generated by train_entity_model.py; do not edit weights."]
    for name, tensor in model.state_dict().items():
        shape = "f32"
        for dim in reversed(tensor.shape):
            shape = f"[{shape}; {dim}]"
        lines.append(
            f"pub static {name.upper().replace('.', '_')}: {shape} = {rust_array(state[name])};"
        )
    lines.append(f"pub const CALIBRATION_SLOPE: f32 = {rust_array(slope)};")
    lines.append(f"pub const CALIBRATION_INTERCEPT: f32 = {rust_array(intercept)};")
    lines.append(f"pub const HORIZON_SECONDS: f32 = {rust_array(horizon)};")
    (out / "weights.rs").write_text("\n".join(lines) + "\n")
    # Actual test examples, plus exact full-player permutations, exercise the exported model.
    p = probabilities(slope * predict(model, values, indices) + intercept)
    selected = indices[np.argsort(p)[np.linspace(0, len(indices) - 1, 12).astype(int)]]
    fixtures = np.array(values[selected])
    swapped = fixtures.copy()
    swapped[:, GLOBAL_COUNT:] = (
        swapped[:, GLOBAL_COUNT:].reshape(-1, 4, 17)[:, [1, 0, 3, 2]].reshape(-1, 68)
    )
    fixtures = np.concatenate((fixtures, swapped))
    with torch.no_grad():
        predictions = probabilities(slope * model(torch.from_numpy(fixtures)).numpy() + intercept)
    pairs = [
        f"(&{rust_array(row.tolist())}, {rust_array(float(p))})"
        for row, p in zip(fixtures, predictions)
    ]
    (out / "parity.rs").write_text("&[\n" + ",\n".join(pairs) + "\n]\n")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("csv", type=Path)
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--task", choices=["frame", "touch"], required=True)
    parser.add_argument("--epochs", type=int, default=30)
    parser.add_argument("--batch-size", type=int, default=4096)
    parser.add_argument("--seed", type=int, default=20260905)
    parser.add_argument("--threads", type=int, default=2)
    parser.add_argument("--goal-counts", type=Path)
    parser.add_argument("--split-file", type=Path)
    args = parser.parse_args()
    torch.set_num_threads(args.threads)
    args.out_dir.mkdir(parents=True, exist_ok=True)
    horizon = 5.0 if args.task == "frame" else 10.0
    values, frame, columns, source = load_dataset(
        args.csv, args.out_dir / "dataset", args.task, horizon
    )
    if args.split_file:
        split_ids = json.loads(args.split_file.read_text())["partitions"]
        parts = {
            name: {"replay_ids": ids, "indices": np.flatnonzero(frame.replay_id.isin(ids))}
            for name, ids in split_ids.items()
        }
    else:
        parts = split_replays(frame)
    y = frame.label.to_numpy(dtype=np.float32)
    # Bound normalization memory while drawing from training data only.
    train = parts["train"]["indices"]
    sample = np.random.default_rng(args.seed).choice(train, min(len(train), 200_000), replace=False)
    mean, scale = normalization(values[sample])
    report = {
        "source": source,
        "task": args.task,
        "horizon": horizon,
        "seed": args.seed,
        "torch": torch.__version__,
        "python": platform.python_version(),
        "numpy": np.__version__,
        "pandas": pd.__version__,
        "sklearn": sklearn.__version__,
        "epochs_budget": args.epochs,
        "batch_size": args.batch_size,
        "source_files_sha256": {
            name: sha256(Path(__file__).with_name(name))
            for name in ["entity_model.py", "train_entity_model.py", "uv.lock"]
        },
        "published_model_fit_on": "train only; checkpoint selected on tune; calibration fitted on calibration; no refit",
        "split_file_sha256": sha256(args.split_file) if args.split_file else None,
        "feature_names": columns,
        "partitions": {name: part["replay_ids"] for name, part in parts.items()},
        "models": {},
    }
    reflection = np.array(
        [
            -1.0
            if name.endswith(("position_x", "velocity_x", "forward_x")) and "spread" not in name
            else 1.0
            for name in columns
        ],
        dtype=np.float32,
    )
    fitted = {}
    for entity in (False, True):
        name = "entity" if entity else "legacy-refit"
        model, slope, intercept, history = fit(
            values, y, parts, mean, scale, entity, args, reflection
        )
        fitted[name] = (model, slope, intercept)
        tune = parts["tune"]["indices"]
        report["models"][name] = {
            "history": history,
            "calibration_slope": slope,
            "calibration_intercept": intercept,
            "tune": metrics(
                y[tune], probabilities(slope * predict(model, values, tune) + intercept)
            ),
        }
    selected = min(report["models"], key=lambda name: report["models"][name]["tune"]["log_loss"])
    report["selected"] = selected
    print(f"Selected on tuning partition before test evaluation: {selected}", flush=True)
    test_predictions = {}
    for name, (model, slope, intercept) in fitted.items():
        result = report["models"][name]
        indices = parts["test"]["indices"]
        p = probabilities(slope * predict(model, values, indices) + intercept)
        test_predictions[name] = p
        result["test"] = metrics(y[indices], p)
        evaluated = frame.iloc[indices].copy()
        evaluated["prediction"] = p
        if args.task == "touch":
            games = evaluated.groupby(["replay_id", "is_team0"]).agg(
                xg=("prediction", "sum"), labeled_goals=("label", "sum")
            )
            if args.goal_counts:
                actual = pd.read_csv(args.goal_counts).set_index(["replay_id", "is_team0"])
                actual = actual.loc[
                    actual.index.get_level_values("replay_id").isin(parts["test"]["replay_ids"])
                ]
                games = actual[["goals"]].join(games).fillna(0)
            else:
                games["goals"] = games.labeled_goals
            result["count_scale"] = {
                "team_games": len(games),
                "mean_xg": games.xg.mean(),
                "mean_goals": games.goals.mean(),
                "mean_labeled_goals": games.labeled_goals.mean(),
                "label_goal_coverage": games.labeled_goals.sum() / games.goals.sum(),
                "correlation": games.xg.corr(games.goals),
                "mae": (games.xg - games.goals).abs().mean(),
                "rmse": np.sqrt(np.mean((games.xg - games.goals) ** 2)),
            }
            games.to_csv(args.out_dir / f"{name}-team-games.csv")
            bins = pd.cut(evaluated.time_to_next_touch, [0, 0.2, 0.5, 1, 3, 10, np.inf])
            result["next_touch_interval_diagnostics"] = {
                str(interval): metrics(group.label.to_numpy(), group.prediction.to_numpy())
                for interval, group in evaluated.groupby(bins, observed=True)
                if len(group)
            }
        result["per_rank"] = {
            str(rank): metrics(group.label.to_numpy(), group.prediction.to_numpy())
            for rank, group in evaluated.groupby("median_rank_tier")
        }
        evaluated[["replay_id", "is_team0", "time", "label", "prediction"]].to_csv(
            args.out_dir / f"{name}-test.csv", index=False
        )
        destination = args.out_dir / name
        destination.mkdir(exist_ok=True)
        export(model, slope, intercept, columns, values, indices, destination, horizon)
        print(name, result["test"], flush=True)
    # Resample entire replays, keeping correlated frames and both teams together.
    indices = parts["test"]["indices"]
    losses = {}
    for name, p in test_predictions.items():
        p = np.clip(p.astype(np.float64), 1e-7, 1 - 1e-7)
        losses[name] = -(y[indices] * np.log(p) + (1 - y[indices]) * np.log1p(-p))
    paired = (
        pd.DataFrame(
            {
                "replay_id": frame.iloc[indices].replay_id.to_numpy(),
                "difference": losses["entity"] - losses["legacy-refit"],
            }
        )
        .groupby("replay_id")
        .difference.agg(["sum", "count"])
    )
    rng = np.random.default_rng(args.seed)
    draws = rng.integers(0, len(paired), (1000, len(paired)))
    means = paired["sum"].to_numpy()[draws].sum(1) / paired["count"].to_numpy()[draws].sum(1)
    report["paired_replay_bootstrap_log_loss_difference_95ci"] = np.quantile(
        means, [0.025, 0.975]
    ).tolist()
    (args.out_dir / "metrics.json").write_text(
        json.dumps(report, indent=2, default=lambda value: value.item()) + "\n"
    )


if __name__ == "__main__":
    main()
