//! Causal whiff candidates and their telemetry features.
//!
//! A candidate is a local minimum of a player's ball contact gap during live
//! play. Every value read for a candidate lies at or before the first frame at
//! least [`WHIFF_RESOLUTION_DELAY_SECONDS`] after its anchor, so the detector
//! emits with a bounded delay. `scripts/whiff_model` trains on these exact
//! vectors (dumped by `whiff_candidate_dump`), so there is a single feature
//! implementation.

use glam::{DQuat, DVec3};

use super::*;

pub const WHIFF_FEATURE_VERSION: &str = "whiff-features-v2";
pub const WHIFF_RESOLUTION_DELAY_SECONDS: f64 = 0.6;
const CANDIDATE_MAX_GAP: f64 = 600.0;
const CANDIDATE_HALF_WINDOW_SECONDS: f64 = 0.25;
const LOOKBACK_SECONDS: f64 = 2.0;
const HISTORY_SECONDS: f64 = LOOKBACK_SECONDS + WHIFF_RESOLUTION_DELAY_SECONDS + 0.3;
const APPROACH_FAR_GAP: f64 = 1000.0;
const NO_TOUCH: f64 = 9.0;
const FIELD_HALF_X: f64 = 4096.0;
const FIELD_HALF_Y: f64 = 5120.0;
const AIRBORNE_Z: f64 = 60.0;
const BALL_RADIUS: f64 = 92.75;
const GRAVITY: DVec3 = DVec3::new(0.0, 0.0, -650.0);

pub const WHIFF_FEATURE_NAMES: [&str; WHIFF_FEATURE_COUNT] = [
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
    "other_touch_rel",
    "other_touch_focal_gap",
    "other_touch_ball_dv",
    "cf_min_dist",
    "cf_minus_actual",
    "dodge_minus_other_touch",
    "other_touch_is_mate",
    "pred_miss_m0_6",
    "pred_miss_m0_3",
    "pred_miss_m0_15",
    "pred_tca_m0_3",
    "accel_align_0_5",
    "accel_mag_0_5",
    "fwd_align_mean_1_0",
    "fwd_align_trend",
    "airborne_time",
    "boost_frac_0_5",
    "ang_speed_mean_0_5",
    "closing_m0_15",
    "focal_prev_touch_dt",
    "gap_max_since_prev_touch",
    "min_center_dist",
];
pub const WHIFF_FEATURE_COUNT: usize = 66;

pub type WhiffFeatures = [f64; WHIFF_FEATURE_COUNT];

const NAN3: DVec3 = DVec3::splat(f64::NAN);

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct WhiffPlayerSample {
    pub(crate) position: DVec3,
    pub(crate) velocity: DVec3,
    pub(crate) angular_velocity: DVec3,
    pub(crate) rotation: DQuat,
    pub(crate) boost: f64,
    pub(crate) boost_active: bool,
    pub(crate) dodge_active: bool,
    pub(crate) double_jump_active: bool,
    /// Ball-surface to oriented-hitbox gap (0 when touching).
    pub(crate) contact_gap: f64,
    /// Ball-center to oriented-hitbox distance.
    pub(crate) hitbox_distance: f64,
    pub(crate) local_ball: DVec3,
}

impl WhiffPlayerSample {
    fn forward(&self) -> DVec3 {
        self.rotation * DVec3::X
    }

