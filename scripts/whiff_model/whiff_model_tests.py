import json
import pathlib
import tempfile
import unittest

import numpy as np

import adjudicate
import labeling
import mining
import modeling


def judgment(*interactions, usable=True):
    return {"window_usable": usable, "unusable_reason": None, "interactions": list(interactions)}


def interaction(t, label, attempt="yes", contact="no", disrupted="no", fake="no"):
    return {
        "closest_approach_t": t,
        "attempt_start_t": None,
        "commit_t": None,
        "attempt_observable": attempt,
        "focal_contact": contact,
        "disrupted_by_other_player": disrupted,
        "possible_fake": fake,
        "label": label,
        "evidence": [],
        "rationale": "",
        "uncertainty_reason": None,
        "confidence": "medium",
    }


class GapMinimaTests(unittest.TestCase):
    def test_minimum_must_dominate_neighbourhood_and_be_live(self):
        time = np.arange(0, 3, 0.05)
        gap = np.full_like(time, 900.0)
        gap[10] = 100.0
        gap[40] = 50.0
        live = np.ones_like(time, dtype=bool)
        live[40] = False
        self.assertEqual(mining.gap_minima(gap, time, live, 600.0, 0.25), [10])

    def test_flat_contact_run_shorter_than_half_window_yields_one_minimum(self):
        time = np.arange(0, 2, 0.05)
        gap = np.full_like(time, 800.0)
        gap[10:14] = 0.0
        live = np.ones_like(time, dtype=bool)
        self.assertEqual(mining.gap_minima(gap, time, live, 600.0, 0.25), [10])


class ValidationTests(unittest.TestCase):
    core = [3.0, 5.0]

    def test_disrupted_and_clean_miss_are_distinct_consistent_labels(self):
        ok = judgment(
            interaction(3.5, "clear_whiff"), interaction(4.5, "disrupted_attempt", disrupted="yes")
        )
        self.assertEqual(labeling.validate_judgment(ok, self.core), [])
        bad = judgment(interaction(3.5, "clear_whiff", disrupted="yes"))
        self.assertTrue(
            any("inconsistent" in e for e in labeling.validate_judgment(bad, self.core))
        )

    def test_possible_fake_cannot_be_clear_whiff(self):
        errors = labeling.validate_judgment(
            judgment(interaction(4.0, "clear_whiff", fake="yes")), self.core
        )
        self.assertTrue(any("possible_fake" in e for e in errors))

    def test_interaction_outside_core_is_rejected(self):
        errors = labeling.validate_judgment(
            judgment(interaction(1.0, "successful_contact", contact="yes")), self.core
        )
        self.assertTrue(any("outside core" in e for e in errors))

    def test_extract_json_handles_fenced_responses(self):
        self.assertEqual(labeling.extract_json('noise ```json\n{"a": 1}\n``` tail'), {"a": 1})

    def test_window_outcome_prefers_misses_over_contacts(self):
        mixed = judgment(
            interaction(3.2, "successful_contact", contact="yes"),
            interaction(4.0, "disrupted_attempt", disrupted="yes"),
        )
        self.assertEqual(labeling.window_outcome(mixed), "disrupted_attempt")
        self.assertEqual(labeling.window_outcome(judgment()), "no_observable_attempt")


class AdjudicationTests(unittest.TestCase):
    def test_matching_pairs_by_time_and_keeps_unmatched(self):
        a = [interaction(3.1, "clear_whiff"), interaction(4.6, "successful_contact", contact="yes")]
        b = [interaction(3.3, "clear_whiff")]
        pairs = adjudicate.match_interactions(a, b)
        self.assertEqual(
            [(adjudicate.label_of(x), adjudicate.label_of(y)) for x, y in pairs],
            [("clear_whiff", "clear_whiff"), ("successful_contact", adjudicate.ABSENT)],
        )

    def test_resolution_tiers(self):
        self.assertEqual(
            adjudicate.resolve("clear_whiff", "clear_whiff", []), ("clear_whiff", "agreed")
        )
        self.assertEqual(
            adjudicate.resolve("clear_whiff", "disrupted_attempt", ["disrupted_attempt"]),
            ("disrupted_attempt", "adjudicated"),
        )
        self.assertEqual(
            adjudicate.resolve(
                "clear_whiff", "disrupted_attempt", ["uncertain", "successful_contact"]
            ),
            ("uncertain", "unresolved"),
        )


class FakeTransport(labeling.Transport):
    name = "fake"

    def __init__(self, responses):
        super().__init__("fake-model")
        self.responses = list(responses)
        self.calls = 0

    def call(self, prompt, image_paths, work_dir):
        self.calls += 1
        return {"ok": True, "raw": self.responses.pop(0), "latency": 0.0, "cost_usd": 0.01}


class LedgerResumeTests(unittest.TestCase):
    def test_valid_labels_are_not_repeated_and_invalid_ones_retry(self):
        task = {
            "neutral_id": "w1",
            "pass": "primary",
            "representation": "combined",
            "image_paths": [],
            "image_sha256": ["abc"],
            "renderer_version": "r",
            "core_interval": [3.0, 5.0],
            "prompt": "p",
        }
        good = json.dumps(judgment(interaction(4.0, "clear_whiff")))
        with tempfile.TemporaryDirectory() as tmp:
            ledger = labeling.Ledger(pathlib.Path(tmp) / "attempts.jsonl")
            transport = FakeTransport(["not json", good])
            stats = labeling.run_tasks([task], transport, ledger, pathlib.Path(tmp), 10, 10.0, 1)
            self.assertEqual((stats["valid"], stats["invalid"], transport.calls), (1, 1, 2))
            again = FakeTransport([])
            labeling.run_tasks([task], again, ledger, pathlib.Path(tmp), 10, 10.0, 1)
            self.assertEqual(again.calls, 0)
            records = ledger.records()
            self.assertEqual(len(records), 2)
            self.assertEqual(len({r["attempt_id"] for r in records}), 2)
            self.assertEqual([r["valid"] for r in records], [False, True])


class EventMetricTests(unittest.TestCase):
    def test_uncertain_labels_do_not_create_false_positives(self):
        preds = [
            {"neutral_id": "a", "t": 10.0, "label": "clear_whiff"},
            {"neutral_id": "a", "t": 20.0, "label": "clear_whiff"},
            {"neutral_id": "b", "t": 5.0, "label": "disrupted_attempt"},
        ]
        labels = [
            {"neutral_id": "a", "t": 10.2, "label": "clear_whiff"},
            {"neutral_id": "a", "t": 20.1, "label": "uncertain"},
            {"neutral_id": "b", "t": 5.1, "label": "clear_whiff"},
        ]
        metrics = modeling.event_metrics(preds, labels)
        self.assertEqual(
            (
                metrics["clear_whiff"]["tp"],
                metrics["clear_whiff"]["fp"],
                metrics["clear_whiff"]["fn"],
            ),
            (1, 0, 1),
        )
        self.assertEqual(
            (metrics["disrupted_attempt"]["tp"], metrics["disrupted_attempt"]["fp"]), (0, 1)
        )

    def test_greedy_match_is_one_to_one(self):
        self.assertEqual(sorted(modeling.greedy_match([1.0, 1.1], [1.05], 0.4)), [(0, 0)])


if __name__ == "__main__":
    unittest.main()
