"""Rocket Sense review campaigns over frozen evaluation manifests.

`playlist` turns manifest windows into the subtr-actor review playlist that
`POST /api/v1/campaigns` imports: replays are referenced by file sha256, each
item is keyed by a content-derived `eventId`, and items carry the focal
player's perspective but no detector verdict or stratum.

The deployed review player offers a fixed confirmed / rejected / uncertain /
bad_candidate vocabulary, so a campaign asks one question, "clean whiff at the
marked moment?", and `finals_from_export` maps the answers onto the anchor
labels `fresh_eval.score` consumes. Only clean-whiff metrics are meaningful:
`rejected` does not distinguish disrupted attempts from contacts.
"""

from __future__ import annotations

STATUS_LABELS = {
    "confirmed": "clear_whiff",
    "rejected": "not_clear_whiff",
    "uncertain": "uncertain",
}
QUESTION = (
    "Did the highlighted player make a clear attempt at the ball around the marked "
    "moment and miss it entirely, without another player's touch or bump taking the "
    "chance away first? Confirm = clean whiff, Reject = touched it / disrupted / no "
    "real attempt, Uncertain = can't tell, Bad candidate = broken clip."
)


def event_id(window: dict, file_sha256: str) -> str:
    return f"{file_sha256[:12]}:whiff_candidate:{window['anchor_frame']}:{window['player_id']}"


def player_names(listing_row: dict) -> dict[str, str]:
    """Map dump player ids (`Platform:online_id`) to display names."""
    return {
        str(player["platform_player_id"]): player["name"]
        for player in listing_row.get("players", [])
        if player.get("platform_player_id") is not None
    }


def playlist(windows: list[dict], listing: dict[str, dict], name: str) -> dict:
    replays = {}
    items = []
    for index, window in enumerate(windows):
        row = listing[window["replay_id"]]
        sha = row["file_sha256"]
        replays.setdefault(
            sha,
            {
                "id": sha,
                "label": row.get("original_file_name") or window["replay_id"],
                "path": f"/api/v1/replays/{window['replay_id']}/file",
                "meta": {"rocketSenseReplayId": window["replay_id"]},
            },
        )
        player_name = player_names(row).get(window["player_id"].split(":", 1)[-1])
        perspective = {"kind": "player", "playerId": window["player_id"], "ballCam": "on"}
        if player_name:
            perspective["playerName"] = player_name
        items.append(
            {
                "id": window["neutral_id"],
                "replay": sha,
                "start": {"kind": "time", "value": window["window_start_time"]},
                "end": {"kind": "time", "value": window["window_end_time"]},
                "label": f"Clip {index + 1:02d} · {player_name or window['player_id']}",
                "perspective": perspective,
                "meta": {
                    "eventId": event_id(window, sha),
                    "eventTypeLabel": "Whiff check",
                    "playerId": window["player_id"],
                    "playerName": player_name,
                    "neutralId": window["neutral_id"],
                    "manifest": name,
                    "target": {
                        "kind": "player-span",
                        "playerId": window["player_id"],
                        "startTime": window["core_start_time"],
                        "endTime": window["core_end_time"],
                        "eventTime": window["anchor_time"],
                        "eventFrame": window["anchor_frame"],
                    },
                },
            }
        )
    return {
        "version": 1,
        "kind": "whiff-review-playlist",
        "label": f"Whiff evaluation ({name})",
        "playback": {"advanceMode": "manual", "endMode": "stop"},
        "replays": list(replays.values()),
        "items": items,
        "meta": {"manifest": name, "question": QUESTION, "generatedBy": "whifflab playlist"},
    }


def finals_from_export(
    windows: list[dict], listing: dict[str, dict], labels: list[dict], reviewer: str | None = None
) -> dict[str, dict]:
    """Per-window final rows from a campaign `labels.jsonl` export.

    With several reviewers and no `reviewer` filter, disagreement on a window
    resolves to `uncertain`.
    """
    by_event = {event_id(w, listing[w["replay_id"]]["file_sha256"]): w for w in windows}
    statuses: dict[str, set[str]] = {}
    for row in labels:
        if reviewer is not None and row.get("reviewer") != reviewer:
            continue
        window = by_event.get(row["candidate"])
        if window is not None:
            statuses.setdefault(window["neutral_id"], set()).add(row["status"])
    finals = {}
    for window in windows:
        found = statuses.get(window["neutral_id"])
        if not found:
            continue
        status = found.pop() if len(found) == 1 else "uncertain"
        finals[window["neutral_id"]] = {
            "neutral_id": window["neutral_id"],
            "usable": status != "bad_candidate",
            "interactions": [
                {"t_replay": window["anchor_time"], "label": STATUS_LABELS.get(status, "uncertain")}
            ],
            "human_status": status,
        }
    return finals
