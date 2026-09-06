import unittest

import numpy as np
import pandas as pd
import torch

from entity_model import FEATURE_COUNT, GLOBAL_COUNT, GoalModel, labels
from train_entity_model import split_replays


class GoalModelTests(unittest.TestCase):
    def test_whole_player_permutations_preserve_prediction(self):
        torch.manual_seed(7)
        model = GoalModel(np.zeros(FEATURE_COUNT), np.ones(FEATURE_COUNT))
        original = torch.randn(10, FEATURE_COUNT)
        swapped = original.clone()
        swapped[:, GLOBAL_COUNT:] = (
            original[:, GLOBAL_COUNT:].reshape(-1, 4, 17)[:, [1, 0, 3, 2]].flatten(1)
        )
        torch.testing.assert_close(model(original), model(swapped), atol=1e-6, rtol=0)

    def test_boost_transfer_preserves_legacy_but_changes_entity_prediction(self):
        torch.manual_seed(7)
        model = GoalModel(np.zeros(FEATURE_COUNT), np.ones(FEATURE_COUNT))
        original = torch.randn(10, FEATURE_COUNT)
        swapped = original.clone()
        first, second = GLOBAL_COUNT + 2 * 17 + 11, GLOBAL_COUNT + 3 * 17 + 11
        swapped[:, [first, second]] = original[:, [second, first]]
        self.assertTrue(torch.equal(original[:, :GLOBAL_COUNT], swapped[:, :GLOBAL_COUNT]))
        self.assertGreater(float((model(original) - model(swapped)).abs().max().detach()), 1e-5)

    def test_touch_target_and_censoring(self):
        frame = pd.DataFrame(
            {
                "time_to_next_goal_for": [2, 2, 2, np.nan, np.nan, 12, 2],
                "time_to_next_goal_against": [np.nan, 1, np.nan, np.nan, np.nan, np.nan, np.nan],
                "time_to_next_touch": [3, 3, 1, 1, np.nan, np.nan, 2],
                "live_end_resolved": [0] * 7,
                "time_to_live_end": [20, 20, 20, 2, 2, 20, 20],
            }
        )
        y, keep = labels(frame, "touch", 10)
        np.testing.assert_array_equal(y, [1, 0, 0, 0, 0, 0, 0])
        np.testing.assert_array_equal(keep, [1, 1, 1, 1, 0, 1, 1])

    def test_goal_after_live_boundary_does_not_label_previous_stretch(self):
        frame = pd.DataFrame(
            {
                "time_to_next_goal_for": [2.0],
                "time_to_next_goal_against": [np.nan],
                "time_to_next_touch": [np.nan],
                "time_to_live_end": [1.0],
                "live_end_resolved": [1],
            }
        )
        for task in ["touch", "frame"]:
            y, observed = labels(frame, task, 5.0)
            np.testing.assert_array_equal(y, [0])
            np.testing.assert_array_equal(observed, [1])

    def test_split_groups_both_teams_and_tied_dates(self):
        frame = pd.DataFrame(
            {"replay_id": np.repeat(np.arange(100), 2), "date": ["2026-01-01"] * 200}
        )
        parts = split_replays(frame.sample(frac=1, random_state=7).reset_index(drop=True))
        sets = [set(part["replay_ids"]) for part in parts.values()]
        self.assertEqual([len(group) for group in sets], [60, 15, 10, 15])
        self.assertEqual(len(set.union(*sets)), 100)
        for index, group in enumerate(sets):
            for other in sets[index + 1 :]:
                self.assertFalse(group & other)


class BoundaryTests(unittest.TestCase):
    def test_stoppage_is_resolved_but_missing_future_is_censored(self):
        frame = pd.DataFrame(
            {
                "time_to_next_goal_for": [np.nan, np.nan, 0.5],
                "time_to_next_goal_against": [np.nan, np.nan, np.nan],
                "time_to_next_touch": [np.nan, np.nan, np.nan],
                "time_to_live_end": [1.0, 1.0, 0.5],
                "live_end_resolved": [1, 0, 1],
            }
        )
        for task in ["touch", "frame"]:
            y, observed = labels(frame, task, 5.0)
            np.testing.assert_array_equal(y, [0, 0, 1])
            np.testing.assert_array_equal(observed, [1, 0, 1])


if __name__ == "__main__":
    unittest.main()
