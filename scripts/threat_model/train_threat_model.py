#!/usr/bin/env python3
"""Train the subtr-actor expected-goals threat model.

Run from this directory with `uv run --locked train_threat_model.py ...`, or
from the repository root with `nix run .#train-threat-model -- ...`.

Input: CSV from `threat_dataset_dump` with columns
  replay_id, playlist, date, min_rank_tier, max_rank_tier, median_rank_tier,
  team_size, is_team0, time,
  <feature columns...>, time_to_next_goal_for, time_to_next_goal_against,
  time_to_replay_end

Label: this team scores within TAU seconds. Rows with no future goal and less
than TAU seconds of remaining observation are censored (dropped).

Outputs (to --out-dir):
  - metrics.txt: log-loss/Brier/AUC vs baseline, overall + per rank tier calibration
  - model_coefficients.rs: generated Rust coefficients for the selected model
  - parity_fixture.rs: feature rows + expected V for a Rust parity test
"""

import argparse
import gc
import hashlib
import importlib.metadata
import json
import pathlib
import platform

import numpy as np
import pandas as pd
from sklearn.ensemble import HistGradientBoostingClassifier
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import brier_score_loss, log_loss, roc_auc_score
from sklearn.neural_network import MLPClassifier
from sklearn.preprocessing import StandardScaler

META_COLS = [
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
]

parser = argparse.ArgumentParser()
parser.add_argument("csv")
parser.add_argument("--tau", type=float, default=5.0)
parser.add_argument("--sample-hz", type=float, default=4.0)
parser.add_argument("--out-dir", default="threat_model_out")
parser.add_argument("--test-frac", type=float, default=0.2)
parser.add_argument("--seed", type=int, default=7)
parser.add_argument(
    "--manifest",
    help="source replay manifest; its SHA-256 is recorded in training provenance",
)
parser.add_argument("--gbt", action="store_true", help="also fit a GBT ceiling reference")
parser.add_argument(
    "--mlp-hidden-units",
    type=int,
    default=8,
    help="hidden width of the smooth model published for Rust",
)
parser.add_argument(
    "--mlp-epochs",
    type=int,
    default=16,
    help="fixed MLP training budget (also acts as regularization)",
)
args = parser.parse_args()
if args.sample_hz <= 0:
    parser.error("--sample-hz must be positive")
if args.mlp_hidden_units <= 0:
    parser.error("--mlp-hidden-units must be positive")
if args.mlp_epochs <= 0:
    parser.error("--mlp-epochs must be positive")


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


out_dir = pathlib.Path(args.out_dir)
out_dir.mkdir(parents=True, exist_ok=True)

csv_path = pathlib.Path(args.csv)
df = pd.read_csv(csv_path)
feature_cols = [c for c in df.columns if c not in META_COLS]
print(f"rows={len(df)} features={feature_cols}")

if set(df["playlist"].dropna().unique()) != {"ranked-doubles"}:
    raise ValueError("threat model training is restricted to ranked-doubles")
if not (df["team_size"].dropna() == 2).all():
    raise ValueError("threat model training requires team_size=2 for every row")

ball_fields = [
    "ball_forward_y",
    "ball_dist_to_goal",
    "ball_height",
    "ball_speed",
    "ball_speed_toward_goal",
    "goal_open_angle",
    "on_target",
    "time_to_goal_line",
]
player_fields = [
    "position_x",
    "position_y",
    "position_z",
    "velocity_x",
    "velocity_y",
    "velocity_z",
    "forward_x",
    "forward_y",
    "forward_z",
    "distance_to_ball",
    "distance_to_goal",
    "boost",
    "is_goalside",
    "in_net",
    "dodge_available",
    "demoed",
]
team_sets = ("own_team", "opponent_team")
aggregates = ("mean", "spread")
expected_feature_cols = ball_fields + [
    f"{team}_{aggregate}_{field}"
    for team in team_sets
    for aggregate in aggregates
    for field in player_fields
]
history_fields = [
    *ball_fields,
    *[
        f"{team}_{aggregate}_{field}"
        for team in team_sets
        for aggregate, fields in (
            (
                "mean",
                (
                    "position_y",
                    "position_z",
                    "velocity_y",
                    "velocity_z",
                    "forward_y",
                    "distance_to_ball",
                    "distance_to_goal",
                    "boost",
                    "dodge_available",
                    "demoed",
                ),
            ),
            (
                "spread",
                (
                    "position_y",
                    "position_z",
                    "velocity_y",
                    "velocity_z",
                    "distance_to_ball",
                    "boost",
                ),
            ),
        )
        for field in fields
    ],
]
for lag in ("0.5", "1"):
    expected_feature_cols.extend(f"delta_{lag}s_{field}" for field in history_fields)
    expected_feature_cols.append(f"history_{lag}s_available")
