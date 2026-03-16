from __future__ import annotations

import hashlib
import json
import importlib.util
import os
import sysconfig
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable
import site

import numpy as np

from .config import ReplayFeatureConfig, WindowSamplingConfig
from .features import (
    LABEL_PLAYER_HEADER,
    TIME_HEADER,
    ensure_label_player_feature,
)


@dataclass(frozen=True)
class ReplayTensorData:
    replay_path: Path
    player_names: tuple[str, ...]
    global_feature_headers: tuple[str, ...]
    player_feature_headers: tuple[str, ...]
    feature_names: tuple[str, ...]
    token_features_by_player: np.ndarray
    label_counts_by_player: np.ndarray
    relative_times: np.ndarray


@dataclass(frozen=True)
class WindowedTrainingData:
    features: np.ndarray
    relative_times: np.ndarray
    labels: np.ndarray
    replay_paths: tuple[str, ...]
    player_names: tuple[str, ...]
    center_frames: np.ndarray
    feature_names: tuple[str, ...]


@dataclass(frozen=True)
class PretrainingWindowData:
    features: np.ndarray
    relative_times: np.ndarray
    replay_paths: tuple[str, ...]
    player_names: tuple[str, ...]
    center_frames: np.ndarray
    feature_names: tuple[str, ...]


def replay_feature_cache_key(feature_config: ReplayFeatureConfig) -> str:
    payload = json.dumps(
        {
            "fps": feature_config.fps,
            "global_feature_adders": feature_config.global_feature_adders,
            "player_feature_adders": feature_config.player_feature_adders,
            "include_geometry_scalars": feature_config.include_geometry_scalars,
        },
        sort_keys=True,
    ).encode("utf-8")
    return hashlib.sha256(payload).hexdigest()[:16]


def _import_subtr_actor() -> Any:
    repo_root = _repo_root()
    ext_suffix = sysconfig.get_config_var("EXT_SUFFIX") or ".so"
    local_package_dir = repo_root / "python" / "subtr_actor"
    if local_package_dir.exists():
        local_candidates = list(local_package_dir.glob(f"*{ext_suffix}")) + sorted(
            local_package_dir.glob("subtr_actor*.so")
        )
        if local_candidates:
            shared_object = local_candidates[0]
            spec = importlib.util.spec_from_file_location(
                "subtr_actor.subtr_actor",
                shared_object,
            )
            if spec is not None and spec.loader is not None:
                module = importlib.util.module_from_spec(spec)
                spec.loader.exec_module(module)
                return module

    try:
        import subtr_actor
        return subtr_actor
    except Exception:
        pass

    for site_packages_dir in site.getsitepackages():
        package_dir = Path(site_packages_dir) / "subtr_actor"
        if not package_dir.exists():
            continue
        shared_objects = sorted(package_dir.glob("subtr_actor*.so"))
        if not shared_objects:
            continue
        shared_object = shared_objects[0]
        spec = importlib.util.spec_from_file_location(
            "subtr_actor.subtr_actor",
            shared_object,
        )
        if spec is None or spec.loader is None:
            continue
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module

    raise RuntimeError(
        "The flip reset transformer project requires the `subtr_actor` Python "
        "bindings. Build them from `python/` with `maturin develop` or install "
        "`subtr-actor-py` into this environment."
    )


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[4]


def _replay_search_roots() -> tuple[Path, ...]:
    repo_root = _repo_root()
    configured_roots = [
        Path(value).expanduser().resolve()
        for value in os.environ.get("SUBTR_ACTOR_REPLAY_SEARCH_ROOTS", "").split(os.pathsep)
        if value
    ]
    default_roots = [repo_root / "target", repo_root / "assets", repo_root]
    seen: set[Path] = set()
    ordered_roots: list[Path] = []
    for root in configured_roots + default_roots:
        if root in seen or not root.exists():
            continue
        seen.add(root)
        ordered_roots.append(root)
    return tuple(ordered_roots)


def resolve_replay_path(replay_path: Path) -> Path | None:
    replay_path = replay_path.resolve() if replay_path.exists() else replay_path
    if replay_path.exists():
        return replay_path

    matches: list[Path] = []
    for search_root in _replay_search_roots():
        matches.extend(search_root.rglob(f"{replay_path.stem}.replay"))
    if not matches:
        return None

    def priority(path: Path) -> tuple[int, int, str]:
        if "replays" in path.parts:
            return (0, len(path.parts), str(path))
        if "flip-reset-ground-truth" in path.parts:
            return (1, len(path.parts), str(path))
        if "cache" in path.parts:
            return (2, len(path.parts), str(path))
        return (3, len(path.parts), str(path))

    return sorted(matches, key=priority)[0].resolve()


