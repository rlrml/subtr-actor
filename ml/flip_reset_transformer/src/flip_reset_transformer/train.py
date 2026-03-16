from __future__ import annotations

import argparse
import gc
import json
from contextlib import nullcontext
from dataclasses import asdict
from pathlib import Path

import numpy as np
import torch
from torch import nn
from torch.nn import functional as F
from torch.utils.data import DataLoader, TensorDataset, WeightedRandomSampler

from .config import ReplayFeatureConfig, TrainConfig, WindowSamplingConfig
from .dataset import (
    ReplayTensorData,
    build_windowed_training_data,
    load_replay_paths,
    load_replay_tensor_data,
)
from .features import (
    global_feature_adders_for_orientation,
    player_feature_adders_for_orientation,
)
from .model import FlipResetTransformer, FlipResetTransformerConfig


def parse_args() -> TrainConfig:
    parser = argparse.ArgumentParser(description="Train a baseline flip reset transformer")
    parser.add_argument("--replay-dir", type=Path, action="append", default=[])
    parser.add_argument("--replay-file", type=Path, action="append", default=[])
    parser.add_argument("--non-recursive-replay-search", action="store_true")
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--replay-cache-dir", type=Path, default=None)
    parser.add_argument("--require-positive-replays", action="store_true")
    parser.add_argument("--include-geometry-features", action="store_true")
    parser.add_argument("--orientation-encoding", choices=("euler", "quaternion", "basis"), default="euler")
    parser.add_argument("--fps", type=float, default=30.0)
    parser.add_argument("--window-radius", type=int, default=45)
    parser.add_argument("--positive-radius-frames", type=int, default=1)
    parser.add_argument("--negative-to-positive-ratio", type=float, default=4.0)
    parser.add_argument("--negative-only-replay-sample-count", type=int, default=64)
    parser.add_argument("--validation-ratio", type=float, default=0.2)
    parser.add_argument("--epochs", type=int, default=10)
    parser.add_argument("--batch-size", type=int, default=128)
    parser.add_argument("--learning-rate", type=float, default=3e-4)
    parser.add_argument("--weight-decay", type=float, default=1e-4)
    parser.add_argument("--model-dim", type=int, default=128)
    parser.add_argument("--encoder-type", choices=("transformer", "tcn"), default="transformer")
    parser.add_argument("--time-feature-dim", type=int, default=9)
    parser.add_argument("--no-additive-position-embedding", action="store_true")
    parser.add_argument("--cnn-layers", type=int, default=2)
    parser.add_argument("--cnn-kernel-size", type=int, default=5)
    parser.add_argument("--encoder-layers", type=int, default=4)
    parser.add_argument("--attention-heads", type=int, default=4)
    parser.add_argument("--feedforward-dim", type=int, default=256)
    parser.add_argument("--dropout", type=float, default=0.1)
    parser.add_argument("--event-match-radius-frames", type=int, default=3)
    parser.add_argument("--cover-loss-weight", type=float, default=1.0)
    parser.add_argument("--false-positive-loss-weight", type=float, default=1.0)
    parser.add_argument("--count-loss-weight", type=float, default=0.25)
    parser.add_argument("--exact-bce-loss-weight", type=float, default=0.0)
    parser.add_argument("--exact-positive-weight", type=float, default=1.0)
    parser.add_argument("--compute-train-metrics", action="store_true")
    parser.add_argument("--sweep-train-thresholds", action="store_true")
    parser.add_argument("--pretrained-encoder-checkpoint", type=Path, default=None)
    parser.add_argument("--seed", type=int, default=7)
    args = parser.parse_args()
    if not args.replay_dir and not args.replay_file:
        raise ValueError("Provide --replay-dir or --replay-file")

    return TrainConfig(
        output_dir=args.output_dir.resolve(),
        replay_dirs=tuple(path.resolve() for path in args.replay_dir),
        replay_files=tuple(path.resolve() for path in args.replay_file),
        recursive_replay_search=not args.non_recursive_replay_search,
        replay_cache_dir=args.replay_cache_dir.resolve() if args.replay_cache_dir is not None else None,
        pretrained_encoder_checkpoint=args.pretrained_encoder_checkpoint.resolve()
        if args.pretrained_encoder_checkpoint is not None
        else None,
        require_positive_replays=args.require_positive_replays,
        replay_features=ReplayFeatureConfig(
            fps=args.fps,
            global_feature_adders=global_feature_adders_for_orientation(args.orientation_encoding),
            player_feature_adders=player_feature_adders_for_orientation(args.orientation_encoding),
            include_geometry_scalars=args.include_geometry_features,
        ),
        sampling=WindowSamplingConfig(
            window_radius=args.window_radius,
            positive_radius_frames=args.positive_radius_frames,
            negative_to_positive_ratio=args.negative_to_positive_ratio,
            negative_only_replay_sample_count=args.negative_only_replay_sample_count,
            random_seed=args.seed,
        ),
        validation_ratio=args.validation_ratio,
        epochs=args.epochs,
        batch_size=args.batch_size,
        learning_rate=args.learning_rate,
        weight_decay=args.weight_decay,
        model_dim=args.model_dim,
        encoder_type=args.encoder_type,
        time_feature_dim=args.time_feature_dim,
        additive_position_embedding=not args.no_additive_position_embedding,
        cnn_layers=args.cnn_layers,
        cnn_kernel_size=args.cnn_kernel_size,
        encoder_layers=args.encoder_layers,
        attention_heads=args.attention_heads,
        feedforward_dim=args.feedforward_dim,
        dropout=args.dropout,
        event_match_radius_frames=args.event_match_radius_frames,
        cover_loss_weight=args.cover_loss_weight,
        false_positive_loss_weight=args.false_positive_loss_weight,
        count_loss_weight=args.count_loss_weight,
        exact_bce_loss_weight=args.exact_bce_loss_weight,
        exact_positive_weight=args.exact_positive_weight,
        compute_train_metrics=args.compute_train_metrics,
        sweep_train_thresholds=args.sweep_train_thresholds,
    )


