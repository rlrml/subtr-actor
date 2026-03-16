from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import torch

from .config import ReplayFeatureConfig
from .dataset import ReplayTensorData, load_replay_paths, load_replay_tensor_data
from .model import FlipResetTransformer, FlipResetTransformerConfig


@dataclass(frozen=True)
class MatchWindow:
    before_seconds: float = 0.20
    after_seconds: float = 0.05

    def contains(self, signed_delta_seconds: float) -> bool:
        return -self.before_seconds <= signed_delta_seconds <= self.after_seconds


@dataclass(frozen=True)
class EventSummary:
    replay_path: str
    player_name: str
    time: float
    score: float


@dataclass
class PlayerPrediction:
    player_name: str
    times: np.ndarray
    frame_probabilities: np.ndarray


@dataclass
class ReplayPredictionBundle:
    replay_path: Path
    exact_events: list[EventSummary]
    player_predictions: list[PlayerPrediction]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Evaluate a saved flip reset checkpoint on its validation split")
    parser.add_argument("--training-run", type=Path, required=True)
    parser.add_argument("--checkpoint", type=Path, default=None)
    parser.add_argument("--device", type=str, default="auto")
    parser.add_argument("--replay-cache-dir", type=Path, default=None)
    parser.add_argument("--replay-dir", type=Path, action="append", default=[])
    parser.add_argument("--replay-file", type=Path, action="append", default=[])
    parser.add_argument("--non-recursive-replay-search", action="store_true")
    parser.add_argument("--batch-size", type=int, default=256)
    parser.add_argument("--threshold", type=float, default=None)
    parser.add_argument("--sweep-thresholds", action="store_true")
    parser.add_argument("--threshold-min", type=float, default=0.05)
    parser.add_argument("--threshold-max", type=float, default=0.95)
    parser.add_argument("--threshold-steps", type=int, default=19)
    parser.add_argument("--match-window-before", type=float, default=0.20)
    parser.add_argument("--match-window-after", type=float, default=0.05)
    parser.add_argument("--debounce-seconds", type=float, default=0.10)
    parser.add_argument("--output", type=Path, default=None)
    return parser.parse_args()


def load_training_artifacts(
    training_run_path: Path,
    checkpoint_path: Path | None,
    device: torch.device,
) -> tuple[dict[str, object], dict[str, object]]:
    training_run = json.loads(training_run_path.read_text())
    if checkpoint_path is None:
        checkpoint_path = Path(training_run["best_model_path"])
    checkpoint = torch.load(checkpoint_path, map_location=device)
    return training_run, checkpoint


def replay_feature_config_from_training_run(training_run: dict[str, object]) -> ReplayFeatureConfig:
    config = training_run["config"]["replay_features"]
    return ReplayFeatureConfig(
        fps=float(config["fps"]),
        global_feature_adders=tuple(config["global_feature_adders"]),
        player_feature_adders=tuple(config["player_feature_adders"]),
        include_geometry_scalars=bool(config.get("include_geometry_scalars", False)),
    )


def resolve_device(device_arg: str) -> torch.device:
    if device_arg == "auto":
        if torch.cuda.is_available():
            return torch.device("cuda")
        return torch.device("cpu")
    return torch.device(device_arg)


def build_model(checkpoint: dict[str, object], device: torch.device) -> FlipResetTransformer:
    model = FlipResetTransformer(FlipResetTransformerConfig(**checkpoint["model_config"]))
    model.load_state_dict(checkpoint["model_state"])
    model.to(device)
    model.eval()
    return model


def standardize_replay_features(
    replay_data: ReplayTensorData,
    mean: np.ndarray,
    std: np.ndarray,
) -> np.ndarray:
    standardized = (replay_data.token_features_by_player - mean[None, None, :]) / std[None, None, :]
    return standardized.astype(np.float32)


