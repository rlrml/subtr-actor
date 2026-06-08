import os

import pytest

from . import test_directory

import subtr_actor


REPLAY_PATH = os.path.join(
    test_directory, "029103f9-4d58-4964-b47a-539b32f6fb33.replay"
)


def test_get_ndarray_with_info_from_replay_filepath():
    meta, ndarray = subtr_actor.get_ndarray_with_info_from_replay_filepath(REPLAY_PATH)

    assert ndarray.shape[0] > 0
    assert ndarray.shape[1] > 0
    assert "replay_meta" in meta


def test_get_stats_module_names():
    module_names = subtr_actor.get_stats_module_names()

    assert "core" in module_names
    assert "boost" in module_names
    assert "movement" in module_names


def test_get_stats_events():
    events = subtr_actor.get_stats_events(REPLAY_PATH)

    assert "timeline" in events
    assert "boost_ledger" in events
    assert "core_player" in events


def test_get_summed_stats_with_module_selection():
    summed_stats = subtr_actor.get_summed_stats(
        REPLAY_PATH,
        module_names=["core", "boost"],
    )

    assert set(summed_stats["modules"]) == {"core", "boost"}
    assert "score" in summed_stats["modules"]["core"]["team_zero"]
    assert "amount_collected" in summed_stats["modules"]["boost"]["team_zero"]


def test_get_summed_stats_rejects_unknown_module_name():
    with pytest.raises(ValueError, match="Unknown builtin stats module"):
        subtr_actor.get_summed_stats(REPLAY_PATH, module_names=["not_a_real_module"])


def test_get_stats_snapshot_data_with_sampling():
    snapshot_data = subtr_actor.get_stats_snapshot_data(
        REPLAY_PATH,
        module_names=["core", "boost"],
        frame_step_seconds=1.0,
    )

    assert set(snapshot_data["modules"]) == {"core", "boost"}
    assert snapshot_data["frames"]
    assert set(snapshot_data["frames"][0]["modules"]).issubset({"core", "boost"})


def test_get_stats_timeline_with_sampling():
    compact_timeline = subtr_actor.get_stats_timeline(REPLAY_PATH)
    sampled_timeline = subtr_actor.get_stats_timeline(
        REPLAY_PATH,
        frame_step_seconds=1.0,
    )

    assert sampled_timeline["frames"]
    assert len(sampled_timeline["frames"]) < len(compact_timeline["frames"])
    for frame in sampled_timeline["frames"]:
        assert frame["team_zero"] == {}
        assert frame["team_one"] == {}
        for player in frame["players"]:
            assert set(player) == {"player_id", "name", "is_team_0"}
    assert "timeline" in sampled_timeline["events"]
    assert "boost_ledger" in sampled_timeline["events"]
    assert "core_player" in sampled_timeline["events"]


def test_get_legacy_stats_timeline_with_module_filtering():
    timeline = subtr_actor.get_legacy_stats_timeline(
        REPLAY_PATH,
        module_names=["core", "boost"],
        frame_step_seconds=1.0,
    )

    assert timeline["frames"]
    assert "boost" in timeline["frames"][-1]["team_zero"]
    assert "core" in timeline["frames"][-1]["team_zero"]


def test_get_stats_timeline_rejects_module_filtering():
    with pytest.raises(ValueError, match="module_names filtering"):
        subtr_actor.get_stats_timeline(REPLAY_PATH, module_names=["core", "boost"])


def test_get_stats_timeline_rejects_invalid_sampling_step():
    with pytest.raises(ValueError, match="frame_step_seconds"):
        subtr_actor.get_stats_timeline(REPLAY_PATH, frame_step_seconds=0.0)