def train_config_to_json(config: TrainConfig) -> dict[str, object]:
    return {
        "replay_dirs": [str(path) for path in config.replay_dirs],
        "replay_files": [str(path) for path in config.replay_files],
        "recursive_replay_search": config.recursive_replay_search,
        "output_dir": str(config.output_dir),
        "replay_cache_dir": str(config.replay_cache_dir) if config.replay_cache_dir is not None else None,
        "pretrained_encoder_checkpoint": str(config.pretrained_encoder_checkpoint)
        if config.pretrained_encoder_checkpoint is not None
        else None,
        "require_positive_replays": config.require_positive_replays,
        "replay_features": asdict(config.replay_features),
        "sampling": asdict(config.sampling),
        "validation_ratio": config.validation_ratio,
        "epochs": config.epochs,
        "batch_size": config.batch_size,
        "learning_rate": config.learning_rate,
        "weight_decay": config.weight_decay,
        "model_dim": config.model_dim,
        "encoder_type": config.encoder_type,
        "time_feature_dim": config.time_feature_dim,
        "additive_position_embedding": config.additive_position_embedding,
        "cnn_layers": config.cnn_layers,
        "cnn_kernel_size": config.cnn_kernel_size,
        "encoder_layers": config.encoder_layers,
        "attention_heads": config.attention_heads,
        "feedforward_dim": config.feedforward_dim,
        "dropout": config.dropout,
        "event_match_radius_frames": config.event_match_radius_frames,
        "cover_loss_weight": config.cover_loss_weight,
        "false_positive_loss_weight": config.false_positive_loss_weight,
        "count_loss_weight": config.count_loss_weight,
        "exact_bce_loss_weight": config.exact_bce_loss_weight,
        "exact_positive_weight": config.exact_positive_weight,
        "compute_train_metrics": config.compute_train_metrics,
        "sweep_train_thresholds": config.sweep_train_thresholds,
    }


def split_replay_paths(
    replay_paths: list[Path], validation_ratio: float, seed: int
) -> tuple[list[Path], list[Path]]:
    if len(replay_paths) < 2:
        return replay_paths, replay_paths

    rng = np.random.default_rng(seed)
    indices = rng.permutation(len(replay_paths))
    validation_count = max(1, int(round(len(replay_paths) * validation_ratio)))
    validation_indices = set(indices[:validation_count].tolist())

    train_paths = [path for index, path in enumerate(replay_paths) if index not in validation_indices]
    validation_paths = [path for index, path in enumerate(replay_paths) if index in validation_indices]
    return train_paths, validation_paths


