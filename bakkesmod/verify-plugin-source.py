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
RUST_SOURCE = REPO_ROOT / "bakkesmod/rust/src/lib.rs"
RUST_SOURCE_DIR = REPO_ROOT / "bakkesmod/rust/src"
PLUGIN_SOURCE = REPO_ROOT / "bakkesmod/plugin/SubtrActorPlugin.cpp"
PLUGIN_HEADER = REPO_ROOT / "bakkesmod/plugin/SubtrActorPlugin.h"
PLUGIN_README = REPO_ROOT / "bakkesmod/README.md"
ABI_HEADER = REPO_ROOT / "bakkesmod/rust/include/subtr_actor_bakkesmod.h"
WEB_PLAYER_CONFIG_SOURCE = REPO_ROOT / "js/stat-evaluation-player/src/playerConfig.ts"
WEB_PLAYER_MAIN_SOURCE = REPO_ROOT / "js/stat-evaluation-player/src/main.ts"
WEB_PLAYER_SOURCE_DIR = REPO_ROOT / "js/stat-evaluation-player/src"
WEB_PLAYER_STYLES_SOURCE = REPO_ROOT / "js/stat-evaluation-player/src/styles.css"
WEB_PLAYER_STYLES_DIR = REPO_ROOT / "js/stat-evaluation-player/src/styles"
WEB_PLAYER_TEMPLATE_SOURCE = REPO_ROOT / "js/stat-evaluation-player/src/appTemplate.ts"
WEB_PLAYER_FLOATING_WINDOWS_SOURCE = (
    REPO_ROOT / "js/stat-evaluation-player/src/floatingWindows.ts"
)
WEB_PLAYER_TIMELINE_MARKERS_SOURCE = REPO_ROOT / "js/stat-evaluation-player/src/timelineMarkers.ts"
WEB_PLAYER_BOOST_PICKUP_FILTERS_SOURCE = (
    REPO_ROOT / "js/stat-evaluation-player/src/boostPickupFilters.ts"
)
WEB_PLAYER_PLAYER_MODULES_SOURCE = (
    REPO_ROOT / "js/stat-evaluation-player/src/stat-modules/playerModules.ts"
)


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
    (
        "subtr_actor_bakkesmod_write_replay_annotation_frame_players",
        "writeReplayAnnotationFramePlayers",
    ),
    (
        "subtr_actor_bakkesmod_replay_annotation_frame_json_len",
        "replayAnnotationFrameJsonLen",
    ),
    (
        "subtr_actor_bakkesmod_write_replay_annotation_frame_json",
        "writeReplayAnnotationFrameJson",
    ),
    ("subtr_actor_bakkesmod_replay_annotation_score_at_time", "replayAnnotationScoreAtTime"),
    ("subtr_actor_bakkesmod_poll_replay_annotations", "pollReplayAnnotations"),
)


def quoted_strings(value: str) -> list[str]:
    return re.findall(r'"([^"]+)"', value)


def expand_local_cpp_includes(source_path: Path, source: str) -> str:
    include_pattern = re.compile(r'^\s*#include\s+"([^"]+\.cpp)"\s*$', re.MULTILINE)

    def replace_include(match: re.Match[str]) -> str:
        include_path = source_path.parent / match.group(1)
        if not include_path.exists():
            return match.group(0)
        return include_path.read_text(encoding="utf-8")

    return include_pattern.sub(replace_include, source)


def read_web_player_sources() -> str:
    ignored_suffixes = (".test.ts", ".slow-test.ts", ".test-helper.ts")
    sources: list[str] = []
    for path in sorted(WEB_PLAYER_SOURCE_DIR.rglob("*.ts")):
        if "generated" in path.parts or path.name.endswith(ignored_suffixes):
            continue
        sources.append(path.read_text(encoding="utf-8"))
    return "\n".join(sources)


def read_web_player_styles() -> str:
    sources = [WEB_PLAYER_STYLES_SOURCE.read_text(encoding="utf-8")]
    sources.extend(path.read_text(encoding="utf-8") for path in sorted(WEB_PLAYER_STYLES_DIR.glob("*.css")))
    return "\n".join(sources)


def read_rust_sources() -> str:
    sources: list[str] = []
    for path in sorted(RUST_SOURCE_DIR.rglob("*.rs")):
        if "lib_tests" in path.parts:
            continue
        sources.append(path.read_text(encoding="utf-8"))
    return "\n".join(sources)


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


def singleton_window_open_variables(source: str) -> dict[str, str]:
    match = re.search(
        r"return\s+\{\{(.*?)\}\};\s*\}\s*\n\s*std::vector<SubtrActorPlugin::SingletonWindowControl>",
        source,
        re.DOTALL,
    )
    if not match:
        raise AssertionError("missing singletonWindowControls return block")

    variables: dict[str, str] = {}
    for control in re.finditer(
        r'\{\s*"[^"]+",\s*"(?P<id>[^"]+)",\s*"[^"]+",\s*"[^"]+",\s*'
        r"(?:true|false),\s*\d+,\s*&(?P<open_var>ui[A-Za-z0-9_]+Open),",
        match.group(1),
        re.DOTALL,
    ):
        variables[control.group("id")] = control.group("open_var")
    if not variables:
        raise AssertionError("could not parse singletonWindowControls open variables")
    return variables


def cpp_bool_defaults(source: str) -> dict[str, bool]:
    return {
        match.group("name"): match.group("value") == "true"
        for match in re.finditer(
            r"\bbool\s+(?P<name>[A-Za-z0-9_]+)\s*=\s*(?P<value>true|false)\s*;",
            source,
        )
    }


def web_initial_singleton_visibility(source: str) -> dict[str, bool]:
    visibility: dict[str, bool] = {}
    for section in re.finditer(r"<section\b(?P<tag>[^>]*)>", source, re.DOTALL):
        tag = section.group("tag")
        window_id = re.search(r'\bdata-window-id="(?P<id>[^"]+)"', tag)
        if window_id:
            visibility[window_id.group("id")] = not re.search(r"\bhidden\b", tag)
    if not visibility:
        raise AssertionError("could not parse web singleton window visibility")
    return visibility


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


def stats_window_kind_details(source: str) -> dict[str, tuple[str, bool, bool, bool]]:
    match = re.search(
        r"SubtrActorPlugin::statsWindowKindControls\(\)\s+const\s+\{\s+return\s+\{\{"
        r"(.*?)\}\};\s*\}",
        source,
        re.DOTALL,
    )
    if not match:
        raise AssertionError("missing statsWindowKindControls return block")

    controls: dict[str, tuple[str, bool, bool, bool]] = {}
    for control in re.finditer(
        r'\{\s*UiStatsWindowKind::\w+,\s*"(?P<id>[^"]+)",\s*"(?P<label>[^"]+)",'
        r'\s*"[^"]+",\s*(?:static_cast<[^>]+>\([^)]*\)|[^,]+),\s*'
        r"(?P<scope_selector>true|false),\s*"
        r"(?P<stat_picker>true|false),\s*"
        r"(?P<web>true|false),\s*"
        r"(?:true|false)\s*\}",
        match.group(1),
        re.DOTALL,
    ):
        controls[control.group("id")] = (
            control.group("label"),
            control.group("scope_selector") == "true",
            control.group("stat_picker") == "true",
            control.group("web") == "true",
        )
    if not controls:
        raise AssertionError("could not parse statsWindowKindControls details")
    return controls


def web_stats_window_titles(source: str) -> dict[str, str]:
    match = re.search(
        r"(?:function\s+|(?:private\s+)?)getStatsWindowTitle\([^)]*\):\s*string\s*\{(?P<body>.*?)\n\s{2}\}",
        source,
        re.DOTALL,
    )
    if not match:
        raise AssertionError("missing getStatsWindowTitle")
    titles = {
        item.group("id"): item.group("title")
        for item in re.finditer(
            r'case\s+"(?P<id>[^"]+)":\s*return\s+"(?P<title>[^"]+)";',
            match.group("body"),
        )
    }
    if not titles:
        raise AssertionError("could not parse getStatsWindowTitle")
    return titles


def web_stats_window_scope_selector_ids(source: str) -> tuple[str, ...]:
    match = re.search(
        r"(?:function\s+|(?:private\s+)?)hasStatsWindowScopeSelector\([^)]*\):\s*boolean\s*\{\s*"
        r"return\s+(?P<body>.*?);\s*\}",
        source,
        re.DOTALL,
    )
    if not match:
        raise AssertionError("missing hasStatsWindowScopeSelector")
    return tuple(re.findall(r'kind\s*===\s*"([^"]+)"', match.group("body")))


def web_stats_window_stat_picker_ids(source: str, kind_ids: tuple[str, ...]) -> tuple[str, ...]:
    match = re.search(
        r"(?:function\s+|(?:private\s+)?)hasStatsWindowStatPicker\([^)]*\):\s*boolean\s*\{\s*"
        r"return\s+(?P<body>.*?);\s*\}",
        source,
        re.DOTALL,
    )
    if not match:
        raise AssertionError("missing hasStatsWindowStatPicker")
    body = match.group("body")
    excluded = set(re.findall(r'kind\s*!==\s*"([^"]+)"', body))
    if excluded:
        return tuple(kind_id for kind_id in kind_ids if kind_id not in excluded)
    return tuple(re.findall(r'kind\s*===\s*"([^"]+)"', body))


