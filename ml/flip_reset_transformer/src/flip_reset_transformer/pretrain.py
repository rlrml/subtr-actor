from __future__ import annotations

import argparse
import json
from dataclasses import asdict
from pathlib import Path

import numpy as np
import torch
from torch import nn
from torch.utils.data import DataLoader, TensorDataset

from .config import ReplayFeatureConfig
from .dataset import (
    build_pretraining_window_data,
    load_replay_paths,
    load_replay_tensor_data,
    ReplayTensorData,
)
from .model import FlipResetTransformerConfig, MaskedFeatureReconstructionModel


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Masked-window pretraining for the flip reset encoder")
    parser.add_argument("--replay-dir", type=Path, action="append", default=[])
    parser.add_argument("--replay-file", type=Path, action="append", default=[])
    parser.add_argument("--non-recursive-replay-search", action="store_true")
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--replay-cache-dir", type=Path, default=None)
    parser.add_argument("--fps", type=float, default=30.0)
    parser.add_argument("--window-radius", type=int, default=45)
    parser.add_argument("--stride", type=int, default=15)
    parser.add_argument("--max-windows-per-player", type=int, default=64)
    parser.add_argument("--max-replays", type=int, default=0)
    parser.add_argument("--epochs", type=int, default=3)
    parser.add_argument("--batch-size", type=int, default=64)
    parser.add_argument("--learning-rate", type=float, default=3e-4)
    parser.add_argument("--weight-decay", type=float, default=1e-4)
    parser.add_argument("--validation-ratio", type=float, default=0.1)
    parser.add_argument("--model-dim", type=int, default=96)
    parser.add_argument("--encoder-layers", type=int, default=3)
    parser.add_argument("--attention-heads", type=int, default=4)
    parser.add_argument("--feedforward-dim", type=int, default=192)
    parser.add_argument("--dropout", type=float, default=0.1)
    parser.add_argument("--mask-ratio", type=float, default=0.2)
    parser.add_argument("--mask-span-length", type=int, default=5)
    parser.add_argument("--seed", type=int, default=7)
    return parser.parse_args()


def standardize_windows(features: np.ndarray) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    mean = features.mean(axis=(0, 1), keepdims=True)
    std = features.std(axis=(0, 1), keepdims=True)
    std = np.where(std < 1e-6, 1.0, std)
    return ((features - mean) / std).astype(np.float32), mean, std


