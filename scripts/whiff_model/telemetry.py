"""Load `whiff_dataset_dump` replay telemetry into numpy arrays."""

from __future__ import annotations

import gzip
import json
import pathlib
from dataclasses import dataclass, field

import numpy as np

BALL_RADIUS = 92.75
FIELD_HALF_X = 4096.0
FIELD_HALF_Y = 5120.0
CEILING_Z = 2044.0
GOAL_HALF_WIDTH = 893.0
GOAL_HEIGHT = 642.0


def _vec_array(values, width: int) -> np.ndarray:
    out = np.full((len(values), width), np.nan, dtype=np.float64)
    for index, value in enumerate(values):
        if value is not None:
            out[index] = value
    return out


def _float_array(values) -> np.ndarray:
    return np.array([np.nan if v is None else v for v in values], dtype=np.float64)


def quat_rotate(quat: np.ndarray, vector: np.ndarray) -> np.ndarray:
    """Rotate `vector` (..., 3) by unit quaternions `quat` (..., 4) in xyzw order."""
    q_vec = quat[..., :3]
    w = quat[..., 3:4]
    t = 2.0 * np.cross(q_vec, vector)
    return vector + w * t + np.cross(q_vec, t)


@dataclass
class PlayerTrack:
    player_id: str
    is_team_0: bool
    hitbox: dict
    pos: np.ndarray
    vel: np.ndarray
    ang_vel: np.ndarray
    quat: np.ndarray
    boost: np.ndarray
    boost_active: np.ndarray
    dodge_active: np.ndarray
    double_jump_active: np.ndarray
    powerslide_active: np.ndarray
    contact_gap: np.ndarray
    local_ball: np.ndarray

    @property
    def present(self) -> np.ndarray:
        return ~np.isnan(self.pos[:, 0])

    def forward(self) -> np.ndarray:
        axis = np.zeros_like(self.pos)
        axis[:, 0] = 1.0
        return quat_rotate(self.quat, axis)

    def up(self) -> np.ndarray:
        axis = np.zeros_like(self.pos)
        axis[:, 2] = 1.0
        return quat_rotate(self.quat, axis)


@dataclass
class ReplayTelemetry:
    replay_id: str
    content_id: str
    source_path: str
    frame: np.ndarray
    time: np.ndarray
    live: np.ndarray
    ball_pos: np.ndarray
    ball_vel: np.ndarray
    players: list[PlayerTrack]
    touches: list[dict]
    whiff_events: list[dict]
    beaten_to_ball_events: list[dict]
    frame_lookup: dict[int, int] = field(default_factory=dict)

    def __post_init__(self):
        self.frame_lookup = {int(f): i for i, f in enumerate(self.frame)}

    def player(self, player_id: str) -> PlayerTrack:
        for track in self.players:
            if track.player_id == player_id:
                return track
        raise KeyError(player_id)

    def index_at_time(self, time: float) -> int:
        index = int(np.searchsorted(self.time, time))
        return min(max(index, 0), len(self.time) - 1)

    def index_for_frame(self, frame: int) -> int | None:
        return self.frame_lookup.get(int(frame))

    def touch_index(self, touch: dict) -> int:
        if "frame_index" in touch:
            return int(touch["frame_index"])
        found = self.index_for_frame(touch["frame"])
        return found if found is not None else self.index_at_time(touch["time"])


def load_dump(path: pathlib.Path) -> ReplayTelemetry:
    with gzip.open(path, "rt") as handle:
        doc = json.load(handle)
    frames = doc["frames"]
    players = []
    for raw in doc["players"]:
        players.append(
            PlayerTrack(
                player_id=raw["id"],
                is_team_0=bool(raw["is_team_0"]),
                hitbox=raw.get("hitbox") or {},
                pos=_vec_array(raw["pos"], 3),
                vel=_vec_array(raw["vel"], 3),
                ang_vel=_vec_array(raw["ang_vel"], 3),
                quat=_vec_array(raw["quat"], 4),
                boost=_float_array(raw["boost"]),
                boost_active=np.array(raw["boost_active"], dtype=bool),
                dodge_active=np.array(raw["dodge_active"], dtype=bool),
                double_jump_active=np.array(raw["double_jump_active"], dtype=bool),
                powerslide_active=np.array(raw["powerslide_active"], dtype=bool),
                contact_gap=_float_array(raw["contact_gap"]),
                local_ball=_vec_array(raw["local_ball"], 3),
            )
        )
    return ReplayTelemetry(
        replay_id=doc["replay_id"],
        content_id=doc["content_id"],
        source_path=doc["source_path"],
        frame=np.array(frames["frame"], dtype=np.int64),
        time=np.array(frames["time"], dtype=np.float64),
        live=np.array(frames["live"], dtype=bool),
        ball_pos=_vec_array(frames["ball_pos"], 3),
        ball_vel=_vec_array(frames["ball_vel"], 3),
        players=players,
        touches=doc["touches"],
        whiff_events=doc["whiff_events"],
        beaten_to_ball_events=doc["beaten_to_ball_events"],
    )


def dump_path(dump_dir: pathlib.Path, replay_id: str) -> pathlib.Path:
    return dump_dir / f"{replay_id}.frames.json.gz"
