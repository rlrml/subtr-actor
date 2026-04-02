use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use boxcars;
use boxcars::HeaderProp;
use serde::{Deserialize, Serialize};

use super::boost_invariants::{boost_invariant_violations, BoostInvariantKind};
use crate::*;

mod frame_input;
pub use frame_input::*;
mod frame_components;
pub use frame_components::*;
mod live_play;
pub use live_play::*;
mod samples;
pub use samples::*;
pub mod backboard;
pub use backboard::*;
pub mod backboard_bounce;
pub use backboard_bounce::*;
pub mod ball_carry;
pub use ball_carry::*;
pub mod boost;
pub use boost::*;
pub mod ceiling_shot;
pub use ceiling_shot::*;
pub mod demo;
pub use demo::*;
mod flip_reset;
pub use flip_reset::*;
mod flip_reset_tuning_set;
pub use flip_reset_tuning_set::*;
pub mod dodge_reset;
pub use dodge_reset::*;
pub mod double_tap;
pub use double_tap::*;
pub mod fifty_fifty;
pub use fifty_fifty::*;
pub mod fifty_fifty_state;
pub use fifty_fifty_state::*;
pub mod match_stats;
pub use match_stats::*;
pub mod movement;
pub use movement::*;
pub mod musty_flick;
pub use musty_flick::*;
pub mod positioning;
pub use positioning::*;
pub mod player_vertical_state;
pub use player_vertical_state::*;
pub mod possession;
pub use possession::*;
pub mod possession_state;
pub use possession_state::*;
pub mod powerslide;
pub use powerslide::*;
pub mod pressure;
pub use pressure::*;
pub mod rush;
pub use rush::*;
pub mod settings;
pub use settings::*;
pub mod speed_flip;
pub use speed_flip::*;
pub mod touch;
pub use touch::*;
pub mod touch_state;
pub use touch_state::*;

fn interval_fraction_in_scalar_range(start: f32, end: f32, min_value: f32, max_value: f32) -> f32 {
    if (end - start).abs() <= f32::EPSILON {
        return ((start >= min_value) && (start < max_value)) as i32 as f32;
    }

    let t_at_min = (min_value - start) / (end - start);
    let t_at_max = (max_value - start) / (end - start);
    let interval_start = t_at_min.min(t_at_max).max(0.0);
    let interval_end = t_at_min.max(t_at_max).min(1.0);
    (interval_end - interval_start).max(0.0)
}

fn interval_fraction_below_threshold(start: f32, end: f32, threshold: f32) -> f32 {
    if (end - start).abs() <= f32::EPSILON {
        return (start < threshold) as i32 as f32;
    }

    let threshold_time = ((threshold - start) / (end - start)).clamp(0.0, 1.0);
    if start < threshold {
        if end < threshold {
            1.0
        } else {
            threshold_time
        }
    } else if end < threshold {
        1.0 - threshold_time
    } else {
        0.0
    }
}

fn interval_fraction_above_threshold(start: f32, end: f32, threshold: f32) -> f32 {
    if (end - start).abs() <= f32::EPSILON {
        return (start > threshold) as i32 as f32;
    }

    let threshold_time = ((threshold - start) / (end - start)).clamp(0.0, 1.0);
    if start > threshold {
        if end > threshold {
            1.0
        } else {
            threshold_time
        }
    } else if end > threshold {
        1.0 - threshold_time
    } else {
        0.0
    }
}