def load_replay_data_by_path(
    replay_paths: list[Path],
    replay_feature_config: ReplayFeatureConfig,
    replay_cache_dir: Path,
) -> dict[Path, ReplayTensorData]:
    replay_data_by_path: dict[Path, ReplayTensorData] = {}
    skipped_replays: list[dict[str, str]] = []
    for replay_path in replay_paths:
        try:
            replay_data_by_path[replay_path] = load_replay_tensor_data(
                replay_path,
                replay_feature_config,
                cache_dir=replay_cache_dir,
            )
        except Exception as exc:
            skipped_replays.append({"replay_path": str(replay_path), "error": str(exc)})

    if skipped_replays:
        print(
            json.dumps(
                {
                    "skipped_replay_count": len(skipped_replays),
                    "skipped_replays_sample": skipped_replays[:10],
                },
                sort_keys=True,
            )
        )
    if not replay_data_by_path:
        raise ValueError("No replay tensors could be loaded from the provided replay corpus")
    return replay_data_by_path


def split_replays_by_positive_labels(
    replay_data_by_path: dict[Path, ReplayTensorData],
) -> tuple[dict[Path, ReplayTensorData], dict[Path, ReplayTensorData]]:
    positive_replays: dict[Path, ReplayTensorData] = {}
    zero_label_replays: dict[Path, ReplayTensorData] = {}
    for replay_path, replay_data in replay_data_by_path.items():
        if float(replay_data.label_counts_by_player.sum()) > 0.0:
            positive_replays[replay_path] = replay_data
        else:
            zero_label_replays[replay_path] = replay_data
    return positive_replays, zero_label_replays


def standardize_windows(
    train_features: np.ndarray,
    validation_features: np.ndarray,
) -> tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    mean = train_features.mean(axis=(0, 1), keepdims=True)
    std = train_features.std(axis=(0, 1), keepdims=True)
    std = np.where(std < 1e-6, 1.0, std)
    np.subtract(train_features, mean, out=train_features)
    np.divide(train_features, std, out=train_features)
    np.subtract(validation_features, mean, out=validation_features)
    np.divide(validation_features, std, out=validation_features)
    return (
        train_features,
        validation_features,
        mean.squeeze(0).squeeze(0).astype(np.float32),
        std.squeeze(0).squeeze(0).astype(np.float32),
    )


def make_loader(
    features: np.ndarray,
    relative_times: np.ndarray,
    labels: np.ndarray,
    batch_size: int,
    shuffle: bool,
) -> DataLoader:
    dataset = TensorDataset(
        torch.from_numpy(features),
        torch.from_numpy(relative_times),
        torch.from_numpy(labels),
    )
    return DataLoader(
        dataset,
        batch_size=batch_size,
        shuffle=shuffle,
        pin_memory=torch.cuda.is_available(),
    )


def binary_classification_metrics(logits: torch.Tensor, labels: torch.Tensor) -> dict[str, float]:
    probabilities = torch.sigmoid(logits)
    return tolerant_sequence_metrics(probabilities, labels, threshold=0.5, radius=3)


def binary_classification_metrics_for_threshold(
    probabilities: torch.Tensor,
    labels: torch.Tensor,
    threshold: float,
    radius: int = 3,
) -> dict[str, float]:
    return tolerant_sequence_metrics(probabilities, labels, threshold=threshold, radius=radius)


def triangular_kernel(radius: int, device: torch.device, dtype: torch.dtype) -> torch.Tensor:
    positions = torch.arange(-radius, radius + 1, device=device, dtype=dtype)
    kernel = 1.0 - positions.abs() / (radius + 1)
    return kernel.clamp(min=0.0)