def replay_path_priority(path: Path) -> tuple[int, int, str]:
    if "replays" in path.parts:
        return (0, len(path.parts), str(path))
    if "flip-reset-ground-truth" in path.parts:
        return (1, len(path.parts), str(path))
    if "cache" in path.parts:
        return (2, len(path.parts), str(path))
    return (3, len(path.parts), str(path))


def should_include_replay_path(path: Path) -> bool:
    if path.name.startswith("._"):
        return False
    if "replay-cache" in path.parts or ".worktrees" in path.parts:
        return False
    return True


def collect_replay_paths(
    replay_dirs: Iterable[Path] = (),
    replay_files: Iterable[Path] = (),
    recursive: bool = True,
) -> list[Path]:
    replay_paths: list[Path] = []
    for replay_dir_arg in replay_dirs:
        replay_dir = replay_dir_arg.resolve()
        globber = replay_dir.rglob if recursive else replay_dir.glob
        replay_paths.extend(
            path.resolve()
            for path in globber("*.replay")
            if should_include_replay_path(path)
        )

    for replay_file_arg in replay_files:
        replay_path = replay_file_arg.resolve()
        if replay_path.is_file() and should_include_replay_path(replay_path):
            replay_paths.append(replay_path)

    replay_paths_by_stem: dict[str, list[Path]] = {}
    for replay_path in sorted(set(replay_paths)):
        replay_paths_by_stem.setdefault(replay_path.stem, []).append(replay_path)

    return [
        sorted(paths, key=replay_path_priority)[0]
        for _, paths in sorted(replay_paths_by_stem.items())
    ]


def load_replay_paths(
    replay_dirs: Iterable[Path] = (),
    replay_files: Iterable[Path] = (),
    recursive: bool = True,
) -> list[Path]:
    replay_paths = collect_replay_paths(
        replay_dirs=replay_dirs,
        replay_files=replay_files,
        recursive=recursive,
    )

    replay_paths_by_stem: dict[str, list[Path]] = {}
    for replay_path in replay_paths:
        replay_paths_by_stem.setdefault(replay_path.stem, []).append(replay_path)

    return [
        sorted(paths, key=replay_path_priority)[0]
        for _, paths in sorted(replay_paths_by_stem.items())
    ]


def _player_names_from_meta(meta: dict[str, Any]) -> tuple[str, ...]:
    replay_meta = meta["replay_meta"]
    return tuple(
        player["name"]
        for player in replay_meta["team_zero"] + replay_meta["team_one"]
    )


def split_input_and_label_channels(
    array: np.ndarray,
    global_headers: list[str],
    player_headers: list[str],
    player_count: int,
) -> tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray, tuple[str, ...]]:
    frame_count = array.shape[0]
    global_count = len(global_headers)
    player_header_count = len(player_headers)
    label_index = player_headers.index(LABEL_PLAYER_HEADER)
    time_index = global_headers.index(TIME_HEADER)

    global_block = array[:, :global_count]
    player_block = array[:, global_count:].reshape(frame_count, player_count, player_header_count)
    player_block = np.transpose(player_block, (1, 0, 2))

    global_input_indices = [index for index, header in enumerate(global_headers) if header != TIME_HEADER]
    player_input_indices = [index for index, header in enumerate(player_headers) if header != LABEL_PLAYER_HEADER]

    global_inputs = global_block[:, global_input_indices]
    player_inputs = player_block[:, :, player_input_indices]
    labels = player_block[:, :, label_index]
    times = global_block[:, time_index]

    feature_names = tuple(
        [global_headers[index] for index in global_input_indices]
        + [player_headers[index] for index in player_input_indices]
    )

    return global_inputs, player_inputs, labels, times, feature_names


def _header_index(headers: tuple[str, ...], name: str) -> int | None:
    try:
        return headers.index(name)
    except ValueError:
        return None


