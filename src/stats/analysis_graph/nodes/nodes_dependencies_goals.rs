use super::*;
use crate::stats::calculators::{
    AerialGoalCalculator, AirDribbleGoalCalculator, CounterAttackGoalCalculator,
    DoubleTapGoalCalculator, EmptyNetGoalCalculator, FlickGoalCalculator, FlipResetGoalCalculator,
    HalfVolleyGoalCalculator, HighAerialGoalCalculator, LongDistanceGoalCalculator,
    OneTimerGoalCalculator, OwnHalfGoalCalculator, PassingGoalCalculator,
};

pub(crate) fn aerial_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<AerialGoalCalculator>(goal_tags::boxed_aerial_goal)
}

pub(crate) fn high_aerial_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<HighAerialGoalCalculator>(goal_tags::boxed_high_aerial_goal)
}

pub(crate) fn long_distance_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<LongDistanceGoalCalculator>(
        goal_tags::boxed_long_distance_goal,
    )
}

pub(crate) fn own_half_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<OwnHalfGoalCalculator>(goal_tags::boxed_own_half_goal)
}

pub(crate) fn empty_net_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<EmptyNetGoalCalculator>(goal_tags::boxed_empty_net_goal)
}

pub(crate) fn counter_attack_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<CounterAttackGoalCalculator>(
        goal_tags::boxed_counter_attack_goal,
    )
}

pub(crate) fn flick_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<FlickGoalCalculator>(goal_tags::boxed_flick_goal)
}

pub(crate) fn double_tap_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<DoubleTapGoalCalculator>(goal_tags::boxed_double_tap_goal)
}

pub(crate) fn one_timer_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<OneTimerGoalCalculator>(goal_tags::boxed_one_timer_goal)
}

pub(crate) fn passing_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<PassingGoalCalculator>(goal_tags::boxed_passing_goal)
}

pub(crate) fn air_dribble_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<AirDribbleGoalCalculator>(goal_tags::boxed_air_dribble_goal)
}

pub(crate) fn flip_reset_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<FlipResetGoalCalculator>(goal_tags::boxed_flip_reset_goal)
}

pub(crate) fn half_volley_goal_dependency() -> AnalysisDependency {
    AnalysisDependency::with_default::<HalfVolleyGoalCalculator>(goal_tags::boxed_half_volley_goal)
}
