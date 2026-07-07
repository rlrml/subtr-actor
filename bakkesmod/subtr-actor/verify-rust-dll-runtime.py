#!/usr/bin/env python3
"""Load a built Rust BakkesMod DLL and exercise its live analysis ABI.

This runs on Windows CI against the produced subtr_actor_bakkesmod.dll artifact.
It intentionally goes beyond export-table validation: the script loads the DLL,
creates an engine, feeds BakkesMod-shaped live frames with every explicit event
family, then calls graph outputs, every advertised analysis node, stats modules,
and drain APIs through ctypes.
"""

from __future__ import annotations

import argparse
import ctypes
import json
import sys
from pathlib import Path


class SaVec3(ctypes.Structure):
    _fields_ = [
        ("x", ctypes.c_float),
        ("y", ctypes.c_float),
        ("z", ctypes.c_float),
    ]


class SaQuat(ctypes.Structure):
    _fields_ = [
        ("x", ctypes.c_float),
        ("y", ctypes.c_float),
        ("z", ctypes.c_float),
        ("w", ctypes.c_float),
    ]


class SaRigidBody(ctypes.Structure):
    _fields_ = [
        ("location", SaVec3),
        ("rotation", SaQuat),
        ("linear_velocity", SaVec3),
        ("angular_velocity", SaVec3),
        ("has_linear_velocity", ctypes.c_uint8),
        ("has_angular_velocity", ctypes.c_uint8),
        ("sleeping", ctypes.c_uint8),
    ]


class SaPlayerFrame(ctypes.Structure):
    _fields_ = [
        ("player_index", ctypes.c_uint32),
        ("player_name", ctypes.c_char_p),
        ("is_team_0", ctypes.c_uint8),
        ("has_rigid_body", ctypes.c_uint8),
        ("rigid_body", SaRigidBody),
        ("boost_amount", ctypes.c_float),
        ("last_boost_amount", ctypes.c_float),
        ("boost_active", ctypes.c_uint8),
        ("jump_active", ctypes.c_uint8),
        ("double_jump_active", ctypes.c_uint8),
        ("dodge_active", ctypes.c_uint8),
        ("powerslide_active", ctypes.c_uint8),
        ("car_body_id", ctypes.c_int32),
        ("has_car_body_id", ctypes.c_uint8),
        ("has_match_stats", ctypes.c_uint8),
        ("match_goals", ctypes.c_int32),
        ("match_assists", ctypes.c_int32),
        ("match_saves", ctypes.c_int32),
        ("match_shots", ctypes.c_int32),
        ("match_score", ctypes.c_int32),
    ]


class SaEventTiming(ctypes.Structure):
    _fields_ = [
        ("frame_number", ctypes.c_uint64),
        ("time", ctypes.c_float),
        ("seconds_remaining", ctypes.c_int32),
        ("has_timing", ctypes.c_uint8),
        ("has_seconds_remaining", ctypes.c_uint8),
    ]


class SaTouchEvent(ctypes.Structure):
    _fields_ = [
        ("timing", SaEventTiming),
        ("player_index", ctypes.c_uint32),
        ("has_player", ctypes.c_uint8),
        ("is_team_0", ctypes.c_uint8),
        ("closest_approach_distance", ctypes.c_float),
        ("has_closest_approach_distance", ctypes.c_uint8),
    ]


class SaDodgeRefreshedEvent(ctypes.Structure):
    _fields_ = [
        ("timing", SaEventTiming),
        ("player_index", ctypes.c_uint32),
        ("is_team_0", ctypes.c_uint8),
        ("counter_value", ctypes.c_int32),
    ]


class SaBoostPadEvent(ctypes.Structure):
    _fields_ = [
        ("timing", SaEventTiming),
        ("pad_id", ctypes.c_uint32),
        ("kind", ctypes.c_int),
        ("sequence", ctypes.c_uint8),
        ("player_index", ctypes.c_uint32),
        ("has_player", ctypes.c_uint8),
    ]


class SaGoalEvent(ctypes.Structure):
    _fields_ = [
        ("timing", SaEventTiming),
        ("scoring_team_is_team_0", ctypes.c_uint8),
        ("player_index", ctypes.c_uint32),
        ("has_player", ctypes.c_uint8),
        ("team_zero_score", ctypes.c_int32),
        ("has_team_zero_score", ctypes.c_uint8),
        ("team_one_score", ctypes.c_int32),
        ("has_team_one_score", ctypes.c_uint8),
    ]