def load_replay_data_items(
    replay_paths: list[Path],
    replay_feature_config: ReplayFeatureConfig,
    replay_cache_dir: Path,
) -> list[ReplayTensorData]:
    loaded_items: list[ReplayTensorData] = []
    skipped_replays: list[dict[str, str]] = []
    for replay_path in replay_paths:
        try:
            loaded_items.append(
                load_replay_tensor_data(
                    replay_path,
                    replay_feature_config,
                    cache_dir=replay_cache_dir,
                )
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
    if not loaded_items:
        raise ValueError("No replay tensors could be loaded from the provided replay corpus")
    return loaded_items


def split_indices(item_count: int, validation_ratio: float, seed: int) -> tuple[np.ndarray, np.ndarray]:
    if item_count < 2:
        indices = np.arange(item_count, dtype=np.int64)
        return indices, indices

    rng = np.random.default_rng(seed)
    indices = rng.permutation(item_count)
    validation_count = max(1, int(round(item_count * validation_ratio)))
    validation_indices = np.sort(indices[:validation_count])
    train_indices = np.sort(indices[validation_count:])
    if train_indices.size == 0:
        train_indices = validation_indices
    return train_indices, validation_indices


def make_span_mask(
    batch_size: int,
    sequence_length: int,
    mask_ratio: float,
    span_length: int,
    device: torch.device,
) -> torch.Tensor:
    mask = torch.zeros(batch_size, sequence_length, device=device, dtype=torch.bool)
    span_length = max(1, min(span_length, sequence_length))
    target_mask_count = max(1, int(round(sequence_length * mask_ratio)))
    span_count = max(1, int(np.ceil(target_mask_count / span_length)))

    max_start = max(sequence_length - span_length, 0)
    for batch_index in range(batch_size):
        starts = torch.randint(0, max_start + 1, (span_count,), device=device)
        for start in starts.tolist():
            mask[batch_index, start : start + span_length] = True

    empty_rows = ~mask.any(dim=1)
    if empty_rows.any():
        random_positions = torch.randint(
            0,
            sequence_length,
            (int(empty_rows.sum().item()),),
            device=device,
        )
        mask[empty_rows, random_positions] = True
    return mask


def run_epoch(
    model: MaskedFeatureReconstructionModel,
    loader: DataLoader,
    optimizer: torch.optim.Optimizer | None,
    device: torch.device,
    mask_ratio: float,
    mask_span_length: int,
) -> float:
    if optimizer is None:
        model.eval()
    else:
        model.train()

    total_loss = 0.0
    for features, relative_times in loader:
        features = features.to(device)
        relative_times = relative_times.to(device)
        mask = make_span_mask(
            features.shape[0],
            features.shape[1],
            mask_ratio,
            mask_span_length,
            device,
        )

        if optimizer is not None:
            optimizer.zero_grad(set_to_none=True)

        reconstructed = model(features, relative_times, masked_positions=mask)
        loss = nn.functional.mse_loss(reconstructed[mask], features[mask])
        if optimizer is not None:
            loss.backward()
            optimizer.step()
        total_loss += loss.item() * features.shape[0]
    return total_loss / max(len(loader.dataset), 1)


def main() -> None:
    args = parse_args()
    if not args.replay_dir and not args.replay_file:
        raise ValueError("Provide --replay-dir or --replay-file")
    args.output_dir.mkdir(parents=True, exist_ok=True)
    replay_cache_dir = (
        args.replay_cache_dir.resolve()
        if args.replay_cache_dir is not None
        else args.output_dir / "replay-cache"
    )

    replay_paths = load_replay_paths(
        replay_dirs=tuple(path.resolve() for path in args.replay_dir),
        replay_files=tuple(path.resolve() for path in args.replay_file),
        recursive=not args.non_recursive_replay_search,
    )
    if args.max_replays > 0:
        replay_paths = replay_paths[: args.max_replays]

    replay_feature_config = ReplayFeatureConfig(fps=args.fps)
    replay_data_items = load_replay_data_items(
        replay_paths,
        replay_feature_config,
        replay_cache_dir,
    )
    pretraining_data = build_pretraining_window_data(
        replay_data_items,
        window_radius=args.window_radius,
        stride=args.stride,
        max_windows_per_player=args.max_windows_per_player,
        random_seed=args.seed,
    )
    train_indices, validation_indices = split_indices(
        pretraining_data.features.shape[0],
        validation_ratio=args.validation_ratio,
        seed=args.seed,
    )
    train_features = pretraining_data.features[train_indices]
    validation_features = pretraining_data.features[validation_indices]
    standardized_train_features, mean, std = standardize_windows(train_features)
    standardized_validation_features = ((validation_features - mean) / std).astype(np.float32)

    train_loader = DataLoader(
        TensorDataset(
            torch.from_numpy(standardized_train_features),
            torch.from_numpy(pretraining_data.relative_times[train_indices]),
        ),
        batch_size=args.batch_size,
        shuffle=True,
    )
    validation_loader = DataLoader(
        TensorDataset(
            torch.from_numpy(standardized_validation_features),
            torch.from_numpy(pretraining_data.relative_times[validation_indices]),
        ),
        batch_size=args.batch_size,
        shuffle=False,
    )

    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    model = MaskedFeatureReconstructionModel(
        FlipResetTransformerConfig(
            input_dim=standardized_train_features.shape[-1],
            window_length=standardized_train_features.shape[1],
            model_dim=args.model_dim,
            encoder_layers=args.encoder_layers,
            attention_heads=args.attention_heads,
            feedforward_dim=args.feedforward_dim,
            dropout=args.dropout,
        )
    ).to(device)
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=args.learning_rate,
        weight_decay=args.weight_decay,
    )

    history = []
    best_loss = float("inf")
    best_checkpoint_path = args.output_dir / "best_pretrained_encoder.pt"
    for epoch in range(1, args.epochs + 1):
        train_loss = run_epoch(
            model,
            train_loader,
            optimizer,
            device,
            args.mask_ratio,
            args.mask_span_length,
        )
        validation_loss = run_epoch(
            model,
            validation_loader,
            None,
            device,
            args.mask_ratio,
            args.mask_span_length,
        )
        summary = {
            "epoch": epoch,
            "train_pretrain_loss": train_loss,
            "validation_pretrain_loss": validation_loss,
        }
        history.append(summary)
        print(json.dumps(summary, sort_keys=True))
        if validation_loss < best_loss:
            best_loss = validation_loss
            torch.save(
                {
                    "encoder_state": model.encoder.state_dict(),
                    "model_config": asdict(model.config),
                    "feature_names": list(pretraining_data.feature_names),
                    "normalization_mean": mean.squeeze(0).squeeze(0).astype(np.float32).tolist(),
                    "normalization_std": std.squeeze(0).squeeze(0).astype(np.float32).tolist(),
                    "replay_count": len(replay_paths),
                },
                best_checkpoint_path,
            )

    (args.output_dir / "pretraining_run.json").write_text(
        json.dumps(
            {
                "config": vars(args),
                "feature_names": list(pretraining_data.feature_names),
                "window_count": int(pretraining_data.features.shape[0]),
                "train_window_count": int(train_indices.shape[0]),
                "validation_window_count": int(validation_indices.shape[0]),
                "history": history,
                "best_checkpoint_path": str(best_checkpoint_path),
            },
            indent=2,
            default=str,
        )
    )


if __name__ == "__main__":
    main()