    fn up(&self) -> DVec3 {
        self.rotation * DVec3::Z
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct WhiffHistoryFrame {
    pub(crate) time: f64,
    pub(crate) frame: usize,
    pub(crate) live: bool,
    pub(crate) ball_position: DVec3,
    pub(crate) ball_velocity: DVec3,
    /// Indexed by player slot; `None` when the car has no rigid body.
    pub(crate) players: Vec<Option<WhiffPlayerSample>>,
    pub(crate) touch_slots: Vec<usize>,
}

impl WhiffHistoryFrame {
    fn player(&self, slot: usize) -> Option<&WhiffPlayerSample> {
        self.players.get(slot).and_then(Option::as_ref)
    }

    fn gap(&self, slot: usize) -> f64 {
        self.player(slot).map_or(f64::NAN, |p| p.contact_gap)
    }

    fn position(&self, slot: usize) -> DVec3 {
        self.player(slot).map_or(NAN3, |p| p.position)
    }

    fn velocity(&self, slot: usize) -> DVec3 {
        self.player(slot).map_or(NAN3, |p| p.velocity)
    }
}

fn dvec(v: glam::Vec3) -> DVec3 {
    v.as_dvec3()
}

fn finite3(v: DVec3) -> bool {
    v.is_finite()
}

fn nan_or(value: f64, default: f64) -> f64 {
    if value.is_finite() { value } else { default }
}

fn alignment(direction: DVec3, vector: DVec3) -> f64 {
    let norm = direction.length() * vector.length();
    if !norm.is_finite() || norm < 1e-6 {
        return f64::NAN;
    }
    direction.dot(vector) / norm
}

fn far_or_unknown(gap: f64) -> bool {
    gap.is_nan() || gap >= APPROACH_FAR_GAP
}

fn nan_min(values: impl Iterator<Item = f64>) -> f64 {
    values.filter(|v| v.is_finite()).fold(
        f64::NAN,
        |acc, v| if acc.is_nan() || v < acc { v } else { acc },
    )
}

fn nan_max(values: impl Iterator<Item = f64>) -> f64 {
    values.filter(|v| v.is_finite()).fold(
        f64::NAN,
        |acc, v| if acc.is_nan() || v > acc { v } else { acc },
    )
}

fn nan_mean(values: impl Iterator<Item = f64>) -> f64 {
    let (sum, count) = values
        .filter(|v| v.is_finite())
        .fold((0.0, 0usize), |(sum, count), v| (sum + v, count + 1));
    if count == 0 {
        f64::NAN
    } else {
        sum / count as f64
    }
}

/// Bounded per-frame history plus the player-slot table the samples index.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct WhiffHistory {
    frames: VecDeque<WhiffHistoryFrame>,
    /// Sequence number of `frames[0]`.
    base_seq: usize,
    slots: Vec<(PlayerId, bool)>,
}

impl WhiffHistory {
    pub(crate) fn slot_player(&self, slot: usize) -> &(PlayerId, bool) {
        &self.slots[slot]
    }

    pub(crate) fn slot_count(&self) -> usize {
        self.slots.len()
    }

    fn slot_for(&mut self, player_id: &PlayerId, is_team_0: bool) -> usize {
        if let Some(slot) = self.slots.iter().position(|(id, _)| id == player_id) {
            self.slots[slot].1 = is_team_0;
            return slot;
        }
        self.slots.push((player_id.clone(), is_team_0));
        self.slots.len() - 1
    }

    /// Sequence number of the most recent frame.
    pub(crate) fn last_seq(&self) -> Option<usize> {
        (!self.frames.is_empty()).then(|| self.base_seq + self.frames.len() - 1)
    }

    pub(crate) fn frame(&self, seq: usize) -> Option<&WhiffHistoryFrame> {
        seq.checked_sub(self.base_seq)
            .and_then(|index| self.frames.get(index))
    }

    pub(crate) fn first_seq(&self) -> usize {
        self.base_seq
    }

    pub(crate) fn clear(&mut self) {
        self.base_seq += self.frames.len();
        self.frames.clear();
    }