class TolerantSequenceLoss(nn.Module):
    def __init__(
        self,
        radius: int,
        cover_loss_weight: float,
        false_positive_loss_weight: float,
        count_loss_weight: float,
        exact_bce_loss_weight: float,
        exact_positive_weight: float,
    ) -> None:
        super().__init__()
        self.radius = radius
        self.cover_loss_weight = cover_loss_weight
        self.false_positive_loss_weight = false_positive_loss_weight
        self.count_loss_weight = count_loss_weight
        self.exact_bce_loss_weight = exact_bce_loss_weight
        self.register_buffer("exact_positive_weight", torch.tensor(float(exact_positive_weight), dtype=torch.float32))

    def forward(self, logits: torch.Tensor, labels: torch.Tensor) -> torch.Tensor:
        probabilities = torch.sigmoid(logits)
        event_mask = (labels > 0).to(probabilities.dtype)
        kernel = triangular_kernel(self.radius, probabilities.device, probabilities.dtype)
        kernel = kernel.view(1, 1, -1)

        coverage_signal = F.conv1d(
            probabilities.unsqueeze(1),
            kernel,
            padding=self.radius,
        ).squeeze(1).clamp(max=1.0)
        near_signal = F.conv1d(
            event_mask.unsqueeze(1),
            kernel,
            padding=self.radius,
        ).squeeze(1).clamp(max=1.0)

        if bool(event_mask.any()):
            cover_loss = -torch.log(coverage_signal[event_mask.bool()].clamp_min(1e-6)).mean()
        else:
            cover_loss = probabilities.new_tensor(0.0)
        false_positive_loss = (probabilities * (1.0 - near_signal)).mean()
        count_loss = F.smooth_l1_loss(
            probabilities.sum(dim=1),
            event_mask.sum(dim=1),
        )
        exact_bce_loss = F.binary_cross_entropy_with_logits(
            logits,
            event_mask,
            pos_weight=self.exact_positive_weight.to(device=logits.device, dtype=logits.dtype),
        )
        return (
            self.cover_loss_weight * cover_loss
            + self.false_positive_loss_weight * false_positive_loss
            + self.count_loss_weight * count_loss
            + self.exact_bce_loss_weight * exact_bce_loss
        )


def tolerant_sequence_metrics(
    probabilities: torch.Tensor,
    labels: torch.Tensor,
    threshold: float,
    radius: int,
) -> dict[str, float]:
    predictions = probabilities >= threshold
    labels_bool = labels > 0
    kernel_size = radius * 2 + 1

    dilated_labels = F.max_pool1d(
        labels_bool.float().unsqueeze(1),
        kernel_size=kernel_size,
        stride=1,
        padding=radius,
    ).squeeze(1) > 0
    dilated_predictions = F.max_pool1d(
        predictions.float().unsqueeze(1),
        kernel_size=kernel_size,
        stride=1,
        padding=radius,
    ).squeeze(1) > 0

    matched_predictions = torch.logical_and(predictions, dilated_labels).sum().item()
    matched_labels = torch.logical_and(labels_bool, dilated_predictions).sum().item()
    prediction_count = predictions.sum().item()
    label_count = labels_bool.sum().item()

    precision = matched_predictions / max(prediction_count, 1)
    recall = matched_labels / max(label_count, 1)
    f1 = 2.0 * precision * recall / max(precision + recall, 1e-6)

    exact_true_positive = torch.logical_and(predictions, labels_bool).sum().item()
    exact_false_positive = torch.logical_and(predictions, ~labels_bool).sum().item()
    exact_false_negative = torch.logical_and(~predictions, labels_bool).sum().item()
    exact_true_negative = torch.logical_and(~predictions, ~labels_bool).sum().item()
    accuracy = (exact_true_positive + exact_true_negative) / max(
        exact_true_positive + exact_false_positive + exact_false_negative + exact_true_negative,
        1,
    )
    return {
        "accuracy": float(accuracy),
        "precision": float(precision),
        "recall": float(recall),
        "f1": float(f1),
    }


def best_threshold_metrics(logits: torch.Tensor, labels: torch.Tensor, radius: int) -> dict[str, float]:
    probabilities = torch.sigmoid(logits)
    best_metrics: dict[str, float] | None = None
    for threshold in np.linspace(0.05, 0.95, 19):
        metrics = binary_classification_metrics_for_threshold(
            probabilities,
            labels,
            float(threshold),
            radius=radius,
        )
        metrics["threshold"] = float(threshold)
        if best_metrics is None or metrics["f1"] > best_metrics["f1"]:
            best_metrics = metrics
    return best_metrics or {
        "accuracy": 0.0,
        "precision": 0.0,
        "recall": 0.0,
        "f1": 0.0,
        "threshold": 0.5,
    }