def _maybe_append_geometry_scalars(
    token_features: np.ndarray,
    feature_names: tuple[str, ...],
) -> tuple[np.ndarray, tuple[str, ...]]:
    required_headers = {
        "relative ball position x": _header_index(feature_names, "relative ball position x"),
        "relative ball position y": _header_index(feature_names, "relative ball position y"),
        "relative ball position z": _header_index(feature_names, "relative ball position z"),
        "relative ball velocity x": _header_index(feature_names, "relative ball velocity x"),
        "relative ball velocity y": _header_index(feature_names, "relative ball velocity y"),
        "relative ball velocity z": _header_index(feature_names, "relative ball velocity z"),
        "Ball - linear velocity x": _header_index(feature_names, "Ball - linear velocity x"),
        "Ball - linear velocity y": _header_index(feature_names, "Ball - linear velocity y"),
        "Ball - linear velocity z": _header_index(feature_names, "Ball - linear velocity z"),
        "linear velocity x": _header_index(feature_names, "linear velocity x"),
        "linear velocity y": _header_index(feature_names, "linear velocity y"),
        "linear velocity z": _header_index(feature_names, "linear velocity z"),
    }
    if any(index is None for index in required_headers.values()):
        return token_features, feature_names

    rel_pos = token_features[
        :,
        [
            required_headers["relative ball position x"],
            required_headers["relative ball position y"],
            required_headers["relative ball position z"],
        ],
    ]
    rel_vel = token_features[
        :,
        [
            required_headers["relative ball velocity x"],
            required_headers["relative ball velocity y"],
            required_headers["relative ball velocity z"],
        ],
    ]
    ball_vel = token_features[
        :,
        [
            required_headers["Ball - linear velocity x"],
            required_headers["Ball - linear velocity y"],
            required_headers["Ball - linear velocity z"],
        ],
    ]
    player_vel = token_features[
        :,
        [
            required_headers["linear velocity x"],
            required_headers["linear velocity y"],
            required_headers["linear velocity z"],
        ],
    ]

    relative_ball_distance = np.linalg.norm(rel_pos, axis=1, keepdims=True)
    relative_ball_horizontal_distance = np.linalg.norm(rel_pos[:, :2], axis=1, keepdims=True)
    relative_ball_speed = np.linalg.norm(rel_vel, axis=1, keepdims=True)
    player_speed = np.linalg.norm(player_vel, axis=1, keepdims=True)
    ball_speed = np.linalg.norm(ball_vel, axis=1, keepdims=True)

    safe_distance = np.clip(relative_ball_distance, 1e-6, None)
    closing_speed = -np.sum(rel_pos * rel_vel, axis=1, keepdims=True) / safe_distance
    vertical_alignment = rel_pos[:, 2:3] / safe_distance

    scalar_features = np.concatenate(
        [
            relative_ball_distance,
            relative_ball_horizontal_distance,
            relative_ball_speed,
            player_speed,
            ball_speed,
            closing_speed,
            vertical_alignment,
        ],
        axis=1,
    ).astype(np.float32)
    scalar_feature_names = (
        "relative ball distance",
        "relative ball horizontal distance",
        "relative ball speed",
        "player speed",
        "ball speed",
        "closing speed",
        "vertical alignment",
    )
    return (
        np.concatenate([token_features, scalar_features], axis=1),
        feature_names + scalar_feature_names,
    )


def _build_player_token_features(
    global_inputs: np.ndarray,
    player_inputs: np.ndarray,
    feature_names: tuple[str, ...],
    include_geometry_scalars: bool = False,
) -> tuple[np.ndarray, tuple[str, ...]]:
    token_features = np.concatenate([global_inputs, player_inputs], axis=1).astype(np.float32)
    augmented_features = feature_names
    if include_geometry_scalars:
        token_features, augmented_features = _maybe_append_geometry_scalars(
            token_features,
            augmented_features,
        )
    return token_features, augmented_features