def build_window_batches(
    player_features: np.ndarray,
    relative_times: np.ndarray,
    window_radius: int,
) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    frame_count = player_features.shape[0]
    if frame_count <= window_radius * 2:
        return (
            np.empty((0, window_radius * 2 + 1, player_features.shape[1]), dtype=np.float32),
            np.empty((0, window_radius * 2 + 1), dtype=np.float32),
            np.empty((0,), dtype=np.int32),
        )

    valid_centers = np.arange(window_radius, frame_count - window_radius, dtype=np.int32)
    window_length = window_radius * 2 + 1
    features = np.empty((valid_centers.shape[0], window_length, player_features.shape[1]), dtype=np.float32)
    window_relative_times = np.empty((valid_centers.shape[0], window_length), dtype=np.float32)
    for row_index, center_frame in enumerate(valid_centers):
        frame_slice = slice(center_frame - window_radius, center_frame + window_radius + 1)
        center_time = relative_times[center_frame]
        features[row_index] = player_features[frame_slice]
        window_relative_times[row_index] = (relative_times[frame_slice] - center_time).astype(np.float32)
    return features, window_relative_times, valid_centers


def predict_frame_probabilities(
    model: FlipResetTransformer,
    player_features: np.ndarray,
    relative_times: np.ndarray,
    window_radius: int,
    batch_size: int,
    device: torch.device,
) -> np.ndarray:
    window_features, window_relative_times, valid_centers = build_window_batches(
        player_features,
        relative_times,
        window_radius,
    )
    frame_count = player_features.shape[0]
    if valid_centers.size == 0:
        return np.zeros(frame_count, dtype=np.float32)

    probability_sums = np.zeros(frame_count, dtype=np.float32)
    probability_counts = np.zeros(frame_count, dtype=np.float32)
    frame_offsets = np.arange(-window_radius, window_radius + 1, dtype=np.int32)
    with torch.inference_mode():
        for batch_start in range(0, valid_centers.shape[0], batch_size):
            batch_end = min(batch_start + batch_size, valid_centers.shape[0])
            batch_features = torch.from_numpy(window_features[batch_start:batch_end]).to(device)
            batch_relative_times = torch.from_numpy(window_relative_times[batch_start:batch_end]).to(device)
            logits = model(
                batch_features,
                batch_relative_times,
            )
            probabilities = torch.sigmoid(logits).cpu().numpy()
            batch_centers = valid_centers[batch_start:batch_end]
            batch_indices = batch_centers[:, None] + frame_offsets[None, :]
            np.add.at(probability_sums, batch_indices, probabilities)
            np.add.at(probability_counts, batch_indices, 1.0)

    averaged = np.zeros_like(probability_sums)
    nonzero_mask = probability_counts > 0
    averaged[nonzero_mask] = probability_sums[nonzero_mask] / probability_counts[nonzero_mask]
    return averaged


def cluster_predicted_events(
    replay_path: Path,
    player_name: str,
    times: np.ndarray,
    frame_probabilities: np.ndarray,
    threshold: float,
) -> list[EventSummary]:
    predicted_frames = np.flatnonzero(frame_probabilities >= threshold)
    if predicted_frames.size == 0:
        return []

    events: list[EventSummary] = []
    cluster_start = 0
    for index in range(1, predicted_frames.size + 1):
        is_cluster_end = index == predicted_frames.size or predicted_frames[index] != predicted_frames[index - 1] + 1
        if not is_cluster_end:
            continue
        cluster_frames = predicted_frames[cluster_start:index]
        best_frame = cluster_frames[np.argmax(frame_probabilities[cluster_frames])]
        events.append(
            EventSummary(
                replay_path=str(replay_path),
                player_name=player_name,
                time=float(times[best_frame]),
                score=float(frame_probabilities[best_frame]),
            )
        )
        cluster_start = index
    return events


def debounce_predicted_events(
    events: list[EventSummary],
    debounce_seconds: float,
) -> list[EventSummary]:
    if debounce_seconds <= 0.0:
        return events

    debounced: list[EventSummary] = []
    for player_name in sorted({event.player_name for event in events}):
        player_events = sorted(
            (event for event in events if event.player_name == player_name),
            key=lambda event: event.time,
        )
        if not player_events:
            continue
        current = player_events[0]
        for event in player_events[1:]:
            if event.time - current.time <= debounce_seconds:
                if event.score > current.score:
                    current = event
                continue
            debounced.append(current)
            current = event
        debounced.append(current)
    return debounced


def exact_events_for_player(
    replay_path: Path,
    player_name: str,
    times: np.ndarray,
    label_counts: np.ndarray,
) -> list[EventSummary]:
    exact_frames = np.flatnonzero(label_counts > 0)
    return [
        EventSummary(
            replay_path=str(replay_path),
            player_name=player_name,
            time=float(times[frame]),
            score=float(label_counts[frame]),
        )
        for frame in exact_frames
    ]