def main() -> int:
    rust_source = RUST_SOURCE.read_text(encoding="utf-8")
    rust_combined_source = read_rust_sources()
    plugin_source = expand_local_cpp_includes(
        PLUGIN_SOURCE,
        PLUGIN_SOURCE.read_text(encoding="utf-8"),
    )
    plugin_header = PLUGIN_HEADER.read_text(encoding="utf-8")
    plugin_readme_source = PLUGIN_README.read_text(encoding="utf-8")
    abi_header = ABI_HEADER.read_text(encoding="utf-8")
    web_player_config_source = WEB_PLAYER_CONFIG_SOURCE.read_text(encoding="utf-8")
    web_player_main_source = read_web_player_sources()
    web_player_styles_source = read_web_player_styles()
    web_player_template_source = WEB_PLAYER_TEMPLATE_SOURCE.read_text(encoding="utf-8")
    web_player_floating_windows_source = WEB_PLAYER_FLOATING_WINDOWS_SOURCE.read_text(
        encoding="utf-8"
    )
    web_player_timeline_markers_source = WEB_PLAYER_TIMELINE_MARKERS_SOURCE.read_text(
        encoding="utf-8"
    )
    web_player_boost_pickup_filters_source = WEB_PLAYER_BOOST_PICKUP_FILTERS_SOURCE.read_text(
        encoding="utf-8"
    )
    web_player_player_modules_source = WEB_PLAYER_PLAYER_MODULES_SOURCE.read_text(
        encoding="utf-8"
    )
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
    web_singleton_window_ids = tuple(
        ts_array(web_player_floating_windows_source, "SINGLETON_WINDOW_IDS")
    )
    if web_singleton_window_ids != web_singleton_type_ids:
        errors.append(
            "stats evaluation player singleton window order differs from its config type: "
            f"type={web_singleton_type_ids!r} array={web_singleton_window_ids!r}"
        )
    # Rocket League already provides camera and playback controls in-game, and the
    # plugin does not support a real multi-replay queue. Keep those replay-player
    # surfaces out of the BakkesMod launcher even though the web player retains them.
    plugin_excluded_web_window_ids = (
        "camera",
        "playback",
        "mechanics-review",
        "replay-loading",
    )
    plugin_expected_web_singleton_window_ids = tuple(
        window_id
        for window_id in web_singleton_window_ids
        if window_id not in plugin_excluded_web_window_ids
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
    if web_window_ids != plugin_expected_web_singleton_window_ids:
        errors.append(
            "web singleton window order drifted from stats evaluation player: "
            f"expected={plugin_expected_web_singleton_window_ids!r} actual={web_window_ids!r}"
        )
    web_launcher_window_buttons = tuple(
        (window_id, label)
        for window_id, label in web_launcher_buttons(
            web_player_template_source,
            "data-window-toggle",
        )
        if window_id not in plugin_excluded_web_window_ids
    )
    if plugin_web_window_controls != web_launcher_window_buttons:
        errors.append(
            "web singleton launcher labels drifted from stats evaluation player: "
            f"expected={web_launcher_window_buttons!r} actual={plugin_web_window_controls!r}"
        )
    for excluded_window_id in plugin_excluded_web_window_ids:
        if any(window_id == excluded_window_id for window_id, _ in plugin_web_window_controls):
            errors.append(
                f"plugin launcher still exposes replay-player-only window {excluded_window_id!r}"
            )
    plugin_window_open_variables = singleton_window_open_variables(plugin_source)
    plugin_bool_defaults = cpp_bool_defaults(plugin_header)
    web_initial_window_visibility = web_initial_singleton_visibility(web_player_template_source)
    plugin_initial_window_visibility: list[tuple[str, bool]] = []
    web_initial_window_visibility_ordered: list[tuple[str, bool]] = []
    for window_id in web_window_ids:
        open_variable = plugin_window_open_variables.get(window_id)
        if open_variable is None:
            errors.append(f"plugin singleton window {window_id!r} is missing an open bool pointer")
            continue
        if open_variable not in plugin_bool_defaults:
            errors.append(
                f"plugin singleton window {window_id!r} points at {open_variable}, "
                "but the header has no default bool value for it"
            )
            continue
        if window_id not in web_initial_window_visibility:
            errors.append(
                f"stats evaluation player template has no initial visibility for {window_id!r}"
            )
            continue
        plugin_initial_window_visibility.append(
            (window_id, plugin_bool_defaults[open_variable])
        )
        web_initial_window_visibility_ordered.append(
            (window_id, web_initial_window_visibility[window_id])
        )
    if plugin_initial_window_visibility != web_initial_window_visibility_ordered:
        errors.append(
            "plugin singleton default visibility drifted from stats evaluation player: "
            f"expected={web_initial_window_visibility_ordered!r} "
            f"actual={plugin_initial_window_visibility!r}"
        )
    require_contains(
        web_player_styles_source,
        ".floating-window,\n.stats-window,\n.scoreboard-window {\n"
        "  position: absolute;\n"
        "  left: clamp(0.8rem, var(--window-x, 1rem), calc(100vw - 18rem));",
        "stats evaluation player floating windows share overlay chrome",
        errors,
    )
    require_contains(
        plugin_source,
        "void pushWebFloatingWindowStyle() {\n"
        "  ImGui::PushStyleVar(ImGuiStyleVar_WindowPadding, ImVec2{14.0f, 12.0f});\n"
        "  ImGui::PushStyleVar(ImGuiStyleVar_WindowRounding, 8.0f);\n"
        "  ImGui::PushStyleVar(ImGuiStyleVar_FrameRounding, 6.0f);\n"
        "  ImGui::PushStyleVar(ImGuiStyleVar_ItemSpacing, ImVec2{8.0f, 8.0f});\n"
        "  ImGui::PushStyleColor(ImGuiCol_WindowBg, ImVec4{0.03f, 0.07f, 0.10f, 0.88f});\n"
        "  ImGui::PushStyleColor(ImGuiCol_Border, ImVec4{1.0f, 1.0f, 1.0f, 0.12f});\n"
        "}",
        "plugin floating windows share web-like overlay chrome",
        errors,
    )
    require_contains(
        plugin_source,
        "if (entry.kind == RenderEntryKind::Stats) {\n"
        "      renderWithFloatingWindowStyle([&]() {\n"
        "        renderStatsWindow(uiStatsWindows[entry.stats_index], entry.stats_index);\n"
        "      });",
        "plugin stats windows use shared web-like overlay chrome",
        errors,
    )
    require_contains(
        plugin_source,
        'if (id == "scoreboard") {\n'
        "      renderScoreboardWindow();\n"
        '    } else if (id == "mechanics") {\n'
        "      renderWithFloatingWindowStyle([&]() { renderEventsWindow(); });",
        "plugin leaves scoreboard pill separate from shared floating chrome",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".floating-window-header h2,\n"
        ".stats-window-header h2 {\n"
        "  margin: 0;\n"
        "  color: #87afd4;\n"
        "  font-size: 0.68rem;\n"
        "  font-weight: 800;\n"
        "  letter-spacing: 0.14em;\n"
        "  text-transform: uppercase;",
        "stats evaluation player window headers render as uppercase labels",
        errors,
    )
    require_contains(
        plugin_source,
        "std::string uppercaseHeaderLabel(std::string_view value) {\n"
        "  std::string label;\n"
        "  label.reserve(value.size());\n"
        "  for (char ch : value) {\n"
        "    label.push_back(static_cast<char>(std::toupper(static_cast<unsigned char>(ch))));\n"
        "  }\n"
        "  return label;\n"
        "}",
        "plugin window headers render as uppercase labels",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".floating-window-hide,\n"
        ".stats-window-action {\n"
        "  flex-shrink: 0;\n"
        "  padding: var(--ui-control-padding-block) var(--ui-control-padding-inline);\n"
        "  border-radius: var(--ui-radius-md);",
        "stats evaluation player window hide buttons share action chrome",
        errors,
    )
    require_contains(
        plugin_source,
        "ImGui::PushStyleVar(ImGuiStyleVar_FrameRounding, 6.0f);\n"
        "  const bool hideClicked = ImGui::Button(hideLabel.c_str());\n"
        "  ImGui::PopStyleVar();",
        "plugin window hide buttons use shared rounded action chrome",
        errors,
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
    plugin_stats_window_details = stats_window_kind_details(plugin_source)
    web_stats_window_title_by_id = web_stats_window_titles(web_player_main_source)
    plugin_web_stats_window_titles = tuple(
        (kind_id, plugin_stats_window_details[kind_id][0])
        for kind_id in plugin_web_stats_window_kind_ids
    )
    web_stats_window_titles_ordered = tuple(
        (kind_id, web_stats_window_title_by_id[kind_id])
        for kind_id in web_stats_window_kind_ids
    )
    if plugin_web_stats_window_titles != web_stats_window_titles_ordered:
        errors.append(
            "web stats window titles drifted from stats evaluation player: "
            f"expected={web_stats_window_titles_ordered!r} "
            f"actual={plugin_web_stats_window_titles!r}"
        )
    plugin_web_stats_scope_selector_ids = tuple(
        kind_id
        for kind_id in plugin_web_stats_window_kind_ids
        if plugin_stats_window_details[kind_id][1]
    )
    web_stats_scope_selector_ids = web_stats_window_scope_selector_ids(web_player_main_source)
    if plugin_web_stats_scope_selector_ids != web_stats_scope_selector_ids:
        errors.append(
            "web stats window scope-selector kinds drifted from stats evaluation player: "
            f"expected={web_stats_scope_selector_ids!r} "
            f"actual={plugin_web_stats_scope_selector_ids!r}"
        )
    plugin_web_stats_picker_ids = tuple(
        kind_id
        for kind_id in plugin_web_stats_window_kind_ids
        if plugin_stats_window_details[kind_id][2]
    )
    web_stats_picker_ids = web_stats_window_stat_picker_ids(
        web_player_main_source,
        web_stats_window_kind_ids,
    )
    if plugin_web_stats_picker_ids != web_stats_picker_ids:
        errors.append(
            "web stats window stat-picker kinds drifted from stats evaluation player: "
            f"expected={web_stats_picker_ids!r} actual={plugin_web_stats_picker_ids!r}"
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
        web_player_main_source,
        'select.className = "stats-window-scope-select";',
        "stats evaluation player scope selector class",
        errors,
    )
    require_contains(
        web_player_main_source,
        "const teamClass = this.getStatsWindowScopeTeamClass(statsWindow);",
        "stats evaluation player scope selector applies team accent class",
        errors,
    )
    require_contains(
        web_player_main_source,
        "select.classList.add(teamClass);",
        "stats evaluation player scope selector applies team accent class",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".stats-window-scope-select.team-blue,\n"
        ".stats-window-scope-select.team-orange,\n"
        ".stats-window-stat-target.team-blue,\n"
        ".stats-window-stat-target.team-orange {\n"
        "  border-color: var(--team-accent);\n"
        "  box-shadow: inset 0.22rem 0 0 var(--team-accent);",
        "stats evaluation player scope selector team accent",
        errors,
    )
    require_contains(
        plugin_source,
        "auto pushStatsScopeSelectorStyle = [](std::optional<LinearColor> teamColor) {\n"
        "    ImGui::SetNextItemWidth(std::min(208.0f, ImGui::GetContentRegionAvail().x));",
        "plugin stats scope selector uses bounded web-like width",
        errors,
    )
    require_contains(
        plugin_source,
        "ImGui::PushStyleColor(ImGuiCol_Border, accent);\n"
        "    ImGui::PushStyleColor(\n"
        "        ImGuiCol_FrameBg,\n"
        "        ImVec4{accent.x * 0.18f, accent.y * 0.18f, accent.z * 0.18f, 0.58f});",
        "plugin stats scope selector applies team accent frame",
        errors,
    )
    require_contains(
        plugin_source,
        "const std::optional<LinearColor> selectedColor =\n"
        "        selected ? std::make_optional(selected->is_team_0 != 0 ? LinearColor{80, 190, 255, 255}",
        "plugin player stats scope selector derives selected team accent",
        errors,
    )
    require_contains(
        plugin_source,
        "const int selectorStyleColors = pushStatsScopeSelectorStyle(selectedColor);\n"
        "    const bool comboOpen = ImGui::BeginCombo(\n"
        "        std::format(\"##stats-window-team-scope-{}\", window.id).c_str(),",
        "plugin team stats scope selector wraps combo in team accent style",
        errors,
    )
    require_contains(
        web_player_main_source,
        'queryInput.placeholder = "Search stats";',
        "stats evaluation player stats picker search placeholder",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::TextDisabled("Search stats");',
        "plugin stats picker shows web-like search prompt",
        errors,
    )
    require_contains(
        plugin_source,
        'std::format("##stats-window-search-{}", window.id).c_str()',
        "plugin stats picker search input uses hidden label",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".stats-window-toolbar {\n  justify-content: end;\n}",
        "stats evaluation player right-aligns non-scoped stats add toolbar",
        errors,
    )
    require_contains(
        plugin_source,
        "const float addButtonX =\n"
        "        std::max(ImGui::GetCursorPosX(), ImGui::GetWindowContentRegionMax().x - addButtonSize);\n"
        "    ImGui::SetCursorPosX(addButtonX);",
        "plugin right-aligns non-scoped stats add toolbar",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".stats-window-add-button {\n"
        "  width: var(--ui-control-height);\n"
        "  min-width: var(--ui-control-height);\n"
        "  padding: 0;",
        "stats evaluation player stats add button is a square control",
        errors,
    )
    require_contains(
        plugin_source,
        "const float addButtonSize = ImGui::GetFrameHeight();",
        "plugin stats add button uses square control sizing",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".stats-window-picker {\n"
        "  display: grid;\n"
        "  gap: var(--ui-gap-sm);\n"
        "  padding: var(--ui-panel-padding);\n"
        "  border-radius: var(--ui-radius-md);\n"
        "  border: 1px solid rgba(255, 255, 255, 0.08);\n"
        "  background: rgba(255, 255, 255, 0.04);",
        "stats evaluation player stats picker is a bordered panel",
        errors,
    )
    require_contains(
        plugin_source,
        "ImGui::PushStyleVar(ImGuiStyleVar_ChildRounding, 6.0f);\n"
        "  ImGui::PushStyleVar(ImGuiStyleVar_WindowPadding, ImVec2{12.0f, 10.0f});\n"
        "  ImGui::PushStyleColor(ImGuiCol_ChildBg, ImVec4{1.0f, 1.0f, 1.0f, 0.04f});\n"
        "  ImGui::PushStyleColor(ImGuiCol_Border, ImVec4{1.0f, 1.0f, 1.0f, 0.08f});",
        "plugin stats picker uses web-like bordered panel chrome",
        errors,
    )
    require_contains(
        web_player_main_source,
        'addGroup.innerHTML = `<span>Add all ${category}</span><strong>${group.length}</strong>`;',
        "stats evaluation player stats picker category row",
        errors,
    )
    require_contains(
        plugin_source,
        'renderStatsPickerItem(\n'
        '            std::format("Add all {}", category),\n'
        '            std::to_string(count),\n'
        '            std::format("all-{}", category))',
        "plugin stats picker category row mirrors web button/count layout",
        errors,
    )
    require_contains(
        web_player_main_source,
        'item.innerHTML = `<span>${definition.label}</span><strong>${definition.scope}</strong>`;',
        "stats evaluation player stats picker stat row",
        errors,
    )
    require_contains(
        plugin_source,
        "renderStatsPickerItem(\n"
        "            definition.label,\n"
        "            uiStatScopeLabel(definition),\n"
        "            definition.id,\n"
        "            disabled)",
        "plugin stats picker stat row mirrors web button label/scope layout",
        errors,
    )
    require_contains(
        plugin_source,
        'const std::string buttonId = std::format("##stats-picker-item-{}-{}", window.id, id);',
        "plugin stats picker rows use dedicated hidden ImGui ids like web button rows",
        errors,
    )
    require_contains(
        plugin_source,
        "drawList->AddText(\n"
        "        ImVec2{rightX, textY},\n"
        "        disabled ? IM_COL32(107, 133, 156, 255) : IM_COL32(135, 175, 212, 255),\n"
        "        metaString.c_str());",
        "plugin stats picker rows draw right-aligned web accent metadata",
        errors,
    )
    require_contains(
        web_player_main_source,
        'empty.textContent = statRegistry.length === 0 ? "No stats available." : "No matching stats.";',
        "stats evaluation player stats picker no-results empty state",
        errors,
    )
    require_contains(
        web_player_main_source,
        'empty.className = "stat-window-empty";',
        "stats evaluation player stats empty state class",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".stat-window-empty {\n"
        "  margin: 0;\n"
        "  color: #9eb4c6;\n"
        "  font-size: 0.88rem;\n"
        "}",
        "stats evaluation player stats empty state styling",
        errors,
    )
    require_contains(
        plugin_source,
        'renderStatsWindowEmpty("No matching stats.");',
        "plugin stats picker no-results empty state mirrors web",
        errors,
    )
    require_contains(
        plugin_source,
        "void renderStatsWindowEmpty(std::string_view message) {\n"
        "  const std::string messageString{message};\n"
        "  ImGui::PushStyleColor(ImGuiCol_Text, ImVec4{0.62f, 0.71f, 0.78f, 1.0f});\n"
        "  ImGui::TextWrapped(\"%s\", messageString.c_str());\n"
        "  ImGui::PopStyleColor();\n"
        "}",
        "plugin stats empty state uses web-like muted body text",
        errors,
    )
    for plugin_only_stats_picker_surface in (
        'std::format("Search stats##{}", window.id).c_str()',
        'ImGui::SmallButton(std::format("Clear##stat-search-{}", window.id).c_str())',
        'std::format("Add all {} ({})##{}-{}", category, count, window.id, category)',
        'std::format("Add all {}   {}##{}-{}", category, count, window.id, category)',
        'std::format(\n        "{}  [{}]##{}-{}",',
        'std::format(\n        "{}   {}##{}-{}",',
        'ImGui::TextDisabled(\n          "%s  [%s selected]",',
        'ImGui::TextDisabled(\n          "%s   %s",',
        "ImGui::Selectable(itemLabel.c_str(), alreadySelected)",
        'ImGui::Text("No matching stats.");',
        'void renderStatsWindowEmpty(std::string_view message) {\n  ImGui::Spacing();\n  ImGui::TextDisabled("%s", std::string{message}.c_str());\n}',
    ):
        reject_contains(
            plugin_source,
            plugin_only_stats_picker_surface,
            "plugin stats picker plugin-only row/search surface",
            errors,
        )
    require_contains(
        web_player_main_source,
        'empty.textContent = "Load a replay to show stats.";',
        "stats evaluation player stats windows no-frame empty state",
        errors,
    )
    require_contains(
        plugin_source,
        'renderStatsWindowEmpty("Load a replay to show stats.");',
        "plugin stats windows no-data empty state mirrors web",
        errors,
    )
    require_contains(
        web_player_main_source,
        'target ? definition.format(definition.read(target)) : "--"',
        "stats evaluation player scoped stats render missing targets as dash rows",
        errors,
    )
    require_contains(
        plugin_source,
        "void SubtrActorPlugin::renderMissingStatsRows(UiStatsWindow &window)",
        "plugin scoped stats render missing targets as dash rows",
        errors,
    )
    require_contains(
        plugin_source,
        'renderStatsWindowValueRow(window, i, uiStatLabel(statId), "--")',
        "plugin scoped stats missing target value mirrors web dash",
        errors,
    )
    for plugin_only_stats_window_empty_surface in (
        'ImGui::Text("Waiting for selected player.");',
        'ImGui::Text("Waiting for sampled players.");',
        'ImGui::TextWrapped("Start live analysis or load replay stats to show team stats.");',
        'ImGui::Columns(3, "player-stat-rows", false);',
        'ImGui::Columns(3, "team-stat-rows", false);',
    ):
        reject_contains(
            plugin_source,
            plugin_only_stats_window_empty_surface,
            "plugin stats window table/waiting surface",
            errors,
        )
    require_contains(
        web_player_main_source,
        "renderAdHocStats(",
        "stats evaluation player ad-hoc stats renderer",
        errors,
    )
    require_contains(
        web_player_main_source,
        'targetSelect.className = "stats-window-stat-target";',
        "stats evaluation player ad-hoc rows expose target selector",
        errors,
    )
    require_contains(
        web_player_main_source,
        "const teamClass = this.getStatTargetTeamClass(definition, entry.targetId);",
        "stats evaluation player ad-hoc target selector applies team accent class",
        errors,
    )
    require_contains(
        web_player_main_source,
        "targetSelect.classList.add(teamClass);",
        "stats evaluation player ad-hoc target selector applies team accent class",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".stats-window-stat-target {\n"
        "  max-width: 7rem;\n"
        "  margin-left: 0.35rem;\n"
        "  padding: 0.16rem 0.3rem;\n"
        "  border-radius: var(--ui-radius-sm);\n"
        "  font-size: 0.7rem;",
        "stats evaluation player ad-hoc target selector compact sizing",
        errors,
    )
    require_contains(
        plugin_source,
        "void SubtrActorPlugin::renderAdHocStatsWindow(UiStatsWindow &window)",
        "plugin ad-hoc stats renderer",
        errors,
    )
    require_contains(
        plugin_source,
        "renderAdHocTargetSelector(window, entry, statId, i);",
        "plugin ad-hoc rows expose target selector",
        errors,
    )
    require_contains(
        plugin_source,
        "auto pushAdHocTargetSelectorStyle = [](std::optional<LinearColor> teamColor) {\n"
        "    ImGui::SetNextItemWidth(std::min(112.0f, ImGui::GetContentRegionAvail().x));",
        "plugin ad-hoc target selector uses compact web-like width",
        errors,
    )
    require_contains(
        plugin_source,
        "const int selectorStyleColors = pushAdHocTargetSelectorStyle(selectedColor);\n"
        "    const bool comboOpen = ImGui::BeginCombo(\n"
        "        std::format(\"##ad-hoc-target-{}-{}\", window.id, index).c_str(),",
        "plugin ad-hoc target selector wraps combo in team accent style",
        errors,
    )
    require_contains(
        plugin_source,
        "ImGui::PushStyleColor(ImGuiCol_Border, accent);\n"
        "    ImGui::PushStyleColor(\n"
        "        ImGuiCol_FrameBg,\n"
        "        ImVec4{accent.x * 0.18f, accent.y * 0.18f, accent.z * 0.18f, 0.58f});",
        "plugin ad-hoc target selector applies team accent frame",
        errors,
    )
    require_contains(
        plugin_source,
        "const std::string statValue = adHocStatValue(statId, entry.target_id);",
        "plugin ad-hoc rows render stat value inline like web",
        errors,
    )
    for plugin_only_ad_hoc_surface in (
        'ImGui::Columns(4, "ad-hoc-stat-rows", false);',
        'ImGui::BeginChild("ad-hoc-events", ImVec2{0.0f, 0.0f}, true);',
        'ImGui::TextColored(toImVec4(event.color), "%.2fs %s", event.time, event.actor.c_str());',
    ):
        reject_contains(
            plugin_source,
            plugin_only_ad_hoc_surface,
            "plugin ad-hoc stats window plugin-only table/event surface",
            errors,
        )
    require_contains(
        web_player_main_source,
        'teamSection.className = `stats-window-team-group ${this.getTeamScopeClass(team)}`;',
        "stats evaluation player all-player stats team groups",
        errors,
    )
    require_contains(
        web_player_main_source,
        'teamHeader.className = "stats-window-team-header";',
        "stats evaluation player all-player stats team headers",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".stats-window-team-header {\n"
        "  display: flex;\n"
        "  align-items: center;\n"
        "  justify-content: space-between;",
        "stats evaluation player all-player stats header layout",
        errors,
    )
    require_contains(
        plugin_source,
        'const std::string teamTitle = std::format("{} team", teamLabel(isTeam0));\n'
        "    const std::string teamMeta =\n"
        '        std::format("{} player{}", playerCount, playerCount == 1 ? "" : "s");',
        "plugin all-player stats team header title/meta mirrors web",
        errors,
    )
    require_contains(
        plugin_source,
        "ImGui::SameLine(metaX);\n"
        "    ImGui::TextColored(teamColor, \"%s\", teamMeta.c_str());\n"
        "    ImGui::PushStyleColor(ImGuiCol_Separator, teamColor);\n"
        "    ImGui::Separator();",
        "plugin all-player stats team header aligns meta and divider",
        errors,
    )
    require_contains(
        web_player_main_source,
        'section.className = `stats-window-entity ${getTeamClass(player.is_team_0)}`;',
        "stats evaluation player all-player stats entity sections",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".stats-window-entity {\n"
        "  display: grid;\n"
        "  gap: var(--ui-gap-xs);\n"
        "  padding-left: 0.5rem;\n"
        "  border-left: 2px solid rgba(255, 255, 255, 0.12);",
        "stats evaluation player grouped stats entity accent rail",
        errors,
    )
    require_contains(
        plugin_source,
        "drawList->AddRectFilled(\n"
        "          ImVec2{entityStart.x, entityStart.y + 2.0f},\n"
        "          ImVec2{entityStart.x + 2.0f, entityStart.y + ImGui::GetTextLineHeight() + 2.0f},\n"
        "          ImGui::GetColorU32(teamColor));\n"
        "      ImGui::Indent(8.0f);",
        "plugin all-player entity rows use web-like accent rail",
        errors,
    )
    require_contains(
        plugin_source,
        "bool SubtrActorPlugin::renderStatsWindowValueRow(",
        "plugin shared stats row renderer",
        errors,
    )
    require_contains(
        web_player_main_source,
        'row.className = "stats-window-stat-row";',
        "stats evaluation player shared stat row class",
        errors,
    )
    require_contains(
        web_player_main_source,
        'name.className = "stats-window-stat-name";',
        "stats evaluation player shared stat row label class",
        errors,
    )
    require_contains(
        web_player_main_source,
        'valueEl.className = "stats-window-stat-value";',
        "stats evaluation player shared stat row value class",
        errors,
    )
    require_contains(
        web_player_main_source,
        'remove.className = "stats-window-stat-remove";',
        "stats evaluation player shared stat row remove class",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".stats-window-stat-row {\n"
        "  display: grid;\n"
        "  grid-template-columns: minmax(0, 1fr) auto auto;\n"
        "  gap: var(--ui-gap-sm);\n"
        "  align-items: center;\n"
        "  min-height: 1.45rem;",
        "stats evaluation player shared stat row grid layout",
        errors,
    )
    require_contains(
        plugin_source,
        "ImGui::AlignTextToFramePadding();\n"
        "  ImGui::TextColored(ImVec4{0.62f, 0.71f, 0.78f, 1.0f}, \"%s\", labelString.c_str());\n"
        "  ImGui::SameLine(valueX);\n"
        "  ImGui::AlignTextToFramePadding();\n"
        "  ImGui::TextColored(ImVec4{0.93f, 0.96f, 0.98f, 1.0f}, \"%s\", valueString.c_str());",
        "plugin shared stat row uses muted label and bright value like web",
        errors,
    )
    require_contains(
        plugin_source,
        "ImGui::PushStyleVar(ImGuiStyleVar_FramePadding, ImVec2{5.0f, 2.0f});\n"
        "  ImGui::PushStyleVar(ImGuiStyleVar_FrameRounding, 4.0f);\n"
        "  if (ImGui::SmallButton(removeLabel.c_str())) {",
        "plugin shared stat row uses compact rounded remove button",
        errors,
    )
    require_contains(
        plugin_source,
        'renderStatsWindowValueRow(\n                window, i, statLabel, statValue, std::format("player-{}", player.player_index))',
        "plugin all-player stats rows use shared row renderer",
        errors,
    )
    require_contains(
        plugin_source,
        'renderStatsWindowValueRow(\n              window, i, statLabel, statValue, std::format("team-{}", isTeam0))',
        "plugin all-team stats rows use shared row renderer",
        errors,
    )
    for plugin_only_grouped_stats_surface in (
        'ImGui::TreeNodeEx(playerName.c_str(), ImGuiTreeNodeFlags_DefaultOpen)',
        'std::format("all-player-stat-rows-{}", player.player_index).c_str()',
        'std::format("all-team-stat-rows-{}", isTeam0).c_str()',
    ):
        reject_contains(
            plugin_source,
            plugin_only_grouped_stats_surface,
            "plugin grouped stats table/tree surface",
            errors,
        )
    require_contains(
        web_player_main_source,
        'list.className = "goal-label-list";',
        "stats evaluation player goal-label list",
        errors,
    )
    require_contains(
        web_player_main_source,
        'item.className = "goal-label-item";',
        "stats evaluation player goal-label items",
        errors,
    )
    require_contains(
        web_player_main_source,
        'meta.textContent = `${formatTime(time)} · ${scorerName}`;',
        "stats evaluation player goal-label meta format",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::TextDisabled(\n        "%s · %s",',
        "plugin goal-label meta format mirrors web",
        errors,
    )
    require_contains(
        web_player_main_source,
        "actions.append(watch, jump);",
        "stats evaluation player goal-label actions are part of each card header layout",
        errors,
    )
    require_contains(
        plugin_source,
        'const float actionsX = std::max(\n'
        "        ImGui::GetCursorPosX(),\n"
        "        ImGui::GetWindowContentRegionMax().x - watchWidth - cueWidth -\n"
        "            ImGui::GetStyle().ItemSpacing.x);\n"
        '    ImGui::TextColored(toImVec4(event.color), "Goal %zu", ordinal + 1);\n'
        "    ImGui::SameLine(actionsX);",
        "plugin goal-label actions align with goal header like web",
        errors,
    )
    require_contains(
        plugin_source,
        'const bool watchClicked = ImGui::SmallButton("Watch");\n'
        "    ImGui::SameLine();\n"
        '    const bool cueClicked = ImGui::SmallButton("Cue");\n'
        "    ImGui::TextDisabled(",
        "plugin goal-label actions render before metadata like web",
        errors,
    )
    require_contains(
        web_player_main_source,
        'empty.textContent = "Unlabeled";',
        "stats evaluation player unlabeled goal chip",
        errors,
    )
    require_contains(
        plugin_source,
        'renderGoalTagChip("Unlabeled", true, 0);',
        "plugin unlabeled goal tag chip mirrors web",
        errors,
    )
    require_contains(
        plugin_source,
        'const std::string label = std::format("{}##goal-tag-chip-{}", text, chipIndex);',
        "plugin goal-label tags render as individual chips",
        errors,
    )
    require_contains(
        plugin_source,
        "for (size_t tagIndex = 0; tagIndex < tags.size(); tagIndex += 1) {\n"
        "        if (tagIndex > 0) {\n"
        "          ImGui::SameLine();\n"
        "        }\n"
        "        renderGoalTagChip(tags[tagIndex], false, tagIndex);\n"
        "      }",
        "plugin goal-label tag chips preserve separate labels",
        errors,
    )
    require_contains(
        web_player_main_source,
        "const performer = formatGoalTagPerformer(tag);\n"
        '          chip.textContent = `${formatMechanicKind(tag.kind)} ${Math.round(tag.metadata.confidence * 100)}%${\n'
        "            performer ? ` - ${performer}` : \"\"\n"
        "          }`;",
        "stats evaluation player goal-label tag confidence and performer chips",
        errors,
    )
    require_contains(
        web_player_main_source,
        "left.kind.localeCompare(right.kind) ||\n          right.metadata.confidence - left.metadata.confidence",
        "stats evaluation player sorts goal tags by kind and metadata confidence",
        errors,
    )
    require_contains(
        web_player_main_source,
        "const orderedGoalIndexes = goalContexts.map((_, index) => index);",
        "stats evaluation player derives goal-label overview from embedded goal context tags",
        errors,
    )
    require_contains(
        plugin_source,
        "auto goalTagChip = [](const UiEventRecord &event) {",
        "plugin goal-label window formats replay goal tags",
        errors,
    )
    require_contains(
        plugin_source,
        "std::sort(tags.begin(), tags.end(), [](const GoalTagChip &left, const GoalTagChip &right) {\n"
        "      if (left.label == right.label) {\n"
        "        return left.confidence > right.confidence;\n"
        "      }\n"
        "      return left.label < right.label;\n"
        "    });",
        "plugin sorts goal tags by label and descending confidence like web",
        errors,
    )
    require_contains(
        plugin_source,
        "const bool samePlayer = goalEvent.has_player == 0 || candidate.has_player == 0 ||",
        "plugin goal-label tags match scorer like web goal tags",
        errors,
    )
    require_contains(
        plugin_source,
        "const bool nearbyTime = std::fabs(candidate.time - goalEvent.time) <= 0.25f;",
        "plugin goal-label tags associate with nearby goal context",
        errors,
    )
    require_contains(
        plugin_source,
        "if (!hasMatchingGoalEvent) {\n      goalEventIndexes.push_back(index);\n    }",
        "plugin keeps tag-only goals in goal-label overview",
        errors,
    )
    reject_contains(
        plugin_source,
        'ImGui::TextDisabled("%s", event.details.empty() ? "Unlabeled" : event.details.c_str());',
        "plugin goal labels use event details instead of goal tag chips",
        errors,
    )
    for plugin_only_goal_labels_surface in (
        'ImGui::BeginChild("goal-labels", ImVec2{0.0f, 0.0f}, true);',
        'ImGui::TextDisabled("%.2fs - %s", event.time, event.actor.c_str());',
        'ImGui::TextWrapped("%s", event.label.c_str());',
        'ImGui::TextWrapped("No goals loaded.");',
        'ImGui::TextDisabled("%s", tags.empty() ? "Unlabeled" : joinStrings(tags, " · ").c_str());',
        'ImGui::TextDisabled("%s", tags.empty() ? "Unlabeled" : joinStrings(tags, " · ").c_str());\n    if (ImGui::SmallButton("Watch"))',
    ):
        reject_contains(
            plugin_source,
            plugin_only_goal_labels_surface,
            "plugin goal labels plugin-only log surface",
            errors,
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
        web_player_template_source,
        '<button id="toggle-playback" disabled>Play</button>',
        "stats evaluation player playback transport button",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".transport-row {\n"
        "  display: flex;\n"
        "  gap: var(--ui-gap-sm);\n"
        "}\n\n"
        ".transport-row > * {\n"
        "  flex: 1 1 auto;\n"
        "}",
        "stats evaluation player transport rows distribute controls",
        errors,
    )
    require_contains(
        plugin_source,
        "const float playbackTransportWidth = ImGui::GetContentRegionAvail().x;\n"
        "  const float playbackTransportGap = ImGui::GetStyle().ItemSpacing.x;\n"
        "  const float playbackTransportItemWidth =\n"
        "      std::max(72.0f, (playbackTransportWidth - playbackTransportGap) * 0.5f);",
        "plugin playback transport row distributes controls like web flex row",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::Button(label, ImVec2{width, 0.0f})',
        "plugin playback transport uses full-row button sizing",
        errors,
    )
    require_contains(
        plugin_source,
        "ImGui::SameLine(0.0f, playbackTransportGap);",
        "plugin playback transport uses explicit web-like gap",
        errors,
    )
    require_contains(
        web_player_template_source,
        '<select id="playback-rate" disabled>',
        "stats evaluation player playback rate select",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::BeginCombo("##playback-rate", playbackRateLabels[playbackRateIndex])',
        "plugin playback rate uses hidden-label web-like selector",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::Checkbox("Skip post-goal resets", &nextSkipPostGoalTransitions)',
        "plugin playback skip post-goal label mirrors web",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::Checkbox("Skip kickoff countdowns", &nextSkipKickoffs)',
        "plugin playback skip kickoff label mirrors web",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::Columns(2, "playback-detail-grid", false);',
        "plugin playback exposes web-like detail grid",
        errors,
    )
    require_contains(
        web_player_template_source,
        '<dd id="duration-readout">0.00s</dd>',
        "stats evaluation player playback duration starts with a numeric readout",
        errors,
    )
    require_contains(
        plugin_source,
        'renderWebDetailGridCell("Duration", std::format("{:.2f}s", durationSeconds));',
        "plugin playback duration uses web-like detail grid readout",
        errors,
    )
    require_contains(
        web_player_main_source,
        "export function getPlaybackConfigSnapshot({",
        "stats evaluation player playback config snapshot is explicit",
        errors,
    )
    require_contains(
        web_player_main_source,
        "currentTime: state?.currentTime,",
        "stats evaluation player playback config snapshot stores current time",
        errors,
    )
    require_contains(
        web_player_main_source,
        "playbackRate.disabled = !enabled;",
        "stats evaluation player disables playback rate without transport",
        errors,
    )
    require_contains(
        plugin_source,
        "const bool playbackRateDisabled = !transportEnabled;\n"
        "  pushPlaybackDisabledStyle(playbackRateDisabled);\n"
        "  ImGui::SetNextItemWidth(playbackTransportItemWidth);\n"
        "  const bool playbackRateOpen =\n"
        '      ImGui::BeginCombo("##playback-rate", playbackRateLabels[playbackRateIndex]);',
        "plugin playback rate selector is disabled and flex-sized with transport",
        errors,
    )
    require_contains(
        plugin_source,
        "if (playbackRateDisabled) {\n"
        "      ImGui::CloseCurrentPopup();\n"
        "      ImGui::EndCombo();",
        "plugin playback rate disabled selector cannot remain open",
        errors,
    )
    reject_contains(
        plugin_source,
        "playbackStatus",
        "plugin playback persists hidden status state",
        errors,
    )
    reject_contains(
        plugin_source,
        'parseJsonStringProperty(*playback, "status")',
        "plugin playback imports hidden status field",
        errors,
    )
    reject_contains(
        plugin_source,
        ',\\"status\\":',
        "plugin playback exports hidden status field",
        errors,
    )
    for plugin_only_playback_surface in (
        'ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "WEB PLAYBACK CONFIG");',
        'ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "ANALYSIS");',
        'ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "TIMING");',
        'playbackButton("Seek", !transportEnabled)',
        'ImGui::Checkbox("Playing", &nextPlaying)',
        'ImGui::InputFloat("Current time"',
    ):
        reject_contains(
            plugin_source,
            plugin_only_playback_surface,
            "plugin playback window plugin-only surface",
            errors,
        )
    reject_contains(
        plugin_source,
        'ImGui::TextDisabled("Duration");\n  if (durationSeconds > 0.0f) {',
        "plugin playback duration renders a dash fallback",
        errors,
    )
    require_contains(
        web_player_template_source,
        '<input id="recording-fps" type="number" min="1" max="120" step="1" value="60" />',
        "stats evaluation player recording FPS input",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".recording-controls {\n"
        "  display: grid;\n"
        "  grid-template-columns: repeat(2, minmax(0, 1fr));\n"
        "  gap: var(--ui-gap-sm);\n"
        "}",
        "stats evaluation player recording controls use two-column grid",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::InputInt(\n          "##recording-fps",',
        "plugin recording FPS uses web-like numeric input",
        errors,
    )
    require_contains(
        web_player_main_source,
        "elements.fps.disabled = isRecording;",
        "stats evaluation player disables recording FPS while recording",
        errors,
    )
    require_contains(
        plugin_source,
        "recordingSettingsLocked ? ImGuiInputTextFlags_ReadOnly : 0",
        "plugin recording FPS input is read-only while recording",
        errors,
    )
    require_contains(
        web_player_template_source,
        '<select id="recording-playback-rate">',
        "stats evaluation player recording playback rate selector",
        errors,
    )
    require_contains(
        plugin_source,
        'const bool recordingPlaybackRateOpen = ImGui::BeginCombo(\n      "##recording-playback-rate",',
        "plugin recording playback rate uses hidden-label web-like selector",
        errors,
    )
    require_contains(
        web_player_main_source,
        "elements.playbackRate.disabled = isRecording;",
        "stats evaluation player disables recording playback rate while recording",
        errors,
    )
    require_contains(
        plugin_source,
        "const bool recordingPlaybackRateDisabled = recordingSettingsLocked;\n"
        "  pushRecordingDisabledStyle(recordingPlaybackRateDisabled);\n"
        "  ImGui::SetNextItemWidth(recordingControlWidth);\n"
        "  const bool recordingPlaybackRateOpen = ImGui::BeginCombo(",
        "plugin recording playback rate selector is disabled and grid-sized while recording",
        errors,
    )
    require_contains(
        plugin_source,
        "const float recordingControlWidth =\n"
        "      std::max(96.0f, (ImGui::GetContentRegionAvail().x - recordingControlGap) * 0.5f);",
        "plugin recording controls use two-column grid sizing",
        errors,
    )
    require_contains(
        plugin_source,
        "const float recordingPrimaryButtonWidth =\n"
        "      std::max(68.0f, (recordingPrimaryRowWidth - recordingControlGap * 2.0f) / 3.0f);",
        "plugin recording primary transport row distributes three buttons",
        errors,
    )
    require_contains(
        plugin_source,
        "const float recordingSecondaryButtonWidth =\n"
        "      std::max(88.0f, (recordingSecondaryRowWidth - recordingControlGap) * 0.5f);",
        "plugin recording secondary transport row distributes two buttons",
        errors,
    )
    require_contains(
        plugin_source,
        "if (recordingPlaybackRateDisabled) {\n"
        "      ImGui::CloseCurrentPopup();\n"
        "      ImGui::EndCombo();",
        "plugin recording playback rate disabled selector cannot remain open",
        errors,
    )
    require_contains(
        web_player_template_source,
        '<button id="recording-download" type="button" disabled>Download</button>',
        "stats evaluation player recording download action",
        errors,
    )
    require_contains(
        web_player_main_source,
        "elements.start.disabled = !hasRecorder || isRecording;",
        "stats evaluation player recording start waits for recorder and idle state",
        errors,
    )
    require_contains(
        plugin_source,
        'recordingButton(\n          "Start",\n          recordingActive || !loaded || !engine,',
        "plugin recording start waits for analysis engine and idle state",
        errors,
    )
    require_contains(
        plugin_source,
        'recordingButton(\n          "Download",\n          recordingActive || !hasGraphSnapshot,',
        "plugin recording exposes web-like download action",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::Columns(2, "recording-detail-grid", false);',
        "plugin recording exposes web-like detail grid",
        errors,
    )
    require_contains(
        web_player_main_source,
        'case "recording":\n      return "Recording";\n    case "stopping":\n      return "Stopping";\n    case "ready":\n      return "Ready";',
        "stats evaluation player recording status uses user-facing state labels",
        errors,
    )
    require_contains(
        plugin_source,
        'const std::string recordingStatusReadout =\n      recordingActive   ? "Recording"\n      : hasGraphSnapshot ? "Ready"\n      : !loaded || !engine ? "No replay"\n                           : recordingStatus;',
        "plugin recording status readout uses web-like state labels",
        errors,
    )
    require_contains(
        plugin_source,
        'renderWebDetailGridCell("Status", recordingStatusReadout);',
        "plugin recording status renders the web-like detail grid readout",
        errors,
    )
    require_contains(
        web_player_main_source,
        'if (bytes <= 0) {\n    return "--";\n  }\n  const units = ["B", "KB", "MB", "GB"];',
        "stats evaluation player recording size formatter uses dash and decimal units",
        errors,
    )
    require_contains(
        plugin_source,
        'if (bytes == 0) {\n    return "--";\n  }\n\n  constexpr std::array<const char *, 4> units{{"B", "KB", "MB", "GB"}};',
        "plugin recording size formatter mirrors web dash and decimal units",
        errors,
    )
    for plugin_only_recording_surface in (
        'ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "RECORDING");',
        'ImGui::Checkbox("Finalize before dump"',
        "recordingFinishBeforeDump",
        '"recording_finish_before_dump"',
        'ImGui::Button("Snapshot")',
        'ImGui::Button("Log folder")',
        'ImGui::Text("%s", recordingStatus.c_str());',
        'return std::format("{} B", bytes);',
        'KiB',
        'MiB',
    ):
        reject_contains(
            plugin_source,
            plugin_only_recording_surface,
            "plugin recording window plugin-only surface",
            errors,
        )
    require_contains(
        web_player_template_source,
        '<span class="label">Camera profile</span>',
        "stats evaluation player camera profile label",
        errors,
    )
    require_contains(
        web_player_main_source,
        "const DEFAULT_CAMERA_DISTANCE_SCALE = 2.25;",
        "stats evaluation player default camera distance scale",
        errors,
    )
    require_contains(
        plugin_header,
        "float cameraDistanceScale = 2.25f;",
        "plugin default camera distance scale mirrors stats evaluation player",
        errors,
    )
    require_contains(
        plugin_source,
        'parseJsonNumberProperty(json, "camera_distance_scale").value_or(cameraDistanceScale)',
        "legacy camera distance config preserves plugin/web default when unset",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::TextDisabled("Camera profile");\n  ImGui::SetNextItemWidth(ImGui::GetContentRegionAvail().x);\n  const bool profileDisabled = !hasCameraContext;\n  pushCameraDisabledStyle(profileDisabled);\n  const bool profileOpen = ImGui::BeginCombo("##attached-player", selectedLabel.c_str());',
        "plugin camera profile selector mirrors web label and hidden control id",
        errors,
    )
    require_contains(
        web_player_main_source,
        "attachedPlayer.disabled = !enabled;",
        "stats evaluation player disables attached player selector with transport",
        errors,
    )
    require_contains(
        web_player_main_source,
        'button.disabled = !hasReplay || (mode === "follow" && !canFollow);',
        "stats evaluation player disables unavailable camera modes",
        errors,
    )
    require_contains(
        web_player_main_source,
        "cameraViewOverheadButton.disabled = !hasReplay;",
        "stats evaluation player disables overhead camera without replay",
        errors,
    )
    require_contains(
        web_player_main_source,
        "cameraViewSideButton.disabled = !hasReplay;",
        "stats evaluation player disables side camera without replay",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".camera-presets {\n"
        "  display: grid;\n"
        "  grid-template-columns: repeat(2, minmax(0, 1fr));\n"
        "  gap: var(--ui-gap-xs);\n"
        "}",
        "stats evaluation player camera presets use a two-column grid",
        errors,
    )
    require_contains(
        web_player_styles_source,
        '.camera-presets button[data-active="true"] {\n'
        "  border-color: rgba(142, 197, 255, 0.42);\n"
        "  background: linear-gradient(180deg, rgba(33, 71, 107, 0.96), rgba(12, 27, 42, 0.98));",
        "stats evaluation player active camera preset has accented button chrome",
        errors,
    )
    require_contains(
        web_player_main_source,
        "cameraDistance.disabled = !hasAttachedCamera;",
        "stats evaluation player updates camera settings availability from active camera state",
        errors,
    )
    require_contains(
        plugin_source,
        "const bool hasCameraContext = !sampledPlayers.empty();",
        "plugin camera derives replay context for disabled controls",
        errors,
    )
    require_contains(
        plugin_source,
        "const float cameraPresetWidth =\n"
        "      std::max(96.0f, (ImGui::GetContentRegionAvail().x - cameraPresetGap) * 0.5f);",
        "plugin camera presets use two-column grid sizing",
        errors,
    )
    require_contains(
        plugin_source,
        "const bool active = cameraViewMode == mode;",
        "plugin camera presets track the active button state",
        errors,
    )
    require_contains(
        plugin_source,
        "ImGui::PushStyleColor(ImGuiCol_Border, ImVec4{0.56f, 0.77f, 1.0f, 0.42f});\n"
        "      ImGui::PushStyleColor(ImGuiCol_Button, ImVec4{0.13f, 0.28f, 0.42f, 0.96f});",
        "plugin camera active preset uses web-like accented button chrome",
        errors,
    )
    require_contains(
        plugin_source,
        "const bool clicked = ImGui::Button(label, ImVec2{cameraPresetWidth, 0.0f});",
        "plugin camera preset controls use equal-width buttons",
        errors,
    )
    require_contains(
        plugin_source,
        "ImGui::SameLine(0.0f, cameraPresetGap);",
        "plugin camera preset grid uses explicit web-like gaps",
        errors,
    )
    require_contains(
        plugin_source,
        'cameraViewButton("Follow##camera-view", 1, !hasCameraContext || selectedPlayer == nullptr)',
        "plugin disables follow camera without replay player context",
        errors,
    )
    require_contains(
        plugin_source,
        'cameraViewButton("Overhead##camera-view", 2, !hasCameraContext)',
        "plugin disables preset camera modes without replay context",
        errors,
    )
    require_contains(
        plugin_source,
        'cameraViewButton("Diagonal##camera-view", 3, !hasCameraContext)',
        "plugin disables side camera mode without replay context",
        errors,
    )
    require_contains(
        plugin_source,
        "targetPlayer = cameraViewMode == 1 ? sampledPlayerByIndex(cameraSelectedPlayerIndex) : nullptr;\n  const std::string activeCameraLabel =\n      targetPlayer == nullptr",
        "plugin camera settings availability uses updated camera mode",
        errors,
    )
    require_contains(
        web_player_template_source,
        '<dl class="detail-grid">',
        "stats evaluation player camera detail grid",
        errors,
    )
    require_contains(
        web_player_main_source,
        'this.renderEmptyProfile("Free camera");',
        "stats evaluation player camera detail grid uses dash readouts without an attached camera",
        errors,
    )
    require_contains(
        web_player_main_source,
        'elements.cameraFovReadout.textContent = "--";',
        "stats evaluation player camera empty profile uses dash FOV readout",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::Columns(2, "camera-detail-grid", false);',
        "plugin camera uses web-like detail grid",
        errors,
    )
    require_contains(
        plugin_source,
        'auto attachedCameraMetric = [&](int precision, float value) {\n'
        "    if (!hasAttachedCamera) {\n"
        '      return std::string{"--"};\n'
        "    }\n",
        "plugin camera detail grid uses dash readouts without an attached camera",
        errors,
    )
    require_contains(
        web_player_template_source,
        "<span>Custom camera settings</span>",
        "stats evaluation player camera custom settings toggle",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::Checkbox("Custom camera settings", &nextCustomSettingsEnabled)',
        "plugin camera custom settings toggle mirrors web label",
        errors,
    )
    require_contains(
        web_player_template_source,
        "<span>Swivel</span>",
        "stats evaluation player camera swivel label",
        errors,
    )
    require_contains(
        plugin_source,
        'renderCustomSlider("Swivel", cameraCustomSwivelSpeed, 1.0f, 10.0f, "%.1f");',
        "plugin camera swivel label mirrors web",
        errors,
    )
    require_contains(
        web_player_template_source,
        "<span>Transition</span>",
        "stats evaluation player camera transition label",
        errors,
    )
    require_contains(
        plugin_source,
        'renderCustomSlider(\n        "Transition",\n        cameraCustomTransitionSpeed,\n        0.5f,\n        2.0f,\n        "%.2f");',
        "plugin camera transition label mirrors web",
        errors,
    )
    require_contains(
        web_player_template_source,
        '<input id="ball-cam" type="checkbox" disabled />',
        "stats evaluation player camera ball-cam toggle",
        errors,
    )
    require_contains(
        plugin_source,
        'renderCustomSlider(\n        "Transition",\n        cameraCustomTransitionSpeed,\n        0.5f,\n        2.0f,\n        "%.2f");\n  }\n\n  bool nextBallCamEnabled = cameraBallCamEnabled;\n  pushCameraDisabledStyle(!hasAttachedCamera);\n  const bool ballCamChanged = ImGui::Checkbox("Ball cam", &nextBallCamEnabled);',
        "plugin camera ball-cam toggle follows custom settings controls like web",
        errors,
    )
    for plugin_only_camera_surface in (
        'ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "CAMERA PROFILE");',
        'ImGui::BeginCombo("Target", selectedLabel.c_str())',
        'ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "READOUT");',
        'ImGui::Button("Open player stats")',
        'ImGui::Checkbox("Custom settings", &nextCustomSettingsEnabled)',
        'renderCustomSlider("Swivel speed", cameraCustomSwivelSpeed',
        'renderCustomSlider(\n        "Transition speed",',
        'ImGui::Text("%.0f", fov);',
        'ImGui::Text("%.0f", height);',
        'ImGui::Text("%.1f", pitch);',
        'ImGui::Text("%.0f", distance);',
        'ImGui::Text("%.2f", stiffness);',
        "ImGui::RadioButton(label,",
        "ImGui::SameLine();\n  if (cameraViewButton(\"Follow##camera-view\"",
    ):
        reject_contains(
            plugin_source,
            plugin_only_camera_surface,
            "plugin camera window plugin-only surface",
            errors,
        )
    for boost_filter_label in ("Pad type", "Activity", "Field half", "Player"):
        web_boost_label_needles = {
            "Pad type": 'createFilterGroup("Pad type",',
            "Activity": '"Activity",\n            BOOST_PICKUP_ACTIVITY_OPTIONS',
            "Field half": '"Field half",\n            BOOST_PICKUP_FIELD_HALF_OPTIONS',
            "Player": 'groupTitle.textContent = "Player";',
        }
        require_contains(
            web_player_boost_pickup_filters_source,
            web_boost_label_needles[boost_filter_label],
            f"stats evaluation player boost pickup filter label {boost_filter_label}",
            errors,
        )
        require_contains(
            plugin_source,
            f'renderBoostFilterGroupTitle("{boost_filter_label}");',
            f"plugin boost pickup filter label {boost_filter_label}",
            errors,
        )
    require_contains(
        web_player_boost_pickup_filters_source,
        'settingsEl.className = "boost-pickup-filter-panel";',
        "stats evaluation player boost pickup filters render a settings panel",
        errors,
    )
    require_contains(
        web_player_boost_pickup_filters_source,
        'header.className = "boost-pickup-filter-summary";',
        "stats evaluation player boost pickup filters render a summary row",
        errors,
    )
    require_contains(
        web_player_boost_pickup_filters_source,
        'grid.className = "boost-pickup-filter-grid";',
        "stats evaluation player boost pickup filters render a grid",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".boost-pickup-filter-grid {\n"
        "  display: grid;\n"
        "  grid-template-columns: repeat(2, minmax(0, 1fr));\n"
        "  gap: 0.75rem 1rem;\n"
        "}",
        "stats evaluation player boost pickup filter grid is two columns",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::Columns(2, "boost-pickup-filter-grid", false);',
        "plugin boost pickup filter groups use web-like two-column grid",
        errors,
    )
    require_contains(
        plugin_source,
        'const std::string pickupReadout =\n      pickupsHidden ? "Hidden"\n                    : constrainedGroups == 0 ? "All labels"',
        "plugin boost pickup filters render web-like summary readout",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::SetCursorPosX(std::max(\n      ImGui::GetCursorPosX(),\n      ImGui::GetWindowContentRegionMax().x - pickupReadoutWidth));',
        "plugin boost pickup summary is right-aligned like web",
        errors,
    )
    require_contains(
        plugin_source,
        "auto renderBoostFilterCheckbox = [&](const char *label, bool &value, bool sameLine)",
        "plugin boost pickup filter options share web-like option renderer",
        errors,
    )
    require_contains(
        web_player_boost_pickup_filters_source,
        'optionText.textContent = `${player.name} (${player.isTeamZero ? "Blue" : "Orange"})`;',
        "stats evaluation player boost pickup player labels include team",
        errors,
    )
    require_contains(
        plugin_source,
        'const std::string label = std::format(\n          "{} ({})##boost-pickup-player-{}",',
        "plugin boost pickup player labels include team like web",
        errors,
    )
    reject_contains(
        plugin_source,
        'std::format(\n          "{}##boost-pickup-player-{}",\n          playerLabel(player.player_index, player.is_team_0),',
        "plugin boost pickup player labels use generic player label without team suffix",
        errors,
    )
    for plugin_only_boost_surface in (
        'ImGui::Text("Pickup labels: %s", pickupReadout.c_str());',
        'ImGui::Text("Known pads: %zu", boostPadIds.size());',
        'ImGui::Text("Pending pad events: %zu", pendingBoostPadEvents.size());',
        'ImGui::Text("Recent boost pickups: %d", recentEventCountForType("boost_pickup"));',
        'ImGui::Button("Show boost pickups")',
        'ImGui::Button("Open boost stats")',
        'ImGui::Button("Inspect boost nodes")',
        'ImGui::Button("Boost output")',
        'ImGui::Button("All filters")',
        'ImGui::Button("Hide pickups")',
        'ImGui::Button("All players")',
        'ImGui::Button("No players")',
        'ImGui::Separator();\n  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "Activity");',
        'ImGui::Separator();\n  ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "Field half");',
    ):
        reject_contains(
            plugin_source,
            plugin_only_boost_surface,
            "plugin boost pickup filters window plugin-only surface",
            errors,
        )
    for touch_settings_label in (
        'eyebrow.textContent = "Touch markers";',
        'title.textContent = "Touch decay";',
        'labelText.textContent = "Keep each marker visible after the touch";',
        'modeEyebrow.textContent = "Overlay";',
        'modeTitle.textContent = "Touch mode";',
        'breakdownEyebrow.textContent = "Stat display";',
        'breakdownTitle.textContent = "Touch breakdown";',
    ):
        require_contains(
            web_player_main_source,
            touch_settings_label,
            f"stats evaluation player touch controls setting {touch_settings_label}",
            errors,
        )
    require_contains(
        web_player_styles_source,
        ".module-settings-card {\n"
        "  display: grid;\n"
        "  gap: 0.75rem;\n"
        "  padding: 0.85rem 0.9rem;\n"
        "  border-radius: 1rem;\n"
        "  border: 1px solid rgba(255, 255, 255, 0.08);\n"
        "  background: rgba(255, 255, 255, 0.035);",
        "stats evaluation player touch controls use module settings card chrome",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".module-settings-subgroup {\n"
        "  display: grid;\n"
        "  gap: 0.65rem;\n"
        "  padding-top: 0.15rem;\n"
        "  border-top: 1px solid rgba(255, 255, 255, 0.06);",
        "stats evaluation player touch controls use settings subgroups",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".module-settings-header {\n"
        "  display: flex;\n"
        "  align-items: start;\n"
        "  justify-content: space-between;",
        "stats evaluation player touch controls use header/readout rows",
        errors,
    )
    for plugin_touch_surface in (
        'ImGui::BeginChild("touch-settings-card", ImVec2{0.0f, 0.0f}, true);',
        'auto renderTouchSettingsHeader = [](const char *eyebrow,\n                                      const char *title,\n                                      const std::string &readout)',
        'renderTouchSettingsHeader(\n      "Touch markers",\n      "Touch decay",\n      std::format("{:.1f}s", touchMarkerDecaySeconds));',
        'ImGui::TextDisabled("Keep each marker visible after the touch");',
        '"##touch-marker-decay-seconds", &touchMarkerDecaySeconds, 1.0f, 10.0f, "%.1fs"',
        'renderTouchSettingsHeader(\n      "Overlay",\n      "Touch mode",\n      touchControlsMode == 1 ? "Advancement" : "Markers");',
        'renderTouchSettingsHeader("Stat display", "Touch breakdown", touchBreakdownReadout());',
    ):
        require_contains(
            plugin_source,
            plugin_touch_surface,
            "plugin touch controls mirror stats evaluation player settings card",
            errors,
        )
    for plugin_only_touch_surface in (
        'ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "LIVE TOUCH STATE");',
        'ImGui::Text("Pending touches: %zu", pendingTouches.size());',
        'ImGui::Text("Pending dodge refreshes: %zu", pendingDodgeRefreshes.size());',
        'ImGui::Text("Recent touch events: %d", recentEventCountForType("touch"));',
        'ImGui::Button("Show touches")',
        'ImGui::Button("Show movement")',
        'ImGui::Button("Open touch stats")',
        'ImGui::Button("Inspect touch nodes")',
        '"Marker decay seconds", &touchMarkerDecaySeconds',
        'ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "Touch markers");',
        'ImGui::Text("Touch decay");',
        'ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "Overlay");',
        'ImGui::Text("Touch mode");',
        'ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "Stat display");',
        'ImGui::Text("Touch breakdown");',
    ):
        reject_contains(
            plugin_source,
            plugin_only_touch_surface,
            "plugin touch controls window plugin-only surface",
            errors,
        )
    reject_contains(
        plugin_source,
        '"Load Replay...", ImVec2{actionButtonWidth, 0.0f}',
        "launcher exposes web replay-file loading action in-game",
        errors,
    )
    reject_contains(
        plugin_source,
        "showSingletonWindow(uiReplayLoadingOpen, replayLoadingPlacement);\n"
        "    resetReplayAnnotations();\n"
        "    tickReplayAnnotations();\n"
        "    hideLauncherWindow();",
        "launcher opens replay-loading queue surface",
        errors,
    )
    reject_contains(
        plugin_source,
        'std::format("{}   {}", window.label, isOpen ? "Hide" : "Show")',
        "plugin launcher web window toggles append show/hide state text",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::TextUnformatted("Open a replay in Rocket League to start.");\n'
        '  if (gameWrapper->IsInReplay() && ImGui::Button("Refresh current replay", ImVec2{190.0f, 0.0f})) {\n'
        "    resetReplayAnnotations();\n"
        "    tickReplayAnnotations();\n"
        "  }\n\n"
        "  ImGui::End();",
        "plugin empty state uses current Rocket League replay instead of a replay queue",
        errors,
    )
    for plugin_only_empty_state_surface in (
        'ImGui::Button("Start live analysis", ImVec2{150.0f, 0.0f})',
        'ImGui::Button("Open menu", ImVec2{150.0f, 0.0f})',
        'ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "subtr-actor");',
    ):
        reject_contains(
            plugin_source,
            plugin_only_empty_state_surface,
            "plugin empty state plugin-only surface",
            errors,
        )
    require_contains(
        web_player_main_source,
        'renderModuleSummaryGroup("Timeline markers", markerToggles)',
        "stats evaluation player launcher module summary timeline marker group",
        errors,
    )
    require_contains(
        web_player_main_source,
        'renderModuleSummaryGroup("Timeline ranges", rangeToggles)',
        "stats evaluation player launcher module summary timeline range group",
        errors,
    )
    require_contains(
        web_player_main_source,
        'renderModuleSummaryGroup("In-game visualizations", inGameVisualizationToggles)',
        "stats evaluation player launcher module summary in-game group",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".module-summary-item {\n"
        "  appearance: none;\n"
        "  display: inline-flex;\n"
        "  align-items: center;\n"
        "  justify-content: space-between;\n"
        "  gap: var(--ui-gap-sm);\n"
        "  min-height: var(--ui-control-height);\n"
        "  padding: var(--ui-control-padding-block) var(--ui-control-padding-inline);\n"
        "  border-radius: var(--ui-radius-pill);\n"
        "  border: 1px solid var(--ui-border-subtle);\n"
        "  background: rgba(255, 255, 255, 0.03);",
        "stats evaluation player module summary items use muted pill chrome",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".module-summary-item[data-active=\"true\"] {\n"
        "  border-color: rgba(75, 148, 255, 0.22);\n"
        "  background: rgba(75, 148, 255, 0.08);\n"
        "  color: #dceafb;\n"
        "}",
        "stats evaluation player active module summary items use blue pill chrome",
        errors,
    )
    require_contains(
        plugin_source,
        "ImGui::PushStyleVar(ImGuiStyleVar_FramePadding, ImVec2{10.0f, 5.0f});\n"
        "  ImGui::PushStyleVar(ImGuiStyleVar_FrameRounding, 999.0f);\n"
        "  ImGui::PushStyleVar(ImGuiStyleVar_FrameBorderSize, 1.0f);",
        "plugin module summary toggles use web-like pill geometry",
        errors,
    )
    require_contains(
        plugin_source,
        "active ? ImVec4{0.29f, 0.58f, 1.0f, 0.08f} : ImVec4{1.0f, 1.0f, 1.0f, 0.03f}",
        "plugin module summary toggles mirror web active/inactive backgrounds",
        errors,
    )
    require_contains(
        plugin_source,
        "active ? ImVec4{0.29f, 0.58f, 1.0f, 0.22f} : ImVec4{1.0f, 1.0f, 1.0f, 0.10f}",
        "plugin module summary toggles mirror web active/inactive borders",
        errors,
    )
    require_contains(
        plugin_source,
        "ImGui::PopStyleColor(5);\n  ImGui::PopStyleVar(3);",
        "plugin module summary toggles restore web-like style stack",
        errors,
    )
    reject_contains(
        plugin_source,
        "ImVec4{0.16f, 0.35f, 0.28f, 1.0f}",
        "plugin module summary toggles use old green active fill",
        errors,
    )
    require_contains(
        plugin_header,
        "bool includePluginControls = true",
        "plugin module summary can hide plugin-only controls for web-like launcher",
        errors,
    )
    require_contains(
        plugin_source,
        'renderModuleSummaryControls("launcher-module-summary", false, 0.0f, false);',
        "launcher module summary excludes plugin-only controls like the web player",
        errors,
    )
    require_contains(
        plugin_source,
        'auto renderTimelineControls = [&]() {\n'
        '    renderEventFilterModuleSummaryToggle("Backboard", "backboard", idSuffix, toggleWidth);\n'
        "    renderBoolModuleSummaryToggle(\n"
        '        "Possession",\n'
        "        timelineRangePossessionEnabled,\n"
        "        idSuffix,\n"
        "        toggleWidth);\n"
        '    renderEventFilterModuleSummaryToggle("50/50", "fifty_fifty", idSuffix, toggleWidth);',
        "plugin launcher module summary starts with web timeline capability order",
        errors,
    )
    require_contains(
        plugin_source,
        '    renderEventFilterModuleSummaryToggle("Whiff", "whiff", idSuffix, toggleWidth);\n'
        "    renderBoolModuleSummaryToggle(\n"
        '        "Boost pickup timeline",\n'
        "        timelineRangeBoostEnabled,\n"
        "        idSuffix,\n"
        "        toggleWidth);\n"
        '    renderEventFilterModuleSummaryToggle("Powerslide", "powerslide", idSuffix, toggleWidth);\n'
        '    renderEventFilterModuleSummaryToggle("Bump", "bump", idSuffix, toggleWidth);\n'
        "    if (includePluginControls) {\n"
        '      renderEventFilterModuleSummaryToggle("Dodge refresh", "dodge_reset", idSuffix, toggleWidth);',
        "plugin-only timeline shortcuts are outside the web-like launcher summary",
        errors,
    )
    reject_contains(
        plugin_source,
        'auto renderTimelineControls = [&]() {\n'
        '    renderEventFilterModuleSummaryToggle("Touch", "touch", idSuffix, toggleWidth);\n'
        '    renderEventFilterModuleSummaryToggle("Dodge refresh", "dodge_reset", idSuffix, toggleWidth);',
        "launcher module summary starts with plugin-only timeline shortcuts",
        errors,
    )
    require_contains(
        web_player_main_source,
        'createStatsWindow(button.dataset.createStatsWindow as StatsWindowKind);',
        "stats evaluation player launcher creates stats windows",
        errors,
    )
    require_contains(
        web_player_main_source,
        "this.options.setLauncherOpen(false);",
        "stats evaluation player closes launcher after stats window creation",
        errors,
    )
    require_contains(
        plugin_source,
        'renderStatsWindowCreationControls("launcher-stats-windows", true, false, false, true);',
        "plugin launcher closes after stats window creation like the web player",
        errors,
    )
    reject_contains(
        plugin_source,
        'renderStatsWindowCreationControls("launcher-stats-windows", false, false, false, true);',
        "plugin launcher keeps menu open after stats window creation",
        errors,
    )
    require_contains(
        web_player_main_source,
        "if (panels.length === 0) {\n      settings.hidden = true;",
        "stats evaluation player hides empty launcher module settings",
        errors,
    )
    require_contains(
        web_player_main_source,
        "const panels = this.options\n"
        "      .getActiveModules()\n"
        '      .filter((mod) => mod.id !== "boost" && mod.id !== TOUCH_MODULE_ID)\n'
        "      .map((mod) => mod.renderSettings?.(ctx) ?? null)",
        "stats evaluation player module settings are active-module panels",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".module-settings-card {\n"
        "  display: grid;\n"
        "  gap: 0.75rem;\n"
        "  padding: 0.85rem 0.9rem;\n"
        "  border-radius: 1rem;\n"
        "  border: 1px solid rgba(255, 255, 255, 0.08);\n"
        "  background: rgba(255, 255, 255, 0.035);",
        "stats evaluation player module settings render card panels",
        errors,
    )
    require_contains(
        plugin_source,
        "if (timelineRangePossessionEnabled) {\n    ImGui::Separator();\n    renderModuleSettingsControls(\"launcher-module-settings\", false, true, true);\n  }",
        "launcher module settings only render active web panels",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::BeginChild(cardId, ImVec2{0.0f, height}, true);',
        "plugin launcher module settings render web-like card panels",
        errors,
    )
    require_contains(
        plugin_source,
        'const bool possessionCard = beginModuleSettingsCard(\n'
        '        "module-settings-card-possession",\n'
        "        includeOpenButtons ? 116.0f : 84.0f);",
        "plugin active possession settings use module-settings card",
        errors,
    )
    require_contains(
        plugin_source,
        "ImGui::SameLine(std::max(\n"
        "          ImGui::GetCursorPosX(),\n"
        "          ImGui::GetWindowContentRegionMax().x - ImGui::CalcTextSize(readout.c_str()).x));",
        "plugin module settings card aligns readout like web header",
        errors,
    )
    require_contains(
        plugin_source,
        'renderModuleSummaryControls("module-controls-summary");',
        "dedicated module controls keep plugin-only module controls",
        errors,
    )
    require_contains(
        plugin_source,
        'renderModuleSettingsControls("module-controls-settings", true);',
        "dedicated module controls keep full module settings",
        errors,
    )
    reject_contains(
        plugin_source,
        'renderModuleSummaryControls("launcher-module-summary", false, 0.0f);',
        "launcher module summary includes plugin-only controls",
        errors,
    )
    reject_contains(
        plugin_source,
        'renderModuleSettingsControls("launcher-module-settings", false, true);',
        "launcher module settings render unconditionally",
        errors,
    )
    require_contains(
        web_player_main_source,
        'allName.textContent = "All events";',
        "stats evaluation player Events window all-events action",
        errors,
    )
    require_contains(
        plugin_source,
        'std::format("All events   {}##event-sources-actions-all", displaySources.size())',
        "plugin Events window all-events action mirrors web count readout",
        errors,
    )
    require_contains(
        web_player_main_source,
        "for (const source of sources) {\n        source.setActive(true);\n      }",
        "stats evaluation player Events window all action enables visible sources",
        errors,
    )
    require_contains(
        plugin_source,
        "selected.clear();\n"
        "    selected.reserve(displaySources.size());\n"
        "    for (const DisplaySource &source : displaySources) {\n"
        "      selected.emplace_back(source.option->value);\n"
        "    }\n"
        "    applySelection();",
        "plugin Events window all action enables displayed sources like web",
        errors,
    )
    require_contains(
        web_player_main_source,
        'noneName.textContent = "No events";',
        "stats evaluation player Events window no-events action",
        errors,
    )
    require_contains(
        plugin_source,
        'renderEventSourceAction("No events   Off##event-sources-actions-none")',
        "plugin Events window no-events action mirrors web off readout",
        errors,
    )
    require_contains(
        web_player_main_source,
        'item.className = "module-summary-item";',
        "stats evaluation player Events window source rows are module summary items",
        errors,
    )
    require_contains(
        web_player_main_source,
        'state.textContent = `${source.active ? "On" : "Off"} ${source.count}`;',
        "stats evaluation player Events window source rows expose state count readout",
        errors,
    )
    require_contains(
        web_player_main_source,
        "return sources.sort((left, right) => left.label.localeCompare(right.label));",
        "stats evaluation player Events window sorts source rows by label",
        errors,
    )
    require_contains(
        plugin_source,
        'std::format(\n        "{}   {} {}##event-sources-{}",\n        option.label,\n        enabled ? "On" : "Off",\n        count,\n        option.value)',
        "plugin Events window source rows expose web-like state count readout",
        errors,
    )
    require_contains(
        plugin_source,
        "std::sort(\n      displaySources.begin(),\n      displaySources.end(),\n      [](const DisplaySource &left, const DisplaySource &right) {\n        return std::string_view{left.option->label} < std::string_view{right.option->label};\n      });",
        "plugin Events window sorts source rows by label like web",
        errors,
    )
    require_contains(
        web_player_main_source,
        'empty.textContent = "No events loaded.";',
        "stats evaluation player Events window empty state",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::TextDisabled("No events loaded.");',
        "plugin Events window empty state mirrors web",
        errors,
    )
    for plugin_only_events_surface in (
        'renderEventFilterCombo("Filter");',
        'ImGui::Button("Clear")',
        'ImGui::Text("%zu visible / %zu recent", visibleCount, recentUiEvents.size());',
        'ImGui::Columns(4, "event-columns", true);',
        'ImGui::TreeNode("Event sources##event-source-controls")',
        'ImGui::SmallButton("All events##event-sources")',
        'ImGui::SmallButton("No events##event-sources")',
        'renderModuleSummaryToggle(\n          "All events",\n          allSelected,\n          "event-sources-actions")',
        'renderModuleSummaryToggle(\n          "No events",\n          selected.empty(),\n          "event-sources-actions")',
        'renderModuleSummaryToggle(label.c_str(), enabled, "event-sources")',
    ):
        reject_contains(
            plugin_source,
            plugin_only_events_surface,
            "plugin Events window plugin-only log/table surface",
            errors,
        )
    require_contains(
        web_player_main_source,
        "summary.textContent = `Filters ${selectedSourceIds.size}/${sources.length}`;",
        "stats evaluation player event playlist filter count label",
        errors,
    )
    require_contains(
        web_player_main_source,
        'const DEFAULT_UNSELECTED_EVENT_PLAYLIST_SOURCE_IDS = new Set(["module:touch", "module:powerslide"]);',
        "stats evaluation player defaults touch and powerslide out of playlist",
        errors,
    )
    require_contains(
        web_player_main_source,
        "return [...replaySources, ...timelineSources];",
        "stats evaluation player event playlist keeps replay goal source before event sources",
        errors,
    )
    require_contains(
        plugin_source,
        'eventPlaylistSourceFilter = "default";',
        "plugin replay review playlist starts from web-like default source selection",
        errors,
    )
    require_contains(
        plugin_source,
        'return optionValue != "all" && optionValue != "mechanics" && optionValue != "touch" &&\n'
        '         optionValue != "powerslide";',
        "plugin event playlist default excludes broad mechanics, touch, and powerslide",
        errors,
    )
    require_contains(
        plugin_source,
        '{"goal", "Goals", "Replay"}',
        "plugin event playlist goal source label mirrors web",
        errors,
    )
    require_contains(
        plugin_source,
        '{"demo", "Demos", "Replay"}',
        "plugin event playlist demo source label mirrors web",
        errors,
    )
    require_contains(
        plugin_source,
        'if (group == "Replay" && std::string_view{option.value} == "goal") {\n'
        "      return std::tuple{groupRank, 0, std::string_view{option.label}};\n"
        "    }",
        "plugin event playlist keeps replay goal source before other event sources",
        errors,
    )
    require_contains(
        plugin_source,
        'isCorePlayerStat ? "core" : "mechanics"',
        "plugin replay/live drained shot-save-assist events use core playlist source",
        errors,
    )
    require_contains(
        plugin_source,
        'if (isCorePlayerStat) {\n    return;\n  }\n  if (!overlayMechanicEnabled(event.kind))',
        "plugin core replay annotations do not route through mechanic overlay filters",
        errors,
    )
    require_contains(
        web_player_main_source,
        "if (activeKey === this.lastActiveKey && !options.forceScroll) {\n"
        "      return;\n"
        "    }",
        "stats evaluation player event playlist avoids repeated auto-follow scroll",
        errors,
    )
    require_contains(
        plugin_source,
        "active && eventPlaylistAutoFollow && activeEventKey != eventPlaylistLastActiveKey",
        "plugin event playlist avoids repeated auto-follow scroll",
        errors,
    )
    require_contains(
        plugin_source,
        "eventPlaylistLastActiveKey = activeEventKey;",
        "plugin event playlist remembers active auto-follow item",
        errors,
    )
    require_contains(
        plugin_source,
        'std::format(\n'
        '      "Filters {}/{}##event-playlist-filter",\n'
        "      selectedSourceCount,\n"
        "      playlistSources.size())",
        "plugin event playlist filter disclosure mirrors web summary",
        errors,
    )
    require_contains(
        plugin_source,
        "const bool filtersOpen = ImGui::TreeNode(filterSummary.c_str());",
        "plugin event playlist filter panel is collapsed like web details",
        errors,
    )
    reject_contains(
        plugin_source,
        'ImGui::Text("Filters %zu/%zu", selectedSourceCount, playlistSources.size());',
        "plugin event playlist renders filter summary as static text",
        errors,
    )
    require_contains(
        web_player_main_source,
        'empty.textContent = this.options.getReplayPlayer()\n'
        '        ? "No events loaded."\n'
        '        : "Load a replay to see events.";',
        "stats evaluation player event playlist empty state distinguishes no replay",
        errors,
    )
    require_contains(
        plugin_source,
        'recentUiEvents.empty() && replayAnnotations == nullptr ? "Load a replay to see events."\n'
        '                                                               : "No events loaded."',
        "plugin event playlist empty state distinguishes no replay",
        errors,
    )
    require_contains(
        web_player_main_source,
        'if (selectedSourceIds.size === 0) {\n'
        '        empty.textContent = "No event types selected.";\n'
        "      } else {\n"
        '        empty.textContent = "No events in selected event types.";\n'
        "      }",
        "stats evaluation player event playlist no-selection empty state",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::TextDisabled("No event types selected.");',
        "plugin event playlist no-selection empty state mirrors web",
        errors,
    )
    require_contains(
        web_player_main_source,
        'allButton.textContent = "All";',
        "stats evaluation player event playlist all action label",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::Button("All##event-playlist-sources-all")',
        "plugin event playlist all action label mirrors web",
        errors,
    )
    require_contains(
        web_player_main_source,
        "this.activeSourceIds = new Set(sources.map((source) => source.id));",
        "stats evaluation player event playlist all action enables current sources",
        errors,
    )
    require_contains(
        plugin_source,
        "selectedSources.clear();\n"
        "      selectedSources.reserve(playlistSources.size());\n"
        "      for (const PlaylistSource &source : playlistSources) {\n"
        "        selectedSources.emplace_back(source.option->value);\n"
        "      }\n"
        "      eventPlaylistSourceFilter = eventFilterFromSelectedSources(selectedSources);",
        "plugin event playlist all action enables displayed sources like web",
        errors,
    )
    reject_contains(
        plugin_source,
        'eventPlaylistSourceFilter = "all";',
        "plugin event playlist all action enables hidden global sources",
        errors,
    )
    require_contains(
        web_player_main_source,
        'noneButton.textContent = "None";',
        "stats evaluation player event playlist none action label",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::Button("None##event-playlist-sources-none")',
        "plugin event playlist none action label mirrors web",
        errors,
    )
    require_contains(
        web_player_main_source,
        'label.className = "toggle event-playlist-filter-option";',
        "stats evaluation player event playlist filter options are checkbox toggles",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::Checkbox(label.c_str(), &enabled)',
        "plugin event playlist filter options are checkbox toggles",
        errors,
    )
    require_contains(
        web_player_main_source,
        'button.className = "event-playlist-item";',
        "stats evaluation player event playlist rows are clickable items",
        errors,
    )
    require_contains(
        plugin_source,
        "auto renderEventPlaylistItem = [&](const std::string &timeLabel,\n"
        "                                     const std::string &eventLabel,\n"
        "                                     const std::string &metaLabel,\n"
        "                                     const ImVec4 &eventColor,\n"
        "                                     bool active)",
        "plugin event playlist rows use a web-like card renderer",
        errors,
    )
    require_contains(
        plugin_source,
        'const bool clicked = ImGui::InvisibleButton(buttonId.c_str(), ImVec2{rowWidth, rowHeight});',
        "plugin event playlist card rows keep button click behavior",
        errors,
    )
    require_contains(
        plugin_source,
        "drawList->AddRectFilled(rowMin, rowMax, ImGui::ColorConvertFloat4ToU32(rowBg), 6.0f);\n"
        "    drawList->AddRect(rowMin, rowMax, ImGui::ColorConvertFloat4ToU32(border), 6.0f);",
        "plugin event playlist rows draw bordered card chrome",
        errors,
    )
    require_contains(
        plugin_source,
        "drawList->AddRectFilled(\n"
        "        rowMin,\n"
        "        ImVec2{rowMin.x + colorRailWidth, rowMax.y},\n"
        "        eventColorU32,\n"
        "        6.0f);",
        "plugin event playlist rows draw the web-like colored left rail",
        errors,
    )
    require_contains(
        web_player_main_source,
        "const EVENT_PLAYLIST_PLAYER_COLORS = [",
        "stats evaluation player event playlist has player color palette",
        errors,
    )
    require_contains(
        web_player_main_source,
        "return EVENT_PLAYLIST_PLAYER_COLORS[playerIndex % EVENT_PLAYLIST_PLAYER_COLORS.length]!",
        "stats evaluation player event playlist colors rows by player",
        errors,
    )
    require_contains(
        plugin_source,
        "LinearColor eventPlaylistPlayerColor(uint32_t playerIndex)",
        "plugin event playlist has player color palette",
        errors,
    )
    require_contains(
        plugin_source,
        "event.has_player != 0 ? eventPlaylistPlayerColor(event.player_index)",
        "plugin event playlist colors player rows by player",
        errors,
    )
    require_contains(
        web_player_main_source,
        "label.textContent = item.event.label ?? item.sourceLabel;",
        "stats evaluation player event playlist row title uses event label",
        errors,
    )
    require_contains(
        plugin_source,
        "const std::string eventLabel = event.label.empty() ? sourceLabel : event.label;",
        "plugin event playlist row title falls back to source label like web",
        errors,
    )
    require_contains(
        plugin_source,
        "const std::string metaLabel = joinStrings(metaParts, \" · \");",
        "plugin event playlist row metadata mirrors web joined meta text",
        errors,
    )
    require_contains(
        plugin_source,
        "drawList->AddText(\n"
        "        ImVec2{timeX, titleY},\n"
        "        IM_COL32(137, 164, 186, 255),\n"
        "        timeLabel.c_str());",
        "plugin event playlist renders time as a separate muted column",
        errors,
    )
    require_contains(
        plugin_source,
        "drawList->AddText(ImVec2{mainX, titleY}, IM_COL32(237, 245, 250, 255), eventLabel.c_str());",
        "plugin event playlist title draws in the main title column",
        errors,
    )
    require_contains(
        plugin_source,
        "drawList->AddText(ImVec2{mainX, metaY}, IM_COL32(137, 164, 186, 255), metaLabel.c_str());",
        "plugin event playlist metadata draws below the title column",
        errors,
    )
    reject_contains(
        plugin_source,
        "const std::string eventLabel = event.label.empty() ? event.type : event.label;",
        "plugin event playlist row title falls back to raw event type",
        errors,
    )
    for plugin_only_event_playlist_row_surface in (
        'ImGui::Selectable(itemLabel.c_str(), active)',
        'const std::string itemLabel = std::format("{}##event-playlist-item", eventLabel);',
        'ImGui::TextDisabled("%s", timeLabel.c_str());\n    ImGui::SameLine(64.0f);',
        'ImGui::SetCursorPosX(64.0f);\n      ImGui::TextDisabled("%s", joinStrings(metaParts, " · ").c_str());',
        "ImGui::Separator();\n    ImGui::PopID();",
    ):
        reject_contains(
            plugin_source,
            plugin_only_event_playlist_row_surface,
            "plugin event playlist plugin-only flat row surface",
            errors,
        )
    require_contains(
        web_player_timeline_markers_source,
        'label: `${playerName} speed flip ${qualityPercent}%`,',
        "stats evaluation player player mechanics include player names in titles",
        errors,
    )
    require_contains(
        plugin_source,
        'std::format("{} {}", playerLabel(event.player_index, event.is_team_0), action);',
        "plugin player mechanics include player names in event titles",
        errors,
    )
    require_contains(
        web_player_timeline_markers_source,
        "label: `${teamName} rush ${matchupLabel}`,",
        "stats evaluation player team events include team names in titles",
        errors,
    )
    require_contains(
        plugin_source,
        '"{} rush {}v{}"',
        "plugin team events include team names in event titles",
        errors,
    )
    require_contains(
        web_player_timeline_markers_source,
        'label: scorerName ? `${scorerName} goal context` : "Goal context",',
        "stats evaluation player goal context includes scorer names in titles",
        errors,
    )
    require_contains(
        plugin_source,
        'std::format("{} {}", actor, goalContextLabel(event))',
        "plugin goal context includes scorer names in event titles",
        errors,
    )
    require_contains(
        web_player_main_source,
        '.join(" · ");',
        "stats evaluation player event playlist row meta uses dot separator",
        errors,
    )
    require_contains(
        plugin_source,
        'joinStrings(metaParts, " · ")',
        "plugin event playlist row meta uses web-like dot separator",
        errors,
    )
    require_contains(
        web_player_main_source,
        "item.sourceLabel,",
        "stats evaluation player event playlist row meta uses source labels",
        errors,
    )
    require_contains(
        plugin_source,
        "const std::string sourceLabel = sourceLabelForEvent(event);",
        "plugin event playlist row meta uses selected source labels",
        errors,
    )
    require_contains(
        plugin_source,
        "return eventTypeDisplayLabel(event.type);",
        "plugin event playlist fallback source labels use web-like event display labels",
        errors,
    )
    require_contains(
        plugin_source,
        "std::string formatEventPlaylistTime(float seconds)",
        "plugin event playlist uses web-like minute time labels",
        errors,
    )
    reject_contains(
        plugin_source,
        'std::format("All ({})", recentUiEvents.size()).c_str()',
        "plugin event playlist all action includes event count",
        errors,
    )
    reject_contains(
        plugin_source,
        'std::format("{}  {}##event-playlist-item", timeLabel, eventLabel)',
        "plugin event playlist row title prefixes time",
        errors,
    )
    for plugin_only_event_playlist_filter_surface in (
        'renderModuleSummaryToggle(\n          "All",\n          allSourcesEnabled,\n          "event-playlist-sources")',
        'renderModuleSummaryToggle("None", noSourcesEnabled, "event-playlist-sources")',
        'renderModuleSummaryToggle(label.c_str(), source.enabled, "event-playlist-sources")',
        'ImGui::Text("%zu selected / %zu recent", playlistEventIndexes.size(), recentUiEvents.size());',
        'ImGui::TextWrapped("Status: %s", eventPlaylistStatus.c_str());',
        "eventPlaylistStatus",
        '"event_playlist_status"',
        "No events match the selected playlist filters.",
    ):
        reject_contains(
            plugin_source,
            plugin_only_event_playlist_filter_surface,
            "plugin event playlist filter module-summary button surface",
            errors,
        )
    require_contains(
        plugin_source,
        "const std::string currentFilter = eventPlaylistSourceFilter;",
        "plugin event playlist uses independent filter state",
        errors,
    )
    require_contains(
        plugin_source,
        "eventPlaylistSourceFilter = eventFilterFromSelectedSources(selectedSources);",
        "plugin event playlist filter changes do not mutate overlay event filters",
        errors,
    )
    reject_contains(
        plugin_source,
        'setCvarString(\n            "subtr_actor_overlay_event_types",\n            eventFilterFromSelectedSources(selectedSources));',
        "plugin event playlist filter mutates overlay event filters",
        errors,
    )
    reject_contains(
        plugin_source,
        'std::format("{} {:.2f}s##event-playlist-cue", active ? ">" : "Cue", event.time)',
        "plugin event playlist visible cue mini-button",
        errors,
    )
    for plugin_only_event_playlist_row_meta in (
        'joinStrings(metaParts, " / ").c_str());\n    }\n    if (!event.details.empty())',
        'ImGui::TextDisabled("%s", event.category.c_str());',
        "playerLabel(event.player_index, event.is_team_0),\n      mechanicLabel(event.kind),",
        "teamEventLabel(event),\n      std::format",
        'if (!event.details.empty()) {\n      ImGui::TextDisabled("%s", event.details.c_str());',
    ):
        reject_contains(
            plugin_source,
            plugin_only_event_playlist_row_meta,
            "plugin event playlist row plugin-only metadata surface",
            errors,
        )
    require_contains(
        web_player_main_source,
        'button.className = "mechanics-review-item";',
        "stats evaluation player mechanics review rows are clickable items",
        errors,
    )
    require_contains(
        plugin_source,
        "auto renderMechanicsReviewItem = [](const std::string &title,\n"
        "                                      const std::string &meta,\n"
        "                                      bool active)",
        "plugin mechanics review rows use a web-like two-column renderer",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::InvisibleButton("##mechanics-review-item", ImVec2{rowWidth, rowHeight})',
        "plugin mechanics review rows keep full-row button behavior",
        errors,
    )
    require_contains(
        plugin_source,
        "drawList->AddRectFilled(rowMin, rowMax, ImGui::ColorConvertFloat4ToU32(bg), 6.0f);\n"
        "    drawList->AddRect(rowMin, rowMax, ImGui::ColorConvertFloat4ToU32(border), 6.0f);",
        "plugin mechanics review rows draw web-like button chrome",
        errors,
    )
    require_contains(
        plugin_source,
        "drawList->AddText(ImVec2{titleX, textY}, IM_COL32(237, 245, 250, 255), title.c_str());",
        "plugin mechanics review rows render title in left column",
        errors,
    )
    require_contains(
        plugin_source,
        "drawList->AddText(ImVec2{metaX, textY}, IM_COL32(137, 164, 186, 255), metaText.c_str());",
        "plugin mechanics review rows render metadata in right column",
        errors,
    )
    require_contains(
        plugin_source,
        "const std::string meta = joinStrings(metaParts, \" · \");",
        "plugin mechanics review rows join metadata before rendering",
        errors,
    )
    require_contains(
        plugin_source,
        "if (renderMechanicsReviewItem(title, meta, active)) {\n"
        "      mechanicsReviewIndex = static_cast<int>(i);\n"
        "      scheduleUiConfigAutosave();\n"
        "    }",
        "plugin mechanics review row activation updates selection like web",
        errors,
    )
    require_contains(
        web_player_main_source,
        "return (\n"
        "    item.label ??\n"
        "    item.meta?.eventTypeLabel ??\n"
        "    item.meta?.mechanicLabel ??\n"
        "    `Review item ${index + 1}`\n"
        "  );",
        "stats evaluation player mechanics review item labels have mechanic fallback",
        errors,
    )
    require_contains(
        plugin_source,
        'return std::format("Review item {}", index + 1);',
        "plugin mechanics review item labels have review-item fallback",
        errors,
    )
    require_contains(
        plugin_source,
        "mechanicsReviewItemTitle(\n"
        "                               *current,\n"
        "                               static_cast<size_t>(mechanicsReviewIndex))",
        "plugin mechanics review current title uses shared web-like item label fallback",
        errors,
    )
    require_contains(
        plugin_source,
        "const std::string title = mechanicsReviewItemTitle(event, i);",
        "plugin mechanics review row title formats unlabeled mechanics",
        errors,
    )
    reject_contains(
        plugin_source,
        "current->label.empty() ? eventTypeDisplayLabel(current->type)\n"
        "                               : current->label;",
        "plugin mechanics review current title omits review item fallback",
        errors,
    )
    reject_contains(
        plugin_source,
        "const std::string title = event.label.empty() ? eventTypeDisplayLabel(event.type) : event.label;",
        "plugin mechanics review row title omits review item fallback",
        errors,
    )
    reject_contains(
        plugin_source,
        'ImGui::TextWrapped("%s", current == nullptr ? "No candidate selected" : current->label.c_str());',
        "plugin mechanics review current title can render an empty raw label",
        errors,
    )
    reject_contains(
        plugin_source,
        "const std::string title = event.label.empty() ? event.type : event.label;",
        "plugin mechanics review row title falls back to raw event type",
        errors,
    )
    require_contains(
        web_player_template_source,
        '<dl class="mechanics-review-fields">',
        "stats evaluation player mechanics review current item field grid",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::Columns(2, "mechanics-review-fields", false);',
        "plugin mechanics review current item uses web-like field grid",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".mechanics-review-fields dt {\n"
        "  color: var(--muted);\n"
        "  font-size: 0.68rem;\n"
        "  text-transform: uppercase;\n"
        "}",
        "stats evaluation player mechanics review fields use muted labels",
        errors,
    )
    require_contains(
        plugin_source,
        'renderWebDetailGridCell("Mechanic", mechanicReadout);',
        "plugin mechanics review mechanic field uses shared detail grid styling",
        errors,
    )
    require_contains(
        plugin_source,
        'renderWebDetailGridCell("Clip", clipReadout);',
        "plugin mechanics review clip field uses shared detail grid styling",
        errors,
    )
    require_contains(
        plugin_source,
        'renderWebDetailGridCell("Event", eventReadout);',
        "plugin mechanics review event field uses shared detail grid styling",
        errors,
    )
    require_contains(
        web_player_template_source,
        '<div class="mechanics-review-wide">\n                    <dt>Reason</dt>',
        "stats evaluation player mechanics review reason spans the field grid",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::Columns(1);\n'
        '  ImGui::Spacing();\n'
        '  ImGui::TextColored(ImVec4{0.54f, 0.64f, 0.73f, 1.0f}, "Reason");',
        "plugin mechanics review reason is rendered as a full-width field",
        errors,
    )
    require_contains(
        plugin_source,
        "ImGui::PushStyleColor(ImGuiCol_Text, ImVec4{0.93f, 0.96f, 0.98f, 1.0f});\n"
        '  ImGui::TextWrapped("%s", reasonReadout.c_str());\n'
        "  ImGui::PopStyleColor();",
        "plugin mechanics review reason uses web-like value color while wrapping",
        errors,
    )
    require_contains(
        web_player_main_source,
        'parts.push(`${Math.max(0, clipEnd - clipStart).toFixed(1)}s clip`);',
        "stats evaluation player mechanics review clip field includes duration",
        errors,
    )
    require_contains(
        plugin_source,
        '"{:.2f}s to {:.2f}s · {:.1f}s clip · {:.1f}s preroll · {:.1f}s postroll"',
        "plugin mechanics review clip field includes duration and padding details",
        errors,
    )
    require_contains(
        web_player_main_source,
        "return [time, frame].filter((part) => part && part !== \"--\").join(\" · \") || \"--\";",
        "stats evaluation player mechanics review event field combines time and frame",
        errors,
    )
    require_contains(
        plugin_source,
        '"{:.2f}s · frame {}"',
        "plugin mechanics review event field combines time and frame",
        errors,
    )
    reject_contains(
        plugin_source,
        'const std::string clipReadout =\n      current == nullptr ? "--" : std::format("{:.2f}s to {:.2f}s", clipStart, clipEnd);',
        "plugin mechanics review clip field omits web-like duration details",
        errors,
    )
    reject_contains(
        plugin_source,
        'std::format("frame {}", static_cast<unsigned long long>(current->frame_number))',
        "plugin mechanics review event field omits event time",
        errors,
    )
    for plugin_only_mechanics_review_field_surface in (
        'ImGui::TextDisabled("Mechanic");\n  const std::string mechanicReadout =',
        'ImGui::TextDisabled("Clip");\n  ImGui::Text("%s", clipReadout.c_str());',
        'ImGui::TextDisabled("Reason");\n  ImGui::TextWrapped("%s", reasonReadout.c_str());',
    ):
        reject_contains(
            plugin_source,
            plugin_only_mechanics_review_field_surface,
            "plugin mechanics review plugin-only raw field surface",
            errors,
        )
    require_contains(
        web_player_main_source,
        "const eventType = item.meta?.eventType ?? item.meta?.mechanic;\n"
        '  return typeof eventType === "string" ? formatMechanicKind(eventType) : "--";',
        "stats evaluation player mechanics review formats mechanic ids",
        errors,
    )
    require_contains(
        plugin_source,
        "std::string eventTypeDisplayLabel(std::string_view value) {",
        "plugin has web-like event type display labels",
        errors,
    )
    require_contains(
        plugin_source,
        "eventTypeDisplayLabel(current->type)",
        "plugin mechanics review current item formats mechanic ids",
        errors,
    )
    require_contains(
        plugin_source,
        "metaParts.push_back(eventTypeDisplayLabel(event.type));",
        "plugin mechanics review rows format mechanic ids",
        errors,
    )
    require_contains(
        web_player_template_source,
        '<div id="mechanics-review-status" class="mechanics-review-status">\n                Load a review playlist.\n              </div>',
        "stats evaluation player mechanics review initial status text",
        errors,
    )
    require_contains(
        plugin_source,
        ': candidates.empty()             ? "Load a review playlist."\n'
        '                                       : "Loaded review playlist.";',
        "plugin mechanics review renders web-like default status text",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::TextWrapped("%s", statusReadout.c_str());',
        "plugin mechanics review status is always visible like web",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::TextWrapped("%s", statusReadout.c_str());\n'
        "  ImGui::Separator();\n"
        '  ImGui::Text("%d / %zu", current == nullptr ? 0 : mechanicsReviewIndex + 1, candidates.size());',
        "plugin mechanics review status is separated before current item like web",
        errors,
    )
    reject_contains(
        plugin_source,
        'ImGui::TextWrapped("%s", currentTitle.c_str());\n  const std::string statusReadout =',
        "plugin mechanics review renders current item before status",
        errors,
    )
    require_contains(
        web_player_main_source,
        "elements.previous.disabled = !review || review.loading || review.currentIndex <= 0;",
        "stats evaluation player mechanics review disables unavailable previous action",
        errors,
    )
    require_contains(
        plugin_source,
        "const bool prevDisabled = current == nullptr || mechanicsReviewIndex <= 0;",
        "plugin mechanics review disables unavailable previous action",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".mechanics-review-actions,\n"
        ".mechanics-review-decision-actions,\n"
        ".mechanics-review-list-header {\n"
        "  display: flex;\n"
        "  align-items: center;\n"
        "  justify-content: space-between;\n"
        "  gap: var(--ui-gap-sm);\n"
        "}",
        "stats evaluation player mechanics review action rows use flex layout",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".mechanics-review-actions button,\n"
        ".mechanics-review-decision-actions button {\n"
        "  flex: 1 1 0;\n"
        "}",
        "stats evaluation player mechanics review action buttons distribute equally",
        errors,
    )
    require_contains(
        plugin_source,
        "const float actionButtonWidth =\n"
        "      std::max(72.0f, (ImGui::GetContentRegionAvail().x - actionGap * 2.0f) / 3.0f);",
        "plugin mechanics review action buttons distribute equally",
        errors,
    )
    require_contains(
        plugin_source,
        "ImGui::SameLine(0.0f, actionGap);",
        "plugin mechanics review action rows use explicit web-like gaps",
        errors,
    )
    require_contains(
        web_player_main_source,
        "elements.replay.disabled = !review || review.loading || !review.currentClip;",
        "stats evaluation player mechanics review disables unavailable replay action",
        errors,
    )
    require_contains(
        plugin_source,
        "const bool replayDisabled = current == nullptr;",
        "plugin mechanics review disables unavailable replay action",
        errors,
    )
    require_contains(
        web_player_main_source,
        "activeReplayPlayer.setAttachedPlayer(playerId);\n"
        "        activeReplayPlayer.setCameraViewMode(\"follow\");\n"
        "        this.options.clearFreeCameraPreset();",
        "stats evaluation player follows the reviewed player when activating a clip",
        errors,
    )
    require_contains(
        plugin_source,
        "cameraSelectedPlayerIndex = current->player_index;\n      cameraSelectedPlayerId = webPlayerIdForIndex(cameraSelectedPlayerIndex);\n      cameraViewMode = 1;\n      cameraFreePreset = -1;",
        "plugin mechanics review replay follows the reviewed player",
        errors,
    )
    require_contains(
        web_player_main_source,
        "elements.confirm.disabled = decisionDisabled;",
        "stats evaluation player mechanics review disables unavailable decisions",
        errors,
    )
    require_contains(
        web_player_styles_source,
        "#mechanics-review-confirm {\n"
        "  border-color: rgba(76, 175, 120, 0.52);\n"
        "}",
        "stats evaluation player mechanics review confirm action is green-accented",
        errors,
    )
    require_contains(
        web_player_styles_source,
        "#mechanics-review-reject {\n"
        "  border-color: rgba(220, 95, 95, 0.58);\n"
        "}",
        "stats evaluation player mechanics review reject action is red-accented",
        errors,
    )
    require_contains(
        plugin_source,
        "const bool decisionDisabled = current == nullptr;",
        "plugin mechanics review disables unavailable decisions",
        errors,
    )
    require_contains(
        plugin_source,
        "ImVec4{0.30f, 0.69f, 0.47f, 0.52f}",
        "plugin mechanics review confirm action uses green border accent",
        errors,
    )
    require_contains(
        plugin_source,
        "ImVec4{0.86f, 0.37f, 0.37f, 0.58f}",
        "plugin mechanics review reject action uses red border accent",
        errors,
    )
    for plugin_only_mechanics_review_action_surface in (
        'const bool clicked = ImGui::Button(label);',
        'ImGui::SameLine();\n  if (mechanicsReviewButton("Replay clip", replayDisabled))',
        'ImGui::SameLine();\n  if (mechanicsReviewButton("Reject", decisionDisabled))',
    ):
        reject_contains(
            plugin_source,
            plugin_only_mechanics_review_action_surface,
            "plugin mechanics review plugin-only natural-width action surface",
            errors,
        )
    require_contains(
        web_player_template_source,
        '<span id="mechanics-review-replay-load-summary">0 replays</span>',
        "stats evaluation player mechanics review replay summary",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::TextDisabled("%s", replayAnnotations ? "1 replay" : "0 replays");',
        "plugin mechanics review replay summary mirrors web",
        errors,
    )
    require_contains(
        web_player_main_source,
        'empty.textContent = "No review playlist loaded.";',
        "stats evaluation player mechanics review empty playlist text",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::TextDisabled("No review playlist loaded.");',
        "plugin mechanics review empty playlist text mirrors web",
        errors,
    )
    for plugin_only_mechanics_review_current_surface in (
        'ImGui::Text("Decision: %s", mechanicsReviewDecisionLabel(current));',
        'ImGui::Text("Mechanic: %s", current.type.c_str());',
        'ImGui::Text("Player: %s", current.actor.c_str());',
        'ImGui::Text("Clip: %.2fs to %.2fs", clipStart, clipEnd);',
        'ImGui::Text("Event: frame %llu", static_cast<unsigned long long>(current.frame_number));',
        'ImGui::TextWrapped("Reason: %s", current.details.c_str());',
        'ImGui::TextWrapped("No candidate selected");\n    ImGui::End();\n    return;',
    ):
        reject_contains(
            plugin_source,
            plugin_only_mechanics_review_current_surface,
            "plugin mechanics review current item debug-style rows",
            errors,
        )
    for mechanics_review_plugin_only_action in (
        'ImGui::Button("Stop clip")',
        'ImGui::Button("Show playlist")',
        'ImGui::Button("Clear decision")',
    ):
        reject_contains(
            plugin_source,
            mechanics_review_plugin_only_action,
            "plugin mechanics review plugin-only action",
            errors,
        )
    require_contains(
        web_player_main_source,
        "formatMechanicsReviewStatus(candidate.meta?.reviewStatus)",
        "stats evaluation player mechanics review rows expose review status",
        errors,
    )
    require_contains(
        web_player_main_source,
        "this.getPlayerName(candidate),",
        "stats evaluation player mechanics review rows expose player attribution",
        errors,
    )
    require_contains(
        plugin_source,
        "if (!event.actor.empty()) {\n      metaParts.push_back(event.actor);\n    }",
        "plugin mechanics review rows expose player attribution",
        errors,
    )
    require_contains(
        plugin_source,
        'metaParts.push_back(mechanicsReviewDecisionLabel(event));',
        "plugin mechanics review rows expose review status",
        errors,
    )
    require_contains(
        web_player_main_source,
        "formatMechanicsReviewStatus(candidate.meta?.reviewStatus),",
        "stats evaluation player mechanics review row meta uses dot separator",
        errors,
    )
    require_contains(
        web_player_main_source,
        ".join(\" · \") || \"--\";",
        "stats evaluation player mechanics review row meta uses dot separator",
        errors,
    )
    require_contains(
        plugin_source,
        'const std::string meta = joinStrings(metaParts, " · ");',
        "plugin mechanics review row meta uses web-like dot separator",
        errors,
    )
    for plugin_only_mechanics_review_row_surface in (
        'std::format(\n        "{}##mechanics-review-item",\n        title)',
        "ImGui::Selectable(label.c_str(), active)",
        'ImGui::TextDisabled("%s", joinStrings(metaParts, " · ").c_str());',
    ):
        reject_contains(
            plugin_source,
            plugin_only_mechanics_review_row_surface,
            "plugin mechanics review plugin-only flat row surface",
            errors,
        )
    require_contains(
        web_player_main_source,
        "export function getStatsPlayerConfigSnapshot({",
        "stats evaluation player has an explicit persisted config snapshot",
        errors,
    )
    require_contains(
        web_player_main_source,
        "pluginRenderEffects: [...initialConfig.overlays.pluginRenderEffects]",
        "stats evaluation player preserves plugin-only render effect config",
        errors,
    )
    require_contains(
        web_player_main_source,
        "pluginHudOverlay: initialConfig.overlays.pluginHudOverlay",
        "stats evaluation player preserves plugin-only HUD overlay config",
        errors,
    )
    reject_contains(
        plugin_source,
        '"mechanics_review_status"',
        "plugin persists transient mechanics review status",
        errors,
    )
    reject_contains(
        plugin_source,
        'std::format(\n        "{} {:.2f}s {} ({})",',
        "plugin mechanics review rows use prefix/raw-seconds/status-in-title shape",
        errors,
    )
    reject_contains(
        plugin_source,
        'std::format(\n        "{}  {}##mechanics-review-item",\n        formatEventPlaylistTime(event.time),',
        "plugin mechanics review rows prefix event time before item title",
        errors,
    )
    for plugin_only_mechanics_review_queue_surface in (
        'ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "REVIEW QUEUE");',
        'ImGui::Checkbox("Mechanics", &eventPlaylistMechanicsEnabled)',
        'ImGui::Checkbox("Team", &eventPlaylistTeamEventsEnabled)',
        'ImGui::Checkbox("Goal context", &eventPlaylistGoalContextEnabled)',
        'ImGui::SliderFloat("Clip lead", &mechanicsReviewClipLeadSeconds, 0.0f, 10.0f, "%.1fs")',
        '"Clip trail", &mechanicsReviewClipTrailSeconds, 0.0f, 10.0f, "%.1fs"',
        'ImGui::Button("Open events")',
        'ImGui::Button("Open playlist")',
        'ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "REPLAY");',
        'ImGui::Text(\n      "Replay annotations: %s",',
        'joinStrings(metaParts, " / ").c_str());\n    ImGui::PopID();',
    ):
        reject_contains(
            plugin_source,
            plugin_only_mechanics_review_queue_surface,
            "plugin mechanics review plugin-only queue/filter surface",
            errors,
        )
    require_contains(
        web_player_template_source,
        '<span id="replay-loading-summary">0 replays</span>',
        "stats evaluation player replay loading summary starts as replay count",
        errors,
    )
    require_contains(
        plugin_source,
        'const std::string replaySummary = hasReplaySource ? "1 replay" : "0 replays";',
        "plugin replay loading summary uses web-like replay count",
        errors,
    )
    require_contains(
        web_player_template_source,
        '<span id="replay-loading-active">Idle</span>',
        "stats evaluation player replay loading active summary exists",
        errors,
    )
    require_contains(
        web_player_main_source,
        "loaded === states.length",
        "stats evaluation player replay loading active summary reports completion",
        errors,
    )
    require_contains(
        plugin_source,
        'const char *activeSummary = !hasReplaySource         ? "No playlist"\n'
        '                              : replayAnnotations      ? "Complete"',
        "plugin replay loading active summary mirrors web completion state",
        errors,
    )
    require_contains(
        plugin_source,
        'ImGui::TextDisabled("%s", activeSummary);',
        "plugin replay loading renders active summary separately from replay count",
        errors,
    )
    require_contains(
        web_player_main_source,
        'if (state.status === "idle") {\n'
        '      return "Pending";\n'
        "    }\n"
        '    if (state.status === "loading") {\n'
        '      return this.replayLoadStateProgress(state.progress) || "Loading";\n'
        "    }",
        "stats evaluation player replay loading row uses pending/loading status labels",
        errors,
    )
    require_contains(
        plugin_source,
        ': !inReplay       ? "Pending"\n                           : replayAnnotations ? "Loaded"',
        "plugin replay loading pending status mirrors web",
        errors,
    )
    require_contains(
        plugin_source,
        ': "Loading";',
        "plugin replay loading active row status mirrors web loading label",
        errors,
    )
    require_contains(
        web_player_main_source,
        '.join(" · ");',
        "stats evaluation player replay loading rows use dot-separated metadata",
        errors,
    )
    require_contains(
        web_player_main_source,
        "const fileName = rawPath\n    .replace(/^path:/, \"\")\n    .split(\"/\")\n    .filter(Boolean)\n    .pop();",
        "stats evaluation player replay loading row title derives a file label",
        errors,
    )
    require_contains(
        plugin_source,
        "std::string replaySourceDisplayLabel(std::string_view path) {",
        "plugin replay loading has a web-like replay source label helper",
        errors,
    )
    require_contains(
        plugin_source,
        'const std::string title = replaySourceDisplayLabel(titlePath);',
        "plugin replay loading row title uses a file label",
        errors,
    )
    require_contains(
        web_player_main_source,
        'row.append(main, status, progress);',
        "stats evaluation player replay loading row places title/status/progress together",
        errors,
    )
    require_contains(
        plugin_source,
        "ImGui::Dummy(ImVec2{rowWidth, rowHeight});\n"
        "    const ImVec2 rowMin = ImGui::GetItemRectMin();\n"
        "    const ImVec2 rowMax = ImGui::GetItemRectMax();",
        "plugin replay loading row reserves a web-like source card",
        errors,
    )
    require_contains(
        plugin_source,
        "drawList->AddRectFilled(\n"
        "        rowMin,\n"
        "        rowMax,\n"
        "        ImGui::ColorConvertFloat4ToU32(ImVec4{1.0f, 1.0f, 1.0f, 0.035f}),\n"
        "        6.0f);\n"
        "    drawList->AddRect(rowMin, rowMax, ImGui::ColorConvertFloat4ToU32(rowBorder), 6.0f);",
        "plugin replay loading row draws web-like card chrome",
        errors,
    )
    require_contains(
        plugin_source,
        "drawList->AddText(\n"
        "        ImVec2{statusX, titleY},\n"
        "        ImGui::ColorConvertFloat4ToU32(statusColor),\n"
        "        status);",
        "plugin replay loading renders status on the title row",
        errors,
    )
    require_contains(
        plugin_source,
        'const std::string meta = joinStrings(replayMeta, " · ");',
        "plugin replay loading source row uses web-like dot-separated metadata",
        errors,
    )
    require_contains(
        web_player_main_source,
        'progress.className = "mechanics-review-replay-load-progress";',
        "stats evaluation player replay loading rows include progress bars",
        errors,
    )
    require_contains(
        plugin_source,
        "drawList->AddRectFilled(\n"
        "        ImVec2{progressMinX, progressY},\n"
        "        ImVec2{progressMinX + (progressMaxX - progressMinX) * replayLoadProgress, progressY + 4.0f},",
        "plugin replay loading source row draws web-like embedded progress bar",
        errors,
    )
    reject_contains(
        plugin_source,
        'ImGui::Text("Summary: %s", replayPath ? "1 replay candidate" : "0 replay candidates");',
        "plugin replay loading summary label/candidate wording",
        errors,
    )
    reject_contains(
        plugin_source,
        'ImGui::Text("Active: %s", status);',
        "plugin replay loading active status label",
        errors,
    )
    reject_contains(
        plugin_source,
        '"Waiting for replay"',
        "plugin replay loading uses plugin-only waiting status label",
        errors,
    )
    reject_contains(
        plugin_source,
        '"Scanning"',
        "plugin replay loading uses plugin-only scanning status label",
        errors,
    )
    reject_contains(
        plugin_source,
        'const std::string title = replayPath ? *replayPath\n                              : !rawReplayPath.empty() ? rawReplayPath\n                                                       : replayAnnotationPath;',
        "plugin replay loading row title uses the full replay path",
        errors,
    )
    reject_contains(
        plugin_source,
        'ImGui::TextWrapped("%s", title.c_str());',
        "plugin replay loading stacks title before status",
        errors,
    )
    for plugin_only_replay_loading_row_surface in (
        "const float statusX =\n        std::max(ImGui::GetCursorPosX(), ImGui::GetWindowContentRegionMax().x - statusWidth);",
        'ImGui::SameLine(statusX);\n    ImGui::TextColored(statusColor, "%s", status);',
        'ImGui::ProgressBar(replayLoadProgress, ImVec2{-1.0f, 0.0f}, "");',
    ):
        reject_contains(
            plugin_source,
            plugin_only_replay_loading_row_surface,
            "plugin replay loading plugin-only stacked row surface",
            errors,
        )
    for plugin_only_replay_loading_surface in (
        'ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "REPLAY LOADING");',
        'ImGui::Text("In replay: %s", inReplay ? "yes" : "no");',
        'ImGui::Text("Replay time: %.2fs", replayServer.GetReplayTimeElapsed());',
        'ImGui::Text("Annotations: %zu", annotationCount);',
        'ImGui::Text("Players: %zu", annotationPlayers.size());',
        'ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "REPLAY SOURCES");',
        'ImGui::BeginChild("replay-loading-players", ImVec2{0.0f, 96.0f}, true);',
        'ImGui::Checkbox("Replay annotations", &annotationsValue)',
        'ImGui::Button("Retry load")',
        'ImGui::Button("Clear load")',
        'ImGui::TextColored(ImVec4{0.53f, 0.69f, 0.83f, 1.0f}, "CURRENT REPLAY");',
        'metaText += " | ";',
    ):
        reject_contains(
            plugin_source,
            plugin_only_replay_loading_surface,
            "plugin replay loading plugin-only management surface",
            errors,
        )
    for label, plugin_needle in (
        ("Shots, saves, assists", '{"core", "Shots, saves, assists", "Replay"}'),
        ("Possession", '"Possession",\n        timelineRangePossessionEnabled'),
        ("Half control", '"Half control",\n        timelineRangePressureEnabled'),
        ("Rush", '"Rush", timelineRangeRushEnabled'),
        ("Position zones", '"Position zones",\n        timelineRangeAbsolutePositioningEnabled'),
    ):
        require_contains(
            web_player_main_source,
            f'"{label}"',
            f"stats evaluation player module summary label {label}",
            errors,
        )
        require_contains(
            plugin_source,
            plugin_needle,
            f"plugin module summary label {label}",
            errors,
        )
    require_contains(
        plugin_source,
        'appendUiEvent(UiEventRecord{\n      "core",',
        "plugin sends shot/save/assist events to the web core event source",
        errors,
    )
    require_contains(
        plugin_source,
        'appendUiEvent(UiEventRecord{\n      "goal",\n      "goal",',
        "plugin sends goals to the web replay goal event source",
        errors,
    )
    require_contains(
        plugin_source,
        "pushGoalEventMessage(event);\n  pendingGoals.push_back(event);",
        "plugin surfaces raw goal events before graph submission",
        errors,
    )
    require_contains(
        plugin_source,
        'pushPlayerStatEventMessage(event);\n      pendingPlayerStatEvents.push_back(event);',
        "plugin surfaces inferred player stat events before graph submission",
        errors,
    )
    require_contains(
        plugin_source,
        'pushPlayerStatEventMessage(event);\n  pendingPlayerStatEvents.push_back(event);',
        "plugin surfaces explicit player stat events before graph submission",
        errors,
    )
    for stale_label in (
        '"Possession timeline"',
        '"Half control timeline"',
        '"Rush timeline"',
        '"Position zones timeline"',
    ):
        reject_contains(
            plugin_source,
            stale_label,
            "plugin module summary stale timeline suffix label",
            errors,
        )
    require_contains(
        web_player_template_source,
        '<div id="module-settings" class="module-settings" hidden></div>',
        "stats evaluation player launcher ends with module settings",
        errors,
    )
    for plugin_only_launcher_surface in (
        'ImGui::TreeNode("Plugin tools##launcher-plugin-tools")',
        '"Live analysis graph",\n            liveAnalysis,\n            "launcher-plugin-tools"',
        'ImGui::Button("Verify graph", ImVec2{pluginToolButtonWidth, 0.0f})',
        'ImGui::Button("Open modules", ImVec2{pluginToolButtonWidth, 0.0f})',
        'ImGui::BeginChild("launcher-graph-stats-modules", ImVec2{0.0f, 130.0f}, true);',
        'renderLauncherWorkspaceControls();',
    ):
        reject_contains(
            plugin_source,
            plugin_only_launcher_surface,
            "plugin-only launcher surface",
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
        'const bool hasRenderEffects = jsonPropertyExists(*overlays, "renderEffects");',
        "web renderEffects import uses JSON parser",
        errors,
    )
    require_contains(
        plugin_source,
        'jsonPropertyExists(*overlays, "pluginRenderEffects")',
        "plugin-only render effects import uses separate plugin config field",
        errors,
    )
    require_contains(
        plugin_source,
        'parseJsonBoolProperty(*overlays, "pluginHudOverlay")',
        "plugin-only HUD overlay master imports from separate plugin config field",
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
        'renderBoostFilterGroupTitle("Player");',
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
        "const bool hasSelectedMechanicFilter = std::any_of(\n"
        "          selectedFilters.begin(),\n"
        "          selectedFilters.end(),",
        "web mechanic timeline event import enables mechanic playlist group",
        errors,
    )
    reject_contains(
        plugin_source,
        'writeOverlayId("mechanics", eventPlaylistMechanicsEnabled);',
        "web timelineEvents exports plugin-only broad mechanics id",
        errors,
    )
    reject_contains(
        plugin_source,
        'writeOverlayId("team", eventPlaylistTeamEventsEnabled);',
        "web timelineEvents exports plugin-only broad team id",
        errors,
    )
    reject_contains(
        plugin_source,
        'writeOverlayId("goal_context", eventPlaylistGoalContextEnabled);',
        "web timelineEvents exports plugin-only broad goal-context id",
        errors,
    )
    require_contains(
        plugin_source,
        "webTimelineEventSourceIdForFilterToken(token)",
        "web timelineEvents exports normalized web source ids from plugin filters",
        errors,
    )
    require_contains(
        plugin_source,
        "for (const char *token : MECHANIC_FILTER_TOKENS) {\n"
        "      writeMechanicFilterId(token);\n"
        "    }",
        "web mechanics config exports concrete mechanic filters for all-mechanics selection",
        errors,
    )
    reject_contains(
        plugin_source,
        'writeOverlayId(\n      "mechanics",\n      hudOverlayEnabled && cvarBool("subtr_actor_overlay_mechanics_enabled", true));',
        "web renderEffects exports plugin-only broad mechanics id",
        errors,
    )
    reject_contains(
        plugin_source,
        'writeOverlayId(\n      "team",\n      hudOverlayEnabled && cvarBool("subtr_actor_overlay_team_events_enabled", true));',
        "web renderEffects exports plugin-only broad team id",
        errors,
    )
    reject_contains(
        plugin_source,
        'writeOverlayId(\n      "goal_context",\n      hudOverlayEnabled && cvarBool("subtr_actor_overlay_goal_context_enabled", true));',
        "web renderEffects exports plugin-only broad goal-context id",
        errors,
    )
    require_contains(
        plugin_source,
        'file << "    \\"pluginRenderEffects\\": [";',
        "plugin-only render effects export uses separate plugin config field",
        errors,
    )
    require_contains(
        plugin_source,
        'file << "    \\"pluginHudOverlay\\": " << (hudOverlayEnabled ? "true" : "false") << ",\\n";',
        "plugin-only HUD overlay master exports separate plugin config field",
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
        web_player_main_source,
        'const SINGLETON_WINDOW_IDS: SingletonWindowId[] = [',
        "stats evaluation player has an explicit singleton window stacking order",
        errors,
    )
    require_contains(
        plugin_source,
        "for (const SingletonWindowControl &window : webSingletonWindowControls()) {\n"
        "    resetSingletonPlacement(window);\n"
        "  }\n"
        "  for (const SingletonWindowControl &window : singletonWindowControls()) {\n"
        "    if (window.web_config) {\n"
        "      continue;\n"
        "    }\n"
        "    resetSingletonPlacement(window);\n"
        "  }",
        "plugin reset placement z-order follows web singleton order before plugin-only windows",
        errors,
    )
    reject_contains(
        plugin_source,
        "for (const SingletonWindowControl &window : singletonWindowControls()) {\n"
        "    if (window.placement == &scoreboardPlacement) {\n"
        "      resetScoreboardWindowPlacement();\n"
        "      continue;\n"
        "    }\n"
        "    resetSingletonWindowPlacement(\n"
        "        *window.placement,\n"
        "        window.x,\n"
        "        window.y,\n"
        "        window.width,\n"
        "        window.height);\n"
        "  }",
        "plugin reset placement z-order uses raw singleton declaration order",
        errors,
    )
    require_contains(
        web_player_main_source,
        'Boolean(target.closest("button, input, select, textarea, option, label, a, [data-no-drag]"))',
        "stats evaluation player window dragging ignores interactive controls",
        errors,
    )
    require_contains(
        web_player_main_source,
        "bringWindowToFront(windowEl) {",
        "stats evaluation player brings windows forward from non-interactive pointer down",
        errors,
    )
    require_contains(
        web_player_main_source,
        "options.getFloatingWindowController()?.bringToFront(windowEl);",
        "stats evaluation player brings windows forward from non-interactive pointer down",
        errors,
    )
    require_contains(
        plugin_source,
        "ImGui::IsWindowHovered(ImGuiHoveredFlags_RootAndChildWindows) &&\n"
        "      ImGui::IsMouseClicked(ImGuiMouseButton_Left) && !ImGui::IsAnyItemActive()",
        "plugin window z-order bumps ignore active controls like web dragging",
        errors,
    )
    reject_contains(
        plugin_source,
        "ImGui::IsWindowHovered(ImGuiHoveredFlags_RootAndChildWindows) &&\n"
        "      ImGui::IsMouseClicked(ImGuiMouseButton_Left))",
        "plugin window z-order bumps on interactive control clicks",
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
    require_contains(
        web_player_styles_source,
        ".floating-window,\n"
        ".stats-window,\n"
        ".scoreboard-window {\n"
        "  position: absolute;\n"
        "  left: clamp(0.8rem, var(--window-x, 1rem), calc(100vw - 18rem));",
        "stats evaluation player scoreboard shares overlay chrome base",
        errors,
    )
    require_contains(
        web_player_styles_source,
        "  border: 1px solid rgba(255, 255, 255, 0.12);\n"
        "  border-radius: var(--ui-radius-lg);\n"
        "  background: var(--ui-overlay-bg);",
        "stats evaluation player scoreboard shares overlay border/background",
        errors,
    )
    require_contains(
        web_player_styles_source,
        ".scoreboard-window {\n"
        "  left: 50%;\n"
        "  top: 0.7rem;\n"
        "  width: auto;",
        "stats evaluation player scoreboard is a centered pill",
        errors,
    )
    require_contains(
        plugin_source,
        "ImGui::PushStyleVar(ImGuiStyleVar_WindowRounding, 999.0f);\n"
        "  ImGui::PushStyleVar(ImGuiStyleVar_WindowBorderSize, 1.0f);\n"
        "  ImGui::PushStyleColor(ImGuiCol_WindowBg, ImVec4{0.03f, 0.07f, 0.10f, 0.88f});\n"
        "  ImGui::PushStyleColor(ImGuiCol_Border, ImVec4{1.0f, 1.0f, 1.0f, 0.12f});",
        "plugin scoreboard uses web-like pill background and border",
        errors,
    )
    require_contains(
        plugin_source,
        "ImGui::PopStyleColor(2);\n  ImGui::PopStyleVar(3);",
        "plugin scoreboard restores pill chrome style stack",
        errors,
    )
    require_contains(
        web_player_main_source,
        "renderScoreboard(state.frameIndex);",
        "stats evaluation player scoreboard follows current replay frame",
        errors,
    )
    require_contains(
        plugin_source,
        "replayAnnotationScoreAtTime(replayAnnotations, replayTime, &score)",
        "plugin scoreboard follows replay annotation score at current playback time",
        errors,
    )
    require_contains(
        plugin_source,
        "return lastTeamScores;\n}\n\nvoid SubtrActorPlugin::renderScoreboardWindow()",
        "plugin scoreboard falls back to live scores after replay annotation scores",
        errors,
    )
    require_contains(
        plugin_source,
        "writeReplayAnnotationFramePlayers(\n        replayAnnotations,\n        replayTime,",
        "plugin replay annotations update sampled players from current replay stats frame",
        errors,
    )
    require_contains(
        rust_combined_source,
        "match_goals: player.core.goals,",
        "replay annotation frame players expose core goals to plugin stats windows",
        errors,
    )
    require_contains(
        plugin_source,
        "replayStatsModuleNamesFromFrameJson(currentReplayFrameJson())",
        "plugin stats module controls can discover modules from replay frame JSON",
        errors,
    )
    require_contains(
        plugin_source,
        "const std::vector<std::string> moduleNames = availableStatsModuleNames();",
        "plugin stats module selectors use live and replay module names",
        errors,
    )
    require_contains(
        plugin_source,
        "renderStatsModuleFrameOverview(json, std::format(\"module-frame-{}\", window.id));",
        "plugin stats module frame windows render structured team and player sections",
        errors,
    )
    require_contains(
        plugin_source,
        "json += \",\\\"name\\\":\";",
        "plugin replay module frame player stats preserve player names for in-game cards",
        errors,
    )
    require_contains(
        plugin_source,
        "renderStatsWindowEmpty(\"No stats added.\");",
        "plugin stats windows show the web-player no-stats empty state",
        errors,
    )
    require_contains(
        plugin_source,
        "renderStatsWindowEmpty(\"Load a replay to show goal labels.\");",
        "plugin goal-label stats windows show the web-player replay empty state",
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
    require_contains(
        plugin_source,
        'cvarManager->registerNotifier(\n      "subtr_actor_apply_ui_config",',
        "plugin exposes console UI config import",
        errors,
    )
    require_contains(
        plugin_source,
        "statsPlayerCfgJsonFromClipboard(configText)",
        "console UI config import accepts stats-player cfg values",
        errors,
    )
    require_contains(
        plugin_readme_source,
        "`subtr_actor_apply_ui_config <json|cfg|url>`",
        "BakkesMod README documents console UI config import",
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
                rust_combined_source + "\n" + plugin_source,
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