def load_replay_tensor_data(
    replay_path: Path,
    feature_config: ReplayFeatureConfig = ReplayFeatureConfig(),
    cache_dir: Path | None = None,
) -> ReplayTensorData:
    replay_path = replay_path.resolve()
    if cache_dir is not None:
        cache_dir = cache_dir.resolve()
        cache_dir.mkdir(parents=True, exist_ok=True)
        cache_path = cache_dir / (
            f"{replay_path.stem}-{replay_feature_cache_key(feature_config)}.npz"
        )
        if cache_path.exists():
            return load_replay_tensor_data_from_cache(cache_path, replay_path)

    subtr_actor = _import_subtr_actor()

    player_feature_adders = ensure_label_player_feature(feature_config.player_feature_adders)
    meta, ndarray = subtr_actor.get_ndarray_with_info_from_replay_filepath(
        replay_path,
        global_feature_adders=list(feature_config.global_feature_adders),
        player_feature_adders=list(player_feature_adders),
        fps=feature_config.fps,
        dtype="float32",
    )

    array = np.asarray(ndarray, dtype=np.float32)
    global_headers = list(meta["column_headers"]["global_headers"])
    player_headers = list(meta["column_headers"]["player_headers"])
    player_names = _player_names_from_meta(meta)

    global_inputs, player_inputs, labels, times, feature_names = split_input_and_label_channels(
        array=array,
        global_headers=global_headers,
        player_headers=player_headers,
        player_count=len(player_names),
    )

    token_features_by_player = []
    final_feature_names = feature_names
    for player_index in range(len(player_names)):
        player_tokens, final_feature_names = _build_player_token_features(
            global_inputs,
            player_inputs[player_index],
            feature_names,
            include_geometry_scalars=feature_config.include_geometry_scalars,
        )
        token_features_by_player.append(player_tokens)

    replay_tensor_data = ReplayTensorData(
        replay_path=replay_path,
        player_names=player_names,
        global_feature_headers=tuple(header for header in global_headers if header != TIME_HEADER),
        player_feature_headers=tuple(header for header in player_headers if header != LABEL_PLAYER_HEADER),
        feature_names=final_feature_names,
        token_features_by_player=np.stack(token_features_by_player, axis=0),
        label_counts_by_player=labels.astype(np.float32),
        relative_times=times.astype(np.float32),
    )
    if cache_dir is not None:
        save_replay_tensor_data_cache(replay_tensor_data, cache_path)
    return replay_tensor_data


def save_replay_tensor_data_cache(replay_tensor_data: ReplayTensorData, cache_path: Path) -> None:
    np.savez_compressed(
        cache_path,
        replay_path=str(replay_tensor_data.replay_path),
        player_names=np.asarray(replay_tensor_data.player_names, dtype=object),
        global_feature_headers=np.asarray(replay_tensor_data.global_feature_headers, dtype=object),
        player_feature_headers=np.asarray(replay_tensor_data.player_feature_headers, dtype=object),
        feature_names=np.asarray(replay_tensor_data.feature_names, dtype=object),
        token_features_by_player=replay_tensor_data.token_features_by_player,
        label_counts_by_player=replay_tensor_data.label_counts_by_player,
        relative_times=replay_tensor_data.relative_times,
    )


def load_replay_tensor_data_from_cache(cache_path: Path, replay_path: Path | None = None) -> ReplayTensorData:
    cached = np.load(cache_path, allow_pickle=True)
    return ReplayTensorData(
        replay_path=replay_path or Path(str(cached["replay_path"].item())),
        player_names=tuple(str(value) for value in cached["player_names"].tolist()),
        global_feature_headers=tuple(str(value) for value in cached["global_feature_headers"].tolist()),
        player_feature_headers=tuple(str(value) for value in cached["player_feature_headers"].tolist()),
        feature_names=tuple(str(value) for value in cached["feature_names"].tolist()),
        token_features_by_player=cached["token_features_by_player"].astype(np.float32),
        label_counts_by_player=cached["label_counts_by_player"].astype(np.float32),
        relative_times=cached["relative_times"].astype(np.float32),
    )


def _dilate_positive_frames(label_counts: np.ndarray, radius: int) -> np.ndarray:
    positives = label_counts > 0
    if radius <= 0:
        return positives

    dilated = positives.copy()
    for offset in range(1, radius + 1):
        dilated[offset:] |= positives[:-offset]
        dilated[:-offset] |= positives[offset:]
    return dilated


def _iter_player_window_samples(
    replay_data: ReplayTensorData,
    player_index: int,
    sampling: WindowSamplingConfig,
    rng: np.random.Generator,
) -> Iterable[tuple[np.ndarray, np.ndarray, np.ndarray, str, str, int]]:
    token_features = replay_data.token_features_by_player[player_index]
    label_counts = replay_data.label_counts_by_player[player_index]
    relative_times = replay_data.relative_times
    frame_count = token_features.shape[0]
    radius = sampling.window_radius

    if frame_count <= radius * 2:
        return []

    valid_centers = np.arange(radius, frame_count - radius)
    positive_mask = _dilate_positive_frames(label_counts, sampling.positive_radius_frames)
    positive_centers = valid_centers[positive_mask[valid_centers]]
    negative_centers = valid_centers[~positive_mask[valid_centers]]

    if positive_centers.size:
        max_negative_count = int(round(positive_centers.size * sampling.negative_to_positive_ratio))
        if max_negative_count < negative_centers.size:
            negative_centers = rng.choice(negative_centers, size=max_negative_count, replace=False)
    else:
        if sampling.negative_only_replay_sample_count < negative_centers.size:
            negative_centers = rng.choice(
                negative_centers,
                size=sampling.negative_only_replay_sample_count,
                replace=False,
            )

    selected_centers = np.concatenate([positive_centers, negative_centers])
    if selected_centers.size == 0:
        return []

    selected_centers = rng.permutation(selected_centers)
    samples = []
    for center_frame in selected_centers:
        frame_slice = slice(center_frame - radius, center_frame + radius + 1)
        center_time = relative_times[center_frame]
        label_window = (label_counts[frame_slice] > 0).astype(np.float32)
        samples.append(
            (
                token_features[frame_slice],
                (relative_times[frame_slice] - center_time).astype(np.float32),
                label_window,
                str(replay_data.replay_path),
                replay_data.player_names[player_index],
                int(center_frame),
            )
        )
    return samples