    pub(crate) fn push(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        controls: &PlayerControlState,
        touch_state: &TouchState,
        live: bool,
    ) {
        let ball_sample = ball.sample();
        let ball_position = ball_sample.map(|ball| ball.position());
        let ball_velocity = ball_sample.map_or(NAN3, |ball| dvec(ball.velocity()));
        let mut samples: Vec<Option<WhiffPlayerSample>> = vec![None; self.slots.len()];
        for player in &players.players {
            let slot = self.slot_for(&player.player_id, player.is_team_0);
            if samples.len() <= slot {
                samples.resize(slot + 1, None);
            }
            let Some(body) = player.rigid_body.as_ref() else {
                continue;
            };
            let position = vec_to_glam(&body.location);
            let rotation = quat_to_glam(&body.rotation);
            let (contact_gap, hitbox_distance, local_ball) = match ball_position {
                Some(ball_position) => (
                    car_hitbox_ball_contact_gap(ball_position, body, player.hitbox)
                        .map_or(f64::NAN, f64::from),
                    car_hitbox_distance(ball_position, body, player.hitbox)
                        .map_or(f64::NAN, f64::from),
                    dvec(rotation.inverse() * (ball_position - position)),
                ),
                None => (f64::NAN, f64::NAN, NAN3),
            };
            samples[slot] = Some(WhiffPlayerSample {
                position: dvec(position),
                velocity: body
                    .linear_velocity
                    .as_ref()
                    .map_or(NAN3, |v| dvec(vec_to_glam(v))),
                angular_velocity: body
                    .angular_velocity
                    .as_ref()
                    .map_or(NAN3, |v| dvec(vec_to_glam(v))),
                rotation: rotation.as_dquat(),
                boost: player.boost_amount.map_or(f64::NAN, f64::from),
                boost_active: player.boost_active,
                dodge_active: player.dodge_active,
                double_jump_active: controls.sample(&player.player_id).double_jump_active,
                contact_gap,
                hitbox_distance,
                local_ball,
            });
        }
        let touch_slots = touch_state
            .touch_events
            .iter()
            .filter_map(|touch| touch.player.as_ref())
            .filter_map(|player_id| self.slots.iter().position(|(id, _)| id == player_id))
            .collect();
        let time = f64::from(frame.time);
        self.frames.push_back(WhiffHistoryFrame {
            time,
            frame: frame.frame_number,
            live,
            ball_position: ball_position.map_or(NAN3, dvec),
            ball_velocity,
            players: samples,
            touch_slots,
        });
        while self
            .frames
            .front()
            .is_some_and(|front| time - front.time > HISTORY_SECONDS)
        {
            self.frames.pop_front();
            self.base_seq += 1;
        }
    }

    /// Whether the frame at `seq` is a candidate anchor for `slot`: a live,
    /// near-ball contact-gap minimum over +/- the candidate half window.
    /// Requires the history to extend past that half window.
    pub(crate) fn is_gap_minimum(&self, slot: usize, seq: usize) -> Option<bool> {
        let frame = self.frame(seq)?;
        let last = self.frame(self.last_seq()?)?;
        if last.time <= frame.time + CANDIDATE_HALF_WINDOW_SECONDS {
            return None;
        }
        let gap = frame.gap(slot);
        if !frame.live || !gap.is_finite() || gap > CANDIDATE_MAX_GAP {
            return Some(false);
        }
        let window = self.window(
            frame.time - CANDIDATE_HALF_WINDOW_SECONDS,
            frame.time + CANDIDATE_HALF_WINDOW_SECONDS,
        );
        Some(
            window
                .into_iter()
                .all(|f| f.gap(slot).is_nan() || f.gap(slot) >= gap),
        )
    }

    /// Player touching the ball nearest in time to the anchor, other than the
    /// focal player, within the disruption window.
    pub(crate) fn nearest_other_touch(&self, slot: usize, anchor_seq: usize) -> Option<PlayerId> {
        let anchor_time = self.frame(anchor_seq)?.time;
        self.frames
            .iter()
            .filter(|f| (-1.0..=WHIFF_RESOLUTION_DELAY_SECONDS).contains(&(f.time - anchor_time)))
            .flat_map(|f| f.touch_slots.iter().map(move |s| (f.time, *s)))
            .filter(|(_, touch_slot)| *touch_slot != slot)
            .min_by(|a, b| {
                (a.0 - anchor_time)
                    .abs()
                    .total_cmp(&(b.0 - anchor_time).abs())
            })
            .map(|(_, touch_slot)| self.slots[touch_slot].0.clone())
    }

    fn window(&self, start: f64, end: f64) -> Vec<&WhiffHistoryFrame> {
        self.frames
            .iter()
            .filter(|f| f.time >= start && f.time <= end)
            .collect()
    }
}

/// Feature extraction for one candidate over a resolved history view.
struct View<'a> {
    history: &'a WhiffHistory,
    /// Buffer indices of the anchor and the resolution (last readable) frame.
    anchor: usize,
    hi: usize,
    slot: usize,
}