class StreamingThresholdMetrics:
    def __init__(self, thresholds: np.ndarray, radius: int) -> None:
        self.thresholds = np.asarray(thresholds, dtype=np.float32)
        self.radius = radius
        zeros = np.zeros(len(self.thresholds), dtype=np.int64)
        self.prediction_counts = zeros.copy()
        self.label_counts = zeros.copy()
        self.matched_prediction_counts = zeros.copy()
        self.matched_label_counts = zeros.copy()
        self.exact_true_positive_counts = zeros.copy()
        self.exact_false_positive_counts = zeros.copy()
        self.exact_false_negative_counts = zeros.copy()
        self.exact_true_negative_counts = zeros.copy()

    def update(self, probabilities: torch.Tensor, labels: torch.Tensor) -> None:
        thresholds = torch.as_tensor(
            self.thresholds,
            device=probabilities.device,
            dtype=probabilities.dtype,
        ).view(-1, 1, 1)
        labels_bool = labels > 0
        kernel_size = self.radius * 2 + 1

        predictions = probabilities.unsqueeze(0) >= thresholds
        dilated_labels = F.max_pool1d(
            labels_bool.float().unsqueeze(1),
            kernel_size=kernel_size,
            stride=1,
            padding=self.radius,
        ).squeeze(1) > 0
        dilated_predictions = F.max_pool1d(
            predictions.float().reshape(-1, 1, predictions.shape[-1]),
            kernel_size=kernel_size,
            stride=1,
            padding=self.radius,
        ).reshape(predictions.shape[0], predictions.shape[1], predictions.shape[2]) > 0

        self.prediction_counts += predictions.sum(dim=(1, 2)).cpu().numpy()
        label_count = int(labels_bool.sum().item())
        self.label_counts += label_count
        self.matched_prediction_counts += torch.logical_and(
            predictions, dilated_labels.unsqueeze(0)
        ).sum(dim=(1, 2)).cpu().numpy()
        self.matched_label_counts += torch.logical_and(
            labels_bool.unsqueeze(0), dilated_predictions
        ).sum(dim=(1, 2)).cpu().numpy()
        self.exact_true_positive_counts += torch.logical_and(
            predictions, labels_bool.unsqueeze(0)
        ).sum(dim=(1, 2)).cpu().numpy()
        self.exact_false_positive_counts += torch.logical_and(
            predictions, ~labels_bool.unsqueeze(0)
        ).sum(dim=(1, 2)).cpu().numpy()
        self.exact_false_negative_counts += torch.logical_and(
            ~predictions, labels_bool.unsqueeze(0)
        ).sum(dim=(1, 2)).cpu().numpy()
        self.exact_true_negative_counts += torch.logical_and(
            ~predictions, ~labels_bool.unsqueeze(0)
        ).sum(dim=(1, 2)).cpu().numpy()

    def _metrics_for_index(self, index: int) -> dict[str, float]:
        prediction_count = int(self.prediction_counts[index])
        label_count = int(self.label_counts[index])
        matched_predictions = int(self.matched_prediction_counts[index])
        matched_labels = int(self.matched_label_counts[index])

        precision = matched_predictions / max(prediction_count, 1)
        recall = matched_labels / max(label_count, 1)
        f1 = 2.0 * precision * recall / max(precision + recall, 1e-6)

        exact_true_positive = int(self.exact_true_positive_counts[index])
        exact_false_positive = int(self.exact_false_positive_counts[index])
        exact_false_negative = int(self.exact_false_negative_counts[index])
        exact_true_negative = int(self.exact_true_negative_counts[index])
        accuracy = (exact_true_positive + exact_true_negative) / max(
            exact_true_positive + exact_false_positive + exact_false_negative + exact_true_negative,
            1,
        )
        return {
            "accuracy": float(accuracy),
            "precision": float(precision),
            "recall": float(recall),
            "f1": float(f1),
            "threshold": float(self.thresholds[index]),
        }

    def metrics_for_threshold(self, threshold: float) -> dict[str, float]:
        threshold_index = int(np.argmin(np.abs(self.thresholds - threshold)))
        return self._metrics_for_index(threshold_index)

    def best_metrics(self) -> dict[str, float]:
        best_index = max(range(len(self.thresholds)), key=lambda index: self._metrics_for_index(index)["f1"])
        return self._metrics_for_index(best_index)


