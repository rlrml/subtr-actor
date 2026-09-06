"""Publish compact evaluation and provenance for the exact deployed goal models."""

import argparse
import json
from pathlib import Path

import numpy as np
import pandas as pd

from train_entity_model import metrics, sha256


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("corpus", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--runtime-report", type=Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    audit_path = args.corpus / "entity_frames_v7.audit.jsonl"
    audit = [json.loads(line) for line in audit_path.read_text().splitlines()]
    usable = [row for row in audit if "error" not in row]
    segments = [segment for row in usable for segment in row["segments"]]
    offsets = [
        segment["end_time"] - segment["goal_time"]
        for segment in segments
        if segment["goal_time"] is not None
    ]
    split_path = args.corpus / "entity_splits_v7.json"
    splits = json.loads(split_path.read_text())
    report = {
        "version": "trained-v7-entity-precontact",
        "corpus": {
            "usable_replays": len(usable),
            "skipped_replays": len(audit) - len(usable),
            "goals": sum(len(row["goals"]) for row in usable),
            "touches_detected": sum(row["touches_detected"] for row in usable),
            "touches_exported": sum(row["touches_exported"] for row in usable),
            "missing_pre_contact_history": sum(
                row["missing_pre_contact_history"] for row in usable
            ),
            "live_segments": len(segments),
            "unresolved_segments": sum(not segment["resolved"] for segment in segments),
            "goal_boundary_offset_quantiles": np.quantile(offsets, [0, 0.5, 0.9, 0.99, 1]).tolist(),
            "nonzero_goal_boundary_offsets": sum(abs(offset) > 0.001 for offset in offsets),
            "audit_sha256": sha256(audit_path),
            "split_sha256": sha256(split_path),
        },
        "player_overlap": splits["player_overlap"],
        "partitions": {name: len(ids) for name, ids in splits["partitions"].items()},
        "tasks": {},
    }
    table = [
        "| Target | Model | Test log loss ↓ | Brier ↓ | AUC ↑ |",
        "|---|---|---:|---:|---:|",
    ]
    for task in ["frame", "touch"]:
        source = args.corpus / f"entity_{task}_v7_out"
        result = json.loads((source / "metrics.json").read_text())
        result.pop("partitions")
        result.pop("feature_names")
        selected = result["selected"]
        result["export_sha256"] = {
            name: sha256(source / selected / name)
            for name in ["model.json", "weights.rs", "parity.rs"]
        }
        if task == "touch":
            metadata = pd.read_pickle(source / "dataset/metadata.pkl")
            evaluated = metadata.loc[metadata.replay_id.isin(splits["partitions"]["test"])].copy()
            for name, model in result["models"].items():
                predicted = pd.read_csv(source / f"{name}-test.csv")
                evaluated["prediction"] = predicted.prediction.to_numpy()
                terminal = evaluated.loc[evaluated.time_to_next_touch.isna()]
                model["next_touch_interval_diagnostics"]["no_next_touch_before_live_end"] = metrics(
                    terminal.label.to_numpy(), terminal.prediction.to_numpy()
                )
            result["interval_diagnostic_caveat"] = (
                "These groups condition on future outcomes and do not measure pre-contact calibration. "
                "A finite next-touch interval ends without a goal; positive labels occur in terminal intervals."
            )
        report["tasks"][task] = result
        for name, model in result["models"].items():
            score = model["test"]
            table.append(
                f"| {task} | {name} | {score['log_loss']:.5f} | {score['brier']:.5f} | {score['auc']:.4f} |"
            )
    if args.runtime_report:
        report["runtime"] = json.loads(args.runtime_report.read_text())
    (args.output / "entity-v7-evaluation.json").write_text(
        json.dumps(report, indent=2, default=lambda value: value.item()) + "\n"
    )
    (args.output / "entity-v7-splits.json").write_bytes(split_path.read_bytes())
    touch = report["tasks"]["touch"]
    counts = touch["models"][touch["selected"]]["count_scale"]
    improvements = {
        task: 100
        * (
            1
            - result["models"]["entity"]["test"]["log_loss"]
            / result["models"]["legacy-refit"]["test"]["log_loss"]
        )
        for task, result in report["tasks"].items()
    }
    text = f"""# Expected goals v7 evaluation

The deployed entity models were selected on the tuning partition. Their exact
exported weights were then evaluated on {report["partitions"]["test"]} historical replay holdouts.
No full-corpus refit or goal-total multiplier was applied.

{chr(10).join(table)}

Relative to the eight-unit legacy model refitted for each target, test log loss
improved by {improvements["touch"]:.2f}% for pre-contact touch xG and
{improvements["frame"]:.2f}% for five-second scoring threat. Both models for a
target received the same epoch budget and early-stopping rule. See the JSON
report for complete training histories, replay-bootstrap confidence intervals,
fixed probability bands, and rank breakdowns. Both legacy fits exhausted their
epoch budgets (30 for frames, 80 for touches); their learning curves were still
improving. The comparison establishes an improvement under these training
budgets, not superiority to every fully optimized legacy fit.

On resolvable touch examples across {counts["team_games"]} test team-games,
summed touch xG averaged
{counts["mean_xg"]:.3f} versus {counts["mean_goals"]:.3f} actual goals.
Mean absolute count error was {counts["mae"]:.3f} and correlation was
{counts["correlation"]:.3f}. Eligible touch labels cover
{100 * counts["label_goal_coverage"]:.2f}% of actual goals; the remaining goals
lack an eligible touch interval or fall beyond its ten-second horizon.
Predictions exceed eligible positive labels by
{100 * (counts["mean_xg"] / counts["mean_labeled_goals"] - 1):.2f}%.
That calibration drift partly offsets the missing goal coverage; the close
aggregate mean alone does not establish calibration.

## Data and interpretation

The audit accepted {len(usable):,} ranked-doubles replays and rejected
{len(audit) - len(usable)} with ambiguous goal attachment. Of
{report["corpus"]["touches_detected"]:,} detected primary touches,
{report["corpus"]["missing_pre_contact_history"]:,} lacked a snapshot 100–200 ms
before contact. These receive unavailable xG. Unknown live-play endings are
censored when their label cannot be resolved. Full goal and coverage counts
are recorded in the JSON audit summary.

Replay-date partitions contain {report["partitions"]["train"]:,} training,
{report["partitions"]["tune"]:,} tuning, {report["partitions"]["calibration"]:,}
calibration, and {report["partitions"]["test"]:,} test replays.
There were {len(splits["duplicate_replays_excluded"])} identical replay files excluded.
Of {report["player_overlap"]["test"]["players"]:,} test players,
{report["player_overlap"]["test"]["shared_with_train"]} also occur in training. This corpus was used in earlier modeling
experiments: this is historical replay validation, not a prospective test or
evidence of generalization to new players, game modes, or patches.

Touch xG predicts conversion before the next own-team touch or live-play end,
within ten seconds. It assigns one opportunity to the primary contact when
same-team contacts coincide. It is not causal contribution credit. The separate
five-second threat model describes current danger; its integral measures
cumulative pressure. Neither episode peaks nor eventual goal outcomes change
a touch's probability.

The future-touch interval groups in the JSON are outcome-conditioned diagnostics,
not calibration tests: any interval ending at another touch necessarily has
label zero. Terminal intervals, included separately, contain the goals. Model
inputs never include future interval length.

[Machine-readable evaluation and hashes](entity-v7-evaluation.json) ·
[Frozen replay splits and exclusions](entity-v7-splits.json) ·
[Training contracts and reproduction](README.md)
"""
    if "runtime" in report:
        runtime = report["runtime"]
        pressure = runtime["threat_integral"]
        touches = runtime["touch_xg"]
        text += f"""
## Native replay validation

All {runtime["replays"]} held-out replays were processed by the deployed Rust
pipeline without rejection. Across {runtime["evaluated_touch_rows_including_unresolved_endings"]:,}
touches, native and offline per-team sums differ by at most
{runtime["maximum_absolute_touch_xg_difference"]:.8f} xG.
The native totals include predictions near unresolved replay endings whose
outcomes cannot enter the supervised evaluation above.

| Runtime quantity | Mean | Actual goals | MAE | Correlation |
|---|---:|---:|---:|---:|
| Touch xG | {touches["mean_prediction"]:.3f} | {touches["mean_goals"]:.3f} | {touches["mae"]:.3f} | {touches["correlation"]:.3f} |
| Cumulative threat | {pressure["mean_prediction"]:.3f} | {pressure["mean_goals"]:.3f} | {pressure["mae"]:.3f} | {pressure["correlation"]:.3f} |

The cumulative threat integral is approximately on a goal-count scale in this
sample. It also observes the ball after contact, so its count correlation is
not a comparison between predictors with the same information.
"""
    (args.output / "EVALUATION.md").write_text(text)


if __name__ == "__main__":
    main()
