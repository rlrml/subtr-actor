"""Compare native replay totals with offline predictions on every exported touch."""

import argparse
import json
from pathlib import Path

import numpy as np
import pandas as pd
import torch

from entity_model import GoalModel
from train_entity_model import probabilities, sha256


def count_metrics(predicted, goals):
    return {
        "mean_prediction": float(predicted.mean()),
        "mean_goals": float(goals.mean()),
        "mae": float((predicted - goals).abs().mean()),
        "rmse": float(np.sqrt(np.mean((predicted - goals) ** 2))),
        "correlation": float(predicted.corr(goals)),
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("corpus", type=Path)
    parser.add_argument("runtime_summary", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    torch.set_num_threads(2)
    native = pd.read_csv(args.runtime_summary).set_index(["replay_id", "is_team0"])
    splits = json.loads((args.corpus / "entity_splits_v7.json").read_text())
    ids = set(splits["partitions"]["test"])
    if set(native.index.get_level_values("replay_id")) != ids or len(native) != 2 * len(ids):
        raise ValueError("Native summary must cover both teams of every frozen test replay")
    artifact = json.loads((args.corpus / "entity_touch_v7_out/entity/model.json").read_text())
    state = artifact["weights"]
    model = GoalModel(
        state["mean"], state["scale"], artifact["entity"], reflection=state["reflection"]
    )
    model.load_state_dict({name: torch.tensor(value) for name, value in state.items()})
    model.eval()
    totals = []
    touch_rows = 0
    for chunk in pd.read_csv(
        args.corpus / "entity_touches_v7.csv",
        usecols=["replay_id", "is_team0", *artifact["feature_names"]],
        chunksize=50_000,
    ):
        selected = chunk.loc[chunk.replay_id.isin(ids)].copy()
        if selected.empty:
            continue
        values = selected[artifact["feature_names"]].to_numpy(dtype=np.float32)
        with torch.no_grad():
            logits = model(torch.from_numpy(values)).numpy()
        selected["prediction"] = probabilities(
            logits * artifact["calibration_slope"] + artifact["calibration_intercept"]
        ).astype(np.float64)
        totals.append(selected.groupby(["replay_id", "is_team0"]).prediction.sum())
        touch_rows += len(selected)
    offline = pd.concat(totals).groupby(level=[0, 1]).sum().reindex(native.index, fill_value=0)
    differences = (offline - native.touch_xg).abs()
    report = {
        "replays": len(ids),
        "team_games": len(native),
        "evaluated_touch_rows_including_unresolved_endings": touch_rows,
        "maximum_absolute_touch_xg_difference": float(differences.max()),
        "tolerance": 0.001,
        "touch_xg": count_metrics(native.touch_xg, native.goals),
        "threat_integral": count_metrics(native.threat_integral, native.goals),
        "runtime_summary_sha256": sha256(args.runtime_summary),
    }
    args.out.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))
    if differences.max() > report["tolerance"]:
        raise AssertionError(f"Native/offline touch totals diverge: {differences.nlargest(5)}")


if __name__ == "__main__":
    main()