def greedy_match_events(
    exact_events: list[EventSummary],
    predicted_events: list[EventSummary],
    match_window: MatchWindow,
) -> tuple[int, int, int]:
    matched_exact = [False] * len(exact_events)
    matched_predicted = [False] * len(predicted_events)

    for exact_index, exact_event in enumerate(exact_events):
        best_candidate_index: int | None = None
        best_abs_delta = float("inf")
        best_score = float("-inf")
        for predicted_index, predicted_event in enumerate(predicted_events):
            if matched_predicted[predicted_index]:
                continue
            if predicted_event.player_name != exact_event.player_name:
                continue
            signed_delta = predicted_event.time - exact_event.time
            if not match_window.contains(signed_delta):
                continue
            abs_delta = abs(signed_delta)
            if abs_delta < best_abs_delta or (
                abs_delta == best_abs_delta and predicted_event.score > best_score
            ):
                best_candidate_index = predicted_index
                best_abs_delta = abs_delta
                best_score = predicted_event.score

        if best_candidate_index is None:
            continue
        matched_exact[exact_index] = True
        matched_predicted[best_candidate_index] = True

    true_positive = sum(matched_exact)
    false_negative = len(exact_events) - true_positive
    false_positive = len(predicted_events) - sum(matched_predicted)
    return true_positive, false_positive, false_negative


def evaluate_bundles_for_threshold(
    bundles: list[ReplayPredictionBundle],
    threshold: float,
    debounce_seconds: float,
    match_window: MatchWindow,
) -> dict[str, object]:
    total_true_positive = 0
    total_false_positive = 0
    total_false_negative = 0
    exact_event_count = 0
    predicted_event_count = 0
    replay_summaries: list[dict[str, object]] = []

    for bundle in bundles:
        replay_predicted_events: list[EventSummary] = []
        for player_prediction in bundle.player_predictions:
            replay_predicted_events.extend(
                cluster_predicted_events(
                    bundle.replay_path,
                    player_prediction.player_name,
                    player_prediction.times,
                    player_prediction.frame_probabilities,
                    threshold,
                )
            )

        replay_predicted_events = debounce_predicted_events(
            replay_predicted_events,
            debounce_seconds,
        )
        true_positive, false_positive, false_negative = greedy_match_events(
            bundle.exact_events,
            replay_predicted_events,
            match_window,
        )
        exact_event_count += len(bundle.exact_events)
        predicted_event_count += len(replay_predicted_events)
        total_true_positive += true_positive
        total_false_positive += false_positive
        total_false_negative += false_negative
        replay_summaries.append(
            {
                "replay_path": str(bundle.replay_path),
                "exact_event_count": len(bundle.exact_events),
                "predicted_event_count": len(replay_predicted_events),
                "true_positive": true_positive,
                "false_positive": false_positive,
                "false_negative": false_negative,
            }
        )

    precision = total_true_positive / max(total_true_positive + total_false_positive, 1)
    recall = total_true_positive / max(total_true_positive + total_false_negative, 1)
    f1 = 2.0 * precision * recall / max(precision + recall, 1e-6)
    return {
        "threshold": threshold,
        "precision": precision,
        "recall": recall,
        "f1": f1,
        "true_positive": total_true_positive,
        "false_positive": total_false_positive,
        "false_negative": total_false_negative,
        "exact_event_count": exact_event_count,
        "predicted_event_count": predicted_event_count,
        "replay_summaries": replay_summaries,
    }


