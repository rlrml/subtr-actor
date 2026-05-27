use super::*;
use crate::stats::calculators::*;
use crate::*;

#[macro_use]
#[path = "goal_tags_macros.rs"]
mod goal_tags_macros;
#[path = "goal_tags_boxed.rs"]
mod goal_tags_boxed;
#[path = "goal_tags_half_volley.rs"]
mod goal_tags_half_volley;
#[path = "goal_tags_mechanics.rs"]
mod goal_tags_mechanics;
#[path = "goal_tags_position.rs"]
mod goal_tags_position;

pub(crate) use goal_tags_boxed::*;
pub use goal_tags_half_volley::HalfVolleyGoalNode;
pub use goal_tags_mechanics::{
    AirDribbleGoalNode, DoubleTapGoalNode, FlickGoalNode, FlipResetGoalNode, OneTimerGoalNode,
    PassingGoalNode,
};
pub use goal_tags_position::{
    AerialGoalNode, CounterAttackGoalNode, EmptyNetGoalNode, HighAerialGoalNode,
    LongDistanceGoalNode, OwnHalfGoalNode,
};
