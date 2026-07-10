use std::collections::{HashMap, HashSet, VecDeque};

use boxcars;
use boxcars::HeaderProp;
use serde::{Deserialize, Serialize};

pub(crate) use crate::stats::common::*;

pub(crate) use crate::stats::accumulators::*;

// These support analysis-graph and live-play integrations that do not exercise
// every helper in the default replay pipeline.
#[allow(dead_code)]
mod frame_input;
pub use frame_input::*;
mod frame_components;
pub use frame_components::*;
mod event_stream;
pub use event_stream::*;
#[allow(dead_code)]
mod in_flight;
pub use in_flight::*;
#[allow(dead_code)]
mod event_definition;
pub use event_definition::*;
mod continuous_ball_control;
pub use continuous_ball_control::*;
#[cfg(test)]
#[path = "ball_control_test_support.rs"]
mod ball_control_test_support;
mod live_play;
pub use live_play::*;
mod samples;
pub use samples::*;
pub mod backboard;
pub use backboard::*;
pub mod backboard_bounce;
pub use backboard_bounce::*;
pub mod air_dribble;
pub use air_dribble::*;
pub mod ball_carry;
pub use ball_carry::*;
pub mod boost;
pub use crate::boost_pad_locations::standard_soccar_boost_pad_layout;
pub(crate) use crate::boost_pad_locations::{
    BOOST_PAD_BACK_CORNER_X, BOOST_PAD_BACK_CORNER_Y, BOOST_PAD_MIDFIELD_TOLERANCE_Y,
    BOOST_PAD_SIDE_LANE_Y, PadPositionEstimate, STANDARD_PAD_MATCH_RADIUS_BIG,
    STANDARD_PAD_MATCH_RADIUS_SMALL, STANDARD_SOCCAR_BOOST_PAD_LAYOUT,
    standard_soccar_boost_pad_position,
};
pub use boost::*;
pub mod bump;
pub use bump::*;
pub mod ceiling_shot;
pub use ceiling_shot::*;
pub mod center;
pub use center::*;
pub mod controlled_play;
pub use controlled_play::*;
pub mod demo;
pub use demo::*;
#[allow(dead_code)]
mod flip_reset;
pub use flip_reset::*;
pub mod dodge_reset;
pub use dodge_reset::*;
pub mod double_tap;
pub use double_tap::*;
pub mod fifty_fifty;
pub use fifty_fifty::*;
pub mod fifty_fifty_state;
pub use fifty_fifty_state::*;
#[allow(dead_code)]
pub mod flick;
pub use flick::*;
pub mod flip_impulse;
pub use flip_impulse::*;
#[allow(dead_code)]
pub mod goal_tags;
pub use goal_tags::*;
pub mod half_flip;
pub use half_flip::*;
pub mod half_volley;
pub use half_volley::*;
pub mod kickoff_types;
pub use kickoff_types::*;
pub mod kickoff;
pub use kickoff::*;
pub mod match_stats;
pub use match_stats::*;
pub mod movement;
pub use movement::*;
pub mod one_timer;
pub use one_timer::*;
pub mod pass;
pub use pass::*;
#[allow(dead_code)]
pub mod player_state_span;
pub use player_state_span::*;
pub mod positioning;
pub use positioning::*;
pub mod player_vertical_state;
pub use player_vertical_state::*;
pub mod player_possession;
pub use player_possession::*;
pub mod loose_possession;
pub use loose_possession::*;
pub mod possession;
pub use possession::*;
pub mod possession_state;
pub use possession_state::*;
pub mod powerslide;
pub use powerslide::*;
pub mod ball_half;
pub use ball_half::*;
pub mod ball_third;
pub use ball_third::*;
pub mod rotation;
pub use rotation::*;
pub mod rush;
pub use rush::*;
pub mod settings;
pub use settings::*;
pub mod speed_flip;
pub use speed_flip::*;
pub mod territorial_pressure;
pub use territorial_pressure::*;
pub mod touch;
pub use touch::*;
pub mod touch_intention;
pub use touch_intention::*;
pub mod touch_state;
pub use touch_state::*;
pub mod wall_aerial;
pub use wall_aerial::*;
pub mod wall_aerial_shot;
pub use wall_aerial_shot::*;
pub mod wavedash;
pub use wavedash::*;
pub mod whiff;
pub use whiff::*;

pub(crate) fn chronological_touch_events(touch_events: &[TouchEvent]) -> Vec<&TouchEvent> {
    let mut touch_events = touch_events.iter().collect::<Vec<_>>();
    touch_events.sort_by(|left, right| {
        TouchEvent::timestamp_ordering(left, right)
            .then_with(|| touch_state::touch_event_ordering(left, right))
    });
    touch_events
}