impl View<'_> {
    fn f(&self, index: usize) -> &WhiffHistoryFrame {
        &self.history.frames[index]
    }

    fn ta(&self) -> f64 {
        self.f(self.anchor).time
    }

    /// First index at or after `time`, clamped to the readable range.
    fn index_at(&self, time: f64) -> usize {
        let frames = &self.history.frames;
        let mut index = frames.partition_point(|f| f.time < time);
        index = index.min(self.hi);
        index
    }

    fn at(&self, offset: f64) -> usize {
        let cutoff = self.ta() + WHIFF_RESOLUTION_DELAY_SECONDS;
        self.index_at((self.ta() + offset).min(cutoff))
    }

    fn gap(&self, index: usize) -> f64 {
        self.f(index).gap(self.slot)
    }

    fn pos(&self, index: usize) -> DVec3 {
        self.f(index).position(self.slot)
    }

    fn vel(&self, index: usize) -> DVec3 {
        self.f(index).velocity(self.slot)
    }

    fn sample(&self, index: usize) -> Option<&WhiffPlayerSample> {
        self.f(index).player(self.slot)
    }

    fn forward_alignment(&self, index: usize) -> f64 {
        let frame = self.f(index);
        self.sample(index).map_or(f64::NAN, |p| {
            alignment(p.forward(), frame.ball_position - p.position)
        })
    }

    fn pred_miss(&self, index: usize) -> (f64, f64) {
        let frame = self.f(index);
        let (p, v) = (self.pos(index), self.vel(index));
        let (bp, bv) = (frame.ball_position, frame.ball_velocity);
        if !(finite3(p) && finite3(v) && finite3(bp)) {
            return (f64::NAN, f64::NAN);
        }
        let airborne = p.z > AIRBORNE_Z;
        let mut best = (f64::INFINITY, 0.0);
        for step in 0..=40 {
            let t = f64::from(step) / 40.0;
            let mut car = p + v * t;
            if airborne {
                car += 0.5 * GRAVITY * t * t;
            }
            let mut ball = bp + bv * t + 0.5 * GRAVITY * t * t;
            ball.z = ball.z.max(BALL_RADIUS);
            let distance = car.distance(ball);
            if distance < best.0 {
                best = (distance, t);
            }
        }
        best
    }

    fn compute(&self) -> WhiffFeatures {
        let mut out = [f64::NAN; WHIFF_FEATURE_COUNT];
        let mut set = |name: &str, value: f64| {
            let index = WHIFF_FEATURE_NAMES
                .iter()
                .position(|n| *n == name)
                .expect("known whiff feature");
            out[index] = value;
        };
        let ta = self.ta();
        let cutoff = ta + WHIFF_RESOLUTION_DELAY_SECONDS;
        let anchor = self.anchor;
        let hi = self.hi;
        let lo = self.index_at(ta - LOOKBACK_SECONDS);
        let slot = self.slot;
        let is_team_0 = self.history.slots[slot].1;

        let gap_anchor = self.gap(anchor);
        set("gap_at_anchor", gap_anchor);
        let gap_at = |offset: f64| self.gap(self.at(offset));
        for (name, offset) in [
            ("gap_m1_0", -1.0),
            ("gap_m0_5", -0.5),
            ("gap_m0_3", -0.3),
            ("gap_m0_15", -0.15),
            ("gap_p0_15", 0.15),
            ("gap_p0_3", 0.3),
            ("gap_p0_6", 0.6),
        ] {
            set(name, gap_at(offset));
        }
        set("closing_rate_0_3", (gap_at(-0.3) - gap_anchor) / 0.3);
        set("opening_rate_0_3", (gap_at(0.3) - gap_anchor) / 0.3);

        let anchor_frame = self.f(anchor);
        let focal = self.sample(anchor);
        let local_ball = focal.map_or(NAN3, |p| p.local_ball);
        set("local_ball_x", local_ball.x);
        set("local_ball_y_abs", local_ball.y.abs());
        set("local_ball_z", local_ball.z);
        let (pos, vel) = (self.pos(anchor), self.vel(anchor));
        let (ball_pos, ball_vel) = (anchor_frame.ball_position, anchor_frame.ball_velocity);
        set("car_speed", vel.length());
        set("ball_speed", ball_vel.length());
        set("rel_speed", (vel - ball_vel).length());
        let pre = self.index_at(ta - 0.3);
        set("forward_alignment", self.forward_alignment(anchor));
        set("forward_alignment_m0_3", self.forward_alignment(pre));
        set("velocity_alignment", alignment(vel, ball_pos - pos));
        set(
            "velocity_alignment_m0_3",
            alignment(self.vel(pre), self.f(pre).ball_position - self.pos(pre)),
        );
        set("car_z", pos.z);
        set("ball_z", ball_pos.z);
        set("ball_z_minus_car_z", ball_pos.z - pos.z);
        set("car_up_z", focal.map_or(f64::NAN, |p| p.up().z));
        set(
            "wall_distance",
            if finite3(pos) {
                (FIELD_HALF_X - pos.x.abs()).min(FIELD_HALF_Y - pos.y.abs())
            } else {
                f64::NAN
            },
        );

        let recent = self.index_at(ta - 0.8)..=self.index_at(ta + 0.1);
        let dodge = |index: usize| self.sample(index).is_some_and(|p| p.dodge_active);
        let dodge_recent = recent.clone().any(dodge);
        set("dodge_recent", f64::from(u8::from(dodge_recent)));
        let last_start = (recent.start() + 1..=*recent.end())
            .rev()
            .find(|&index| dodge(index) && !dodge(index - 1))
            .or_else(|| dodge(*recent.start()).then_some(*recent.start()));
        set(
            "dodge_start_rel",
            last_start.map_or(-2.0, |index| self.f(index).time - ta),
        );
        set(
            "double_jump_recent",
            f64::from(u8::from(recent.clone().any(|index| {
                self.sample(index).is_some_and(|p| p.double_jump_active)
            }))),
        );
        let boost_fraction = |start: usize| {
            let count = anchor + 1 - start;
            (start..=anchor)
                .filter(|&index| self.sample(index).is_some_and(|p| p.boost_active))
                .count() as f64
                / count as f64
        };
        set("boost_fraction_m1", boost_fraction(self.index_at(ta - 1.0)));
        set("boost_amount", focal.map_or(f64::NAN, |p| p.boost));
        set(
            "car_ang_speed",
            focal.map_or(f64::NAN, |p| p.angular_velocity.length()),
        );

        let far = (lo..=anchor)
            .rev()
            .find(|&index| far_or_unknown(self.gap(index)));
        set(
            "approach_duration",
            far.map_or(LOOKBACK_SECONDS, |index| ta - self.f(index).time),
        );

        let ball_dv =
            |index: usize| (self.f(index + 1).ball_velocity - self.f(index).ball_velocity).length();
        let post_start = anchor.saturating_sub(3).max(lo);
        let post_end = self.index_at(ta + 0.3);
        set(
            "ball_dv_max_post",
            nan_or(nan_max((post_start..post_end.min(hi)).map(ball_dv)), 0.0),
        );
        set(
            "ball_dv_max_pre",
            nan_or(
                nan_max((self.index_at(ta - 1.0)..post_start).map(ball_dv)),
                0.0,
            ),
        );
        let before = self.f(self.index_at(ta - 0.1)).ball_velocity;
        let after = self.f(self.at(0.3)).ball_velocity;
        set(
            "ball_dir_change_post",
            nan_or(1.0 - alignment(before, after), 0.0),
        );

        let (mut focal_dt, mut mate_dt, mut opp_dt) = (NO_TOUCH, NO_TOUCH, NO_TOUCH);
        // Latest other-player touch in [-1.0, +0.1]: (rel, index, is teammate).
        let mut other_touch: Option<(f64, usize, bool)> = None;
        let mut prev_focal: Option<(f64, usize)> = None;
        for index in self.index_at(ta - LOOKBACK_SECONDS)..=hi {
            let rel = self.f(index).time - ta;
            for &touch_slot in &self.f(index).touch_slots {
                if touch_slot == slot {
                    if (-LOOKBACK_SECONDS..-0.3).contains(&rel) {
                        prev_focal = Some((rel, index));
                    }
                } else if (-1.0..=0.1).contains(&rel) {
                    let mate = self.history.slots[touch_slot].1 == is_team_0;
                    other_touch = Some((rel, index, mate));
                }
                if !(-1.0..=WHIFF_RESOLUTION_DELAY_SECONDS).contains(&rel) {
                    continue;
                }
                if touch_slot == slot {
                    if rel >= -0.3 && rel.abs() < focal_dt.abs() {
                        focal_dt = rel;
                    }
                } else if self.history.slots[touch_slot].1 == is_team_0 {
                    if rel.abs() < mate_dt.abs() {
                        mate_dt = rel;
                    }
                } else if rel.abs() < opp_dt.abs() {
                    opp_dt = rel;
                }
            }
        }
        set("focal_touch_dt", focal_dt);
        set("teammate_touch_dt", mate_dt);
        set("opponent_touch_dt", opp_dt);

        let near = self.index_at(ta - 0.5)..=self.index_at((ta + 0.5).min(cutoff));
        let (mut mate_gap, mut opp_gap) = (f64::INFINITY, f64::INFINITY);
        let mut others = 0.0;
        let mut nearest = f64::INFINITY;
        for other in 0..self.history.slots.len() {
            if other == slot {
                continue;
            }
            let value = nan_or(
                nan_min(near.clone().map(|index| self.f(index).gap(other))),
                f64::INFINITY,
            );
            if self.history.slots[other].1 == is_team_0 {
                mate_gap = mate_gap.min(value);
            } else {
                opp_gap = opp_gap.min(value);
            }
            let other_pos = anchor_frame.position(other);
            if finite3(other_pos) && finite3(ball_pos) && other_pos.distance(ball_pos) < 800.0 {
                others += 1.0;
            }
            if finite3(other_pos) && finite3(pos) {
                nearest = nearest.min(other_pos.distance(pos));
            }
        }
        set("teammate_min_gap", mate_gap.min(5000.0));
        set("opponent_min_gap", opp_gap.min(5000.0));
        set("others_near_ball", others);
        set("nearest_car_distance", nearest.min(10000.0));
        let span = hi + 1 - lo;
        set(
            "present_fraction",
            (lo..=hi)
                .filter(|&index| self.sample(index).is_some())
                .count() as f64
                / span as f64,
        );
        set(
            "max_sample_dt",
            (lo..hi)
                .map(|index| self.f(index + 1).time - self.f(index).time)
                .fold(0.0, f64::max),
        );

        match other_touch {
            Some((rel, touch, mate)) => {
                set("other_touch_rel", rel);
                set("other_touch_focal_gap", nan_or(self.gap(touch), 5000.0));
                let pre_touch = touch.saturating_sub(1).max(lo);
                let before = self.f(pre_touch).ball_velocity;
                let after = self.f((touch + 2).min(hi)).ball_velocity;
                set(
                    "other_touch_ball_dv",
                    nan_or((after - before).length(), 0.0),
                );
                // Distance the car would have had to the pre-touch ball path.
                let (p0, v0, t0) = (
                    self.f(pre_touch).ball_position,
                    self.f(pre_touch).ball_velocity,
                    self.f(pre_touch).time,
                );
                let (mut counterfactual, mut actual) = (f64::INFINITY, f64::INFINITY);
                for index in touch..=self.index_at(ta + 0.3).min(hi) {
                    let car = self.pos(index);
                    if !finite3(car) {
                        continue;
                    }
                    let dt = self.f(index).time - t0;
                    let mut ball = p0 + v0 * dt + 0.5 * GRAVITY * dt * dt;
                    ball.z = ball.z.max(BALL_RADIUS);
                    if car.distance(ball) < counterfactual {
                        counterfactual = car.distance(ball);
                    }
                    let real = car.distance(self.f(index).ball_position);
                    if real < actual {
                        actual = real;
                    }
                }
                let counterfactual = counterfactual.min(5000.0);
                set("cf_min_dist", counterfactual);
                set("cf_minus_actual", counterfactual - actual.min(5000.0));
                let dodge_start = (self.index_at(ta - LOOKBACK_SECONDS).max(1)
                    ..=self.index_at(ta + 0.2))
                    .rev()
                    .find(|&index| dodge(index) && !dodge(index - 1));
                set(
                    "dodge_minus_other_touch",
                    dodge_start.map_or(-9.0, |index| self.f(index).time - self.f(touch).time),
                );
                set("other_touch_is_mate", f64::from(u8::from(mate)));
            }
            None => {
                set("other_touch_rel", -9.0);
                set("other_touch_focal_gap", 5000.0);
                set("other_touch_ball_dv", 0.0);
                set("cf_min_dist", 5000.0);
                set("cf_minus_actual", 0.0);
                set("dodge_minus_other_touch", -9.0);
                set("other_touch_is_mate", -1.0);
            }
        }

        for (name, offset) in [
            ("pred_miss_m0_6", -0.6),
            ("pred_miss_m0_3", -0.3),
            ("pred_miss_m0_15", -0.15),
        ] {
            let (distance, time) = self.pred_miss(self.index_at(ta + offset));
            set(name, distance);
            if name == "pred_miss_m0_3" {
                set("pred_tca_m0_3", time);
            }
        }

        let lo5 = self.index_at(ta - 0.5);
        if anchor >= lo5 + 2 {
            let mut alignments = Vec::new();
            let mut magnitudes = Vec::new();
            for index in lo5 + 1..=anchor {
                let dt = (self.f(index).time - self.f(index - 1).time).max(1e-3);
                let mut accel = (self.vel(index) - self.vel(index - 1)) / dt;
                let car = self.pos(index);
                if car.z > AIRBORNE_Z {
                    accel -= GRAVITY;
                }
                magnitudes.push(accel.length());
                let a = alignment(accel, self.f(index).ball_position - car);
                alignments.push(a);
            }
            set("accel_align_0_5", nan_mean(alignments.into_iter()));
            set("accel_mag_0_5", nan_mean(magnitudes.into_iter()));
        }
        set(
            "fwd_align_mean_1_0",
            nan_mean((self.index_at(ta - 1.0)..=anchor).map(|index| self.forward_alignment(index))),
        );
        set(
            "fwd_align_trend",
            self.forward_alignment(anchor) - self.forward_alignment(lo5),
        );
        let airborne_since = (lo..=anchor)
            .rev()
            .find(|&index| self.pos(index).z.is_nan() || self.pos(index).z <= AIRBORNE_Z)
            .map_or(self.f(lo).time, |index| self.f(index).time);
        set(
            "airborne_time",
            if pos.z > AIRBORNE_Z {
                ta - airborne_since
            } else {
                0.0
            },
        );
        set("boost_frac_0_5", boost_fraction(lo5));
        set(
            "ang_speed_mean_0_5",
            nan_mean((lo5..=anchor).map(|index| {
                self.sample(index)
                    .map_or(f64::NAN, |p| p.angular_velocity.length())
            })),
        );
        let i15 = self.index_at(ta - 0.15);
        let to_ball = self.f(i15).ball_position - self.pos(i15);
        set(
            "closing_m0_15",
            (self.vel(i15) - self.f(i15).ball_velocity).dot(to_ball) / to_ball.length().max(1e-6),
        );
        match prev_focal {
            Some((rel, touch)) => {
                set("focal_prev_touch_dt", rel);
                set(
                    "gap_max_since_prev_touch",
                    nan_max((touch..=anchor).map(|index| self.gap(index))),
                );
            }
            None => {
                set("focal_prev_touch_dt", -9.0);
                set("gap_max_since_prev_touch", 5000.0);
            }
        }
        set(
            "min_center_dist",
            nan_min(
                (self.index_at(ta - 0.3)..=self.at(0.3))
                    .map(|index| self.pos(index).distance(self.f(index).ball_position)),
            ),
        );
        out
    }
}

