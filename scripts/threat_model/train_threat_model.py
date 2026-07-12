#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "numpy>=1.26",
#     "pandas>=2.1",
#     "scikit-learn>=1.4",
# ]
# ///
"""Train the subtr-actor expected-goals threat model.

Run with `uv run train_threat_model.py ...` — the PEP 723 block above (and
the adjacent uv lock) pin the training environment so published coefficients
have reproducible provenance.

Input: CSV from `threat_dataset_dump` with columns
  replay_id, playlist, min_rank_tier, max_rank_tier, team_size, is_team0, time,
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
import json
import pathlib
import sys

import numpy as np
import pandas as pd
from sklearn.ensemble import HistGradientBoostingClassifier
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import brier_score_loss, log_loss, roc_auc_score
from sklearn.model_selection import GroupShuffleSplit
from sklearn.preprocessing import StandardScaler

META_COLS = [
    "replay_id",
    "playlist",
    "min_rank_tier",
    "max_rank_tier",
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
parser.add_argument("--out-dir", default="threat_model_out")
parser.add_argument("--test-frac", type=float, default=0.2)
parser.add_argument("--seed", type=int, default=7)
parser.add_argument("--gbt", action="store_true", help="also fit a GBT ceiling reference")
args = parser.parse_args()

out_dir = pathlib.Path(args.out_dir)
out_dir.mkdir(parents=True, exist_ok=True)

df = pd.read_csv(args.csv)
feature_cols = [c for c in df.columns if c not in META_COLS]
print(f"rows={len(df)} features={feature_cols}")

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
X = df[feature_cols].to_numpy(dtype=np.float64)
groups = df["replay_id"].to_numpy()

print(f"kept={len(df)} censored_dropped={int(censored.sum())} base_rate={y.mean():.5f}")

bad = ~np.isfinite(X).all(axis=0)
if bad.any():
    print("WARNING: non-finite feature columns:", [feature_cols[i] for i in np.where(bad)[0]])
    X = np.nan_to_num(X, nan=0.0, posinf=0.0, neginf=0.0)

splitter = GroupShuffleSplit(n_splits=1, test_size=args.test_frac, random_state=args.seed)
train_idx, test_idx = next(splitter.split(X, y, groups))
Xtr, Xte, ytr, yte = X[train_idx], X[test_idx], y[train_idx], y[test_idx]
print(f"train={len(train_idx)} test={len(test_idx)} (grouped by replay)")

scaler = StandardScaler().fit(Xtr)
Xtr_s, Xte_s = scaler.transform(Xtr), scaler.transform(Xte)

lr = LogisticRegression(max_iter=2000, C=1.0)
lr.fit(Xtr_s, ytr)
p_lr = lr.predict_proba(Xte_s)[:, 1]

lines = []


def report(name, p, yt):
    base = np.full_like(p, yt.mean() if name.endswith("(train-rate)") else ytr.mean())
    msg = (
        f"{name}: log_loss={log_loss(yt, p):.5f} brier={brier_score_loss(yt, p):.5f} "
        f"auc={roc_auc_score(yt, p):.4f} | baseline log_loss={log_loss(yt, base):.5f} "
        f"brier={brier_score_loss(yt, base):.5f}"
    )
    print(msg)
    lines.append(msg)


report("logistic", p_lr, yte)

if args.gbt:
    gbt = HistGradientBoostingClassifier(max_iter=300, learning_rate=0.1, random_state=args.seed)
    gbt.fit(Xtr, ytr)
    p_gbt = gbt.predict_proba(Xte)[:, 1]
    report("gbt-ceiling", p_gbt, yte)


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


lines.append("\ncalibration (predicted, observed, n):")
for pred, obs, n in calibration_table(p_lr, yte):
    lines.append(f"  {pred:.4f}  {obs:.4f}  n={n}")

# Per-rank calibration on test set
test_df = df.iloc[test_idx]
lines.append("\nper-rank-tier test metrics (logistic):")
tiers = test_df["min_rank_tier"].to_numpy()
for tier in sorted(pd.unique(tiers[~pd.isna(tiers)])):
    m = tiers == tier
    if m.sum() < 2000 or yte[m].sum() < 20:
        lines.append(f"  tier={tier}: n={int(m.sum())} (too small, skipped)")
        continue
    lines.append(
        f"  tier={int(tier)}: n={int(m.sum())} base={yte[m].mean():.5f} pred_mean={p_lr[m].mean():.5f} "
        f"log_loss={log_loss(yte[m], p_lr[m]):.5f} brier={brier_score_loss(yte[m], p_lr[m]):.5f}"
    )

(out_dir / "metrics.txt").write_text("\n".join(lines) + "\n")

# Fold standardization into raw-feature coefficients:
# z = (x - mu) / sigma ; w_z . z + b = (w_z / sigma) . x + (b - w_z . mu / sigma)
w_z = lr.coef_[0]
mu, sigma = scaler.mean_, scaler.scale_
w_raw = w_z / sigma
b_raw = float(lr.intercept_[0] - np.sum(w_z * mu / sigma))

coeffs = {name: float(w) for name, w in zip(feature_cols, w_raw)}
(out_dir / "coefficients.json").write_text(
    json.dumps({"bias": b_raw, "weights": coeffs, "tau": tau}, indent=1)
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

# Parity fixture: 6 test rows spanning the prediction range
order = np.argsort(p_lr)
picks = [order[0], order[len(order) // 4], order[len(order) // 2], order[3 * len(order) // 4], order[-2], order[-1]]
fix = ["// (features in FEATURE_NAMES order, expected_v)"]
for i in picks:
    raw = Xte[i]
    vals = ", ".join(f32(x) for x in raw)
    fix.append(f"(&[{vals}], {f32(p_lr[i])}),")
(out_dir / "parity_fixture.rs").write_text("\n".join(fix) + "\n")

print(f"wrote {out_dir}/metrics.txt, coefficients.json, model_coefficients.rs, parity_fixture.rs")