pub(crate) fn sequential_touch_events(touch_events: &[TouchEvent]) -> Vec<&TouchEvent> {
    let mut touch_events = touch_events.iter().collect::<Vec<_>>();
    // Sequential calculators often update a single "last touch" slot while iterating.
    // Put stronger exact-tie contacts later so those assignments retain the primary touch.
    touch_events.sort_by(|left, right| {
        TouchEvent::timestamp_ordering(left, right)
            .then_with(|| touch_state::touch_event_ordering(right, left))
    });
    touch_events
}

#[cfg(test)]
#[path = "ordering_tests.rs"]
mod ordering_tests;

/// How long after a ball-hit the `CarComponent_Dodge` `ReplicatedActive` byte
/// may take to replicate. The hit and the dodge activation that produced it
/// routinely land on adjacent frames, but the lag is not always "a frame or
/// two": on lower-FPS / downsampled replays a flick contact has been observed
/// to precede its dodge byte by ~0.23s (≈5 sampled frames). Detectors that pair
/// a touch with the dodge that powered it (touch dodge-upgrades, flip-reset
/// confirmation, flick detection) all key off this same physical quantity, so
/// they share this single tolerance rather than each guessing their own.
pub(crate) const DODGE_ACTIVE_BYTE_LAG_TOLERANCE_SECONDS: f32 = 0.25;

const SUPERSONIC_SPEED_THRESHOLD: f32 = 2200.0;
const BOOST_SPEED_THRESHOLD: f32 = 1410.0;
const POWERSLIDE_MAX_Z_THRESHOLD: f32 = 40.0;
const BALL_RADIUS_Z: f32 = 92.75;
const BALL_CARRY_MIN_BALL_Z: f32 = BALL_RADIUS_Z + 5.0;
const BALL_CARRY_MAX_BALL_Z: f32 = 600.0;
const BALL_CARRY_MAX_HORIZONTAL_GAP: f32 = BALL_RADIUS_Z * 1.4;
const BALL_CARRY_MAX_VERTICAL_GAP: f32 = 220.0;
const BALL_CARRY_MIN_DURATION: f32 = 1.0;
const WALL_CONTACT_MIN_PLAYER_Z: f32 = 120.0;
const SIDE_WALL_CONTACT_ABS_X: f32 = 3600.0;
const BACK_WALL_CONTACT_ABS_Y: f32 = 5000.0;
const BACK_WALL_GOAL_MOUTH_HALF_WIDTH_X: f32 = 900.0;
const FIELD_ZONE_BOUNDARY_Y: f32 = BOOST_PAD_SIDE_LANE_Y;
const DEFAULT_MOST_BACK_FORWARD_THRESHOLD_Y: f32 = 236.0;
const SMALL_PAD_AMOUNT_RAW: f32 = BOOST_MAX_AMOUNT * 12.0 / 100.0;
const BOOST_FULL_BAND_MIN_RAW: f32 = BOOST_MAX_AMOUNT - 1.0;

const SOCCAR_CEILING_Z: f32 = 2044.0;
const CEILING_CONTACT_MAX_GAP: f32 = 90.0;

fn normalized_y(is_team_0: bool, position: glam::Vec3) -> f32 {
    if is_team_0 { position.y } else { -position.y }
}

fn is_enemy_side(is_team_0: bool, position: glam::Vec3) -> bool {
    normalized_y(is_team_0, position) > BOOST_PAD_MIDFIELD_TOLERANCE_Y
}

fn player_is_on_wall(position: glam::Vec3) -> bool {
    let is_side_wall = position.x.abs() >= SIDE_WALL_CONTACT_ABS_X;
    let is_back_wall = position.y.abs() >= BACK_WALL_CONTACT_ABS_Y
        && position.x.abs() > BACK_WALL_GOAL_MOUTH_HALF_WIDTH_X;

    position.z >= WALL_CONTACT_MIN_PLAYER_Z && (is_side_wall || is_back_wall)
}

fn player_is_on_ceiling(position: glam::Vec3) -> bool {
    SOCCAR_CEILING_Z - position.z <= CEILING_CONTACT_MAX_GAP
}

fn player_sample_is_touching_surface(player: &PlayerSample) -> bool {
    let Some(position) = player.position() else {
        return false;
    };

    player
        .rigid_body
        .as_ref()
        .is_some_and(|body| car_hitbox_touches_floor(body, player.hitbox))
        || PlayerVerticalBand::from_height(position.z).is_grounded()
        || player_is_on_wall(position)
        || player_is_on_ceiling(position)
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