class SaPlayerStatEvent(ctypes.Structure):
    _fields_ = [
        ("timing", SaEventTiming),
        ("player_index", ctypes.c_uint32),
        ("is_team_0", ctypes.c_uint8),
        ("kind", ctypes.c_int),
        ("has_shot_ball", ctypes.c_uint8),
        ("shot_ball", SaRigidBody),
        ("has_shot_player", ctypes.c_uint8),
        ("shot_player", SaRigidBody),
    ]


class SaDemolishEvent(ctypes.Structure):
    _fields_ = [
        ("timing", SaEventTiming),
        ("attacker_index", ctypes.c_uint32),
        ("victim_index", ctypes.c_uint32),
        ("attacker_velocity", SaVec3),
        ("victim_velocity", SaVec3),
        ("victim_location", SaVec3),
        ("active_duration_seconds", ctypes.c_float),
    ]


class SaLiveFrame(ctypes.Structure):
    _fields_ = [
        ("frame_number", ctypes.c_uint64),
        ("time", ctypes.c_float),
        ("dt", ctypes.c_float),
        ("seconds_remaining", ctypes.c_int32),
        ("has_seconds_remaining", ctypes.c_uint8),
        ("game_state", ctypes.c_int32),
        ("has_game_state", ctypes.c_uint8),
        ("kickoff_countdown_time", ctypes.c_int32),
        ("has_kickoff_countdown_time", ctypes.c_uint8),
        ("ball_has_been_hit", ctypes.c_uint8),
        ("has_ball_has_been_hit", ctypes.c_uint8),
        ("team_zero_score", ctypes.c_int32),
        ("has_team_zero_score", ctypes.c_uint8),
        ("team_one_score", ctypes.c_int32),
        ("has_team_one_score", ctypes.c_uint8),
        ("possession_team_is_team_0", ctypes.c_uint8),
        ("has_possession_team", ctypes.c_uint8),
        ("scored_on_team_is_team_0", ctypes.c_uint8),
        ("has_scored_on_team", ctypes.c_uint8),
        ("live_play", ctypes.c_uint8),
        ("has_live_play", ctypes.c_uint8),
        ("has_ball", ctypes.c_uint8),
        ("ball", SaRigidBody),
        ("players", ctypes.POINTER(SaPlayerFrame)),
        ("player_count", ctypes.c_size_t),
        ("touches", ctypes.POINTER(SaTouchEvent)),
        ("touch_count", ctypes.c_size_t),
        ("dodge_refreshes", ctypes.POINTER(SaDodgeRefreshedEvent)),
        ("dodge_refresh_count", ctypes.c_size_t),
        ("boost_pad_events", ctypes.POINTER(SaBoostPadEvent)),
        ("boost_pad_event_count", ctypes.c_size_t),
        ("goals", ctypes.POINTER(SaGoalEvent)),
        ("goal_count", ctypes.c_size_t),
        ("player_stat_events", ctypes.POINTER(SaPlayerStatEvent)),
        ("player_stat_event_count", ctypes.c_size_t),
        ("demolishes", ctypes.POINTER(SaDemolishEvent)),
        ("demolish_count", ctypes.c_size_t),
    ]


class SaMechanicEvent(ctypes.Structure):
    _fields_ = [
        ("kind", ctypes.c_int),
        ("player_index", ctypes.c_uint32),
        ("is_team_0", ctypes.c_uint8),
        ("frame_number", ctypes.c_uint64),
        ("time", ctypes.c_float),
        ("confidence", ctypes.c_float),
    ]


class SaTeamEvent(ctypes.Structure):
    _fields_ = [
        ("kind", ctypes.c_int),
        ("is_team_0", ctypes.c_uint8),
        ("start_frame", ctypes.c_uint64),
        ("end_frame", ctypes.c_uint64),
        ("start_time", ctypes.c_float),
        ("end_time", ctypes.c_float),
        ("attackers", ctypes.c_uint32),
        ("defenders", ctypes.c_uint32),
        ("confidence", ctypes.c_float),
    ]