const CAR_MAX_SPEED: f32 = 2300.0;
const SUPERSONIC_SPEED_THRESHOLD: f32 = 2200.0;
const BOOST_SPEED_THRESHOLD: f32 = 1410.0;
const POWERSLIDE_MAX_Z_THRESHOLD: f32 = 40.0;
const BALL_RADIUS_Z: f32 = 92.75;
const BALL_CARRY_MIN_BALL_Z: f32 = BALL_RADIUS_Z + 5.0;
const BALL_CARRY_MAX_BALL_Z: f32 = 600.0;
const BALL_CARRY_MAX_HORIZONTAL_GAP: f32 = BALL_RADIUS_Z * 1.4;
const BALL_CARRY_MAX_VERTICAL_GAP: f32 = 220.0;
const BALL_CARRY_MIN_DURATION: f32 = 1.0;
const FIELD_ZONE_BOUNDARY_Y: f32 = BOOST_PAD_SIDE_LANE_Y;
const DEFAULT_MOST_BACK_FORWARD_THRESHOLD_Y: f32 = 236.0;
const SMALL_PAD_AMOUNT_RAW: f32 = BOOST_MAX_AMOUNT * 12.0 / 100.0;
const BOOST_ZERO_BAND_RAW: f32 = 1.0;
const BOOST_FULL_BAND_MIN_RAW: f32 = BOOST_MAX_AMOUNT - 1.0;
const STANDARD_PAD_MATCH_RADIUS_SMALL: f32 = 450.0;
const STANDARD_PAD_MATCH_RADIUS_BIG: f32 = 1000.0;
const BOOST_PAD_MIDFIELD_TOLERANCE_Y: f32 = 128.0;
const BOOST_PAD_SMALL_Z: f32 = 70.0;
const BOOST_PAD_BIG_Z: f32 = 73.0;
const BOOST_PAD_BACK_CORNER_X: f32 = 3072.0;
const BOOST_PAD_BACK_CORNER_Y: f32 = 4096.0;
const BOOST_PAD_BACK_LANE_X: f32 = 1792.0;
const BOOST_PAD_BACK_LANE_Y: f32 = 4184.0;
const BOOST_PAD_BACK_MID_X: f32 = 940.0;
const BOOST_PAD_BACK_MID_Y: f32 = 3308.0;
const BOOST_PAD_CENTER_BACK_Y: f32 = 2816.0;
const BOOST_PAD_SIDE_WALL_X: f32 = 3584.0;
const BOOST_PAD_SIDE_WALL_Y: f32 = 2484.0;
const BOOST_PAD_SIDE_LANE_X: f32 = 1788.0;
const BOOST_PAD_SIDE_LANE_Y: f32 = 2300.0;
const BOOST_PAD_FRONT_LANE_X: f32 = 2048.0;
const BOOST_PAD_FRONT_LANE_Y: f32 = 1036.0;
const BOOST_PAD_CENTER_X: f32 = 1024.0;
const BOOST_PAD_CENTER_MID_Y: f32 = 1024.0;
const BOOST_PAD_GOAL_LINE_Y: f32 = 4240.0;

fn push_pad(
    pads: &mut Vec<(glam::Vec3, BoostPadSize)>,
    x: f32,
    y: f32,
    z: f32,
    size: BoostPadSize,
) {
    pads.push((glam::Vec3::new(x, y, z), size));
}

fn push_mirror_x(
    pads: &mut Vec<(glam::Vec3, BoostPadSize)>,
    x: f32,
    y: f32,
    z: f32,
    size: BoostPadSize,
) {
    push_pad(pads, -x, y, z, size);
    push_pad(pads, x, y, z, size);
}

fn push_mirror_y(
    pads: &mut Vec<(glam::Vec3, BoostPadSize)>,
    x: f32,
    y: f32,
    z: f32,
    size: BoostPadSize,
) {
    push_pad(pads, x, -y, z, size);
    push_pad(pads, x, y, z, size);
}

fn push_mirror_xy(
    pads: &mut Vec<(glam::Vec3, BoostPadSize)>,
    x: f32,
    y: f32,
    z: f32,
    size: BoostPadSize,
) {
    push_mirror_x(pads, x, -y, z, size);
    push_mirror_x(pads, x, y, z, size);
}