if feature_cols != expected_feature_cols:
    raise ValueError(
        "unexpected or out-of-order threat feature schema:\n"
        f"expected={expected_feature_cols}\nactual={feature_cols}"
    )

for team in team_sets:
    for aggregate in aggregates:
        prefix = f"{team}_{aggregate}_"
        actual = [name.removeprefix(prefix) for name in feature_cols if name.startswith(prefix)]
        if actual != player_fields:
            raise ValueError(
                f"asymmetric or out-of-order player feature schema for {team}/{aggregate}: {actual}"
            )

for col in feature_cols:
    df[col] = df[col].astype(np.float32)

# Label + censoring
tau = args.tau
has_goal = df["time_to_next_goal_for"].notna()
label = has_goal & (df["time_to_next_goal_for"] <= tau)
censored = ~has_goal & (df["time_to_replay_end"] < tau)
keep = ~censored
df = df[keep].copy()
y = label[keep].to_numpy()
X = df[feature_cols].to_numpy(dtype=np.float32)
groups = df["replay_id"].to_numpy()

print(f"kept={len(df)} censored_dropped={int(censored.sum())} base_rate={y.mean():.5f}")

bad = ~np.isfinite(X).all(axis=0)
if bad.any():
    print(
        "WARNING: non-finite feature columns:",
        [feature_cols[i] for i in np.where(bad)[0]],
    )
    X = np.nan_to_num(X, nan=0.0, posinf=0.0, neginf=0.0)

dates = pd.to_datetime(df["date"], utc=True, errors="raise")
replay_dates = (
    pd.DataFrame({"replay_id": groups, "date": dates})
    .groupby("replay_id", sort=False)["date"]
    .min()
    .sort_values()
)
test_replay_count = max(1, int(np.ceil(len(replay_dates) * args.test_frac)))
test_replays = set(replay_dates.index[-test_replay_count:])
test_mask = np.fromiter((replay_id in test_replays for replay_id in groups), dtype=bool)
train_idx = np.flatnonzero(~test_mask)
test_idx = np.flatnonzero(test_mask)
if not len(train_idx) or not len(test_idx):
    raise ValueError("temporal replay split produced an empty partition")
Xtr, Xte, ytr, yte = X[train_idx], X[test_idx], y[train_idx], y[test_idx]
temporal_cutoff = replay_dates.loc[list(test_replays)].min().isoformat()
print(
    f"train={len(train_idx)} test={len(test_idx)} "
    f"(latest {test_replay_count} replays held out from {temporal_cutoff})"
)


def replay_set_hash(indices) -> str:
    replay_ids = sorted({str(groups[index]) for index in indices})
    return hashlib.sha256("\n".join(replay_ids).encode()).hexdigest()


provenance = {
    "dataset": str(csv_path),
    "dataset_sha256": sha256_file(csv_path),
    "manifest": args.manifest,
    "manifest_sha256": sha256_file(pathlib.Path(args.manifest)) if args.manifest else None,
    "seed": args.seed,
    "test_fraction": args.test_frac,
    "split": "temporal_by_replay_date",
    "temporal_test_cutoff": temporal_cutoff,
    "tau_seconds": tau,
    "sample_hz": args.sample_hz,
    "publish_model": "mlp-tanh",
    "mlp_hidden_units": args.mlp_hidden_units,
    "mlp_epochs": args.mlp_epochs,
    "train_replay_ids_sha256": replay_set_hash(train_idx),
    "test_replay_ids_sha256": replay_set_hash(test_idx),
    "python": platform.python_version(),
    "packages": {
        name: importlib.metadata.version(name) for name in ("numpy", "pandas", "scikit-learn")
    },
}
(out_dir / "training_provenance.json").write_text(json.dumps(provenance, indent=2) + "\n")

scaler = StandardScaler().fit(Xtr)
Xtr_s, Xte_s = scaler.transform(Xtr), scaler.transform(Xte)


def new_mlp():
    return MLPClassifier(
        hidden_layer_sizes=(args.mlp_hidden_units,),
        activation="tanh",
        solver="adam",
        alpha=1e-4,
        batch_size=4096,
        learning_rate_init=1e-3,
        max_iter=args.mlp_epochs,
        shuffle=True,
        random_state=args.seed,
        tol=1e-5,
        n_iter_no_change=3,
        verbose=True,
    )


