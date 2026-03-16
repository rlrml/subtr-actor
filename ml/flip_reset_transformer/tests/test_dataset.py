from __future__ import annotations

import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

import numpy as np

from flip_reset_transformer.config import WindowSamplingConfig
from flip_reset_transformer.dataset import (
    _build_player_token_features,
    build_pretraining_window_data,
    ReplayTensorData,
    build_windowed_training_data,
    collect_replay_paths,
    split_input_and_label_channels,
)


class DatasetTests(unittest.TestCase):
    def test_split_input_and_label_channels_excludes_label_and_time(self) -> None:
        array = np.array(
            [
                [
                    10.0,
                    1.0,
                    20.0,
                    30.0,
                    0.0,
                    40.0,
                    50.0,
                    1.0,
                ],
                [
                    11.0,
                    2.0,
                    21.0,
                    31.0,
                    1.0,
                    41.0,
                    51.0,
                    0.0,
                ],
            ],
            dtype=np.float32,
        )
        global_headers = ["Ball - position x", "current time"]
        player_headers = ["position x", "boost level (raw replay units)", "dodge refresh count"]

        global_inputs, player_inputs, labels, times, feature_names = split_input_and_label_channels(
            array=array,
            global_headers=global_headers,
            player_headers=player_headers,
            player_count=2,
        )

        self.assertEqual(global_inputs.shape, (2, 1))
        self.assertEqual(player_inputs.shape, (2, 2, 2))
        self.assertEqual(labels.shape, (2, 2))
        self.assertEqual(feature_names, ("Ball - position x", "position x", "boost level (raw replay units)"))
        np.testing.assert_allclose(times, np.array([1.0, 2.0], dtype=np.float32))
        np.testing.assert_allclose(labels[0], np.array([0.0, 1.0], dtype=np.float32))

    def test_build_windowed_training_data_produces_positive_and_negative_windows(self) -> None:
        frame_count = 7
        feature_count = 4
        replay = ReplayTensorData(
            replay_path=Path("/tmp/example.replay"),
            player_names=("player-1",),
            global_feature_headers=("Ball - position x",),
            player_feature_headers=("position x", "linear velocity x"),
            feature_names=("Ball - position x", "position x", "linear velocity x", "relative ball position x"),
            token_features_by_player=np.arange(frame_count * feature_count, dtype=np.float32).reshape(
                1, frame_count, feature_count
            ),
            label_counts_by_player=np.array([[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0]], dtype=np.float32),
            relative_times=np.linspace(0.0, 0.2, frame_count, dtype=np.float32),
        )

        dataset = build_windowed_training_data(
            [replay],
            WindowSamplingConfig(
                window_radius=1,
                positive_radius_frames=0,
                negative_to_positive_ratio=1.0,
                random_seed=3,
            ),
        )

        self.assertEqual(dataset.features.shape[1:], (3, feature_count))
        self.assertEqual(dataset.labels.shape[1], 3)
        self.assertTrue(np.any(dataset.labels.sum(axis=1) > 0.0))
        self.assertTrue(np.any(dataset.labels.sum(axis=1) == 0.0))
        self.assertTrue(np.any(np.all(dataset.labels == np.array([0.0, 1.0, 0.0], dtype=np.float32), axis=1)))

    def test_build_windowed_training_data_keeps_exact_sequence_labels(self) -> None:
        replay = ReplayTensorData(
            replay_path=Path("/tmp/example.replay"),
            player_names=("player-1",),
            global_feature_headers=("Ball - position x",),
            player_feature_headers=("position x",),
            feature_names=("Ball - position x", "position x"),
            token_features_by_player=np.arange(18, dtype=np.float32).reshape(1, 9, 2),
            label_counts_by_player=np.array([[0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0]], dtype=np.float32),
            relative_times=np.linspace(0.0, 0.3, 9, dtype=np.float32),
        )
        dataset = build_windowed_training_data(
            [replay],
            WindowSamplingConfig(
                window_radius=1,
                positive_radius_frames=0,
                negative_to_positive_ratio=0.0,
                random_seed=5,
            ),
        )
        label_windows = {tuple(row.tolist()) for row in dataset.labels}
        self.assertIn((0.0, 1.0, 0.0), label_windows)

    def test_build_player_token_features_preserves_rust_relative_features(self) -> None:
        global_inputs = np.array([[100.0], [110.0]], dtype=np.float32)
        player_inputs = np.array(
            [
                [90.0, 10.0],
                [95.0, 15.0],
            ],
            dtype=np.float32,
        )
        feature_names = (
            "Ball - position x",
            "position x",
            "relative ball position x",
        )

        token_features, augmented_features = _build_player_token_features(
            global_inputs,
            player_inputs,
            feature_names,
            include_geometry_scalars=False,
        )

        self.assertEqual(augmented_features, feature_names)
        np.testing.assert_allclose(
            token_features,
            np.array(
                [
                    [100.0, 90.0, 10.0],
                    [110.0, 95.0, 15.0],
                ],
                dtype=np.float32,
            ),
        )

    def test_build_player_token_features_adds_geometry_scalars(self) -> None:
        global_inputs = np.array(
            [
                [100.0, 0.0, 50.0, 10.0, 0.0, 0.0],
                [110.0, 0.0, 60.0, 10.0, 0.0, 0.0],
            ],
            dtype=np.float32,
        )
        player_inputs = np.array(
            [
                [90.0, 0.0, 30.0, 5.0, 0.0, 0.0, 10.0, 0.0, 20.0, 5.0, 0.0, 0.0],
                [100.0, 0.0, 45.0, 3.0, 4.0, 0.0, 10.0, 0.0, 15.0, 7.0, -4.0, 0.0],
            ],
            dtype=np.float32,
        )
        feature_names = (
            "Ball - position x",
            "Ball - position y",
            "Ball - position z",
            "Ball - linear velocity x",
            "Ball - linear velocity y",
            "Ball - linear velocity z",
            "position x",
            "position y",
            "position z",
            "linear velocity x",
            "linear velocity y",
            "linear velocity z",
            "relative ball position x",
            "relative ball position y",
            "relative ball position z",
            "relative ball velocity x",
            "relative ball velocity y",
            "relative ball velocity z",
        )

        token_features, augmented_features = _build_player_token_features(
            global_inputs,
            player_inputs,
            feature_names,
            include_geometry_scalars=True,
        )

        self.assertIn("relative ball distance", augmented_features)
        self.assertIn("closing speed", augmented_features)
        distance_index = augmented_features.index("relative ball distance")
        closing_speed_index = augmented_features.index("closing speed")
        np.testing.assert_allclose(token_features[:, distance_index], np.array([22.36068, 18.027756], dtype=np.float32))
        self.assertLess(token_features[0, closing_speed_index], 0.0)

    def test_build_pretraining_window_data(self) -> None:
        replay = ReplayTensorData(
            replay_path=Path("/tmp/example.replay"),
            player_names=("player-1",),
            global_feature_headers=("Ball - position x",),
            player_feature_headers=("position x",),
            feature_names=("Ball - position x", "position x"),
            token_features_by_player=np.arange(20, dtype=np.float32).reshape(1, 10, 2),
            label_counts_by_player=np.zeros((1, 10), dtype=np.float32),
            relative_times=np.linspace(0.0, 0.3, 10, dtype=np.float32),
        )
        pretraining = build_pretraining_window_data(
            [replay],
            window_radius=1,
            stride=2,
            max_windows_per_player=10,
            random_seed=7,
        )
        self.assertEqual(pretraining.features.shape[1:], (3, 2))
        self.assertGreater(pretraining.features.shape[0], 0)

    def test_collect_replay_paths_prefers_replays_directory_and_dedupes_by_stem(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            replay_dir = root / "replays"
            replay_dir.mkdir()
            cache_dir = root / "cache"
            cache_dir.mkdir()
            preferred = replay_dir / "example.replay"
            duplicate = cache_dir / "example.replay"
            extra = root / "other.replay"
            preferred.write_bytes(b"replay-a")
            duplicate.write_bytes(b"replay-b")
            extra.write_bytes(b"replay-c")

            replay_paths = collect_replay_paths(replay_dirs=(root,))

            self.assertEqual(replay_paths, [preferred.resolve(), extra.resolve()])


if __name__ == "__main__":
    unittest.main()
