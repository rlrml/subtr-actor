"""Causal candidate generation and telemetry features for the whiff model.

Candidates are local minima of the focal car's ball contact gap during live
play. A candidate can be resolved `RESOLUTION_DELAY_SECONDS` after its anchor:
both the minimum test and every feature only read frames up to that cutoff, so
the same computation can run in a live detector. Candidate-source names,
detector verdicts, player identities, and ranks are never features.
"""

from __future__ import annotations

import numpy as np

from telemetry import FIELD_HALF_X, FIELD_HALF_Y, ReplayTelemetry

FEATURE_VERSION = "whiff-features-v1"
CANDIDATE_MAX_GAP = 600.0
CANDIDATE_HALF_WINDOW_SECONDS = 0.25
RESOLUTION_DELAY_SECONDS = 0.6
LOOKBACK_SECONDS = 2.0
APPROACH_FAR_GAP = 1000.0


def candidate_indices(r: ReplayTelemetry, player_index: int) -> list[int]:
    """Gap minima that are the lowest value within +/-0.25 s (a subset of the cutoff)."""
    gap = r.players[player_index].contact_gap
    finite = np.isfinite(gap) & r.live
    out = []
    last_time = -1e9
    for index in np.flatnonzero(finite & (gap <= CANDIDATE_MAX_GAP)):
        t = r.time[index]
        lo = np.searchsorted(r.time, t - CANDIDATE_HALF_WINDOW_SECONDS)
        hi = np.searchsorted(r.time, t + CANDIDATE_HALF_WINDOW_SECONDS, side="right")
        window = gap[lo:hi]
        window = window[np.isfinite(window)]
        if gap[index] > window.min() or t - last_time < CANDIDATE_HALF_WINDOW_SECONDS:
            continue
        out.append(int(index))
        last_time = t
    return out


def _at(r: ReplayTelemetry, values: np.ndarray, t: float, cutoff: float):
    t = min(t, cutoff)
    return values[r.index_at_time(t)]


def _nan_to(value, default=np.nan):
    return default if value is None or not np.isfinite(value) else float(value)


FEATURE_NAMES = [
    "gap_at_anchor",
    "gap_m1_0",
    "gap_m0_5",
    "gap_m0_3",
    "gap_m0_15",
    "gap_p0_15",
    "gap_p0_3",
    "gap_p0_6",
    "closing_rate_0_3",
    "opening_rate_0_3",
    "local_ball_x",
    "local_ball_y_abs",
    "local_ball_z",
    "car_speed",
    "ball_speed",
    "rel_speed",
    "forward_alignment",
    "forward_alignment_m0_3",
    "velocity_alignment",
    "velocity_alignment_m0_3",
    "car_z",
    "ball_z",
    "ball_z_minus_car_z",
    "car_up_z",
    "wall_distance",
    "dodge_recent",
    "dodge_start_rel",
    "double_jump_recent",
    "boost_fraction_m1",
    "boost_amount",
    "car_ang_speed",
    "approach_duration",
    "ball_dv_max_post",
    "ball_dv_max_pre",
    "ball_dir_change_post",
    "focal_touch_dt",
    "teammate_touch_dt",
    "opponent_touch_dt",
    "teammate_min_gap",
    "opponent_min_gap",
    "others_near_ball",
    "nearest_car_distance",
    "present_fraction",
    "max_sample_dt",
]

NO_TOUCH = 9.0


def _alignment(direction: np.ndarray, vector: np.ndarray) -> float:
    norm = np.linalg.norm(vector) * np.linalg.norm(direction)
    if not np.isfinite(norm) or norm < 1e-6:
        return np.nan
    return float(np.dot(direction, vector) / norm)