def build_windowed_training_data(
    replay_data_items: Iterable[ReplayTensorData],
    sampling: WindowSamplingConfig = WindowSamplingConfig(),
) -> WindowedTrainingData:
    rng = np.random.default_rng(sampling.random_seed)

    feature_rows: list[np.ndarray] = []
    relative_time_rows: list[np.ndarray] = []
    labels: list[np.ndarray] = []
    replay_paths: list[str] = []
    player_names: list[str] = []
    center_frames: list[int] = []
    feature_names: tuple[str, ...] | None = None

    for replay_data in replay_data_items:
        if feature_names is None:
            feature_names = replay_data.feature_names
        elif feature_names != replay_data.feature_names:
            raise ValueError("All replays must produce the same feature layout")

        for player_index in range(len(replay_data.player_names)):
            for (
                token_features,
                relative_times,
                label,
                replay_path,
                player_name,
                center_frame,
            ) in _iter_player_window_samples(replay_data, player_index, sampling, rng):
                feature_rows.append(token_features)
                relative_time_rows.append(relative_times)
                labels.append(label)
                replay_paths.append(replay_path)
                player_names.append(player_name)
                center_frames.append(center_frame)

    if not feature_rows:
        raise ValueError("No training windows were produced from the provided replays")

    return WindowedTrainingData(
        features=np.stack(feature_rows, axis=0).astype(np.float32),
        relative_times=np.stack(relative_time_rows, axis=0).astype(np.float32),
        labels=np.asarray(labels, dtype=np.float32),
        replay_paths=tuple(replay_paths),
        player_names=tuple(player_names),
        center_frames=np.asarray(center_frames, dtype=np.int32),
        feature_names=feature_names or tuple(),
    )


def build_pretraining_window_data(
    replay_data_items: Iterable[ReplayTensorData],
    window_radius: int,
    stride: int,
    max_windows_per_player: int,
    random_seed: int,
) -> PretrainingWindowData:
    rng = np.random.default_rng(random_seed)
    feature_rows: list[np.ndarray] = []
    relative_time_rows: list[np.ndarray] = []
    replay_paths: list[str] = []
    player_names: list[str] = []
    center_frames: list[int] = []
    feature_names: tuple[str, ...] | None = None

    for replay_data in replay_data_items:
        if feature_names is None:
            feature_names = replay_data.feature_names
        elif feature_names != replay_data.feature_names:
            raise ValueError("All replays must produce the same feature layout")

        for player_index, player_name in enumerate(replay_data.player_names):
            token_features = replay_data.token_features_by_player[player_index]
            relative_times = replay_data.relative_times
            frame_count = token_features.shape[0]
            if frame_count <= window_radius * 2:
                continue

            valid_centers = np.arange(window_radius, frame_count - window_radius, stride)
            if valid_centers.size == 0:
                continue
            if max_windows_per_player < valid_centers.size:
                valid_centers = np.sort(
                    rng.choice(valid_centers, size=max_windows_per_player, replace=False)
                )

            for center_frame in valid_centers:
                frame_slice = slice(center_frame - window_radius, center_frame + window_radius + 1)
                center_time = relative_times[center_frame]
                feature_rows.append(token_features[frame_slice])
                relative_time_rows.append(
                    (relative_times[frame_slice] - center_time).astype(np.float32)
                )
                replay_paths.append(str(replay_data.replay_path))
                player_names.append(player_name)
                center_frames.append(int(center_frame))

    if not feature_rows:
        raise ValueError("No pretraining windows were produced from the provided replays")

    return PretrainingWindowData(
        features=np.stack(feature_rows, axis=0).astype(np.float32),
        relative_times=np.stack(relative_time_rows, axis=0).astype(np.float32),
        replay_paths=tuple(replay_paths),
        player_names=tuple(player_names),
        center_frames=np.asarray(center_frames, dtype=np.int32),
        feature_names=feature_names or tuple(),
    )
