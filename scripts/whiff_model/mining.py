"""Mine review windows from replay telemetry without relying on the current detector.

Several deliberately broad proposal sources are unioned per player and merged
when they refer to the same interaction. Random live-play windows and tiled
continuous segments are added so that the benchmark can measure misses that
fall outside every proposal gate.
"""

from __future__ import annotations

import hashlib
import random

import numpy as np

from telemetry import FIELD_HALF_X, FIELD_HALF_Y, ReplayTelemetry

WINDOW_LEAD_SECONDS = 4.0
WINDOW_TAIL_SECONDS = 2.0
CORE_HALF_SECONDS = 1.0
MERGE_SECONDS = 0.5
GAP_MINIMUM_HALF_WINDOW_SECONDS = 0.25
PROPOSAL_MAX_GAP = 600.0
NEAR_MISS_GAP = 150.0
DODGE_NEAR_BALL_GAP = 700.0
DISRUPTION_RADIUS_SECONDS = 1.0
DISRUPTION_MAX_GAP = 700.0

SPLIT_WEIGHTS = (("train", 60), ("dev", 20), ("test", 20))


def stable_unit(*parts: object) -> float:
    digest = hashlib.sha256(":".join(str(p) for p in parts).encode()).digest()
    return int.from_bytes(digest[:8], "big") / 2**64


def replay_split(content_id: str, salt: str = "whiff-split-v1") -> str:
    value = stable_unit(salt, content_id) * 100
    cumulative = 0
    for name, weight in SPLIT_WEIGHTS:
        cumulative += weight
        if value < cumulative:
            return name
    return SPLIT_WEIGHTS[-1][0]


def gap_minima(
    gap: np.ndarray, time: np.ndarray, live: np.ndarray, max_gap: float, half_window: float
) -> list[int]:
    """Indices of local contact-gap minima below `max_gap` during live play.

    A minimum must be the smallest finite gap within `half_window` seconds on
    both sides. Flat runs (e.g. sustained contact at gap 0) yield their first
    frame.
    """
    finite = np.isfinite(gap) & live
    candidates = np.flatnonzero(finite & (gap <= max_gap))
    minima = []
    last_kept = -1
    for index in candidates:
        lo = np.searchsorted(time, time[index] - half_window)
        hi = np.searchsorted(time, time[index] + half_window, side="right")
        window = gap[lo:hi]
        window = window[np.isfinite(window)]
        if window.size == 0 or gap[index] > window.min():
            continue
        if last_kept >= 0 and time[index] - time[last_kept] < half_window:
            continue
        minima.append(int(index))
        last_kept = index
    return minima


def _window_bounds(r: ReplayTelemetry, anchor_time: float) -> tuple[int, int]:
    start = r.index_at_time(anchor_time - WINDOW_LEAD_SECONDS)
    end = r.index_at_time(anchor_time + WINDOW_TAIL_SECONDS)
    return start, end


def scenario_tags(r: ReplayTelemetry, player_index: int, anchor: int) -> list[str]:
    track = r.players[player_index]
    tags = []
    pos = track.pos[anchor]
    ball = r.ball_pos[anchor]
    if np.isfinite(pos).all():
        if pos[2] > 300:
            tags.append("aerial")
        if (abs(pos[0]) > FIELD_HALF_X - 300 or abs(pos[1]) > FIELD_HALF_Y - 300) and pos[2] > 150:
            tags.append("wall")
        if pos[2] <= 300:
            tags.append("grounded_car")
    if np.isfinite(ball).all() and ball[2] > 400:
        tags.append("ball_high")
    if np.isfinite(ball).all():
        others = 0
        for other_index, other in enumerate(r.players):
            if other_index == player_index:
                continue
            other_pos = other.pos[anchor]
            if np.isfinite(other_pos).all() and np.linalg.norm(other_pos - ball) < 700:
                others += 1
        if others >= 1:
            tags.append("congested")
    lb = track.local_ball[anchor]
    if np.isfinite(lb).all():
        if lb[0] < -50:
            tags.append("ball_behind_car")
        elif abs(lb[1]) > abs(lb[0]):
            tags.append("ball_beside_car")
    start, end = _window_bounds(r, r.time[anchor])
    present = track.present[start : end + 1]
    gaps = np.diff(r.time[start : end + 1])
    if present.mean() < 0.95 or (gaps.size and gaps.max() > 0.15):
        tags.append("poor_telemetry")
    return tags


