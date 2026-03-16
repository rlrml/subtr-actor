from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path

from .features import DEFAULT_GLOBAL_FEATURE_ADDERS, DEFAULT_PLAYER_FEATURE_ADDERS


@dataclass(frozen=True)
class ReplayFeatureConfig:
    fps: float = 30.0
    global_feature_adders: tuple[str, ...] = DEFAULT_GLOBAL_FEATURE_ADDERS
    player_feature_adders: tuple[str, ...] = DEFAULT_PLAYER_FEATURE_ADDERS
    include_geometry_scalars: bool = False


@dataclass(frozen=True)
class WindowSamplingConfig:
    window_radius: int = 45
    positive_radius_frames: int = 1
    negative_to_positive_ratio: float = 4.0
    negative_only_replay_sample_count: int = 64
    random_seed: int = 7

    @property
    def window_length(self) -> int:
        return self.window_radius * 2 + 1


@dataclass(frozen=True)
class PretrainSamplingConfig:
    window_radius: int = 45
    stride: int = 15
    max_windows_per_player: int = 64
    random_seed: int = 7

    @property
    def window_length(self) -> int:
        return self.window_radius * 2 + 1


@dataclass(frozen=True)
class TrainConfig:
    output_dir: Path
    replay_dirs: tuple[Path, ...] = ()
    replay_files: tuple[Path, ...] = ()
    recursive_replay_search: bool = True
    replay_cache_dir: Path | None = None
    pretrained_encoder_checkpoint: Path | None = None
    require_positive_replays: bool = False
    replay_features: ReplayFeatureConfig = field(default_factory=ReplayFeatureConfig)
    sampling: WindowSamplingConfig = field(default_factory=WindowSamplingConfig)
    validation_ratio: float = 0.2
    epochs: int = 10
    batch_size: int = 128
    learning_rate: float = 3e-4
    weight_decay: float = 1e-4
    model_dim: int = 128
    encoder_type: str = "transformer"
    time_feature_dim: int = 9
    additive_position_embedding: bool = True
    cnn_layers: int = 2
    cnn_kernel_size: int = 5
    encoder_layers: int = 4
    attention_heads: int = 4
    feedforward_dim: int = 256
    dropout: float = 0.1
    event_match_radius_frames: int = 3
    cover_loss_weight: float = 1.0
    false_positive_loss_weight: float = 1.0
    count_loss_weight: float = 0.25
    exact_bce_loss_weight: float = 0.0
    exact_positive_weight: float = 1.0
    compute_train_metrics: bool = False
    sweep_train_thresholds: bool = False
    replay_cache_dir_name: str = "replay-cache"
