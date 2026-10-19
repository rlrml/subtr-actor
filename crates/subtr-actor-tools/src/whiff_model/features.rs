//! Causal whiff candidates and features, mirroring `scripts/whiff_model/features.py`.
//!
//! Candidates are local minima of a player's ball contact gap during live play.
//! Every value read for a candidate lies at or before its anchor plus
//! [`RESOLUTION_DELAY_SECONDS`], so a live detector can emit at that delay.
//! Arithmetic is in `f64` over the rounded telemetry so the Python trainer and
//! this implementation produce the same feature vectors.

use super::telemetry::WhiffTelemetry;

pub const FEATURE_VERSION: &str = "whiff-features-v1";
pub const CANDIDATE_MAX_GAP: f64 = 600.0;
pub const CANDIDATE_HALF_WINDOW_SECONDS: f64 = 0.25;
pub const RESOLUTION_DELAY_SECONDS: f64 = 0.6;
const LOOKBACK_SECONDS: f64 = 2.0;
const APPROACH_FAR_GAP: f64 = 1000.0;
const NO_TOUCH: f64 = 9.0;
const FIELD_HALF_X: f64 = 4096.0;
const FIELD_HALF_Y: f64 = 5120.0;

pub const FEATURE_NAMES: [&str; 44] = [
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
];

type V3 = [f64; 3];
const NAN3: V3 = [f64::NAN; 3];

/// Numeric view of [`WhiffTelemetry`]: missing samples are NaN.
pub struct NumericTelemetry {
    pub time: Vec<f64>,
    pub live: Vec<bool>,
    pub ball_pos: Vec<V3>,
    pub ball_vel: Vec<V3>,
    pub players: Vec<NumericPlayer>,
    /// `(player index, frame index)` for every attributed touch.
    pub touches: Vec<(usize, usize)>,
}

pub struct NumericPlayer {
    pub id: String,
    pub is_team_0: bool,
    pub pos: Vec<V3>,
    pub vel: Vec<V3>,
    pub ang_vel: Vec<V3>,
    pub forward: Vec<V3>,
    pub up: Vec<V3>,
    pub boost: Vec<f64>,
    pub boost_active: Vec<bool>,
    pub dodge_active: Vec<bool>,
    pub double_jump_active: Vec<bool>,
    pub contact_gap: Vec<f64>,
    pub local_ball: Vec<V3>,
}

fn v3(value: &Option<[f32; 3]>) -> V3 {
    value.map_or(NAN3, |v| [v[0] as f64, v[1] as f64, v[2] as f64])
}

fn scalar(value: &Option<f32>) -> f64 {
    value.map_or(f64::NAN, f64::from)
}

fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Rotate `v` by the xyzw quaternion `q` exactly as `telemetry.quat_rotate`.
fn quat_rotate(q: &Option<[f32; 4]>, v: V3) -> V3 {
    let Some(q) = q else { return NAN3 };
    let qv = [q[0] as f64, q[1] as f64, q[2] as f64];
    let w = q[3] as f64;
    let c = cross(qv, v);
    let t = [2.0 * c[0], 2.0 * c[1], 2.0 * c[2]];
    let c2 = cross(qv, t);
    [
        v[0] + w * t[0] + c2[0],
        v[1] + w * t[1] + c2[1],
        v[2] + w * t[2] + c2[2],
    ]
}

impl NumericTelemetry {
    pub fn from_telemetry(telemetry: &WhiffTelemetry) -> Self {
        let frames = &telemetry.frames;
        let players: Vec<NumericPlayer> = telemetry
            .players
            .iter()
            .map(|track| NumericPlayer {
                id: track.id.clone(),
                is_team_0: track.is_team_0,
                pos: track.pos.iter().map(v3).collect(),
                vel: track.vel.iter().map(v3).collect(),
                ang_vel: track.ang_vel.iter().map(v3).collect(),
                forward: track
                    .quat
                    .iter()
                    .map(|q| quat_rotate(q, [1.0, 0.0, 0.0]))
                    .collect(),
                up: track
                    .quat
                    .iter()
                    .map(|q| quat_rotate(q, [0.0, 0.0, 1.0]))
                    .collect(),
                boost: track.boost.iter().map(scalar).collect(),
                boost_active: track.boost_active.clone(),
                dodge_active: track.dodge_active.clone(),
                double_jump_active: track.double_jump_active.clone(),
                contact_gap: track.contact_gap.iter().map(scalar).collect(),
                local_ball: track.local_ball.iter().map(v3).collect(),
            })
            .collect();
        let touches = telemetry
            .touches
            .iter()
            .filter_map(|touch| {
                let id = touch.get("player_id")?.as_str()?;
                let player = players.iter().position(|p| p.id == id)?;
                let frame_index = touch.get("frame_index")?.as_u64()? as usize;
                Some((player, frame_index))
            })
            .collect();
        Self {
            time: frames.time.iter().map(|t| f64::from(*t)).collect(),
            live: frames.live.clone(),
            ball_pos: frames.ball_pos.iter().map(v3).collect(),
            ball_vel: frames.ball_vel.iter().map(v3).collect(),
            players,
            touches,
        }
    }

