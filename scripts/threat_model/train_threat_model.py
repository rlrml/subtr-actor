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
  - model_coefficients.rs: raw-feature logistic coefficients (standardization folded)
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
args = parser.parse_args()
if args.sample_hz <= 0:
    parser.error("--sample-hz must be positive")


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


feature_knockout("all-player-state", lambda column: column not in ball_fields, Xte_s)
feature_knockout("boost", lambda column: column.endswith("_boost"), Xte_s)
feature_knockout("dodge-available", lambda column: column.endswith("_dodge_available"), Xte_s)
feature_knockout("demo-state", lambda column: column.endswith("_demoed"), Xte_s)

if args.gbt:
    gbt = HistGradientBoostingClassifier(max_iter=300, learning_rate=0.1, random_state=args.seed)
    gbt.fit(Xtr, ytr)
    p_gbt = gbt.predict_proba(Xte)[:, 1]
    report("gbt-ceiling", p_gbt, yte)
    del gbt, p_gbt
    gc.collect()


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


calibration = calibration_table(p_lr, yte)
ece = sum(abs(pred - obs) * n for pred, obs, n in calibration) / len(yte)
lines.append(f"\nexpected calibration error (15 equal-frequency bins): {ece:.5f}")
lines.append("calibration (predicted, observed, n):")
for pred, obs, n in calibration:
    lines.append(f"  {pred:.4f}  {obs:.4f}  n={n}")

# Count-scale validation: integrate overlapping five-second probabilities for
# each held-out replay/team, then compare against distinct observed goal times.
count_scale = df.iloc[test_idx].copy()
count_scale["prediction"] = p_lr
count_scale = count_scale.sort_values(["replay_id", "is_team0", "time"])
group_keys = ["replay_id", "is_team0"]
count_scale["dt"] = count_scale.groupby(group_keys, sort=False)["time"].diff()
# Rows exist only during live play. A large replay-time gap crosses a kickoff
# or goal stoppage and must contribute zero rather than integrating through
# time in which the model was not evaluated.
max_live_sample_gap = 2.0 / args.sample_hz
count_scale["dt"] = count_scale["dt"].where(
    count_scale["dt"].between(0.0, max_live_sample_gap), 0.0
)
count_scale["xg_contribution"] = count_scale["prediction"] * count_scale["dt"].fillna(0.0) / tau
count_scale["goal_time"] = (count_scale["time"] + count_scale["time_to_next_goal_for"]).round(2)
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
        "\nheld-out replay/team count-scale validation:",
        f"  team_games={len(team_games)} mean_xg={xg_mean:.4f} mean_goals={goal_mean:.4f} "
        f"ratio={xg_mean / goal_mean:.4f}",
        f"  mae={count_mae:.4f} rmse={count_rmse:.4f} correlation={count_correlation:.4f}",
    ]
)

# Per-rank calibration on test set
test_df = df.iloc[test_idx]
lines.append("\nper-rank-tier test metrics (logistic):")
tiers = test_df["median_rank_tier"].to_numpy()
for tier in sorted(pd.unique(tiers[~pd.isna(tiers)])):
    m = tiers == tier
    if m.sum() < 2000 or yte[m].sum() < 20:
        lines.append(f"  tier={tier}: n={int(m.sum())} (too small, skipped)")
        continue
    lines.append(
        f"  tier={tier:g}: n={int(m.sum())} base={yte[m].mean():.5f} pred_mean={p_lr[m].mean():.5f} "
        f"log_loss={log_loss(yte[m], p_lr[m]):.5f} brier={brier_score_loss(yte[m], p_lr[m]):.5f}"
    )

(out_dir / "metrics.txt").write_text("\n".join(lines) + "\n")
del count_scale, team_games, Xtr_s, Xte_s
gc.collect()

# After the frozen temporal evaluation, refit the publishable coefficients on
# the full corpus. Held-out metrics above always come from `lr`; generated
# coefficients and parity values below always come from `publish_lr`.
publish_scaler = StandardScaler().fit(X)
X_publish = publish_scaler.transform(X)
publish_lr = LogisticRegression(max_iter=2000, C=1.0)
publish_lr.fit(X_publish, y)

# Fold standardization into raw-feature coefficients:
# z = (x - mu) / sigma ; w_z . z + b = (w_z / sigma) . x + (b - w_z . mu / sigma)
w_z = publish_lr.coef_[0]
mu, sigma = publish_scaler.mean_, publish_scaler.scale_
w_raw = w_z / sigma
b_raw = float(publish_lr.intercept_[0] - np.sum(w_z * mu / sigma))

coeffs = {name: float(w) for name, w in zip(feature_cols, w_raw)}
(out_dir / "coefficients.json").write_text(
    json.dumps(
        {"bias": b_raw, "weights": coeffs, "tau": tau, "provenance": provenance},
        indent=1,
    )
)

# Emitted in the exact shape of the GENERATED COEFFICIENTS section of
# src/stats/calculators/expected_goals_model.rs; paste between the markers and
# bump THREAT_MODEL_VERSION. Literals are shortest-f32 so clippy's
# excessive_precision lint stays quiet.


def f32(x) -> str:
    text = str(np.float32(x))
    return text if any(c in text for c in ".e") else text + ".0"


rust = ["// Generated by scripts/threat_model/train_threat_model.py — do not hand-edit values.\n"]
rust.append(f"pub const THREAT_MODEL_BIAS: f32 = {f32(b_raw)};")
rust.append("pub const THREAT_MODEL_WEIGHTS: [(&str, f32); THREAT_FEATURE_COUNT] = [")
for name in feature_cols:
    rust.append(f'    ("{name}", {f32(coeffs[name])}),')
rust.append("];")
(out_dir / "model_coefficients.rs").write_text("\n".join(rust) + "\n")

# Parity fixture: 6 temporal-test rows spanning the published model's range.
p_publish_test = publish_lr.predict_proba(publish_scaler.transform(Xte))[:, 1]
order = np.argsort(p_publish_test)
picks = [
    order[0],
    order[len(order) // 4],
    order[len(order) // 2],
    order[3 * len(order) // 4],
    order[-2],
    order[-1],
]
fix = ["// (features in FEATURE_NAMES order, expected_v)"]
for i in picks:
    raw = Xte[i]
    vals = ", ".join(f32(x) for x in raw)
    fix.append(f"(&[{vals}], {f32(p_publish_test[i])}),")
(out_dir / "parity_fixture.rs").write_text("\n".join(fix) + "\n")

print(
    f"wrote {out_dir}/metrics.txt, training_provenance.json, coefficients.json, "
    "model_coefficients.rs, parity_fixture.rs"
)