def run_epoch(
    model: FlipResetTransformer,
    loader: DataLoader,
    loss_fn: nn.Module,
    optimizer: torch.optim.Optimizer | None,
    device: torch.device,
    event_match_radius_frames: int,
    compute_metrics: bool,
    sweep_thresholds: bool,
) -> tuple[float, dict[str, float], dict[str, float]]:
    if optimizer is None:
        model.eval()
    else:
        model.train()

    total_loss = 0.0
    metric_thresholds = (
        np.linspace(0.05, 0.95, 19, dtype=np.float32)
        if sweep_thresholds
        else np.asarray([0.5], dtype=np.float32)
    )
    metrics_accumulator = (
        StreamingThresholdMetrics(metric_thresholds, radius=event_match_radius_frames)
        if compute_metrics
        else None
    )

    context = torch.inference_mode if optimizer is None else nullcontext
    with context():
        for features, relative_times, labels in loader:
            features = features.to(device, non_blocking=True)
            relative_times = relative_times.to(device, non_blocking=True)
            labels = labels.to(device, non_blocking=True)

            if optimizer is not None:
                optimizer.zero_grad(set_to_none=True)

            logits = model(features, relative_times)
            loss = loss_fn(logits, labels)

            if optimizer is not None:
                loss.backward()
                optimizer.step()

            total_loss += loss.item() * features.shape[0]
            if metrics_accumulator is not None:
                metrics_accumulator.update(
                    torch.sigmoid(logits.detach()),
                    labels.detach(),
                )

    average_loss = total_loss / max(len(loader.dataset), 1)
    if metrics_accumulator is None:
        empty_metrics = {
            "accuracy": 0.0,
            "precision": 0.0,
            "recall": 0.0,
            "f1": 0.0,
        }
        return average_loss, empty_metrics, {**empty_metrics, "threshold": 0.5}

    default_metrics = metrics_accumulator.metrics_for_threshold(0.5)
    best_metrics = metrics_accumulator.best_metrics() if sweep_thresholds else default_metrics
    return (
        average_loss,
        {key: value for key, value in default_metrics.items() if key != "threshold"},
        best_metrics,
    )