def _add(proposals: dict, player_index: int, anchor: int, source: str, r: ReplayTelemetry):
    proposals.setdefault(player_index, []).append((float(r.time[anchor]), anchor, source))


def proposals_for_replay(r: ReplayTelemetry) -> list[dict]:
    raw: dict[int, list[tuple[float, int, str]]] = {}
    id_to_index = {track.player_id: i for i, track in enumerate(r.players)}

    for i, track in enumerate(r.players):
        for anchor in gap_minima(
            track.contact_gap, r.time, r.live, PROPOSAL_MAX_GAP, GAP_MINIMUM_HALF_WINDOW_SECONDS
        ):
            gap = track.contact_gap[anchor]
            _add(raw, i, anchor, "gap_min_near" if gap <= NEAR_MISS_GAP else "gap_min_wide", r)
        dodge_starts = np.flatnonzero(track.dodge_active[1:] & ~track.dodge_active[:-1]) + 1
        for start in dodge_starts:
            if not r.live[start]:
                continue
            lo = r.index_at_time(r.time[start] - 0.3)
            hi = r.index_at_time(r.time[start] + 0.7)
            segment = track.contact_gap[lo : hi + 1]
            if not np.isfinite(segment).any() or np.nanmin(segment) > DODGE_NEAR_BALL_GAP:
                continue
            _add(raw, i, lo + int(np.nanargmin(segment)), "dodge_near_ball", r)

    for event in r.whiff_events:
        index = id_to_index.get(event["player_id"])
        anchor = r.index_for_frame(event["frame"])
        if index is not None and anchor is not None:
            _add(raw, index, anchor, "detector_whiff", r)
    for event in r.beaten_to_ball_events:
        index = id_to_index.get(event["player_id"])
        anchor = r.index_for_frame(event["frame"])
        if index is not None and anchor is not None:
            _add(raw, index, anchor, "detector_beaten_to_ball", r)

    for touch in r.touches:
        touch_index = r.touch_index(touch)
        if not r.live[touch_index]:
            continue
        toucher = id_to_index.get(touch.get("player_id"))
        if toucher is not None:
            _add(raw, toucher, touch_index, "focal_touch", r)
        lo = r.index_at_time(r.time[touch_index] - DISRUPTION_RADIUS_SECONDS)
        hi = r.index_at_time(r.time[touch_index] + DISRUPTION_RADIUS_SECONDS)
        for i, track in enumerate(r.players):
            if i == toucher:
                continue
            segment = track.contact_gap[lo : hi + 1]
            if np.isfinite(segment).any() and np.nanmin(segment) <= DISRUPTION_MAX_GAP:
                _add(raw, i, lo + int(np.nanargmin(segment)), "other_touch_nearby", r)

    merged = []
    for player_index, items in raw.items():
        items.sort()
        groups: list[list[tuple[float, int, str]]] = []
        for item in items:
            if groups and item[0] - groups[-1][-1][0] <= MERGE_SECONDS:
                groups[-1].append(item)
            else:
                groups.append([item])
        track = r.players[player_index]
        for group in groups:
            indices = [anchor for _, anchor, _ in group]
            gaps = track.contact_gap[indices]
            anchor = indices[int(np.nanargmin(gaps))] if np.isfinite(gaps).any() else indices[0]
            merged.append(
                make_window(r, player_index, anchor, sorted({source for *_, source in group}))
            )
    return merged