/// Features for the candidate anchored at `anchor_seq`, reading frames up to
/// the most recent one in the history.
pub(crate) fn whiff_candidate_features(
    history: &WhiffHistory,
    slot: usize,
    anchor_seq: usize,
) -> Option<WhiffFeatures> {
    let anchor = anchor_seq.checked_sub(history.base_seq)?;
    if anchor >= history.frames.len() {
        return None;
    }
    let view = View {
        history,
        anchor,
        hi: history.frames.len() - 1,
        slot,
    };
    Some(view.compute())
}

/// Time and frame where the approach to the anchor began: the first frame
/// after the player was last at least the far gap away, bounded by the
/// lookback.
pub(crate) fn whiff_attempt_start(
    history: &WhiffHistory,
    slot: usize,
    anchor_seq: usize,
) -> (f64, usize) {
    let anchor = anchor_seq - history.base_seq;
    let anchor_time = history.frames[anchor].time;
    let lo = history
        .frames
        .partition_point(|f| f.time < anchor_time - LOOKBACK_SECONDS)
        .min(anchor);
    let start = (lo..=anchor)
        .rev()
        .find(|&index| far_or_unknown(history.frames[index].gap(slot)))
        .map_or(lo, |index| (index + 1).min(anchor));
    let frame = &history.frames[start];
    (frame.time, frame.frame)
}
