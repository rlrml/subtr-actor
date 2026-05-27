#!/usr/bin/env python3
"""Validate the BakkesMod C++ source against the Rust live-event ABI.

This is intentionally a source contract check, not a substitute for in-game
validation. It catches drift between the Rust-declared graph/event registries
and the C++ plugin code that samples, queues, and attaches live event families.
"""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
RUST_SOURCE = REPO_ROOT / "crates/subtr-actor-bakkesmod/src/lib.rs"
PLUGIN_SOURCE = REPO_ROOT / "bakkesmod/SubtrActorPlugin.cpp"
PLUGIN_HEADER = REPO_ROOT / "bakkesmod/SubtrActorPlugin.h"
ABI_HEADER = REPO_ROOT / "crates/subtr-actor-bakkesmod/include/subtr_actor_bakkesmod.h"


@dataclass(frozen=True)
class EventFamily:
    graph_field: str
    frame_pointer: str
    frame_count: str
    pending_vector: str
    attach_pointer: str
    attach_count: str
    producers: tuple[str, ...]


EVENT_FAMILIES = (
    EventFamily(
        graph_field="demo_events",
        frame_pointer="demolishes",
        frame_count="demolish_count",
        pending_vector="pendingDemolishes",
        attach_pointer="frame.demolishes",
        attach_count="frame.demolish_count",
        producers=("recordDemolish(", "CAR_DEMOLISHED_EVENT"),
    ),
    EventFamily(
        graph_field="boost_pad_events",
        frame_pointer="boost_pad_events",
        frame_count="boost_pad_event_count",
        pending_vector="pendingBoostPadEvents",
        attach_pointer="frame.boost_pad_events",
        attach_count="frame.boost_pad_event_count",
        producers=("recordBoostPadEvent(", "BOOST_PICKED_UP_EVENT", "BOOST_SPAWNED_EVENT"),
    ),
    EventFamily(
        graph_field="touch_events",
        frame_pointer="touches",
        frame_count="touch_count",
        pending_vector="pendingTouches",
        attach_pointer="frame.touches",
        attach_count="frame.touch_count",
        producers=("recordTouch(", "BALL_TOUCH_EVENT"),
    ),
    EventFamily(
        graph_field="dodge_refreshed_events",
        frame_pointer="dodge_refreshes",
        frame_count="dodge_refresh_count",
        pending_vector="pendingDodgeRefreshes",
        attach_pointer="frame.dodge_refreshes",
        attach_count="frame.dodge_refresh_count",
        producers=("recordDodgeRefreshFromJumpState(", "samplePlayer(CarWrapper"),
    ),
    EventFamily(
        graph_field="player_stat_events",
        frame_pointer="player_stat_events",
        frame_count="player_stat_event_count",
        pending_vector="pendingPlayerStatEvents",
        attach_pointer="frame.player_stat_events",
        attach_count="frame.player_stat_event_count",
        producers=(
            "recordPlayerStatDeltas(",
            "recordExplicitPlayerStat(",
            "SaPlayerStatEventKindShot",
            "SaPlayerStatEventKindSave",
            "SaPlayerStatEventKindAssist",
        ),
    ),
    EventFamily(
        graph_field="goal_events",
        frame_pointer="goals",
        frame_count="goal_count",
        pending_vector="pendingGoals",
        attach_pointer="frame.goals",
        attach_count="frame.goal_count",
        producers=("recordGoal(", "GOAL_SCORED_EVENT"),
    ),
)

DERIVED_EVENT_FIELDS = {
    "active_demos": (
        "sync_active_demos(",
        "self.active_demos",
        ".live_events",
        "DEMO_ACTIVE_DURATION_SECONDS",
    ),
}