lr = LogisticRegression(max_iter=2000, C=1.0)
lr.fit(Xtr_s, ytr)
p_lr = lr.predict_proba(Xte_s)[:, 1]

lines = [
    "provenance:",
    *(f"  {key}={value}" for key, value in provenance.items() if key != "packages"),
    *(f"  {name}={version}" for name, version in provenance["packages"].items()),
    "",
]


def report(name, p, yt):
    base = np.full_like(p, ytr.mean())
    msg = (
        f"{name}: log_loss={log_loss(yt, p):.5f} brier={brier_score_loss(yt, p):.5f} "
        f"auc={roc_auc_score(yt, p):.4f} | baseline log_loss={log_loss(yt, base):.5f} "
        f"brier={brier_score_loss(yt, base):.5f}"
    )
    print(msg)
    lines.append(msg)


report("logistic", p_lr, yte)
predictions = {"logistic": p_lr}


def feature_knockout(name, predicate, standardized_test):
    """Measure reliance by replacing one feature family with its training mean."""
    knocked_out = standardized_test.copy()
    columns = [index for index, column in enumerate(feature_cols) if predicate(column)]
    knocked_out[:, columns] = 0.0
    probabilities = lr.predict_proba(knocked_out)[:, 1]
    delta = log_loss(yte, probabilities) - log_loss(yte, p_lr)
    msg = f"knockout {name}: columns={len(columns)} delta_log_loss={delta:+.5f}"
    print(msg)
    lines.append(msg)


feature_knockout(
    "all-player-state",
    lambda column: (
        not any(column == field or column.endswith(f"s_{field}") for field in ball_fields)
    ),
    Xte_s,
)
feature_knockout("boost", lambda column: column.endswith("_boost"), Xte_s)
feature_knockout("dodge-available", lambda column: column.endswith("_dodge_available"), Xte_s)
feature_knockout("demo-state", lambda column: column.endswith("_demoed"), Xte_s)
feature_knockout(
    "all-history",
    lambda column: column.startswith("delta_") or column.startswith("history_"),
    Xte_s,
)

if args.gbt:
    gbt = HistGradientBoostingClassifier(max_iter=300, learning_rate=0.1, random_state=args.seed)
    gbt.fit(Xtr, ytr)
    p_gbt = gbt.predict_proba(Xte)[:, 1]
    report("gbt-ceiling", p_gbt, yte)
    predictions["gbt-ceiling"] = p_gbt
    del gbt
    gc.collect()

mlp = new_mlp()
mlp.fit(Xtr_s, ytr)
p_mlp = mlp.predict_proba(Xte_s)[:, 1]
report(f"mlp-tanh-{args.mlp_hidden_units}", p_mlp, yte)
predictions[f"mlp-tanh-{args.mlp_hidden_units}"] = p_mlp


def calibration_table(p, yt, n_bins=15):
    qs = np.quantile(p, np.linspace(0, 1, n_bins + 1))
    qs[0], qs[-1] = -np.inf, np.inf
    rows = []
    for lo, hi in zip(qs[:-1], qs[1:]):
        m = (p > lo) & (p <= hi)
        if m.sum() == 0:
            continue
        rows.append((p[m].mean(), yt[m].mean(), int(m.sum())))
    return rows


for model_name, model_predictions in predictions.items():
    calibration = calibration_table(model_predictions, yte)
    ece = sum(abs(pred - obs) * n for pred, obs, n in calibration) / len(yte)
    lines.append(f"\n{model_name} expected calibration error (15 equal-frequency bins): {ece:.5f}")
    lines.append("calibration (predicted, observed, n):")
    for pred, obs, n in calibration:
        lines.append(f"  {pred:.4f}  {obs:.4f}  n={n}")
    lines.append("fixed probability bands (range, predicted, observed, n):")
    for low, high in zip(
        (0.0, 0.01, 0.02, 0.05, 0.10, 0.15, 0.25, 0.50, 0.75),
        (0.01, 0.02, 0.05, 0.10, 0.15, 0.25, 0.50, 0.75, 1.01),
    ):
        mask = (model_predictions >= low) & (model_predictions < high)
        if mask.any():
            lines.append(
                f"  [{low:.2f},{high:.2f}) {model_predictions[mask].mean():.4f} "
                f"{yte[mask].mean():.4f} n={mask.sum()}"
            )