fn build_standard_soccar_boost_pad_layout() -> Vec<(glam::Vec3, BoostPadSize)> {
    let mut pads = Vec::with_capacity(34);

    push_mirror_y(
        &mut pads,
        0.0,
        BOOST_PAD_GOAL_LINE_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_xy(
        &mut pads,
        BOOST_PAD_BACK_LANE_X,
        BOOST_PAD_BACK_LANE_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_xy(
        &mut pads,
        BOOST_PAD_BACK_CORNER_X,
        BOOST_PAD_BACK_CORNER_Y,
        BOOST_PAD_BIG_Z,
        BoostPadSize::Big,
    );
    push_mirror_xy(
        &mut pads,
        BOOST_PAD_BACK_MID_X,
        BOOST_PAD_BACK_MID_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_y(
        &mut pads,
        0.0,
        BOOST_PAD_CENTER_BACK_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_xy(
        &mut pads,
        BOOST_PAD_SIDE_WALL_X,
        BOOST_PAD_SIDE_WALL_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_xy(
        &mut pads,
        BOOST_PAD_SIDE_LANE_X,
        BOOST_PAD_SIDE_LANE_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_xy(
        &mut pads,
        BOOST_PAD_FRONT_LANE_X,
        BOOST_PAD_FRONT_LANE_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_y(
        &mut pads,
        0.0,
        BOOST_PAD_CENTER_MID_Y,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );
    push_mirror_x(
        &mut pads,
        BOOST_PAD_SIDE_WALL_X,
        0.0,
        BOOST_PAD_BIG_Z,
        BoostPadSize::Big,
    );
    push_mirror_x(
        &mut pads,
        BOOST_PAD_CENTER_X,
        0.0,
        BOOST_PAD_SMALL_Z,
        BoostPadSize::Small,
    );

    pads
}

static STANDARD_SOCCAR_BOOST_PAD_LAYOUT: LazyLock<Vec<(glam::Vec3, BoostPadSize)>> =
    LazyLock::new(build_standard_soccar_boost_pad_layout);

pub fn standard_soccar_boost_pad_layout() -> &'static [(glam::Vec3, BoostPadSize)] {
    STANDARD_SOCCAR_BOOST_PAD_LAYOUT.as_slice()
}

fn normalized_y(is_team_0: bool, position: glam::Vec3) -> f32 {
    if is_team_0 {
        position.y
    } else {
        -position.y
    }
}

fn is_enemy_side(is_team_0: bool, position: glam::Vec3) -> bool {
    normalized_y(is_team_0, position) > BOOST_PAD_MIDFIELD_TOLERANCE_Y
}

fn standard_soccar_boost_pad_position(index: usize) -> glam::Vec3 {
    STANDARD_SOCCAR_BOOST_PAD_LAYOUT[index].0
}

#[derive(Debug, Clone, Default)]
struct PadPositionEstimate {
    observations: Vec<glam::Vec3>,
}

impl PadPositionEstimate {
    fn observe(&mut self, position: glam::Vec3) {
        self.observations.push(position);
    }

    fn observations(&self) -> &[glam::Vec3] {
        self.observations.as_slice()
    }

    fn mean(&self) -> Option<glam::Vec3> {
        if self.observations.is_empty() {
            return None;
        }

        let sum = self
            .observations
            .iter()
            .copied()
            .fold(glam::Vec3::ZERO, |acc, position| acc + position);
        Some(sum / self.observations.len() as f32)
    }
}

fn header_prop_to_f32(prop: &HeaderProp) -> Option<f32> {
    match prop {
        HeaderProp::Float(value) => Some(*value),
        HeaderProp::Int(value) => Some(*value as f32),
        HeaderProp::QWord(value) => Some(*value as f32),
        _ => None,
    }
}

fn get_header_f32(stats: &HashMap<String, HeaderProp>, keys: &[&str]) -> Option<f32> {
    keys.iter()
        .find_map(|key| stats.get(*key).and_then(header_prop_to_f32))
}