class SaGoalContextEvent(ctypes.Structure):
    _fields_ = [
        ("frame_number", ctypes.c_uint64),
        ("time", ctypes.c_float),
        ("scoring_team_is_team_0", ctypes.c_uint8),
        ("has_scorer", ctypes.c_uint8),
        ("scorer_index", ctypes.c_uint32),
        ("has_scoring_team_most_back_player", ctypes.c_uint8),
        ("scoring_team_most_back_player_index", ctypes.c_uint32),
        ("has_defending_team_most_back_player", ctypes.c_uint8),
        ("defending_team_most_back_player_index", ctypes.c_uint32),
        ("has_ball_position", ctypes.c_uint8),
        ("ball_position", SaVec3),
        ("has_ball_air_time_before_goal", ctypes.c_uint8),
        ("ball_air_time_before_goal", ctypes.c_float),
        ("goal_buildup", ctypes.c_int),
    ]


class RustAbi:
    def __init__(self, path: Path):
        self.dll = ctypes.CDLL(str(path.resolve()))
        self._configure_functions()

    def _configure_functions(self) -> None:
        self.dll.subtr_actor_bakkesmod_engine_create.argtypes = []
        self.dll.subtr_actor_bakkesmod_engine_create.restype = ctypes.c_void_p
        self.dll.subtr_actor_bakkesmod_engine_destroy.argtypes = [ctypes.c_void_p]
        self.dll.subtr_actor_bakkesmod_engine_destroy.restype = None
        self.dll.subtr_actor_bakkesmod_finish.argtypes = [ctypes.c_void_p]
        self.dll.subtr_actor_bakkesmod_finish.restype = ctypes.c_int32
        self.dll.subtr_actor_bakkesmod_process_frame.argtypes = [
            ctypes.c_void_p,
            ctypes.POINTER(SaLiveFrame),
        ]
        self.dll.subtr_actor_bakkesmod_process_frame.restype = ctypes.c_int32

        for name in (
            "events",
            "frame",
            "timeline",
            "stats",
            "graph_info",
            "analysis_node_names",
        ):
            len_fn = getattr(self.dll, f"subtr_actor_bakkesmod_{name}_json_len")
            write_fn = getattr(self.dll, f"subtr_actor_bakkesmod_write_{name}_json")
            len_fn.argtypes = [ctypes.c_void_p]
            len_fn.restype = ctypes.c_size_t
            write_fn.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t]
            write_fn.restype = ctypes.c_size_t

        for name in (
            "stats_module",
            "stats_module_frame",
            "stats_module_config",
            "graph_output",
            "analysis_node",
        ):
            len_fn = getattr(self.dll, f"subtr_actor_bakkesmod_{name}_json_len")
            write_fn = getattr(self.dll, f"subtr_actor_bakkesmod_write_{name}_json")
            len_fn.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
            len_fn.restype = ctypes.c_size_t
            write_fn.argtypes = [
                ctypes.c_void_p,
                ctypes.c_char_p,
                ctypes.POINTER(ctypes.c_uint8),
                ctypes.c_size_t,
            ]
            write_fn.restype = ctypes.c_size_t

        self.dll.subtr_actor_bakkesmod_pending_event_count.argtypes = [ctypes.c_void_p]
        self.dll.subtr_actor_bakkesmod_pending_event_count.restype = ctypes.c_size_t
        self.dll.subtr_actor_bakkesmod_pending_team_event_count.argtypes = [ctypes.c_void_p]
        self.dll.subtr_actor_bakkesmod_pending_team_event_count.restype = ctypes.c_size_t
        self.dll.subtr_actor_bakkesmod_pending_goal_context_event_count.argtypes = [ctypes.c_void_p]
        self.dll.subtr_actor_bakkesmod_pending_goal_context_event_count.restype = ctypes.c_size_t
        self.dll.subtr_actor_bakkesmod_drain_events.argtypes = [
            ctypes.c_void_p,
            ctypes.POINTER(SaMechanicEvent),
            ctypes.c_size_t,
        ]
        self.dll.subtr_actor_bakkesmod_drain_events.restype = ctypes.c_size_t
        self.dll.subtr_actor_bakkesmod_drain_team_events.argtypes = [
            ctypes.c_void_p,
            ctypes.POINTER(SaTeamEvent),
            ctypes.c_size_t,
        ]
        self.dll.subtr_actor_bakkesmod_drain_team_events.restype = ctypes.c_size_t
        self.dll.subtr_actor_bakkesmod_drain_goal_context_events.argtypes = [
            ctypes.c_void_p,
            ctypes.POINTER(SaGoalContextEvent),
            ctypes.c_size_t,
        ]
        self.dll.subtr_actor_bakkesmod_drain_goal_context_events.restype = ctypes.c_size_t

    def json_value(self, engine: int, name: str) -> object:
        len_fn = getattr(self.dll, f"subtr_actor_bakkesmod_{name}_json_len")
        write_fn = getattr(self.dll, f"subtr_actor_bakkesmod_write_{name}_json")
        return read_json(engine, len_fn, write_fn)

    def named_json_value(self, engine: int, name: str, key: str) -> object:
        len_fn = getattr(self.dll, f"subtr_actor_bakkesmod_{name}_json_len")
        write_fn = getattr(self.dll, f"subtr_actor_bakkesmod_write_{name}_json")
        return read_named_json(engine, len_fn, write_fn, key)


