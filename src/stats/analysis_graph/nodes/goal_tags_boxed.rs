use super::*;

pub(crate) fn boxed_aerial_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(AerialGoalNode::new())
}

pub(crate) fn boxed_high_aerial_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(HighAerialGoalNode::new())
}

pub(crate) fn boxed_long_distance_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(LongDistanceGoalNode::new())
}

pub(crate) fn boxed_own_half_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(OwnHalfGoalNode::new())
}

pub(crate) fn boxed_empty_net_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(EmptyNetGoalNode::new())
}

pub(crate) fn boxed_counter_attack_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(CounterAttackGoalNode::new())
}

pub(crate) fn boxed_flick_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(FlickGoalNode::new())
}

pub(crate) fn boxed_double_tap_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(DoubleTapGoalNode::new())
}

pub(crate) fn boxed_one_timer_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(OneTimerGoalNode::new())
}

pub(crate) fn boxed_passing_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(PassingGoalNode::new())
}

pub(crate) fn boxed_air_dribble_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(AirDribbleGoalNode::new())
}

pub(crate) fn boxed_flip_reset_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(FlipResetGoalNode::new())
}

pub(crate) fn boxed_half_volley_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(HalfVolleyGoalNode::new())
}