REQUIRED_PLUGIN_ABI_EXPORTS = (
    ("subtr_actor_bakkesmod_engine_create", "engineCreate"),
    ("subtr_actor_bakkesmod_engine_destroy", "engineDestroy"),
    ("subtr_actor_bakkesmod_engine_reset", "engineReset"),
    ("subtr_actor_bakkesmod_finish", "engineFinish"),
    ("subtr_actor_bakkesmod_process_frame", "processFrame"),
    ("subtr_actor_bakkesmod_events_json_len", "eventsJsonLen"),
    ("subtr_actor_bakkesmod_write_events_json", "writeEventsJson"),
    ("subtr_actor_bakkesmod_frame_json_len", "frameJsonLen"),
    ("subtr_actor_bakkesmod_write_frame_json", "writeFrameJson"),
    ("subtr_actor_bakkesmod_timeline_json_len", "timelineJsonLen"),
    ("subtr_actor_bakkesmod_write_timeline_json", "writeTimelineJson"),
    ("subtr_actor_bakkesmod_stats_json_len", "statsJsonLen"),
    ("subtr_actor_bakkesmod_write_stats_json", "writeStatsJson"),
    ("subtr_actor_bakkesmod_stats_module_json_len", "statsModuleJsonLen"),
    ("subtr_actor_bakkesmod_write_stats_module_json", "writeStatsModuleJson"),
    ("subtr_actor_bakkesmod_stats_module_frame_json_len", "statsModuleFrameJsonLen"),
    ("subtr_actor_bakkesmod_write_stats_module_frame_json", "writeStatsModuleFrameJson"),
    ("subtr_actor_bakkesmod_stats_module_config_json_len", "statsModuleConfigJsonLen"),
    ("subtr_actor_bakkesmod_write_stats_module_config_json", "writeStatsModuleConfigJson"),
    ("subtr_actor_bakkesmod_graph_output_json_len", "graphOutputJsonLen"),
    ("subtr_actor_bakkesmod_write_graph_output_json", "writeGraphOutputJson"),
    ("subtr_actor_bakkesmod_analysis_node_json_len", "analysisNodeJsonLen"),
    ("subtr_actor_bakkesmod_write_analysis_node_json", "writeAnalysisNodeJson"),
    ("subtr_actor_bakkesmod_analysis_node_names_json_len", "analysisNodeNamesJsonLen"),
    ("subtr_actor_bakkesmod_write_analysis_node_names_json", "writeAnalysisNodeNamesJson"),
    ("subtr_actor_bakkesmod_graph_info_json_len", "graphInfoJsonLen"),
    ("subtr_actor_bakkesmod_write_graph_info_json", "writeGraphInfoJson"),
    ("subtr_actor_bakkesmod_drain_events", "drainEvents"),
    ("subtr_actor_bakkesmod_drain_team_events", "drainTeamEvents"),
    ("subtr_actor_bakkesmod_drain_goal_context_events", "drainGoalContextEvents"),
    ("subtr_actor_bakkesmod_replay_annotations_create", "replayAnnotationsCreate"),
    ("subtr_actor_bakkesmod_replay_annotations_destroy", "replayAnnotationsDestroy"),
    ("subtr_actor_bakkesmod_replay_annotation_count", "replayAnnotationCount"),
    ("subtr_actor_bakkesmod_replay_annotation_player_count", "replayAnnotationPlayerCount"),
    ("subtr_actor_bakkesmod_write_replay_annotation_players", "writeReplayAnnotationPlayers"),
    ("subtr_actor_bakkesmod_poll_replay_annotations", "pollReplayAnnotations"),
)


def quoted_strings(value: str) -> list[str]:
    return re.findall(r'"([^"]+)"', value)


def rust_array(source: str, name: str) -> list[str]:
    match = re.search(
        rf"const\s+{re.escape(name)}\s*:\s*&\s*\[\s*&str\s*\]\s*=\s*&\[(.*?)\];",
        source,
        re.DOTALL,
    )
    if not match:
        raise AssertionError(f"missing Rust array {name}")
    return quoted_strings(match.group(1))


def cpp_array(source: str, name: str) -> list[str]:
    match = re.search(
        rf"constexpr\s+std::array<[^>]+>\s+{re.escape(name)}\s*\{{(.*?)\}};",
        source,
        re.DOTALL,
    )
    if not match:
        raise AssertionError(f"missing C++ array {name}")
    return quoted_strings(match.group(1))


def require_contains(source: str, needle: str, label: str, errors: list[str]) -> None:
    if needle not in source:
        errors.append(f"missing {label}: {needle}")


