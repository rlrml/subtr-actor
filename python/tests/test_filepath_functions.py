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


def test_get_stats_with_module_selection():
    stats = subtr_actor.get_stats(REPLAY_PATH, module_names=["core", "boost"])

    assert set(stats["modules"]) == {"core", "boost"}
    assert "player_stats" in stats["modules"]["core"]
    assert "player_stats" in stats["modules"]["boost"]


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
    full_timeline = subtr_actor.get_stats_timeline(REPLAY_PATH, module_names=["core", "boost"])
    sampled_timeline = subtr_actor.get_stats_timeline(
        REPLAY_PATH,
        module_names=["core", "boost"],
        frame_step_seconds=1.0,
    )

    assert sampled_timeline["frames"]
    assert len(sampled_timeline["frames"]) < len(full_timeline["frames"])
    assert "boost" in sampled_timeline["frames"][-1]["team_zero"]
    assert "timeline" in sampled_timeline["events"]


def test_get_stats_timeline_rejects_invalid_sampling_step():
    with pytest.raises(ValueError, match="frame_step_seconds"):
        subtr_actor.get_stats_timeline(REPLAY_PATH, frame_step_seconds=0.0)


def test_get_stats_rejects_unknown_module_name():
    with pytest.raises(ValueError, match="Unknown builtin stats module"):
        subtr_actor.get_stats(REPLAY_PATH, module_names=["not_a_real_module"])
