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
WEB_PLAYER_CONFIG_SOURCE = REPO_ROOT / "js/stat-evaluation-player/src/playerConfig.ts"
WEB_PLAYER_MAIN_SOURCE = REPO_ROOT / "js/stat-evaluation-player/src/main.ts"
WEB_PLAYER_TEMPLATE_SOURCE = REPO_ROOT / "js/stat-evaluation-player/src/appTemplate.ts"


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
    (
        "subtr_actor_bakkesmod_decoded_stats_player_config_json_len",
        "decodedStatsPlayerConfigJsonLen",
    ),
    (
        "subtr_actor_bakkesmod_write_decoded_stats_player_config_json",
        "writeDecodedStatsPlayerConfigJson",
    ),
    (
        "subtr_actor_bakkesmod_encoded_stats_player_config_len",
        "encodedStatsPlayerConfigLen",
    ),
    (
        "subtr_actor_bakkesmod_write_encoded_stats_player_config",
        "writeEncodedStatsPlayerConfig",
    ),
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


def ts_array(source: str, name: str) -> list[str]:
    match = re.search(
        rf"const\s+{re.escape(name)}\s*:[^=]+=\s*\[(.*?)\];",
        source,
        re.DOTALL,
    )
    if not match:
        raise AssertionError(f"missing TypeScript array {name}")
    return quoted_strings(match.group(1))


def ts_type_alias_strings(source: str, name: str) -> list[str]:
    match = re.search(
        rf"export\s+type\s+{re.escape(name)}\s*=\s*(.*?);",
        source,
        re.DOTALL,
    )
    if not match:
        raise AssertionError(f"missing TypeScript type alias {name}")
    return quoted_strings(match.group(1))


def ts_interface_fields(source: str, name: str) -> tuple[str, ...]:
    match = re.search(
        rf"export\s+interface\s+{re.escape(name)}\s*\{{(?P<body>.*?)\n\}}",
        source,
        re.DOTALL,
    )
    if not match:
        raise AssertionError(f"missing TypeScript interface {name}")
    fields = []
    for line in match.group("body").splitlines():
        field = re.search(r"\breadonly\s+([A-Za-z0-9_]+)\??\s*:", line)
        if field:
            fields.append(field.group(1))
    if not fields:
        raise AssertionError(f"could not parse TypeScript interface {name}")
    return tuple(fields)


def cpp_lambda_body(source: str, name: str) -> str:
    match = re.search(
        rf"auto\s+{re.escape(name)}\s*=\s*\[\]\((?P<args>.*?)\)\s*\{{(?P<body>.*?)\n\s*\}};",
        source,
        re.DOTALL,
    )
    if not match:
        raise AssertionError(f"missing C++ lambda {name}")
    return match.group("body")


def web_launcher_buttons(source: str, attribute: str) -> list[tuple[str, str]]:
    return [
        (match.group("id"), match.group("label").strip())
        for match in re.finditer(
            rf'<button\b(?=[^>]*\b{re.escape(attribute)}="(?P<id>[^"]+)")[^>]*>'
            r"(?P<label>[^<]+)</button>",
            source,
            re.DOTALL,
        )
    ]


def require_contains(source: str, needle: str, label: str, errors: list[str]) -> None:
    if needle not in source:
        errors.append(f"missing {label}: {needle}")


def reject_contains(source: str, needle: str, label: str, errors: list[str]) -> None:
    if needle in source:
        errors.append(f"unexpected {label}: {needle}")


def singleton_window_controls(source: str) -> list[tuple[str, str, bool, int]]:
    match = re.search(
        r"return\s+\{\{(.*?)\}\};\s*\}\s*\n\s*std::vector<SubtrActorPlugin::SingletonWindowControl>",
        source,
        re.DOTALL,
    )
    if not match:
        raise AssertionError("missing singletonWindowControls return block")

    controls: list[tuple[str, str, bool, int]] = []
    for control in re.finditer(
        r'\{\s*"(?P<label>[^"]+)",\s*"(?P<id>[^"]+)",\s*"[^"]+",\s*"[^"]+",\s*'
        r"(?P<web>true|false),\s*(?P<order>\d+),",
        match.group(1),
        re.DOTALL,
    ):
        controls.append(
            (
                control.group("id"),
                control.group("label"),
                control.group("web") == "true",
                int(control.group("order")),
            )
        )
    if not controls:
        raise AssertionError("could not parse singletonWindowControls")
    return controls