    /// `numpy.searchsorted(time, t, side="left")`.
    fn search_left(&self, t: f64) -> usize {
        self.time.partition_point(|value| *value < t)
    }

    fn search_right(&self, t: f64) -> usize {
        self.time.partition_point(|value| *value <= t)
    }

    fn index_at_time(&self, t: f64) -> usize {
        self.search_left(t).min(self.time.len().saturating_sub(1))
    }
}

fn norm(v: V3) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn finite3(v: V3) -> bool {
    v.iter().all(|x| x.is_finite())
}

fn alignment(direction: V3, vector: V3) -> f64 {
    let n = norm(vector) * norm(direction);
    if !n.is_finite() || n < 1e-6 {
        return f64::NAN;
    }
    (direction[0] * vector[0] + direction[1] * vector[1] + direction[2] * vector[2]) / n
}

fn nan_max(values: impl Iterator<Item = f64>) -> Option<f64> {
    values.filter(|v| v.is_finite()).reduce(f64::max)
}

fn nan_min(values: impl Iterator<Item = f64>) -> Option<f64> {
    values.filter(|v| v.is_finite()).reduce(f64::min)
}

/// Candidate anchors (frame indices) for one player.
pub fn candidate_indices(r: &NumericTelemetry, player_index: usize) -> Vec<usize> {
    let gap = &r.players[player_index].contact_gap;
    let mut out = Vec::new();
    let mut last_time = -1e9;
    for index in 0..gap.len() {
        if !(gap[index].is_finite() && r.live[index] && gap[index] <= CANDIDATE_MAX_GAP) {
            continue;
        }
        let t = r.time[index];
        let lo = r.search_left(t - CANDIDATE_HALF_WINDOW_SECONDS);
        let hi = r.search_right(t + CANDIDATE_HALF_WINDOW_SECONDS);
        let window_min = nan_min(gap[lo..hi].iter().copied()).unwrap_or(f64::INFINITY);
        if gap[index] > window_min || t - last_time < CANDIDATE_HALF_WINDOW_SECONDS {
            continue;
        }
        out.push(index);
        last_time = t;
    }
    out
}