def main() -> None:
    config = parse_args()
    config.output_dir.mkdir(parents=True, exist_ok=True)
    replay_cache_dir = (
        config.replay_cache_dir
        if config.replay_cache_dir is not None
        else config.output_dir / config.replay_cache_dir_name
    )

    replay_paths = load_replay_paths(
        replay_dirs=config.replay_dirs,
        replay_files=config.replay_files,
        recursive=config.recursive_replay_search,
    )
    replay_data_by_path = load_replay_data_by_path(
        replay_paths,
        config.replay_features,
        replay_cache_dir,
    )
    positive_replays, zero_label_replays = split_replays_by_positive_labels(replay_data_by_path)
    if config.require_positive_replays:
        if not positive_replays:
            raise ValueError("No replay tensors with positive dodge-refresh labels were found")
        replay_data_by_path = positive_replays
        print(
            json.dumps(
                {
                    "filtered_zero_label_replay_count": len(zero_label_replays),
                    "positive_replay_count": len(positive_replays),
                },
                sort_keys=True,
            )
        )
    train_paths, validation_paths = split_replay_paths(
        list(replay_data_by_path),
        validation_ratio=config.validation_ratio,
        seed=config.sampling.random_seed,
    )
    training_replay_count = len(replay_data_by_path)

    train_replays = [replay_data_by_path[path] for path in train_paths]
    validation_replays = [replay_data_by_path[path] for path in validation_paths]

    train_data = build_windowed_training_data(train_replays, config.sampling)
    try:
        validation_data = build_windowed_training_data(validation_replays, config.sampling)
    except ValueError:
        validation_replays = train_replays
        validation_data = build_windowed_training_data(validation_replays, config.sampling)
    train_features, validation_features, mean, std = standardize_windows(
        train_data.features,
        validation_data.features,
    )
    del replay_data_by_path
    del positive_replays
    del train_replays
    del validation_replays
    gc.collect()

    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    model = FlipResetTransformer(
        FlipResetTransformerConfig(
            input_dim=train_features.shape[-1],
            window_length=train_features.shape[1],
            model_dim=config.model_dim,
            encoder_type=config.encoder_type,
            time_feature_dim=config.time_feature_dim,
            cnn_layers=config.cnn_layers,
            cnn_kernel_size=config.cnn_kernel_size,
            encoder_layers=config.encoder_layers,
            attention_heads=config.attention_heads,
            feedforward_dim=config.feedforward_dim,
            dropout=config.dropout,
            additive_position_embedding=config.additive_position_embedding,
        )
    ).to(device)
    if config.pretrained_encoder_checkpoint is not None:
        checkpoint = torch.load(config.pretrained_encoder_checkpoint, map_location=device)
        model.encoder.load_state_dict(checkpoint["encoder_state"])

    train_has_event = train_data.labels.max(axis=1) > 0
    positive_count = float(train_has_event.sum())
    negative_count = float(train_has_event.shape[0] - positive_count)
    loss_fn = TolerantSequenceLoss(
        radius=config.event_match_radius_frames,
        cover_loss_weight=config.cover_loss_weight,
        false_positive_loss_weight=config.false_positive_loss_weight,
        count_loss_weight=config.count_loss_weight,
        exact_bce_loss_weight=config.exact_bce_loss_weight,
        exact_positive_weight=config.exact_positive_weight,
    ).to(device)
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=config.learning_rate,
        weight_decay=config.weight_decay,
    )

    train_loader = make_loader(
        train_features,
        train_data.relative_times,
        train_data.labels,
        batch_size=config.batch_size,
        shuffle=False,
    )
    sample_weights = np.where(
        train_has_event,
        negative_count / max(positive_count, 1.0),
        1.0,
    )
    train_loader = DataLoader(
        train_loader.dataset,
        batch_size=config.batch_size,
        sampler=WeightedRandomSampler(
            weights=torch.from_numpy(sample_weights.astype(np.float32)),
            num_samples=len(sample_weights),
            replacement=True,
        ),
        pin_memory=torch.cuda.is_available(),
    )
    validation_loader = make_loader(
        validation_features,
        validation_data.relative_times,
        validation_data.labels,
        batch_size=config.batch_size,
        shuffle=False,
    )

    best_validation_f1 = -1.0
    best_checkpoint_path = config.output_dir / "best_model.pt"

    history = []
    for epoch in range(1, config.epochs + 1):
        train_loss, train_metrics, train_best_threshold_metrics = run_epoch(
            model,
            train_loader,
            loss_fn,
            optimizer,
            device,
            config.event_match_radius_frames,
            compute_metrics=config.compute_train_metrics,
            sweep_thresholds=config.sweep_train_thresholds,
        )
        validation_loss, validation_metrics, validation_best_threshold_metrics = run_epoch(
            model,
            validation_loader,
            loss_fn,
            None,
            device,
            config.event_match_radius_frames,
            compute_metrics=True,
            sweep_thresholds=True,
        )

        epoch_summary = {
            "epoch": epoch,
            "train_loss": train_loss,
            "validation_loss": validation_loss,
            "train_metrics": train_metrics,
            "train_best_threshold_metrics": train_best_threshold_metrics,
            "validation_metrics": validation_metrics,
            "validation_best_threshold_metrics": validation_best_threshold_metrics,
        }
        history.append(epoch_summary)
        print(json.dumps(epoch_summary, sort_keys=True))

        if validation_best_threshold_metrics["f1"] > best_validation_f1:
            best_validation_f1 = validation_best_threshold_metrics["f1"]
            torch.save(
                {
                    "model_state": model.state_dict(),
                    "model_config": asdict(model.config),
                    "feature_names": list(train_data.feature_names),
                    "normalization_mean": mean.tolist(),
                    "normalization_std": std.tolist(),
                    "train_replays": [str(path) for path in train_paths],
                    "validation_replays": [str(path) for path in validation_paths],
                    "best_validation_threshold": validation_best_threshold_metrics["threshold"],
                },
                best_checkpoint_path,
            )

    metadata_path = config.output_dir / "training_run.json"
    metadata_path.write_text(
        json.dumps(
            {
                "config": train_config_to_json(config),
                "train_window_count": int(train_data.labels.shape[0]),
                "validation_window_count": int(validation_data.labels.shape[0]),
                "loaded_replay_count": len(replay_paths),
                "training_replay_count": training_replay_count,
                "filtered_zero_label_replay_count": len(zero_label_replays)
                if config.require_positive_replays
                else 0,
                "feature_names": list(train_data.feature_names),
                "replay_cache_dir": str(replay_cache_dir),
                "history": history,
                "best_model_path": str(best_checkpoint_path),
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