def stats_window_kind_controls(source: str) -> list[tuple[str, str, bool]]:
    match = re.search(
        r"SubtrActorPlugin::statsWindowKindControls\(\)\s+const\s+\{\s+return\s+\{\{"
        r"(.*?)\}\};\s*\}",
        source,
        re.DOTALL,
    )
    if not match:
        raise AssertionError("missing statsWindowKindControls return block")

    controls: list[tuple[str, str, bool]] = []
    for control in re.finditer(
        r'\{\s*UiStatsWindowKind::\w+,\s*"(?P<id>[^"]+)",\s*"[^"]+",\s*"(?P<label>[^"]+)",\s*'
        r"(?:static_cast<[^>]+>\([^)]*\)|[^,]+),\s*"
        r"(?P<scope_selector>true|false),\s*"
        r"(?P<stat_picker>true|false),\s*"
        r"(?P<web>true|false),\s*"
        r"(?P<default_window>true|false)\s*\}",
        match.group(1),
        re.DOTALL,
    ):
        controls.append(
            (control.group("id"), control.group("label"), control.group("web") == "true")
        )
    if not controls:
        raise AssertionError("could not parse statsWindowKindControls")
    return controls


def main() -> int:
    rust_source = RUST_SOURCE.read_text(encoding="utf-8")
    plugin_source = PLUGIN_SOURCE.read_text(encoding="utf-8")
    plugin_header = PLUGIN_HEADER.read_text(encoding="utf-8")
    abi_header = ABI_HEADER.read_text(encoding="utf-8")
    web_player_config_source = WEB_PLAYER_CONFIG_SOURCE.read_text(encoding="utf-8")
    web_player_main_source = WEB_PLAYER_MAIN_SOURCE.read_text(encoding="utf-8")
    web_player_template_source = WEB_PLAYER_TEMPLATE_SOURCE.read_text(encoding="utf-8")
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

    web_singleton_type_ids = tuple(
        ts_type_alias_strings(web_player_config_source, "SingletonWindowId")
    )
    web_singleton_window_ids = tuple(ts_array(web_player_main_source, "SINGLETON_WINDOW_IDS"))
    if web_singleton_window_ids != web_singleton_type_ids:
        errors.append(
            "stats evaluation player singleton window order differs from its config type: "
            f"type={web_singleton_type_ids!r} array={web_singleton_window_ids!r}"
        )
    web_stats_window_kind_ids = tuple(
        ts_type_alias_strings(web_player_config_source, "StatsWindowKind")
    )
    web_window_placement_fields = ts_interface_fields(
        web_player_config_source,
        "WindowPlacementConfig",
    )
    expected_web_window_placement_fields = ("x", "y", "viewport", "zIndex", "visible")
    if web_window_placement_fields != expected_web_window_placement_fields:
        errors.append(
            "stats evaluation player WindowPlacementConfig fields changed: "
            f"expected={expected_web_window_placement_fields!r} "
            f"actual={web_window_placement_fields!r}"
        )

    plugin_web_window_controls = tuple(
        (window_id, label)
        for window_id, label, web_config, _ in sorted(
            singleton_window_controls(plugin_source),
            key=lambda control: (control[3], control[0]),
        )
        if web_config
    )
    web_window_ids = tuple(window_id for window_id, _ in plugin_web_window_controls)
    if web_window_ids != web_singleton_window_ids:
        errors.append(
            "web singleton window order drifted from stats evaluation player: "
            f"expected={web_singleton_window_ids!r} actual={web_window_ids!r}"
        )
    web_launcher_window_buttons = tuple(
        web_launcher_buttons(web_player_template_source, "data-window-toggle")
    )
    if plugin_web_window_controls != web_launcher_window_buttons:
        errors.append(
            "web singleton launcher labels drifted from stats evaluation player: "
            f"expected={web_launcher_window_buttons!r} actual={plugin_web_window_controls!r}"
        )

    plugin_web_stats_window_controls = tuple(
        (kind_id, label)
        for kind_id, label, web_config in stats_window_kind_controls(plugin_source)
        if web_config
    )
    plugin_web_stats_window_kind_ids = tuple(
        kind_id for kind_id, _ in plugin_web_stats_window_controls
    )
    if plugin_web_stats_window_kind_ids != web_stats_window_kind_ids:
        errors.append(
            "web stats window kind order drifted from stats evaluation player: "
            f"expected={web_stats_window_kind_ids!r} actual={plugin_web_stats_window_kind_ids!r}"
        )
    web_launcher_stats_buttons = tuple(
        web_launcher_buttons(web_player_template_source, "data-create-stats-window")
    )
    if plugin_web_stats_window_controls != web_launcher_stats_buttons:
        errors.append(
            "web stats launcher labels drifted from stats evaluation player: "
            f"expected={web_launcher_stats_buttons!r} actual={plugin_web_stats_window_controls!r}"
        )
    require_contains(
        web_player_template_source,
        '<input id="skip-post-goal-transitions" type="checkbox" checked />',
        "stats evaluation player default skips post-goal resets",
        errors,
    )
    require_contains(
        plugin_header,
        "bool playbackSkipPostGoalTransitions = true;",
        "plugin playback default skips post-goal resets like web player",
        errors,
    )
    require_contains(
        web_player_template_source,
        '<input id="skip-kickoffs" type="checkbox" />',
        "stats evaluation player default does not skip kickoffs",
        errors,
    )
    require_contains(
        plugin_header,
        "bool playbackSkipKickoffs = false;",
        "plugin playback default does not skip kickoffs like web player",
        errors,
    )
    require_contains(
        plugin_source,
        '"Load Replay...", ImVec2{actionButtonWidth, 0.0f}',
        "launcher actions expose replay loading like the web player",
        errors,
    )
    require_contains(
        plugin_source,
        '"Live analysis graph",\n            liveAnalysis,\n            "launcher-plugin-tools"',
        "plugin-specific live analysis toggle lives under launcher plugin tools",
        errors,
    )
    reject_contains(
        plugin_source,
        '"Live analysis graph",\n          liveAnalysis,\n          "launcher-actions"',
        "plugin-specific live analysis toggle in web-like launcher actions",
        errors,
    )
    require_contains(
        plugin_source,
        "std::optional<std::string> SubtrActorPlugin::webPlayerIdForWindowConfig(",
        "nullable web stats window playerId helper",
        errors,
    )
    require_contains(
        plugin_source,
        "std::optional<std::string> SubtrActorPlugin::webCameraPlayerIdConfig() const",
        "nullable web camera attachedPlayerId helper",
        errors,
    )
    require_contains(
        plugin_source,
        "if (const auto playerId = webPlayerIdForWindowConfig(window))",
        "web stats window config uses nullable playerId helper",
        errors,
    )
    require_contains(
        plugin_source,
        "if (const auto playerId = webPlayerIdForIndexIfKnown(window.selected_player_index))",
        "stats window player selection only synthesizes known player ids",
        errors,
    )
    require_contains(
        plugin_source,
        "if (const auto attachedPlayerId = webCameraPlayerIdConfig())",
        "web camera config uses nullable attachedPlayerId helper",
        errors,
    )
    require_contains(
        plugin_source,
        "bool jsonPropertyIsNull(const std::string &json, const std::string &propertyName)",
        "JSON null property helper",
        errors,
    )
    require_contains(
        plugin_source,
        'jsonPropertyIsNull(*camera, "customSettings")',
        "camera customSettings null import uses JSON parser",
        errors,
    )
    require_contains(
        plugin_header,
        "int cameraFreePreset = -1;",
        "camera free preset has nullable native state",
        errors,
    )
    require_contains(
        plugin_source,
        'std::clamp(parseJsonNumberProperty(json, "camera_free_preset").value_or(-1.0), -1.0, 1.0)',
        "legacy camera free preset import preserves null state",
        errors,
    )
    require_contains(
        plugin_source,
        'jsonPropertyIsNull(*camera, "freePreset")',
        "web camera freePreset null import uses JSON parser",
        errors,
    )
    require_contains(
        plugin_source,
        'cameraFreePreset = -1;',
        "plain camera modes clear free preset",
        errors,
    )
    reject_contains(
        plugin_source,
        'camera->find("\\"customSettings\\":null")',
        "camera customSettings null import uses exact substring",
        errors,
    )
    require_contains(
        plugin_source,
        'file << "null";',
        "web config can emit null values",
        errors,
    )
    reject_contains(
        plugin_source,
        'file << ",\\"playerId\\":\\"" << escapeJsonString(webPlayerIdForWindow(window)) << "\\""',
        "web stats window config always serializes playerId as string",
        errors,
    )
    require_contains(
        plugin_source,
        'file << ",\\"team\\":\\"" << (window.selected_team_is_team_0 != 0 ? "blue" : "orange") << "\\""',
        "web stats window config always emits team",
        errors,
    )
    require_contains(
        plugin_source,
        'const bool hasLegacyStatsWindows = jsonPropertyExists(json, "stats_windows");',
        "stats window import checks legacy property presence",
        errors,
    )
    require_contains(
        plugin_source,
        'const bool hasWebStatsWindows = jsonPropertyExists(json, "statsWindows");',
        "stats window import checks web property presence",
        errors,
    )
    require_contains(
        plugin_source,
        "const bool statsWindowObjectsFromWeb = !hasLegacyStatsWindows && hasWebStatsWindows;",
        "stats window import tracks web source",
        errors,
    )
    require_contains(
        plugin_source,
        'const auto idString = parseJsonStringProperty(object, "id");\n    if (statsWindowObjectsFromWeb && !idString) {\n      continue;\n    }',
        "web stats window import rejects entries without string ids",
        errors,
    )
    require_contains(
        plugin_source,
        'hasLegacyStatsWindows\n          ? parseJsonObjectArrayProperty(json, "stats_windows")',
        "stats window import prefers present legacy plugin config",
        errors,
    )
    require_contains(
        plugin_source,
        'hasWebStatsWindows ? parseJsonObjectArrayProperty(json, "statsWindows")',
        "stats window import falls back to present web config",
        errors,
    )
    require_contains(
        plugin_source,
        'std::optional<std::string> placement = parseJsonObjectProperty(object, "placement");',
        "window array import can default missing web placement",
        errors,
    )
    require_contains(
        plugin_source,
        'if (!webConfig) {\n          continue;\n        }\n        placement = R"({"x":8,"y":8,"viewport":{"width":1,"height":1},"visible":true})";',
        "web singleton window import mirrors normalizePlacement defaults",
        errors,
    )
    require_contains(
        plugin_source,
        "if (!placement && statsWindowObjectsFromWeb) {\n      placement = R\"({\"x\":8,\"y\":8,\"viewport\":{\"width\":1,\"height\":1},\"visible\":true})\";\n    }",
        "web stats window import mirrors normalizePlacement defaults",
        errors,
    )
    require_contains(
        plugin_source,
        "if (!statsWindowObjectsFromWeb && !hasEntriesProperty && window.entries.empty() &&",
        "web stats window import preserves missing entries as empty",
        errors,
    )
    require_contains(
        plugin_source,
        'if (jsonPropertyExists(*overlays, "renderEffects")) {',
        "web renderEffects import uses JSON parser",
        errors,
    )
    require_contains(
        plugin_source,
        'if (jsonPropertyExists(*boostConfig, "playerIds")) {',
        "web boost playerIds import uses JSON parser",
        errors,
    )
    require_contains(
        plugin_source,
        'boostPickupPlayerFilterEnabled = !jsonPropertyIsNull(*boostConfig, "playerIds");',
        "web boost playerIds null preserves all-player filter",
        errors,
    )
    require_contains(
        plugin_source,
        "writeStringArray(file, boostPickupPlayerIds);",
        "web boost playerIds config emits selected players",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "PLAYER");',
        "boost pickup filters expose web player filter group",
        errors,
    )
    reject_contains(
        plugin_source,
        'overlays->find("\\"renderEffects\\"")',
        "web renderEffects import uses exact substring",
        errors,
    )
    require_contains(
        plugin_source,
        'std::string statId = statsWindowObjectsFromWeb\n                               ? parseJsonStringProperty(entryObject, "statId").value_or("")\n                               : parseJsonStringProperty(entryObject, "stat_id").value_or("");',
        "web selected stat import only reads statId",
        errors,
    )
    require_contains(
        plugin_source,
        'std::string targetId = statsWindowObjectsFromWeb\n                                 ? parseJsonStringProperty(entryObject, "targetId").value_or("")\n                                 : parseJsonStringProperty(entryObject, "target_id").value_or("");',
        "web selected stat import only reads targetId",
        errors,
    )
    require_contains(
        plugin_source,
        "if (statsWindowObjectsFromWeb) {\n        continue;\n      }",
        "web selected stat import rejects string-array entries",
        errors,
    )
    reject_contains(
        plugin_source,
        'if (statsWindowObjects.empty()) {\n    statsWindowObjects = parseJsonObjectArrayProperty(json, "statsWindows");\n  }',
        "stats window import falls back based on parsed legacy emptiness",
        errors,
    )
    require_contains(
        plugin_source,
        "auto writeStatsPlayerPlacement = [](",
        "dedicated web placement writer",
        errors,
    )
    require_contains(
        plugin_source,
        "writeStatsPlayerPlacement(file, *window.placement, visible);",
        "singletonWindows use web placement shape",
        errors,
    )
    require_contains(
        plugin_source,
        "auto writeStatsPlayerStatsWindowPlacement = [](",
        "dedicated stats window web placement writer",
        errors,
    )
    require_contains(
        plugin_source,
        "writeStatsPlayerStatsWindowPlacement(file, window);",
        "stats windows use web placement shape",
        errors,
    )
    require_contains(
        plugin_source,
        "placement.viewport_width > 0.0f ? placement.viewport_width : std::max(1.0f, displaySize.x)",
        "singleton web placement viewport width is positive",
        errors,
    )
    require_contains(
        plugin_source,
        "placement.viewport_height > 0.0f ? placement.viewport_height\n"
        "                                                                  : std::max(1.0f, displaySize.y)",
        "singleton web placement viewport height is positive",
        errors,
    )
    require_contains(
        plugin_source,
        "window.viewport_width > 0.0f ? window.viewport_width : std::max(1.0f, displaySize.x)",
        "stats web placement viewport width is positive",
        errors,
    )
    require_contains(
        plugin_source,
        "window.viewport_height > 0.0f ? window.viewport_height : std::max(1.0f, displaySize.y)",
        "stats web placement viewport height is positive",
        errors,
    )
    require_contains(
        plugin_source,
        "writePlacement(file, *window.placement, visible);",
        "plugin windows keep plugin placement shape",
        errors,
    )
    web_singleton_placement_writer = cpp_lambda_body(plugin_source, "writeStatsPlayerPlacement")
    web_stats_placement_writer = cpp_lambda_body(
        plugin_source,
        "writeStatsPlayerStatsWindowPlacement",
    )
    for writer_name, writer_source in (
        ("singleton web placement writer", web_singleton_placement_writer),
        ("stats window web placement writer", web_stats_placement_writer),
    ):
        for field in web_window_placement_fields:
            require_contains(
                writer_source,
                f'\\"{field}\\"',
                f"{writer_name} emits WindowPlacementConfig.{field}",
                errors,
            )
        for legacy_field in (
            "has_placement",
            '\\"viewport_width\\"',
            '\\"viewport_height\\"',
            "placement.width",
            "placement.height",
            "window.width",
            "window.height",
        ):
            reject_contains(
                writer_source,
                legacy_field,
                f"{writer_name} emits plugin-only placement field",
                errors,
            )
    require_contains(
        plugin_source,
        'std::format("##stats-window-player-scope-{}", window.id).c_str()',
        "player stats scope combo uses hidden label",
        errors,
    )
    require_contains(
        plugin_source,
        'std::format("##stats-window-team-scope-{}", window.id).c_str()',
        "team stats scope combo uses hidden label",
        errors,
    )
    reject_contains(
        plugin_source,
        'ImGui::BeginCombo("Player", selectedLabel.c_str())',
        "visible player stats scope combo label",
        errors,
    )
    reject_contains(
        plugin_source,
        'ImGui::BeginCombo("Team", selectedTeam)',
        "visible team stats scope combo label",
        errors,
    )

    require_contains(
        plugin_source,
        "constexpr ImGuiWindowFlags UI_FLOATING_WINDOW_FLAGS",
        "managed floating window flags",
        errors,
    )
    require_contains(
        plugin_source,
        "UI_FLOATING_WINDOW_FLAGS =\n"
        "    ImGuiWindowFlags_NoTitleBar | ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoCollapse |\n"
        "    ImGuiWindowFlags_NoSavedSettings;",
        "managed floating windows opt out of implicit ImGui persistence",
        errors,
    )
    require_contains(
        plugin_source,
        "scoreboardFlags =\n"
        "      ImGuiWindowFlags_NoTitleBar | ImGuiWindowFlags_AlwaysAutoResize |\n"
        "      ImGuiWindowFlags_NoScrollbar | ImGuiWindowFlags_NoCollapse |\n"
        "      ImGuiWindowFlags_NoSavedSettings;",
        "scoreboard opts out of implicit ImGui persistence",
        errors,
    )
    reject_contains(
        plugin_source,
        "ImGui::SetNextWindowFocus();\n"
        "      placement.z_index = nextUiWindowZIndex++;\n"
        "      placement.pending_focus = false;",
        "singleton pending-focus render path bumps z-index",
        errors,
    )
    reject_contains(
        plugin_source,
        "ImGui::SetNextWindowFocus();\n"
        "      scoreboardPlacement.z_index = nextUiWindowZIndex++;\n"
        "      scoreboardPlacement.pending_focus = false;",
        "scoreboard pending-focus render path bumps z-index",
        errors,
    )
    reject_contains(
        plugin_source,
        "ImGui::SetNextWindowFocus();\n"
        "    window.z_index = nextUiWindowZIndex++;\n"
        "    window.pending_focus = false;",
        "stats pending-focus render path bumps z-index",
        errors,
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