/// Feature vector for one candidate, in [`FEATURE_NAMES`] order.
pub fn candidate_features(r: &NumericTelemetry, player_index: usize, anchor: usize) -> Vec<f64> {
    let track = &r.players[player_index];
    let ta = r.time[anchor];
    let cutoff = ta + RESOLUTION_DELAY_SECONDS;
    let lo = r.index_at_time(ta - LOOKBACK_SECONDS);
    let hi = r.index_at_time(cutoff);
    let gap = &track.contact_gap;
    let gap_at = |offset: f64| gap[r.index_at_time((ta + offset).min(cutoff))];

    let mut f = Vec::with_capacity(FEATURE_NAMES.len());
    let gap_anchor = gap[anchor];
    f.push(gap_anchor);
    let offsets = [-1.0, -0.5, -0.3, -0.15, 0.15, 0.3, 0.6];
    let samples: Vec<f64> = offsets.iter().map(|o| gap_at(*o)).collect();
    f.extend(&samples);
    f.push((samples[2] - gap_anchor) / 0.3);
    f.push((samples[5] - gap_anchor) / 0.3);

    let lb = track.local_ball[anchor];
    f.push(lb[0]);
    f.push(lb[1].abs());
    f.push(lb[2]);

    let pos = track.pos[anchor];
    let vel = track.vel[anchor];
    let ball_pos = r.ball_pos[anchor];
    let ball_vel = r.ball_vel[anchor];
    f.push(norm(vel));
    f.push(norm(ball_vel));
    f.push(norm(sub(vel, ball_vel)));
    let pre = r.index_at_time(ta - 0.3);
    f.push(alignment(track.forward[anchor], sub(ball_pos, pos)));
    f.push(alignment(
        track.forward[pre],
        sub(r.ball_pos[pre], track.pos[pre]),
    ));
    f.push(alignment(vel, sub(ball_pos, pos)));
    f.push(alignment(
        track.vel[pre],
        sub(r.ball_pos[pre], track.pos[pre]),
    ));
    f.push(pos[2]);
    f.push(ball_pos[2]);
    f.push(ball_pos[2] - pos[2]);
    f.push(track.up[anchor][2]);
    f.push(if finite3(pos) {
        (FIELD_HALF_X - pos[0].abs()).min(FIELD_HALF_Y - pos[1].abs())
    } else {
        f64::NAN
    });

    let recent_lo = r.index_at_time(ta - 0.8);
    let recent_hi = r.index_at_time(ta + 0.1);
    let dodge = &track.dodge_active[recent_lo..=recent_hi];
    f.push(if dodge.iter().any(|d| *d) { 1.0 } else { 0.0 });
    let last_start = (1..dodge.len()).rev().find(|k| dodge[*k] && !dodge[k - 1]);
    f.push(match last_start {
        Some(k) => r.time[recent_lo + k] - ta,
        None if dodge[0] => r.time[recent_lo] - ta,
        None => -2.0,
    });
    f.push(
        if track.double_jump_active[recent_lo..=recent_hi]
            .iter()
            .any(|d| *d)
        {
            1.0
        } else {
            0.0
        },
    );
    let boost_lo = r.index_at_time(ta - 1.0);
    let boost_window = &track.boost_active[boost_lo..=anchor];
    f.push(boost_window.iter().filter(|b| **b).count() as f64 / boost_window.len() as f64);
    f.push(track.boost[anchor]);
    f.push(norm(track.ang_vel[anchor]));

    let far = (lo..=anchor)
        .rev()
        .find(|k| gap[*k].is_nan() || gap[*k] >= APPROACH_FAR_GAP);
    f.push(far.map_or(LOOKBACK_SECONDS, |k| ta - r.time[k]));

    let dv: Vec<f64> = (lo..hi)
        .map(|k| norm(sub(r.ball_vel[k + 1], r.ball_vel[k])))
        .collect();
    let rel_anchor = anchor - lo;
    let split = rel_anchor.saturating_sub(3);
    let post_end = r.index_at_time(ta + 0.3).saturating_sub(lo).min(dv.len());
    let post = if post_end > split {
        &dv[split..post_end]
    } else {
        &dv[0..0]
    };
    let pre_start = r.index_at_time(ta - 1.0).saturating_sub(lo).min(dv.len());
    let pre_end = split.min(dv.len());
    let pre_dv = if pre_end > pre_start {
        &dv[pre_start..pre_end]
    } else {
        &dv[0..0]
    };
    f.push(nan_max(post.iter().copied()).unwrap_or(0.0));
    f.push(nan_max(pre_dv.iter().copied()).unwrap_or(0.0));
    let before = r.ball_vel[r.index_at_time(ta - 0.1)];
    let after = r.ball_vel[r.index_at_time((ta + 0.3).min(cutoff))];
    let change = 1.0 - alignment(before, after);
    f.push(if change.is_finite() { change } else { 0.0 });

    let (mut focal_dt, mut mate_dt, mut opp_dt) = (NO_TOUCH, NO_TOUCH, NO_TOUCH);
    for (index, frame_index) in &r.touches {
        let rel = r.time[*frame_index] - ta;
        if rel > RESOLUTION_DELAY_SECONDS || rel < -1.0 {
            continue;
        }
        if *index == player_index {
            if rel >= -0.3 && rel.abs() < focal_dt.abs() {
                focal_dt = rel;
            }
        } else if r.players[*index].is_team_0 == track.is_team_0 {
            if rel.abs() < mate_dt.abs() {
                mate_dt = rel;
            }
        } else if rel.abs() < opp_dt.abs() {
            opp_dt = rel;
        }
    }
    f.extend([focal_dt, mate_dt, opp_dt]);

    let near_lo = r.index_at_time(ta - 0.5);
    let near_hi = r.index_at_time((ta + 0.5).min(cutoff));
    let (mut mate_gap, mut opp_gap) = (f64::INFINITY, f64::INFINITY);
    let mut others = 0usize;
    let mut nearest = f64::INFINITY;
    for (index, other) in r.players.iter().enumerate() {
        if index == player_index {
            continue;
        }
        let value =
            nan_min(other.contact_gap[near_lo..=near_hi].iter().copied()).unwrap_or(f64::INFINITY);
        if other.is_team_0 == track.is_team_0 {
            mate_gap = mate_gap.min(value);
        } else {
            opp_gap = opp_gap.min(value);
        }
        let other_pos = other.pos[anchor];
        if finite3(other_pos) && finite3(ball_pos) && norm(sub(other_pos, ball_pos)) < 800.0 {
            others += 1;
        }
        if finite3(other_pos) && finite3(pos) {
            nearest = nearest.min(norm(sub(other_pos, pos)));
        }
    }
    f.push(mate_gap.min(5000.0));
    f.push(opp_gap.min(5000.0));
    f.push(others as f64);
    f.push(nearest.min(10000.0));
    let present = track.pos[lo..=hi].iter().filter(|p| !p[0].is_nan()).count();
    f.push(present as f64 / (hi - lo + 1) as f64);
    f.push(
        r.time[lo..=hi]
            .windows(2)
            .map(|w| w[1] - w[0])
            .reduce(f64::max)
            .unwrap_or(0.0),
    );
    debug_assert_eq!(f.len(), FEATURE_NAMES.len());
    f
}

/// One causal candidate with its features.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub player_index: usize,
    pub anchor_index: usize,
    pub anchor_time: f64,
    pub features: Vec<f64>,
}

pub fn replay_candidates(r: &NumericTelemetry) -> Vec<Candidate> {
    let mut out = Vec::new();
    for player_index in 0..r.players.len() {
        for anchor in candidate_indices(r, player_index) {
            out.push(Candidate {
                player_index,
                anchor_index: anchor,
                anchor_time: r.time[anchor],
                features: candidate_features(r, player_index, anchor),
            });
        }
    }
    out
}

#[cfg(test)]
#[path = "features_tests.rs"]
mod tests;