def temporal_stability(p, indices):
    samples = df.iloc[indices][["replay_id", "is_team0", "time"]].copy()
    samples["prediction"] = p
    samples = samples.sort_values(["replay_id", "is_team0", "time"])
    group_keys = ["replay_id", "is_team0"]
    dt = samples.groupby(group_keys, sort=False)["time"].diff().to_numpy()
    delta = samples.groupby(group_keys, sort=False)["prediction"].diff().abs().to_numpy()
    max_live_sample_gap = 2.0 / args.sample_hz
    contiguous = np.isfinite(dt) & (dt > 0.0) & (dt <= max_live_sample_gap)
    contiguous_delta = delta[contiguous]
    live_minutes = float(dt[contiguous].sum() / 60.0)
    previous = samples.groupby(group_keys, sort=False)["prediction"].shift().to_numpy()
    current = samples["prediction"].to_numpy()
    crossings = {}
    for threshold in (0.05, 0.15):
        crossed = contiguous & (
            ((previous <= threshold) & (current > threshold))
            | ((previous > threshold) & (current <= threshold))
        )
        crossings[threshold] = float(crossed.sum() / live_minutes)
    return {
        "mean_abs_step": float(contiguous_delta.mean()),
        "p95_abs_step": float(np.quantile(contiguous_delta, 0.95)),
        "p99_abs_step": float(np.quantile(contiguous_delta, 0.99)),
        "crossings_005_per_minute": crossings[0.05],
        "crossings_015_per_minute": crossings[0.15],
    }


for model_name, model_predictions in predictions.items():
    stability = temporal_stability(model_predictions, test_idx)
    lines.append(f"\n{model_name} held-out temporal stability (contiguous 4 Hz samples):")
    lines.append(
        "  mean_abs_step={mean_abs_step:.5f} p95_abs_step={p95_abs_step:.5f} "
        "p99_abs_step={p99_abs_step:.5f}".format(**stability)
    )
    lines.append(
        "  crossings/min: 0.05={crossings_005_per_minute:.3f} "
        "0.15={crossings_015_per_minute:.3f}".format(**stability)
    )

# Count-scale validation: integrate overlapping five-second probabilities for
# each held-out replay/team, then compare against distinct observed goal times.
count_scale_base = df.iloc[test_idx].copy()
group_keys = ["replay_id", "is_team0"]
for model_name, model_predictions in predictions.items():
    count_scale = count_scale_base.copy()
    count_scale["prediction"] = model_predictions
    count_scale = count_scale.sort_values(["replay_id", "is_team0", "time"])
    count_scale["dt"] = count_scale.groupby(group_keys, sort=False)["time"].diff()
    # Rows exist only during live play. A large replay-time gap crosses a kickoff
    # or goal stoppage and must contribute zero rather than integrating through
    # time in which the model was not evaluated.
    max_live_sample_gap = 2.0 / args.sample_hz
    count_scale["dt"] = count_scale["dt"].where(
        count_scale["dt"].between(0.0, max_live_sample_gap), 0.0
    )
    count_scale["goal_time"] = (count_scale["time"] + count_scale["time_to_next_goal_for"]).round(2)
    count_scale["xg_contribution"] = count_scale["prediction"] * count_scale["dt"].fillna(0.0) / tau
    team_games = count_scale.groupby(group_keys, sort=False).agg(
        predicted_xg=("xg_contribution", "sum"),
        goals=("goal_time", lambda values: values.dropna().nunique()),
    )
    goal_mean = float(team_games["goals"].mean())
    xg_mean = float(team_games["predicted_xg"].mean())
    count_mae = float((team_games["predicted_xg"] - team_games["goals"]).abs().mean())
    count_rmse = float(np.sqrt(np.mean((team_games["predicted_xg"] - team_games["goals"]) ** 2)))
    count_correlation = float(team_games["predicted_xg"].corr(team_games["goals"]))
    lines.extend(
        [
            f"\n{model_name} held-out replay/team count-scale validation:",
            f"  team_games={len(team_games)} mean_xg={xg_mean:.4f} mean_goals={goal_mean:.4f} "
            f"ratio={xg_mean / goal_mean:.4f}",
            f"  mae={count_mae:.4f} rmse={count_rmse:.4f} correlation={count_correlation:.4f}",
        ]
    )