def candidate_features(r: ReplayTelemetry, player_index: int, anchor: int) -> np.ndarray:
    track = r.players[player_index]
    ta = r.time[anchor]
    cutoff = ta + RESOLUTION_DELAY_SECONDS
    lo = r.index_at_time(ta - LOOKBACK_SECONDS)
    hi = r.index_at_time(cutoff)
    gap = track.contact_gap

    def gap_at(offset):
        return _nan_to(_at(r, gap, ta + offset, cutoff))

    features = {}
    features["gap_at_anchor"] = _nan_to(gap[anchor])
    for name, offset in (
        ("gap_m1_0", -1.0),
        ("gap_m0_5", -0.5),
        ("gap_m0_3", -0.3),
        ("gap_m0_15", -0.15),
        ("gap_p0_15", 0.15),
        ("gap_p0_3", 0.3),
        ("gap_p0_6", 0.6),
    ):
        features[name] = gap_at(offset)
    features["closing_rate_0_3"] = (features["gap_m0_3"] - features["gap_at_anchor"]) / 0.3
    features["opening_rate_0_3"] = (features["gap_p0_3"] - features["gap_at_anchor"]) / 0.3

    lb = track.local_ball[anchor]
    features["local_ball_x"] = _nan_to(lb[0])
    features["local_ball_y_abs"] = _nan_to(abs(lb[1]))
    features["local_ball_z"] = _nan_to(lb[2])

    pos, vel = track.pos[anchor], track.vel[anchor]
    ball_pos, ball_vel = r.ball_pos[anchor], r.ball_vel[anchor]
    features["car_speed"] = _nan_to(np.linalg.norm(vel))
    features["ball_speed"] = _nan_to(np.linalg.norm(ball_vel))
    features["rel_speed"] = _nan_to(np.linalg.norm(vel - ball_vel))
    forward = track.forward()
    pre = r.index_at_time(ta - 0.3)
    features["forward_alignment"] = _alignment(forward[anchor], ball_pos - pos)
    features["forward_alignment_m0_3"] = _alignment(forward[pre], r.ball_pos[pre] - track.pos[pre])
    features["velocity_alignment"] = _alignment(vel, ball_pos - pos)
    features["velocity_alignment_m0_3"] = _alignment(
        track.vel[pre], r.ball_pos[pre] - track.pos[pre]
    )
    features["car_z"] = _nan_to(pos[2])
    features["ball_z"] = _nan_to(ball_pos[2])
    features["ball_z_minus_car_z"] = _nan_to(ball_pos[2] - pos[2])
    features["car_up_z"] = _nan_to(track.up()[anchor][2])
    features["wall_distance"] = _nan_to(
        min(FIELD_HALF_X - abs(pos[0]), FIELD_HALF_Y - abs(pos[1]))
        if np.isfinite(pos).all()
        else np.nan
    )

    recent = slice(r.index_at_time(ta - 0.8), r.index_at_time(ta + 0.1) + 1)
    dodge = track.dodge_active[recent]
    features["dodge_recent"] = float(dodge.any())
    starts = np.flatnonzero(dodge[1:] & ~dodge[:-1]) + 1
    first = recent.start + (int(starts[-1]) if starts.size else (0 if dodge[:1].any() else -1))
    features["dodge_start_rel"] = (
        float(r.time[first] - ta) if (starts.size or dodge[:1].any()) else -2.0
    )
    features["double_jump_recent"] = float(track.double_jump_active[recent].any())
    boost_window = slice(r.index_at_time(ta - 1.0), anchor + 1)
    features["boost_fraction_m1"] = float(track.boost_active[boost_window].mean())
    features["boost_amount"] = _nan_to(track.boost[anchor])
    features["car_ang_speed"] = _nan_to(np.linalg.norm(track.ang_vel[anchor]))

    far = np.flatnonzero(~(gap[lo : anchor + 1] < APPROACH_FAR_GAP))
    features["approach_duration"] = (
        float(ta - r.time[lo + far[-1]]) if far.size else LOOKBACK_SECONDS
    )

    dv = np.linalg.norm(np.diff(r.ball_vel[lo : hi + 1], axis=0), axis=1)
    rel_anchor = anchor - lo
    post = dv[max(rel_anchor - 3, 0) : r.index_at_time(ta + 0.3) - lo]
    pre_dv = dv[r.index_at_time(ta - 1.0) - lo : max(rel_anchor - 3, 0)]
    features["ball_dv_max_post"] = (
        float(np.nanmax(post)) if post.size and np.isfinite(post).any() else 0.0
    )
    features["ball_dv_max_pre"] = (
        float(np.nanmax(pre_dv)) if pre_dv.size and np.isfinite(pre_dv).any() else 0.0
    )
    before = r.ball_vel[r.index_at_time(ta - 0.1)]
    after = r.ball_vel[r.index_at_time(min(ta + 0.3, cutoff))]
    features["ball_dir_change_post"] = _nan_to(1.0 - _alignment(before, after), 0.0)

    focal_dt, mate_dt, opp_dt = NO_TOUCH, NO_TOUCH, NO_TOUCH
    id_to_index = {t.player_id: i for i, t in enumerate(r.players)}
    for touch in r.touches:
        index = id_to_index.get(touch.get("player_id"))
        if index is None:
            continue
        tt = r.time[r.touch_index(touch)]
        rel = tt - ta
        if rel > RESOLUTION_DELAY_SECONDS or rel < -1.0:
            continue
        if index == player_index:
            if rel >= -0.3 and abs(rel) < abs(focal_dt):
                focal_dt = rel
        elif r.players[index].is_team_0 == track.is_team_0:
            if abs(rel) < abs(mate_dt):
                mate_dt = rel
        elif abs(rel) < abs(opp_dt):
            opp_dt = rel
    features["focal_touch_dt"] = focal_dt
    features["teammate_touch_dt"] = mate_dt
    features["opponent_touch_dt"] = opp_dt

    near = slice(r.index_at_time(ta - 0.5), r.index_at_time(min(ta + 0.5, cutoff)) + 1)
    mate_gap, opp_gap, others, nearest = np.inf, np.inf, 0, np.inf
    for index, other in enumerate(r.players):
        if index == player_index:
            continue
        segment = other.contact_gap[near]
        value = float(np.nanmin(segment)) if np.isfinite(segment).any() else np.inf
        if other.is_team_0 == track.is_team_0:
            mate_gap = min(mate_gap, value)
        else:
            opp_gap = min(opp_gap, value)
        other_pos = other.pos[anchor]
        if np.isfinite(other_pos).all() and np.isfinite(ball_pos).all():
            others += int(np.linalg.norm(other_pos - ball_pos) < 800)
        if np.isfinite(other_pos).all() and np.isfinite(pos).all():
            nearest = min(nearest, float(np.linalg.norm(other_pos - pos)))
    features["teammate_min_gap"] = min(mate_gap, 5000.0)
    features["opponent_min_gap"] = min(opp_gap, 5000.0)
    features["others_near_ball"] = float(others)
    features["nearest_car_distance"] = min(nearest, 10000.0)
    features["present_fraction"] = float(track.present[lo : hi + 1].mean())
    steps = np.diff(r.time[lo : hi + 1])
    features["max_sample_dt"] = float(steps.max()) if steps.size else 0.0
    return np.array([features[name] for name in FEATURE_NAMES], dtype=np.float64)


def replay_candidates(r: ReplayTelemetry) -> list[dict]:
    out = []
    for player_index, track in enumerate(r.players):
        for anchor in candidate_indices(r, player_index):
            out.append(
                {
                    "replay_id": r.replay_id,
                    "player_id": track.player_id,
                    "player_index": player_index,
                    "anchor_index": anchor,
                    "anchor_frame": int(r.frame[anchor]),
                    "anchor_time": float(r.time[anchor]),
                    "features": candidate_features(r, player_index, anchor),
                }
            )
    return out