def rigid_body(location: SaVec3, linear_velocity: SaVec3 | None = None) -> SaRigidBody:
    body = SaRigidBody()
    body.location = location
    body.rotation = SaQuat()
    body.linear_velocity = linear_velocity if linear_velocity is not None else SaVec3()
    body.angular_velocity = SaVec3()
    body.has_linear_velocity = 1
    body.has_angular_velocity = 1
    return body


def vec3(x: float, y: float, z: float) -> SaVec3:
    return SaVec3(ctypes.c_float(x), ctypes.c_float(y), ctypes.c_float(z))


def timing(frame_number: int) -> SaEventTiming:
    value = SaEventTiming()
    value.frame_number = frame_number
    value.time = ctypes.c_float(frame_number * 0.1)
    value.seconds_remaining = 299
    value.has_timing = 1
    value.has_seconds_remaining = 1
    return value


def player(player_index: int, is_team_0: bool, location: SaVec3) -> SaPlayerFrame:
    value = SaPlayerFrame()
    value.player_index = player_index
    value.player_name = None
    value.is_team_0 = 1 if is_team_0 else 0
    value.has_rigid_body = 1
    value.rigid_body = rigid_body(location)
    value.boost_amount = 33.0
    value.last_boost_amount = 33.0
    value.car_body_id = 23
    value.has_car_body_id = 1
    value.has_match_stats = 1
    value.match_goals = player_index
    value.match_assists = player_index + 1
    value.match_saves = player_index + 2
    value.match_shots = player_index + 3
    value.match_score = player_index + 100
    return value


def live_frame(frame_number: int, players, ball: SaRigidBody) -> SaLiveFrame:
    frame = SaLiveFrame()
    frame.frame_number = frame_number
    frame.time = ctypes.c_float(frame_number * 0.1)
    frame.dt = ctypes.c_float(0.1)
    frame.seconds_remaining = 299
    frame.has_seconds_remaining = 1
    frame.ball_has_been_hit = 1
    frame.has_ball_has_been_hit = 1
    frame.live_play = 1
    frame.has_live_play = 1
    frame.has_ball = 1
    frame.ball = ball
    frame.players = ctypes.cast(players, ctypes.POINTER(SaPlayerFrame))
    frame.player_count = len(players)
    return frame


