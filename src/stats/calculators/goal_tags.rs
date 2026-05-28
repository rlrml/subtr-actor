use super::*;

const DEFAULT_AERIAL_GOAL_MIN_BALL_Z: f32 = 600.0;
const DEFAULT_HIGH_AERIAL_GOAL_MIN_BALL_Z: f32 = 700.0;
const DEFAULT_LONG_DISTANCE_GOAL_MAX_ATTACKING_Y: f32 = 1024.0;
const DEFAULT_OWN_HALF_GOAL_MAX_ATTACKING_Y: f32 = 0.0;
// Avoid labeling long delayed clears as own-half goals solely from replay goal credit.
const OWN_HALF_GOAL_MAX_TOUCH_TO_GOAL_SECONDS: f32 = 8.0;
const DEFAULT_EMPTY_NET_MIN_DEFENDER_Y_MARGIN: f32 = 700.0;
const DEFAULT_EMPTY_NET_MIN_DEFENDER_DISTANCE: f32 = 1000.0;
const DEFAULT_EMPTY_NET_MAX_TOUCH_ATTACKING_Y: f32 = 3600.0;
const DEFAULT_FLICK_GOAL_MAX_EVENT_TO_GOAL_SECONDS: f32 = 3.0;
const DEFAULT_DOUBLE_TAP_GOAL_MAX_EVENT_TO_GOAL_SECONDS: f32 = 3.0;
const DEFAULT_ONE_TIMER_GOAL_MAX_EVENT_TO_GOAL_SECONDS: f32 = 3.0;
const DEFAULT_PASSING_GOAL_MAX_PASS_TO_GOAL_SECONDS: f32 = 3.0;
const DEFAULT_AIR_DRIBBLE_GOAL_MAX_END_TO_GOAL_SECONDS: f32 = 3.0;
const DEFAULT_FLIP_RESET_GOAL_MAX_EVENT_TO_GOAL_SECONDS: f32 = 8.0;
const DEFAULT_HALF_VOLLEY_GOAL_MAX_TOUCH_TO_GOAL_SECONDS: f32 = 3.0;
const DEFAULT_HALF_VOLLEY_GOAL_MIN_GOAL_ALIGNMENT: f32 = 0.55;

#[path = "goal_tags_calculator_types.rs"]
mod calculator_types;
#[path = "goal_tags_calculator_structs.rs"]
mod calculator_structs;
#[path = "goal_tags_combined.rs"]
mod combined;
#[path = "goal_tags_config.rs"]
mod config;
#[path = "goal_tags_config_half_volley.rs"]
mod config_half_volley;
#[path = "goal_tags_config_mechanics.rs"]
mod config_mechanics;
#[path = "goal_tags_config_position.rs"]
mod config_position;
#[path = "goal_tags_evidence.rs"]
mod evidence;
#[path = "goal_tags_event_builders.rs"]
mod event_builders;
#[path = "goal_tags_matching.rs"]
mod matching;
#[path = "goal_tags_matching_air_dribble.rs"]
mod matching_air_dribble;
#[path = "goal_tags_matching_pass.rs"]
mod matching_pass;
#[path = "goal_tags_matching_point.rs"]
mod matching_point;
#[path = "goal_tags_matching_position.rs"]
mod matching_position;
#[path = "goal_tags_mechanics_air_dribble.rs"]
mod mechanics_air_dribble;
#[path = "goal_tags_mechanics_flip_reset.rs"]
mod mechanics_flip_reset;
#[path = "goal_tags_mechanics_half_volley.rs"]
mod mechanics_half_volley;
#[path = "goal_tags_mechanics_passing.rs"]
mod mechanics_passing;
#[path = "goal_tags_mechanics_point.rs"]
mod mechanics_point;
#[path = "goal_tags_point_events.rs"]
mod point_events;
#[path = "goal_tags_position.rs"]
mod position;
#[path = "goal_tags_position_empty_net.rs"]
mod position_empty_net;
#[path = "goal_tags_types.rs"]
mod types;

pub use calculator_structs::*;
pub use combined::*;
pub use config::*;
use evidence::*;
use event_builders::*;
use matching::*;
use point_events::*;
use self::types::GoalTaggingContext;
pub use self::types::{
    GoalTagEvent, GoalTagEvidence, GoalTagEvidenceKind, GoalTagKind, GoalTagModifier,
};

#[cfg(test)]
#[path = "goal_tags_tests.rs"]
mod tests;
