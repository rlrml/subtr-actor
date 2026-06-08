#!/usr/bin/env python3
"""Validate BakkesMod graph JSON dumps.

Run after `subtr_actor_dump_graph finish` writes files under BakkesMod's
`data/subtr-actor` directory.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


GRAPH_DUMP_FILES = {
    "events": "graph-events.json",
    "frame": "graph-frame.json",
    "timeline": "graph-timeline.json",
    "stats": "graph-stats.json",
    "analysis_nodes": "graph-analysis-nodes.json",
    "event_history": "graph-event-history.json",
    "graph_info": "graph-info.json",
}

REQUIRED_GRAPH_OUTPUTS = tuple(GRAPH_DUMP_FILES.keys())
DEFAULT_GRAPH_EVENT_FIELDS = (
    "timeline",
    "mechanics",
    "goal_context",
    "core_player",
    "core_player_goal_context",
    "possession",
    "pressure",
    "territorial_pressure",
    "movement",
    "positioning_activity",
    "positioning_possession",
    "positioning_field_zone",
    "positioning_ball_depth",
    "positioning_teammate_role",
    "positioning_ball_proximity",
    "positioning_goal_context",
    "rotation_player",
    "rotation_team",
    "rotation_role_span",
    "rotation_depth_span",
    "rotation_first_man_stint",
    "backboard",
    "ball_carry",
    "ceiling_shot",
    "wall_aerial",
    "wall_aerial_shot",
    "center",
    "double_tap",
    "fifty_fifty",
    "kickoff",
    "flick",
    "musty_flick",
    "one_timer",
    "pass",
    "rush",
    "flip_impulse",
    "speed_flip",
    "half_flip",
    "half_volley",
    "wavedash",
    "whiff",
    "dodge_reset",
    "powerslide",
    "boost_pickups",
    "boost_ledger",
    "boost_state",
    "bump",
    "touch",
    "touch_ball_movement",
)
DEFAULT_REQUIRED_GRAPH_EVENT_FIELDS = ("timeline", "goal_context", "boost_pickups")
DEFAULT_EVENT_HISTORY_FIELDS = (
    "active_demos",
    "demo_events",
    "boost_pad_events",
    "touch_events",
    "dodge_refreshed_events",
    "player_stat_events",
    "goal_events",
)
DEFAULT_REQUIRED_EVENT_HISTORY_FIELDS = (
    "demo_events",
    "boost_pad_events",
    "touch_events",
    "dodge_refreshed_events",
    "player_stat_events",
    "goal_events",
)


class Validation:
    def __init__(self) -> None:
        self.errors: list[str] = []

    def require(self, condition: bool, message: str) -> None:
        if not condition:
            self.errors.append(message)

    def require_string_list(self, value: Any, name: str) -> list[str]:
        if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
            self.errors.append(f"{name} must be an array of strings")
            return []
        return value

    def require_object(self, value: Any, name: str) -> dict[str, Any]:
        if not isinstance(value, dict):
            self.errors.append(f"{name} must be a JSON object")
            return {}
        return value


def read_json(path: Path, validation: Validation) -> Any:
    if not path.is_file():
        validation.errors.append(f"missing dump file: {path}")
        return None
    if path.stat().st_size == 0:
        validation.errors.append(f"empty dump file: {path}")
        return None
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        validation.errors.append(f"invalid JSON in {path}: {error}")
        return None


def require_array_field(
    validation: Validation,
    payload: dict[str, Any],
    field_name: str,
    label: str,
    require_nonzero: bool,
) -> None:
    value = payload.get(field_name)
    if not isinstance(value, list):
        validation.errors.append(f"{label} field {field_name!r} must be an array")
        return
    print(f"{label} field {field_name!r} has {len(value)} entries")
    if require_nonzero and not value:
        validation.errors.append(f"{label} field {field_name!r} has no entries")


def validate_dump(dump_dir: Path, require_event_history: bool, require_graph_events: bool) -> int:
    validation = Validation()
    payloads = {
        name: read_json(dump_dir / file_name, validation)
        for name, file_name in GRAPH_DUMP_FILES.items()
    }
    if validation.errors:
        return report(validation)

    graph_info = validation.require_object(payloads["graph_info"], "graph-info.json")
    graph_output_names = validation.require_string_list(
        graph_info.get("graph_output_names"), "graph_info.graph_output_names"
    )
    callable_node_names = validation.require_string_list(
        graph_info.get("callable_analysis_node_names"),
        "graph_info.callable_analysis_node_names",
    )
    resolved_node_names = validation.require_string_list(
        graph_info.get("node_names"), "graph_info.node_names"
    )
    graph_event_fields = validation.require_string_list(
        graph_info.get("graph_event_field_names"), "graph_info.graph_event_field_names"
    ) or list(DEFAULT_GRAPH_EVENT_FIELDS)
    required_graph_event_fields = validation.require_string_list(
        graph_info.get("required_graph_event_field_names"),
        "graph_info.required_graph_event_field_names",
    ) or list(DEFAULT_REQUIRED_GRAPH_EVENT_FIELDS)
    event_history_fields = validation.require_string_list(
        graph_info.get("event_history_field_names"), "graph_info.event_history_field_names"
    ) or list(DEFAULT_EVENT_HISTORY_FIELDS)
    required_event_history_fields = validation.require_string_list(
        graph_info.get("required_event_history_field_names"),
        "graph_info.required_event_history_field_names",
    ) or list(DEFAULT_REQUIRED_EVENT_HISTORY_FIELDS)

    for output_name in REQUIRED_GRAPH_OUTPUTS:
        validation.require(
            output_name in graph_output_names,
            f"graph_info.graph_output_names missing {output_name!r}",
        )
    for output_name in graph_output_names:
        validation.require(
            output_name in GRAPH_DUMP_FILES,
            f"graph_info.graph_output_names contains unsupported dump output {output_name!r}",
        )
    for field_name in DEFAULT_GRAPH_EVENT_FIELDS:
        validation.require(
            field_name in graph_event_fields,
            f"graph_info.graph_event_field_names missing known field {field_name!r}",
        )
    for field_name in DEFAULT_REQUIRED_GRAPH_EVENT_FIELDS:
        validation.require(
            field_name in required_graph_event_fields,
            f"graph_info.required_graph_event_field_names missing strict field {field_name!r}",
        )
    for field_name in required_graph_event_fields:
        validation.require(
            field_name in graph_event_fields,
            f"required graph event field {field_name!r} is not declared",
        )
    for field_name in DEFAULT_EVENT_HISTORY_FIELDS:
        validation.require(
            field_name in event_history_fields,
            f"graph_info.event_history_field_names missing known field {field_name!r}",
        )
    for field_name in DEFAULT_REQUIRED_EVENT_HISTORY_FIELDS:
        validation.require(
            field_name in required_event_history_fields,
            f"graph_info.required_event_history_field_names missing strict field {field_name!r}",
        )
    for field_name in required_event_history_fields:
        validation.require(
            field_name in event_history_fields,
            f"required event_history field {field_name!r} is not declared",
        )

    analysis_nodes = validation.require_object(
        payloads["analysis_nodes"], "graph-analysis-nodes.json"
    )
    analysis_node_keys = set(analysis_nodes.keys())
    for node_name in callable_node_names:
        validation.require(
            node_name in analysis_node_keys,
            f"graph-analysis-nodes.json missing callable node {node_name!r}",
        )
    for node_name in analysis_node_keys:
        validation.require(
            node_name in callable_node_names,
            f"graph-analysis-nodes.json contains unexpected node {node_name!r}",
        )
    for node_name in resolved_node_names:
        validation.require(
            node_name in callable_node_names,
            f"resolved graph node {node_name!r} is not callable by name",
        )

    events = validation.require_object(payloads["events"], "graph-events.json")
    for field_name in graph_event_fields:
        require_array_field(
            validation,
            events,
            field_name,
            "events",
            require_graph_events and field_name in required_graph_event_fields,
        )

    event_history = validation.require_object(
        payloads["event_history"], "graph-event-history.json"
    )
    for field_name in event_history_fields:
        require_array_field(
            validation,
            event_history,
            field_name,
            "event_history",
            require_event_history and field_name in required_event_history_fields,
        )

    print(f"graph outputs declared: {len(graph_output_names)}")
    print(f"callable analysis nodes declared: {len(callable_node_names)}")
    print(f"resolved graph nodes declared: {len(resolved_node_names)}")
    return report(validation)


def report(validation: Validation) -> int:
    if validation.errors:
        for error in validation.errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    print("BakkesMod graph dump validation passed")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Validate JSON files written by subtr_actor_dump_graph finish."
    )
    parser.add_argument("dump_dir", type=Path, help="BakkesMod data/subtr-actor dump directory")
    parser.add_argument(
        "--require-event-history",
        action="store_true",
        help="Require strict cumulative event_history fields to be nonzero",
    )
    parser.add_argument(
        "--require-graph-events",
        action="store_true",
        help="Require strict graph-generated events fields to be nonzero",
    )
    args = parser.parse_args()
    return validate_dump(
        args.dump_dir,
        require_event_history=args.require_event_history,
        require_graph_events=args.require_graph_events,
    )


if __name__ == "__main__":
    raise SystemExit(main())