def build_synthetic_frames() -> tuple[list[SaLiveFrame], list[object]]:
    keepalive: list[object] = []
    players = (SaPlayerFrame * 2)(
        player(0, True, vec3(0.0, 0.0, 92.75)),
        player(1, False, vec3(120.0, 0.0, 92.75)),
    )
    keepalive.append(players)

    shot_ball = rigid_body(vec3(300.0, 100.0, 120.0), vec3(1000.0, 500.0, 100.0))
    shot_player = rigid_body(vec3(240.0, 90.0, 92.75), vec3(800.0, 300.0, 0.0))
    touches = (SaTouchEvent * 1)(
        SaTouchEvent(timing(1), 0, 1, 1, ctypes.c_float(12.0), 1),
    )
    dodge_refreshes = (SaDodgeRefreshedEvent * 1)(
        SaDodgeRefreshedEvent(timing(1), 0, 1, 1),
    )
    boost_pad_events = (SaBoostPadEvent * 1)(
        SaBoostPadEvent(timing(1), 34, 1, 1, 0, 1),
    )
    goals = (SaGoalEvent * 1)(
        SaGoalEvent(timing(1), 1, 0, 1, 1, 1, 0, 1),
    )
    player_stat_events = (SaPlayerStatEvent * 3)(
        SaPlayerStatEvent(timing(1), 0, 1, 1, 1, shot_ball, 1, shot_player),
        SaPlayerStatEvent(timing(1), 1, 0, 2, 0, SaRigidBody(), 0, SaRigidBody()),
        SaPlayerStatEvent(timing(1), 0, 1, 3, 0, SaRigidBody(), 0, SaRigidBody()),
    )
    demolishes = (SaDemolishEvent * 1)(
        SaDemolishEvent(
            timing(1),
            0,
            1,
            vec3(2300.0, 0.0, 0.0),
            SaVec3(),
            vec3(120.0, 0.0, 92.75),
            ctypes.c_float(0.25),
        ),
    )
    keepalive.extend(
        [touches, dodge_refreshes, boost_pad_events, goals, player_stat_events, demolishes]
    )

    frames = [
        live_frame(
            frame_number,
            players,
            rigid_body(vec3(frame_number * 25.0, 0.0, 120.0)),
        )
        for frame_number in range(1, 4)
    ]
    frames[0].touches = ctypes.cast(touches, ctypes.POINTER(SaTouchEvent))
    frames[0].touch_count = len(touches)
    frames[0].dodge_refreshes = ctypes.cast(
        dodge_refreshes, ctypes.POINTER(SaDodgeRefreshedEvent)
    )
    frames[0].dodge_refresh_count = len(dodge_refreshes)
    frames[0].boost_pad_events = ctypes.cast(
        boost_pad_events, ctypes.POINTER(SaBoostPadEvent)
    )
    frames[0].boost_pad_event_count = len(boost_pad_events)
    frames[0].goals = ctypes.cast(goals, ctypes.POINTER(SaGoalEvent))
    frames[0].goal_count = len(goals)
    frames[0].player_stat_events = ctypes.cast(
        player_stat_events, ctypes.POINTER(SaPlayerStatEvent)
    )
    frames[0].player_stat_event_count = len(player_stat_events)
    frames[0].demolishes = ctypes.cast(demolishes, ctypes.POINTER(SaDemolishEvent))
    frames[0].demolish_count = len(demolishes)
    return frames, keepalive


def read_json(engine: int, len_fn, write_fn) -> object:
    json_len = len_fn(engine)
    if json_len <= 0:
        raise AssertionError(f"{len_fn.__name__} returned empty JSON")
    buffer = (ctypes.c_uint8 * json_len)()
    written = write_fn(engine, buffer, json_len)
    if written != json_len:
        raise AssertionError(f"{write_fn.__name__} wrote {written}, expected {json_len}")
    return json.loads(bytes(buffer).decode("utf-8"))


def read_named_json(engine: int, len_fn, write_fn, key: str) -> object:
    encoded = key.encode("utf-8")
    json_len = len_fn(engine, encoded)
    if json_len <= 0:
        raise AssertionError(f"{len_fn.__name__} returned empty JSON for {key!r}")
    buffer = (ctypes.c_uint8 * json_len)()
    written = write_fn(engine, encoded, buffer, json_len)
    if written != json_len:
        raise AssertionError(f"{write_fn.__name__} wrote {written}, expected {json_len}")
    return json.loads(bytes(buffer).decode("utf-8"))


def require_string_list(value: object, label: str) -> list[str]:
    if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
        raise AssertionError(f"{label} should be a list of strings")
    return value


def require_object(value: object, label: str) -> dict[str, object]:
    if not isinstance(value, dict):
        raise AssertionError(f"{label} should be an object")
    return value


def validate_event_outputs(graph_info: dict[str, object], event_history: object, events: object) -> None:
    history = require_object(event_history, "event_history")
    graph_events = require_object(events, "events")
    history_fields = require_string_list(
        graph_info.get("event_history_field_names"),
        "graph_info.event_history_field_names",
    )
    required_history_fields = require_string_list(
        graph_info.get("required_event_history_field_names"),
        "graph_info.required_event_history_field_names",
    )
    required_graph_event_fields = require_string_list(
        graph_info.get("required_graph_event_field_names"),
        "graph_info.required_graph_event_field_names",
    )
    for field_name in required_history_fields:
        if field_name not in history_fields:
            raise AssertionError(f"required event_history field {field_name!r} is not declared")
        entries = history.get(field_name)
        if not isinstance(entries, list) or not entries:
            raise AssertionError(f"event_history field {field_name!r} should be nonempty")
    for field_name in required_graph_event_fields:
        entries = graph_events.get(field_name)
        if not isinstance(entries, list) or not entries:
            raise AssertionError(f"events field {field_name!r} should be nonempty")


