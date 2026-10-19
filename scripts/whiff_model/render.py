"""Render blinded visual evidence packets for one review window.

Three representations are produced from the same telemetry:

* `replay`: contact sheets of pseudo-3D frames from a ball-cam behind the focal
  car and a fixed overview camera (dense inside the core interval, sparse in
  the lead-in).
* `trajectory`: timestamped top, side, and car-frame trajectory plots.
* `combined`: both of the above plus a telemetry strip (contact gap, inferred
  touches, dodge state, ball speed).

Rendered frames are drawn only at observed replay samples; nothing is
interpolated. Every panel carries window-relative time.
"""

from __future__ import annotations

import hashlib
import json
import pathlib

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
import numpy as np  # noqa: E402
from matplotlib.patches import Circle, Polygon, Rectangle  # noqa: E402

from telemetry import (  # noqa: E402
    BALL_RADIUS,
    CEILING_Z,
    FIELD_HALF_X,
    FIELD_HALF_Y,
    GOAL_HALF_WIDTH,
    GOAL_HEIGHT,
    ReplayTelemetry,
    quat_rotate,
)

RENDERER_VERSION = "whiff-render-v1"
FOCAL_COLOR = "#e81cff"
TEAMMATE_COLOR = "#2f7bff"
OPPONENT_COLOR = "#ff8a1f"
BALL_COLOR = "#f2f2f2"
CORE_DENSE_FPS = 10.0
CONTEXT_FPS = 2.0
JPEG_QUALITY = 82


def _hitbox_corners(pos: np.ndarray, quat: np.ndarray, hitbox: dict) -> np.ndarray:
    length = hitbox.get("length", 118.0)
    width = hitbox.get("width", 84.2)
    height = hitbox.get("height", 36.2)
    angle = np.radians(hitbox.get("angle", -0.55))
    center = np.array([hitbox.get("offset", 13.88), 0.0, hitbox.get("elevation", 17.05)])
    signs = np.array(
        [[sx, sy, sz] for sx in (-1, 1) for sy in (-1, 1) for sz in (-1, 1)], dtype=float
    )
    local = signs * np.array([length / 2, width / 2, height / 2])
    cos_a, sin_a = np.cos(angle), np.sin(angle)
    rot_y = np.array([[cos_a, 0, sin_a], [0, 1, 0], [-sin_a, 0, cos_a]])
    local = local @ rot_y.T + center
    return pos + quat_rotate(np.broadcast_to(quat, (8, 4)), local)


# Corner index = 4*ix + 2*iy + iz where i* is 0 for the negative side.
_FACES = {
    "front": [4, 5, 7, 6],
    "back": [0, 1, 3, 2],
    "left": [0, 1, 5, 4],
    "right": [2, 3, 7, 6],
    "top": [1, 3, 7, 5],
    "bottom": [0, 2, 6, 4],
}


