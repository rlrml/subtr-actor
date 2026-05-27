use std::collections::{HashMap, HashSet, VecDeque};

use boxcars;
use serde::{Deserialize, Serialize};

use super::boost_invariants::{boost_invariant_violations, BoostInvariantKind};
use crate::*;

mod field_constants;
mod header_values;
mod interval_math;
mod labels;
mod soccar_boost_pads;

pub(crate) use field_constants::*;
pub(crate) use header_values::*;
pub(crate) use interval_math::*;
pub(crate) use labels::*;
pub(crate) use soccar_boost_pads::*;

macro_rules! export_calculator_module {
    (pub $module:ident) => {
        pub mod $module;
        pub use self::$module::*;
    };
    (private $module:ident) => {
        mod $module;
        pub use self::$module::*;
    };
}

export_calculator_module!(private frame_input);
export_calculator_module!(private frame_components);
export_calculator_module!(private continuous_ball_control);
#[cfg(test)]
#[path = "ball_control_test_support.rs"]
mod ball_control_test_support;
export_calculator_module!(private live_play);
export_calculator_module!(private samples);
export_calculator_module!(pub backboard);
export_calculator_module!(pub backboard_bounce);
export_calculator_module!(pub air_dribble);
export_calculator_module!(pub ball_carry);
export_calculator_module!(pub boost);
export_calculator_module!(pub bump);
export_calculator_module!(pub ceiling_shot);
export_calculator_module!(pub center);
export_calculator_module!(pub demo);
export_calculator_module!(private flip_reset);
export_calculator_module!(private flip_reset_tuning_set);
export_calculator_module!(pub dodge_reset);
export_calculator_module!(pub double_tap);
export_calculator_module!(pub fifty_fifty);
export_calculator_module!(pub fifty_fifty_state);
export_calculator_module!(pub flick);
export_calculator_module!(pub goal_tags);
export_calculator_module!(pub half_flip);
export_calculator_module!(pub half_volley);
export_calculator_module!(pub match_stats);
export_calculator_module!(pub movement);
export_calculator_module!(pub musty_flick);
export_calculator_module!(pub one_timer);
export_calculator_module!(pub pass);
export_calculator_module!(pub positioning);
export_calculator_module!(pub player_vertical_state);
export_calculator_module!(pub possession);
export_calculator_module!(pub possession_state);
export_calculator_module!(pub powerslide);
export_calculator_module!(pub pressure);
export_calculator_module!(pub rotation);
export_calculator_module!(pub rush);
export_calculator_module!(pub settings);
export_calculator_module!(pub speed_flip);
export_calculator_module!(pub territorial_pressure);
export_calculator_module!(pub touch);
export_calculator_module!(pub touch_state);
export_calculator_module!(pub wall_aerial);
export_calculator_module!(pub wall_aerial_shot);
export_calculator_module!(pub wavedash);
export_calculator_module!(pub whiff);