# Per-rank calibration on test set
test_df = df.iloc[test_idx]
tiers = test_df["median_rank_tier"].to_numpy()
for model_name, model_predictions in predictions.items():
    lines.append(f"\nper-rank-tier test metrics ({model_name}):")
    for tier in sorted(pd.unique(tiers[~pd.isna(tiers)])):
        m = tiers == tier
        if m.sum() < 2000 or yte[m].sum() < 20:
            lines.append(f"  tier={tier}: n={int(m.sum())} (too small, skipped)")
            continue
        lines.append(
            f"  tier={tier:g}: n={int(m.sum())} base={yte[m].mean():.5f} "
            f"pred_mean={model_predictions[m].mean():.5f} "
            f"log_loss={log_loss(yte[m], model_predictions[m]):.5f} "
            f"brier={brier_score_loss(yte[m], model_predictions[m]):.5f}"
        )

(out_dir / "metrics.txt").write_text("\n".join(lines) + "\n")
del count_scale, team_games, Xtr_s, Xte_s
gc.collect()

# After the frozen temporal evaluation, refit the MLP on the full corpus.
# Held-out metrics above always come from models fit only
# on `Xtr`; generated coefficients and parity values below come from this
# full-corpus refit.
publish_scaler = StandardScaler().fit(X)
X_publish = publish_scaler.transform(X)
mu, sigma = publish_scaler.mean_, publish_scaler.scale_

# Emitted in the exact shape of expected_goals_model_weights.rs. Literals are
# shortest-f32 so clippy's excessive_precision lint stays quiet.


def f32(x) -> str:
    text = str(np.float32(x))
    return text if any(c in text for c in ".e") else text + ".0"


rust = ["// Generated by scripts/threat_model/train_threat_model.py — do not hand-edit values.\n"]
published_estimator = new_mlp()
published_estimator.fit(X_publish, y)
hidden_weights_z = published_estimator.coefs_[0]
hidden_weights_raw = hidden_weights_z / sigma[:, np.newaxis]
hidden_biases_raw = published_estimator.intercepts_[0] - ((mu / sigma) @ hidden_weights_z)
output_weights = published_estimator.coefs_[1][:, 0]
output_bias = float(published_estimator.intercepts_[1][0])
input_weights = {
    name: [float(weight) for weight in weights]
    for name, weights in zip(feature_cols, hidden_weights_raw)
}
artifact = {
    "model": "mlp-tanh",
    "hidden_units": args.mlp_hidden_units,
    "hidden_biases": [float(value) for value in hidden_biases_raw],
    "input_weights": input_weights,
    "output_bias": output_bias,
    "output_weights": [float(value) for value in output_weights],
    "tau": tau,
    "provenance": provenance,
}
rust.append(f"pub const THREAT_MODEL_HIDDEN_UNITS: usize = {args.mlp_hidden_units};")
rust.append(
    "pub const THREAT_MODEL_HIDDEN_BIASES: [f32; THREAT_MODEL_HIDDEN_UNITS] = ["
    + ", ".join(f32(value) for value in hidden_biases_raw)
    + "];"
)
rust.append(
    "pub const THREAT_MODEL_INPUT_WEIGHTS: "
    "[(&str, [f32; THREAT_MODEL_HIDDEN_UNITS]); THREAT_MODEL_FEATURE_COUNT] = ["
)
for name in feature_cols:
    weights = ", ".join(f32(value) for value in input_weights[name])
    rust.append(f'    ("{name}", [{weights}]),')
rust.append("];")
rust.append(f"pub const THREAT_MODEL_OUTPUT_BIAS: f32 = {f32(output_bias)};")
rust.append(
    "pub const THREAT_MODEL_OUTPUT_WEIGHTS: [f32; THREAT_MODEL_HIDDEN_UNITS] = ["
    + ", ".join(f32(value) for value in output_weights)
    + "];"
)

(out_dir / "coefficients.json").write_text(json.dumps(artifact, indent=1) + "\n")
(out_dir / "model_coefficients.rs").write_text("\n".join(rust) + "\n")

# Parity fixture: 6 temporal-test rows spanning the published model's range.
p_publish_test = published_estimator.predict_proba(publish_scaler.transform(Xte))[:, 1]
order = np.argsort(p_publish_test)
picks = [
    order[0],
    order[len(order) // 4],
    order[len(order) // 2],
    order[3 * len(order) // 4],
    order[-2],
    order[-1],
]
fix = ["&[", "    // (features in ThreatModelFeatures::feature_names() order, expected_v)"]
for i in picks:
    raw = Xte[i]
    vals = ", ".join(f32(x) for x in raw)
    fix.append(f"    (&[{vals}], {f32(p_publish_test[i])}),")
fix.append("]")
(out_dir / "parity_fixture.rs").write_text("\n".join(fix) + "\n")

print(
    f"wrote {out_dir}/metrics.txt, training_provenance.json, coefficients.json, "
    "model_coefficients.rs, parity_fixture.rs"
)