class Camera:
    def __init__(self, position: np.ndarray, target: np.ndarray, fov_degrees: float):
        self.position = position
        forward = target - position
        forward /= max(np.linalg.norm(forward), 1e-6)
        right = np.cross(forward, np.array([0.0, 0.0, 1.0]))
        if np.linalg.norm(right) < 1e-6:
            right = np.array([1.0, 0.0, 0.0])
        right /= np.linalg.norm(right)
        self.forward, self.right, self.up = forward, right, np.cross(right, forward)
        self.focal = 1.0 / np.tan(np.radians(fov_degrees) / 2)

    def project(self, points: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
        rel = np.atleast_2d(points) - self.position
        depth = rel @ self.forward
        safe = np.where(depth > 1.0, depth, np.nan)
        x = (rel @ self.right) / safe * self.focal
        y = (rel @ self.up) / safe * self.focal
        return np.stack([x, y], axis=-1), depth


def _field_lines() -> list[np.ndarray]:
    lines = []
    for x in np.arange(-FIELD_HALF_X, FIELD_HALF_X + 1, 1024):
        lines.append(np.array([[x, y, 0] for y in np.linspace(-FIELD_HALF_Y, FIELD_HALF_Y, 60)]))
    for y in np.arange(-FIELD_HALF_Y, FIELD_HALF_Y + 1, 1024):
        lines.append(np.array([[x, y, 0] for x in np.linspace(-FIELD_HALF_X, FIELD_HALF_X, 50)]))
    for z in (CEILING_Z,):
        for sx in (-1, 1):
            lines.append(
                np.array(
                    [
                        [sx * FIELD_HALF_X, y, z]
                        for y in np.linspace(-FIELD_HALF_Y, FIELD_HALF_Y, 60)
                    ]
                )
            )
        for sy in (-1, 1):
            lines.append(
                np.array(
                    [
                        [x, sy * FIELD_HALF_Y, z]
                        for x in np.linspace(-FIELD_HALF_X, FIELD_HALF_X, 50)
                    ]
                )
            )
    for sx in (-1, 1):
        for sy in (-1, 1):
            lines.append(
                np.array(
                    [
                        [sx * FIELD_HALF_X, sy * FIELD_HALF_Y, z]
                        for z in np.linspace(0, CEILING_Z, 12)
                    ]
                )
            )
    for sy in (-1, 1):
        y = sy * FIELD_HALF_Y
        lines.append(
            np.array(
                [
                    [-GOAL_HALF_WIDTH, y, 0],
                    [-GOAL_HALF_WIDTH, y, GOAL_HEIGHT],
                    [GOAL_HALF_WIDTH, y, GOAL_HEIGHT],
                    [GOAL_HALF_WIDTH, y, 0],
                ]
            )
        )
    return lines


FIELD_LINES = _field_lines()


def _player_color(r: ReplayTelemetry, focal_index: int, index: int) -> str:
    if index == focal_index:
        return FOCAL_COLOR
    same_team = r.players[index].is_team_0 == r.players[focal_index].is_team_0
    return TEAMMATE_COLOR if same_team else OPPONENT_COLOR


def _draw_3d_panel(
    ax, r: ReplayTelemetry, focal: int, i: int, camera: Camera, title: str, core: bool
):
    ax.set_facecolor("#13241a")
    ax.set_xlim(-1.6, 1.6)
    ax.set_ylim(-1.0, 1.0)
    ax.set_xticks([])
    ax.set_yticks([])
    for spine in ax.spines.values():
        spine.set_edgecolor("#ffd400" if core else "#555555")
        spine.set_linewidth(2.5 if core else 0.8)
    for line in FIELD_LINES:
        screen, depth = camera.project(line)
        ax.plot(screen[:, 0], screen[:, 1], color="#5d7a63", linewidth=0.5)

    drawables = []
    ball = r.ball_pos[i]
    if np.isfinite(ball).all():
        shadow = np.array(
            [
                [ball[0] + BALL_RADIUS * np.cos(a), ball[1] + BALL_RADIUS * np.sin(a), 0]
                for a in np.linspace(0, 2 * np.pi, 16)
            ]
        )
        sh, sh_depth = camera.project(shadow)
        if np.isfinite(sh).all():
            drawables.append((float(np.mean(sh_depth)) + 1e5, "poly", sh, "#000000", 0.45))
        center, depth = camera.project(ball)
        if np.isfinite(center).all():
            radius = camera.focal * BALL_RADIUS / depth[0]
            drawables.append((float(depth[0]), "ball", center[0], radius, None))

    for index, track in enumerate(r.players):
        pos, quat = track.pos[i], track.quat[i]
        if not (np.isfinite(pos).all() and np.isfinite(quat).all()):
            continue
        corners = _hitbox_corners(pos, quat, track.hitbox)
        screen, depth = camera.project(corners)
        color = _player_color(r, focal, index)
        for face, ids in _FACES.items():
            pts = screen[ids]
            if not np.isfinite(pts).all():
                continue
            shade = color if face != "front" else "#ffffff"
            alpha = 0.95 if face in ("top", "front") else 0.7
            drawables.append((float(np.mean(depth[ids])), "poly", pts, shade, alpha))
        if index == focal:
            label, _ = camera.project(pos + np.array([0, 0, 90.0]))
            if np.isfinite(label).all():
                drawables.append((0.0, "text", label[0], "F", None))

    for item in sorted(drawables, key=lambda d: -d[0]):
        kind = item[1]
        if kind == "poly":
            ax.add_patch(
                Polygon(
                    item[2],
                    closed=True,
                    facecolor=item[3],
                    edgecolor="#111111",
                    linewidth=0.4,
                    alpha=item[4],
                )
            )
        elif kind == "ball":
            ax.add_patch(
                Circle(item[2], item[3], facecolor=BALL_COLOR, edgecolor="#222222", linewidth=0.8)
            )
        elif kind == "text":
            ax.text(
                *item[2], item[3], color=FOCAL_COLOR, fontsize=9, fontweight="bold", ha="center"
            )
    ax.text(
        -1.55,
        0.88,
        title,
        color="white",
        fontsize=8,
        fontweight="bold",
        bbox=dict(facecolor="black", alpha=0.6, pad=1, edgecolor="none"),
    )


def _ball_cam(r: ReplayTelemetry, focal: int, i: int) -> Camera | None:
    car = r.players[focal].pos[i]
    ball = r.ball_pos[i]
    if not (np.isfinite(car).all() and np.isfinite(ball).all()):
        return None
    direction = ball - car
    horizontal = np.array([direction[0], direction[1], 0.0])
    norm = np.linalg.norm(horizontal)
    horizontal = horizontal / norm if norm > 1e-3 else r.players[focal].forward()[i]
    position = car - horizontal * 320 + np.array([0, 0, 130.0])
    return Camera(position, ball, 100.0)


def _overview_camera(r: ReplayTelemetry, focal: int, core: tuple[int, int]) -> Camera:
    lo, hi = core
    car = r.players[focal].pos[lo : hi + 1]
    ball = r.ball_pos[lo : hi + 1]
    points = np.concatenate([car[np.isfinite(car).all(1)], ball[np.isfinite(ball).all(1)]])
    mid = points.mean(axis=0) if points.size else np.zeros(3)
    motion = car[np.isfinite(car).all(1)]
    if len(motion) >= 2:
        d = motion[-1] - motion[0]
        d[2] = 0
    else:
        d = np.array([1.0, 0, 0])
    if np.linalg.norm(d) < 1e-3:
        d = np.array([1.0, 0, 0])
    d /= np.linalg.norm(d)
    perp = np.array([-d[1], d[0], 0.0])
    # Put the camera on the side of the field interior so walls do not occlude.
    if np.dot(perp, -mid) < 0:
        perp = -perp
    spread = np.ptp(points, axis=0).max() if points.size else 1000
    distance = float(np.clip(spread * 1.3 + 900, 1300, 3200))
    position = mid + perp * distance + np.array([0, 0, distance * 0.45])
    return Camera(position, mid, 70.0)


def _sample_indices(r: ReplayTelemetry, t0: float, t1: float, fps: float) -> list[int]:
    out = []
    for t in np.arange(t0, t1 + 1e-6, 1.0 / fps):
        i = r.index_at_time(t)
        if not out or out[-1] != i:
            out.append(i)
    return out


def _sheet(r, focal, indices, window, camera_fn, path, cols, heading):
    rows = int(np.ceil(len(indices) / cols))
    fig, axes = plt.subplots(rows, cols, figsize=(cols * 3.2, rows * 2.0 + 0.4), dpi=80)
    axes = np.atleast_1d(axes).ravel()
    frames = []
    for ax, i in zip(axes, indices):
        rel = r.time[i] - window["window_start_time"]
        core = window["core_start_time"] <= r.time[i] <= window["core_end_time"]
        camera = camera_fn(i)
        if camera is None:
            ax.set_facecolor("black")
            ax.text(
                0.5,
                0.5,
                f"t={rel:.2f}s\nfocal car not replicated",
                color="white",
                ha="center",
                transform=ax.transAxes,
            )
            ax.set_xticks([])
            ax.set_yticks([])
        else:
            _draw_3d_panel(ax, r, focal, i, camera, f"t={rel:.2f}s", core)
        frames.append(
            {"frame": int(r.frame[i]), "replay_time": float(r.time[i]), "t": round(float(rel), 3)}
        )
    for ax in axes[len(indices) :]:
        ax.axis("off")
    fig.suptitle(heading, fontsize=10)
    fig.tight_layout(rect=(0, 0, 1, 0.97))
    fig.savefig(path, pil_kwargs={"quality": JPEG_QUALITY})
    plt.close(fig)
    return frames


def _rel_time(r, window, i):
    return r.time[i] - window["window_start_time"]


def _trajectory_figure(r: ReplayTelemetry, window: dict, path: pathlib.Path):
    focal = window["player_index"]
    lo = r.index_at_time(window["window_start_time"])
    hi = r.index_at_time(window["window_end_time"])
    sl = slice(lo, hi + 1)
    t_rel = r.time[sl] - window["window_start_time"]
    duration = window["window_end_time"] - window["window_start_time"]
    core_lo = window["core_start_time"] - window["window_start_time"]
    core_hi = window["core_end_time"] - window["window_start_time"]
    in_core = (t_rel >= core_lo) & (t_rel <= core_hi)
    cmap = plt.get_cmap("viridis")
    norm = plt.Normalize(0, duration)
    focal_track = r.players[focal]
    car = focal_track.pos[sl]
    ball = r.ball_pos[sl]

    motion = car[np.isfinite(car).all(1)]
    axis = (motion[-1] - motion[0]) if len(motion) >= 2 else np.array([1.0, 0, 0])
    axis[2] = 0
    if np.linalg.norm(axis) < 1e-3:
        axis = np.array([1.0, 0, 0])
    axis /= np.linalg.norm(axis)

    fig, grid = plt.subplots(2, 2, figsize=(16, 13), dpi=80)
    top, side, local, field = grid[0, 0], grid[0, 1], grid[1, 0], grid[1, 1]
    axes = [top, side, local, field]
    core_points = np.concatenate([car[in_core], ball[in_core]])
    core_points = core_points[np.isfinite(core_points).all(1)]
    if core_points.size == 0:
        core_points = np.zeros((1, 3))

    def zoom(ax, xy):
        lo_xy, hi_xy = xy.min(axis=0), xy.max(axis=0)
        center = (lo_xy + hi_xy) / 2
        half = max((hi_xy - lo_xy).max() / 2 + 350, 600)
        ax.set_xlim(center[0] - half, center[0] + half)
        ax.set_ylim(center[1] - half, center[1] + half)

    def draw_paths(ax, project, title, xlabel, ylabel):
        for index, track in enumerate(r.players):
            pts = track.pos[sl]
            ok = np.isfinite(pts).all(1)
            if not ok.any():
                continue
            xy = project(pts)
            color = _player_color(r, focal, index)
            width = 2.2 if index == focal else 1.0
            ax.plot(
                xy[ok, 0],
                xy[ok, 1],
                color=color,
                linewidth=width,
                alpha=0.9 if index == focal else 0.55,
            )
            if index == focal:
                ax.scatter(
                    xy[ok & in_core, 0],
                    xy[ok & in_core, 1],
                    c=t_rel[ok & in_core],
                    cmap=cmap,
                    norm=norm,
                    s=22,
                    marker="s",
                    edgecolors=FOCAL_COLOR,
                    linewidths=0.8,
                    zorder=4,
                )
        ok = np.isfinite(ball).all(1)
        bxy = project(ball)
        ax.plot(bxy[ok, 0], bxy[ok, 1], color="#444444", linewidth=1.2, linestyle="--")
        ax.scatter(
            bxy[ok & in_core, 0],
            bxy[ok & in_core, 1],
            c=t_rel[ok & in_core],
            cmap=cmap,
            norm=norm,
            s=30,
            marker="o",
            edgecolors="black",
            linewidths=0.6,
            zorder=5,
        )
        for k in range(len(t_rel)):
            tick = round(t_rel[k] * 2) / 2
            if abs(t_rel[k] - tick) < 0.018 and np.isfinite(ball[k]).all():
                ax.annotate(
                    f"{tick:.1f}",
                    bxy[k],
                    fontsize=7,
                    color="#222222",
                    xytext=(3, 3),
                    textcoords="offset points",
                )
                if np.isfinite(car[k]).all():
                    cxy = project(car[k : k + 1])[0]
                    ax.annotate(
                        f"{tick:.1f}",
                        cxy,
                        fontsize=7,
                        color=FOCAL_COLOR,
                        xytext=(3, -9),
                        textcoords="offset points",
                    )
        ax.set_title(title, fontsize=10)
        ax.set_xlabel(xlabel)
        ax.set_ylabel(ylabel)
        ax.set_aspect("equal", adjustable="box")
        ax.grid(alpha=0.3)

    def top_project(p):
        return p[:, :2]

    draw_paths(top, top_project, "Top view zoomed on core interval (field x/y, uu)", "x", "y")
    zoom(top, top_project(core_points))
    draw_paths(field, top_project, "Whole field, full window (context)", "x", "y")
    field.set_xlim(-FIELD_HALF_X - 300, FIELD_HALF_X + 300)
    field.set_ylim(-FIELD_HALF_Y - 300, FIELD_HALF_Y + 300)
    field.add_patch(
        Rectangle(
            (-FIELD_HALF_X, -FIELD_HALF_Y),
            2 * FIELD_HALF_X,
            2 * FIELD_HALF_Y,
            fill=False,
            edgecolor="black",
            linewidth=1.5,
        )
    )
    forward = focal_track.forward()[sl]
    for k in np.flatnonzero(in_core)[::3]:
        if np.isfinite(car[k]).all() and np.isfinite(focal_track.quat[lo + k]).all():
            corners = _hitbox_corners(car[k], focal_track.quat[lo + k], focal_track.hitbox)
            top.add_patch(
                Polygon(
                    corners[[0, 2, 6, 4]][:, :2],
                    closed=True,
                    fill=False,
                    edgecolor=cmap(norm(t_rel[k])),
                    linewidth=1.0,
                )
            )
            fwd = forward[k]
            top.arrow(
                car[k, 0],
                car[k, 1],
                fwd[0] * 120,
                fwd[1] * 120,
                color=FOCAL_COLOR,
                width=4,
                head_width=25,
                alpha=0.6,
            )
        if np.isfinite(ball[k]).all():
            top.add_patch(
                Circle(
                    ball[k, :2],
                    BALL_RADIUS,
                    fill=False,
                    edgecolor=cmap(norm(t_rel[k])),
                    linewidth=0.8,
                )
            )
    top.add_patch(
        Rectangle(
            (-FIELD_HALF_X, -FIELD_HALF_Y),
            2 * FIELD_HALF_X,
            2 * FIELD_HALF_Y,
            fill=False,
            edgecolor="black",
            linewidth=1.5,
        )
    )

    def side_project(p):
        return np.stack([p @ axis, p[:, 2]], axis=-1)

    draw_paths(
        side,
        side_project,
        "Side view (along focal travel vs height)",
        "distance along travel (uu)",
        "z (uu)",
    )
    for k in np.flatnonzero(in_core)[::3]:
        if np.isfinite(ball[k]).all():
            side.add_patch(
                Circle(
                    side_project(ball[k : k + 1])[0],
                    BALL_RADIUS,
                    fill=False,
                    edgecolor=cmap(norm(t_rel[k])),
                    linewidth=0.8,
                )
            )
    side.axhline(0, color="black", linewidth=1.2)
    side_xy = side_project(core_points)
    zoom(side, side_xy)
    bottom, top_z = side.get_ylim()
    if bottom < -150:
        side.set_ylim(-150, -150 + (top_z - bottom))

    lb = focal_track.local_ball[sl]
    ok = np.isfinite(lb).all(1)
    local.plot(lb[ok, 1] * -1, lb[ok, 0], color="#666666", linewidth=1.0)
    sc = local.scatter(
        -lb[ok, 1],
        lb[ok, 0],
        c=t_rel[ok],
        cmap=cmap,
        norm=norm,
        s=np.where(in_core[ok], 30, 8),
        edgecolors="black",
        linewidths=0.3,
        zorder=4,
    )
    hb = focal_track.hitbox
    length, width, offset = hb.get("length", 118), hb.get("width", 84), hb.get("offset", 13.9)
    local.add_patch(
        Rectangle(
            (-width / 2, offset - length / 2),
            width,
            length,
            facecolor=FOCAL_COLOR,
            alpha=0.5,
            edgecolor="black",
        )
    )
    local.add_patch(
        Circle(
            (0, offset + length / 2 + BALL_RADIUS),
            BALL_RADIUS,
            fill=False,
            linestyle=":",
            edgecolor="#999999",
        )
    )
    local.annotate(
        "car nose",
        (0, offset + length / 2),
        fontsize=8,
        ha="center",
        xytext=(0, 4),
        textcoords="offset points",
    )
    for k in np.flatnonzero(ok):
        tick = round(t_rel[k] * 2) / 2
        if abs(t_rel[k] - tick) < 0.018:
            local.annotate(
                f"{tick:.1f}",
                (-lb[k, 1], lb[k, 0]),
                fontsize=7,
                xytext=(3, 3),
                textcoords="offset points",
            )
    local.set_xlim(-1500, 1500)
    local.set_ylim(-1500, 1800)
    local.set_aspect("equal")
    local.grid(alpha=0.3)
    local.set_title(
        "Ball center relative to focal car (car frame, top-down;\ncar points up the page; dotted = ball touching nose)",
        fontsize=9,
    )
    local.set_xlabel("right of car (uu)")
    local.set_ylabel("ahead of car (uu)")
    fig.colorbar(sc, ax=axes, shrink=0.5, label="window time (s); core interval drawn with markers")
    fig.suptitle(
        f"Trajectories over window 0-{duration:.1f}s. Magenta = focal car, blue = focal's teammate,\n"
        f"orange = opponents, dashed grey = ball. Core interval {core_lo:.1f}-{core_hi:.1f}s; "
        "outlines = car hitbox / ball every 0.1 s in core.",
        fontsize=10,
    )
    fig.savefig(path, pil_kwargs={"quality": JPEG_QUALITY})
    plt.close(fig)


def _telemetry_figure(r: ReplayTelemetry, window: dict, path: pathlib.Path):
    focal = window["player_index"]
    lo = r.index_at_time(window["window_start_time"])
    hi = r.index_at_time(window["window_end_time"])
    sl = slice(lo, hi + 1)
    t_rel = r.time[sl] - window["window_start_time"]
    fig, (gap_ax, speed_ax) = plt.subplots(
        2, 1, figsize=(14, 6), dpi=80, sharex=True, gridspec_kw={"height_ratios": [3, 1.3]}
    )
    for index, track in enumerate(r.players):
        color = _player_color(r, focal, index)
        gap_ax.plot(
            t_rel,
            track.contact_gap[sl],
            color=color,
            linewidth=2.2 if index == focal else 1.0,
            alpha=1 if index == focal else 0.7,
        )
    gap_ax.axhline(0, color="black", linewidth=1)
    gap_ax.set_ylim(-20, 1200)
    gap_ax.set_ylabel("ball surface to car hitbox gap (uu)\n0 = touching (estimated)")
    dodge = r.players[focal].dodge_active[sl]
    for k in np.flatnonzero(dodge):
        gap_ax.axvspan(
            t_rel[k] - 0.017, t_rel[k] + 0.017, color=FOCAL_COLOR, alpha=0.08, linewidth=0
        )
    id_to_index = {track.player_id: i for i, track in enumerate(r.players)}
    for touch in r.touches:
        ti = r.touch_index(touch)
        if lo <= ti <= hi:
            index = id_to_index.get(touch.get("player_id"))
            if index is None:
                continue
            who = (
                "focal"
                if index == focal
                else (
                    "teammate"
                    if r.players[index].is_team_0 == r.players[focal].is_team_0
                    else "opponent"
                )
            )
            color = _player_color(r, focal, index)
            gap_ax.axvline(
                r.time[ti] - window["window_start_time"], color=color, linestyle="--", linewidth=1.5
            )
            gap_ax.text(
                r.time[ti] - window["window_start_time"],
                1120,
                f"touch: {who}",
                rotation=90,
                fontsize=8,
                color=color,
                va="top",
                ha="right",
            )
    for span in (gap_ax, speed_ax):
        span.axvspan(
            window["core_start_time"] - window["window_start_time"],
            window["core_end_time"] - window["window_start_time"],
            color="#ffd400",
            alpha=0.12,
        )
    missing = ~r.players[focal].present[sl]
    if missing.any():
        gap_ax.scatter(
            t_rel[missing],
            np.full(missing.sum(), -10),
            marker="x",
            color="red",
            s=12,
            label="focal car sample missing",
        )
        gap_ax.legend(loc="upper left", fontsize=8)
    gap_ax.set_title(
        "Telemetry (MEASURED: positions/velocities; ESTIMATED: hitbox gap; INFERRED: touch markers come from a "
        "heuristic touch detector and can be wrong). Shaded magenta = focal dodge active. Yellow = core interval.",
        fontsize=9,
    )
    ball_speed = np.linalg.norm(r.ball_vel[sl], axis=1)
    car_speed = np.linalg.norm(r.players[focal].vel[sl], axis=1)
    speed_ax.plot(t_rel, ball_speed, color="#444444", label="ball speed")
    speed_ax.plot(t_rel, car_speed, color=FOCAL_COLOR, label="focal car speed")
    speed_ax.set_ylabel("uu/s")
    speed_ax.set_xlabel("window time (s)")
    speed_ax.legend(loc="upper left", fontsize=8)
    speed_ax.grid(alpha=0.3)
    gap_ax.grid(alpha=0.3)
    fig.tight_layout()
    fig.savefig(path, pil_kwargs={"quality": JPEG_QUALITY})
    plt.close(fig)


REPRESENTATIONS = ("replay", "trajectory", "combined")


def render_window(r: ReplayTelemetry, window: dict, out_dir: pathlib.Path) -> dict:
    """Render every representation for one window; returns the render manifest."""
    out_dir.mkdir(parents=True, exist_ok=True)
    focal = window["player_index"]
    core_lo = r.index_at_time(window["core_start_time"])
    core_hi = r.index_at_time(window["core_end_time"])
    overview = _overview_camera(r, focal, (core_lo, core_hi))
    images = {}
    frames = {}
    dense = _sample_indices(r, window["core_start_time"], window["core_end_time"], CORE_DENSE_FPS)
    context = _sample_indices(
        r, window["window_start_time"], window["window_end_time"], CONTEXT_FPS
    )
    frames["ball_cam_core"] = _sheet(
        r,
        focal,
        dense,
        window,
        lambda i: _ball_cam(r, focal, i),
        out_dir / "ball_cam_core.jpg",
        5,
        "Ball-cam behind focal car (magenta, 'F'), core interval at ~10 frames/s. White face = car nose.",
    )
    frames["overview_core"] = _sheet(
        r,
        focal,
        dense,
        window,
        lambda i: overview,
        out_dir / "overview_core.jpg",
        5,
        "Fixed overview camera, core interval at ~10 frames/s. Magenta = focal, blue = teammate, orange = opponents.",
    )
    frames["overview_context"] = _sheet(
        r,
        focal,
        context,
        window,
        lambda i: overview,
        out_dir / "overview_context.jpg",
        5,
        "Fixed overview camera, full window at 2 frames/s (yellow border = core interval).",
    )
    _trajectory_figure(r, window, out_dir / "trajectory.jpg")
    _telemetry_figure(r, window, out_dir / "telemetry.jpg")
    images["replay"] = ["ball_cam_core.jpg", "overview_core.jpg", "overview_context.jpg"]
    images["trajectory"] = ["trajectory.jpg"]
    images["combined"] = images["replay"] + images["trajectory"] + ["telemetry.jpg"]
    hashes = {}
    for name in set(images["combined"]):
        hashes[name] = hashlib.sha256((out_dir / name).read_bytes()).hexdigest()
    manifest = {
        "renderer_version": RENDERER_VERSION,
        "window_id": window["window_id"],
        "images": images,
        "image_sha256": hashes,
        "frame_refs": frames,
        "window_duration": window["window_end_time"] - window["window_start_time"],
        "core_interval": [
            window["core_start_time"] - window["window_start_time"],
            window["core_end_time"] - window["window_start_time"],
        ],
    }
    (out_dir / "render.json").write_text(json.dumps(manifest, indent=1))
    return manifest


CLOSEUP_FPS = 20.0


def render_closeup(
    r: ReplayTelemetry, window: dict, center_t: float, out_dir: pathlib.Path
) -> dict:
    """Dense (~20 frames/s) ball-cam and overview sheets over +/-0.5 s of `center_t` (window time)."""
    out_dir.mkdir(parents=True, exist_ok=True)
    focal = window["player_index"]
    center = window["window_start_time"] + center_t
    core_lo = r.index_at_time(window["core_start_time"])
    core_hi = r.index_at_time(window["core_end_time"])
    overview = _overview_camera(r, focal, (core_lo, core_hi))
    indices = _sample_indices(r, center - 0.5, center + 0.5, CLOSEUP_FPS)
    tag = f"{center_t:.2f}".replace(".", "_")
    names = [f"closeup_ball_cam_{tag}.jpg", f"closeup_overview_{tag}.jpg"]
    refs = {
        names[0]: _sheet(
            r,
            focal,
            indices,
            window,
            lambda i: _ball_cam(r, focal, i),
            out_dir / names[0],
            5,
            "Close-up ball-cam, every replay sample (~20-30/s) for +/-0.5 s",
        ),
        names[1]: _sheet(
            r,
            focal,
            indices,
            window,
            lambda i: overview,
            out_dir / names[1],
            5,
            "Close-up fixed overview, every replay sample for +/-0.5 s",
        ),
    }
    return {
        "images": names,
        "image_sha256": {n: hashlib.sha256((out_dir / n).read_bytes()).hexdigest() for n in names},
        "frame_refs": refs,
    }