def make_window(r: ReplayTelemetry, player_index: int, anchor: int, sources: list[str]) -> dict:
    track = r.players[player_index]
    anchor_time = float(r.time[anchor])
    start, end = _window_bounds(r, anchor_time)
    gap = track.contact_gap[anchor]
    return {
        "window_id": hashlib.sha256(
            f"{r.content_id}:{track.player_id}:{int(r.frame[anchor])}".encode()
        ).hexdigest()[:16],
        "replay_id": r.replay_id,
        "content_id": r.content_id,
        "player_id": track.player_id,
        "player_index": player_index,
        "anchor_index": int(anchor),
        "anchor_frame": int(r.frame[anchor]),
        "anchor_time": anchor_time,
        "window_start_time": float(r.time[start]),
        "window_end_time": float(r.time[end]),
        "core_start_time": anchor_time - CORE_HALF_SECONDS,
        "core_end_time": anchor_time + CORE_HALF_SECONDS,
        "live_fraction": float(r.live[start : end + 1].mean()),
        "anchor_gap": None if not np.isfinite(gap) else float(gap),
        "sources": sources,
        "tags": scenario_tags(r, player_index, anchor),
    }


def random_windows(r: ReplayTelemetry, per_player: int, seed: str) -> list[dict]:
    rng = random.Random(f"{seed}:{r.content_id}")
    live_indices = np.flatnonzero(r.live)
    out = []
    if live_indices.size == 0:
        return out
    for player_index in range(len(r.players)):
        for _ in range(per_player):
            anchor = int(live_indices[rng.randrange(live_indices.size)])
            out.append(make_window(r, player_index, anchor, ["random_live"]))
    return out


def tiled_windows(r: ReplayTelemetry, segment_seconds: float, seed: str) -> list[dict]:
    """Contiguous core intervals covering one random live segment per player."""
    rng = random.Random(f"tile:{seed}:{r.content_id}")
    out = []
    live = r.live
    for player_index in range(len(r.players)):
        for _attempt in range(50):
            start_time = rng.uniform(r.time[0] + WINDOW_LEAD_SECONDS, r.time[-1] - segment_seconds)
            lo = r.index_at_time(start_time)
            hi = r.index_at_time(start_time + segment_seconds)
            if live[lo : hi + 1].mean() > 0.95:
                break
        else:
            continue
        anchor_time = start_time + CORE_HALF_SECONDS
        segment_id = hashlib.sha256(
            f"{r.content_id}:{player_index}:{start_time:.3f}".encode()
        ).hexdigest()[:12]
        while anchor_time + CORE_HALF_SECONDS <= start_time + segment_seconds + 1e-6:
            anchor = r.index_at_time(anchor_time)
            window = make_window(r, player_index, anchor, ["tiled_segment"])
            window["core_start_time"] = anchor_time - CORE_HALF_SECONDS
            window["core_end_time"] = anchor_time + CORE_HALF_SECONDS
            window["anchor_time"] = anchor_time
            window["segment_id"] = segment_id
            window["window_id"] = hashlib.sha256(
                f"{segment_id}:{anchor_time:.3f}".encode()
            ).hexdigest()[:16]
            out.append(window)
            anchor_time += 2 * CORE_HALF_SECONDS
    return out


STRATA = (
    ("detector_whiff", lambda w: "detector_whiff" in w["sources"]),
    (
        "near_miss_no_touch",
        lambda w: "gap_min_near" in w["sources"] and "focal_touch" not in w["sources"],
    ),
    ("other_touch_nearby", lambda w: "other_touch_nearby" in w["sources"]),
    ("dodge_near_ball", lambda w: "dodge_near_ball" in w["sources"]),
    ("focal_touch", lambda w: "focal_touch" in w["sources"]),
    ("wide_approach", lambda w: "gap_min_wide" in w["sources"]),
    ("random_live", lambda w: "random_live" in w["sources"]),
)


def stratum(window: dict) -> str:
    for name, predicate in STRATA:
        if predicate(window):
            return name
    return "other"