def drain_pending_events(abi: RustAbi, engine: int) -> tuple[int, int, int]:
    mechanic_events = (SaMechanicEvent * 64)()
    team_events = (SaTeamEvent * 64)()
    goal_context_events = (SaGoalContextEvent * 64)()
    mechanic_count = abi.dll.subtr_actor_bakkesmod_drain_events(engine, mechanic_events, 64)
    team_count = abi.dll.subtr_actor_bakkesmod_drain_team_events(engine, team_events, 64)
    goal_context_count = abi.dll.subtr_actor_bakkesmod_drain_goal_context_events(
        engine,
        goal_context_events,
        64,
    )
    if mechanic_count == 0:
        raise AssertionError("expected at least one drainable player-owned event")
    if goal_context_count == 0:
        raise AssertionError("expected at least one drainable goal-context event")
    return mechanic_count, team_count, goal_context_count


def run_smoke_test(path: Path) -> None:
    abi = RustAbi(path)
    engine = abi.dll.subtr_actor_bakkesmod_engine_create()
    if not engine:
        raise AssertionError("subtr_actor_bakkesmod_engine_create returned null")
    try:
        frames, keepalive = build_synthetic_frames()
        for frame in frames:
            status = abi.dll.subtr_actor_bakkesmod_process_frame(engine, ctypes.byref(frame))
            if status != 0:
                raise AssertionError(
                    f"subtr_actor_bakkesmod_process_frame failed for frame "
                    f"{frame.frame_number}: {status}"
                )
        if abi.dll.subtr_actor_bakkesmod_finish(engine) != 0:
            raise AssertionError("subtr_actor_bakkesmod_finish failed")

        graph_info = require_object(abi.json_value(engine, "graph_info"), "graph_info")
        graph_output_names = require_string_list(
            graph_info.get("graph_output_names"),
            "graph_info.graph_output_names",
        )
        callable_nodes = require_string_list(
            graph_info.get("callable_analysis_node_names"),
            "graph_info.callable_analysis_node_names",
        )
        stats_modules = require_string_list(
            graph_info.get("builtin_stats_module_names"),
            "graph_info.builtin_stats_module_names",
        )
        node_names = require_string_list(
            abi.json_value(engine, "analysis_node_names"),
            "analysis_node_names",
        )
        if node_names != callable_nodes:
            raise AssertionError("analysis_node_names output does not match graph_info")

        outputs = {
            output_name: abi.named_json_value(engine, "graph_output", output_name)
            for output_name in graph_output_names
        }
        analysis_nodes_output = require_object(outputs["analysis_nodes"], "analysis_nodes")
        if set(analysis_nodes_output) != set(callable_nodes):
            raise AssertionError("analysis_nodes output keys do not match callable nodes")
        for node_name in callable_nodes:
            abi.named_json_value(engine, "analysis_node", node_name)
        for module_name in stats_modules:
            abi.named_json_value(engine, "stats_module", module_name)
            abi.named_json_value(engine, "stats_module_frame", module_name)
            abi.named_json_value(engine, "stats_module_config", module_name)

        validate_event_outputs(graph_info, outputs["event_history"], outputs["events"])
        abi.json_value(engine, "events")
        abi.json_value(engine, "frame")
        abi.json_value(engine, "timeline")
        abi.json_value(engine, "stats")
        mechanic_count, team_count, goal_context_count = drain_pending_events(abi, engine)
        print(
            f"Loaded {path}; processed {len(frames)} frames, called "
            f"{len(callable_nodes)} analysis nodes, {len(stats_modules)} stats modules, "
            f"drained {mechanic_count} player events, {team_count} team events, "
            f"{goal_context_count} goal-context events"
        )
        keepalive.clear()
    finally:
        abi.dll.subtr_actor_bakkesmod_engine_destroy(engine)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("dll", nargs="+", type=Path, help="subtr_actor_bakkesmod.dll artifact")
    args = parser.parse_args()

    errors: list[str] = []
    for dll_path in args.dll:
        try:
            run_smoke_test(dll_path)
        except Exception as exc:
            errors.append(f"{dll_path}: {exc}")
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