def evaluate_checkpoint(args: argparse.Namespace) -> dict[str, object]:
    device = resolve_device(args.device)
    training_run, checkpoint = load_training_artifacts(
        args.training_run.resolve(),
        args.checkpoint.resolve() if args.checkpoint is not None else None,
        device,
    )
    model = build_model(checkpoint, device)
    threshold = (
        float(args.threshold)
        if args.threshold is not None
        else float(checkpoint["best_validation_threshold"])
    )
    replay_cache_dir = (
        args.replay_cache_dir.resolve()
        if args.replay_cache_dir is not None
        else Path(training_run["replay_cache_dir"]).resolve()
    )
    replay_feature_config = replay_feature_config_from_training_run(training_run)
    mean = np.asarray(checkpoint["normalization_mean"], dtype=np.float32)
    std = np.asarray(checkpoint["normalization_std"], dtype=np.float32)
    if args.replay_dir or args.replay_file:
        validation_replays = load_replay_paths(
            replay_dirs=args.replay_dir,
            replay_files=args.replay_file,
            recursive=not args.non_recursive_replay_search,
        )
    else:
        validation_replays = [Path(path) for path in checkpoint["validation_replays"]]
    model_config = FlipResetTransformerConfig(**checkpoint["model_config"])
    match_window = MatchWindow(
        before_seconds=float(args.match_window_before),
        after_seconds=float(args.match_window_after),
    )

    bundles: list[ReplayPredictionBundle] = []

    for replay_path in validation_replays:
        replay_data = load_replay_tensor_data(
            replay_path,
            replay_feature_config,
            cache_dir=replay_cache_dir,
        )
        standardized = standardize_replay_features(replay_data, mean, std)
        replay_exact_events: list[EventSummary] = []
        player_predictions: list[PlayerPrediction] = []

        for player_index, player_name in enumerate(replay_data.player_names):
            frame_probabilities = predict_frame_probabilities(
                model,
                standardized[player_index],
                replay_data.relative_times,
                model_config.window_length // 2,
                batch_size=int(args.batch_size),
                device=device,
            )
            replay_exact_events.extend(
                exact_events_for_player(
                    replay_path,
                    player_name,
                    replay_data.relative_times,
                    replay_data.label_counts_by_player[player_index],
                )
            )
            player_predictions.append(
                PlayerPrediction(
                    player_name=player_name,
                    times=replay_data.relative_times,
                    frame_probabilities=frame_probabilities,
                )
            )

        bundles.append(
            ReplayPredictionBundle(
                replay_path=replay_path,
                exact_events=replay_exact_events,
                player_predictions=player_predictions,
            )
        )

    result = evaluate_bundles_for_threshold(
        bundles,
        threshold,
        float(args.debounce_seconds),
        match_window,
    )
    threshold_sweep: list[dict[str, float]] = []
    best_threshold_result: dict[str, object] | None = None
    if args.sweep_thresholds:
        for sweep_threshold in np.linspace(
            float(args.threshold_min),
            float(args.threshold_max),
            int(args.threshold_steps),
        ):
            sweep_result = evaluate_bundles_for_threshold(
                bundles,
                float(sweep_threshold),
                float(args.debounce_seconds),
                match_window,
            )
            threshold_sweep.append(
                {
                    "threshold": float(sweep_result["threshold"]),
                    "precision": float(sweep_result["precision"]),
                    "recall": float(sweep_result["recall"]),
                    "f1": float(sweep_result["f1"]),
                    "predicted_event_count": int(sweep_result["predicted_event_count"]),
                }
            )
            if best_threshold_result is None or float(sweep_result["f1"]) > float(best_threshold_result["f1"]):
                best_threshold_result = sweep_result

    result = {
        "checkpoint_path": str(args.checkpoint or training_run["best_model_path"]),
        "device": str(device),
        "validation_replay_count": len(validation_replays),
        "debounce_seconds": float(args.debounce_seconds),
        "match_window_before": match_window.before_seconds,
        "match_window_after": match_window.after_seconds,
        **result,
    }
    if threshold_sweep:
        result["threshold_sweep"] = threshold_sweep
    if best_threshold_result is not None:
        result["best_threshold_result"] = {
            "threshold": float(best_threshold_result["threshold"]),
            "precision": float(best_threshold_result["precision"]),
            "recall": float(best_threshold_result["recall"]),
            "f1": float(best_threshold_result["f1"]),
            "true_positive": int(best_threshold_result["true_positive"]),
            "false_positive": int(best_threshold_result["false_positive"]),
            "false_negative": int(best_threshold_result["false_negative"]),
            "predicted_event_count": int(best_threshold_result["predicted_event_count"]),
            "exact_event_count": int(best_threshold_result["exact_event_count"]),
        }
    return result


def main() -> None:
    args = parse_args()
    result = evaluate_checkpoint(args)
    output_path = (
        args.output.resolve()
        if args.output is not None
        else args.training_run.resolve().parent / "event_level_eval.json"
    )
    output_path.write_text(json.dumps(result, indent=2))
    print(json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    main()