def main() -> int:
    rust_source = RUST_SOURCE.read_text(encoding="utf-8")
    plugin_source = PLUGIN_SOURCE.read_text(encoding="utf-8")
    plugin_header = PLUGIN_HEADER.read_text(encoding="utf-8")
    abi_header = ABI_HEADER.read_text(encoding="utf-8")
    cpp_combined = plugin_header + "\n" + plugin_source
    errors: list[str] = []

    registry_pairs = (
        ("LIVE_GRAPH_OUTPUT_NAMES", "VERIFY_GRAPH_OUTPUTS"),
        ("LIVE_EVENT_HISTORY_FIELD_NAMES", "FRAME_EVENTS_STATE_EVENT_FIELDS"),
        ("REQUIRED_EVENT_HISTORY_FIELD_NAMES", "REQUIRED_EVENT_HISTORY_FIELDS"),
        ("LIVE_GRAPH_EVENT_FIELD_NAMES", "GRAPH_EVENT_FIELDS"),
        ("REQUIRED_GRAPH_EVENT_FIELD_NAMES", "REQUIRED_GRAPH_EVENT_FIELDS"),
    )
    for rust_name, cpp_name in registry_pairs:
        rust_values = rust_array(rust_source, rust_name)
        cpp_values = cpp_array(plugin_source, cpp_name)
        if rust_values != cpp_values:
            errors.append(
                f"registry mismatch {rust_name} != {cpp_name}: "
                f"Rust={rust_values!r} C++={cpp_values!r}"
            )

    required_event_fields = set(rust_array(rust_source, "REQUIRED_EVENT_HISTORY_FIELD_NAMES"))
    covered_event_fields = {family.graph_field for family in EVENT_FAMILIES}
    covered_event_fields.update(DERIVED_EVENT_FIELDS)
    missing_coverage = sorted(required_event_fields - covered_event_fields)
    if missing_coverage:
        errors.append(f"required event fields missing C++ producer contract: {missing_coverage}")

    loaded_exports = set(
        re.findall(r'GetProcAddress\(rustLibrary,\s*"([^"]+)"\)', plugin_source)
    )
    expected_exports = {symbol for symbol, _ in REQUIRED_PLUGIN_ABI_EXPORTS}
    missing_loaded_exports = sorted(expected_exports - loaded_exports)
    unexpected_loaded_exports = sorted(loaded_exports - expected_exports)
    if missing_loaded_exports:
        errors.append(f"required Rust ABI exports not loaded by C++ plugin: {missing_loaded_exports}")
    if unexpected_loaded_exports:
        errors.append(f"C++ plugin loads unexpected Rust ABI exports: {unexpected_loaded_exports}")
    for symbol, pointer_name in REQUIRED_PLUGIN_ABI_EXPORTS:
        require_contains(
            abi_header,
            f"{symbol}(",
            f"{symbol} checked-in ABI declaration",
            errors,
        )
        require_contains(
            plugin_header,
            f"{pointer_name} = nullptr",
            f"{symbol} function pointer member",
            errors,
        )
        require_contains(
            plugin_source,
            f'"{symbol}"',
            f"{symbol} GetProcAddress load",
            errors,
        )
        require_contains(
            plugin_source,
            f"!{pointer_name}",
            f"{symbol} load failure guard",
            errors,
        )
        require_contains(
            plugin_source,
            f"{pointer_name} = nullptr",
            f"{symbol} unload reset",
            errors,
        )

    for family in EVENT_FAMILIES:
        require_contains(
            abi_header,
            f"*{family.frame_pointer};",
            f"{family.graph_field} C ABI pointer",
            errors,
        )
        require_contains(
            abi_header,
            f"size_t {family.frame_count};",
            f"{family.graph_field} C ABI count",
            errors,
        )
        require_contains(
            cpp_combined,
            family.pending_vector,
            f"{family.graph_field} pending queue",
            errors,
        )
        require_contains(
            plugin_source,
            family.attach_pointer,
            f"{family.graph_field} frame pointer attachment",
            errors,
        )
        require_contains(
            plugin_source,
            family.attach_count,
            f"{family.graph_field} frame count attachment",
            errors,
        )
        require_contains(
            plugin_source,
            f"{family.pending_vector}.push_back",
            f"{family.graph_field} queue append",
            errors,
        )
        for producer in family.producers:
            require_contains(
                plugin_source,
                producer,
                f"{family.graph_field} producer path",
                errors,
            )
    for graph_field, required_paths in DERIVED_EVENT_FIELDS.items():
        for producer in required_paths:
            require_contains(
                rust_source + "\n" + plugin_source,
                producer,
                f"{graph_field} derived event path",
                errors,
            )
    require_contains(
        plugin_source,
        "subtr_actor_self_test_graph",
        "synthetic graph self-test command",
        errors,
    )
    require_contains(
        plugin_source,
        "graph self-test writing synthetic graph dump",
        "synthetic graph self-test dump mode",
        errors,
    )
    require_contains(
        plugin_source,
        "require_event_history",
        "strict self-test event-history mode",
        errors,
    )
    require_contains(
        plugin_source,
        "require_graph_events",
        "strict self-test graph-events mode",
        errors,
    )

    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1

    print("BakkesMod plugin source contract validation passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
