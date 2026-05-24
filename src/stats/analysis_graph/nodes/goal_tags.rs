use super::*;
use crate::stats::calculators::*;
use crate::*;

macro_rules! goal_tag_node {
    ($node:ident, $calculator:ident, $name:literal) => {
        pub struct $node {
            calculator: $calculator,
        }

        impl $node {
            pub fn new() -> Self {
                Self {
                    calculator: $calculator::new(),
                }
            }
        }

        impl Default for $node {
            fn default() -> Self {
                Self::new()
            }
        }

        impl AnalysisNode for $node {
            type State = $calculator;

            fn name(&self) -> &'static str {
                $name
            }

            fn dependencies(&self) -> NodeDependencies {
                vec![match_stats_dependency()]
            }

            fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
                self.calculator.update(ctx.get::<MatchStatsCalculator>()?)
            }

            fn state(&self) -> &Self::State {
                &self.calculator
            }
        }
    };
}

macro_rules! mechanic_goal_tag_node {
    ($node:ident, $calculator:ident, $name:literal, $dependency:ident, $dependency_type:ty) => {
        pub struct $node {
            calculator: $calculator,
        }

        impl $node {
            pub fn new() -> Self {
                Self {
                    calculator: $calculator::new(),
                }
            }
        }

        impl Default for $node {
            fn default() -> Self {
                Self::new()
            }
        }

        impl AnalysisNode for $node {
            type State = $calculator;

            fn name(&self) -> &'static str {
                $name
            }

            fn dependencies(&self) -> NodeDependencies {
                vec![match_stats_dependency(), $dependency()]
            }

            fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
                self.calculator.update(
                    ctx.get::<MatchStatsCalculator>()?,
                    ctx.get::<$dependency_type>()?,
                )
            }

            fn state(&self) -> &Self::State {
                &self.calculator
            }
        }
    };
}

goal_tag_node!(AerialGoalNode, AerialGoalCalculator, "aerial_goal");
goal_tag_node!(
    HighAerialGoalNode,
    HighAerialGoalCalculator,
    "high_aerial_goal"
);
goal_tag_node!(
    LongDistanceGoalNode,
    LongDistanceGoalCalculator,
    "long_distance_goal"
);
goal_tag_node!(OwnHalfGoalNode, OwnHalfGoalCalculator, "own_half_goal");
goal_tag_node!(EmptyNetGoalNode, EmptyNetGoalCalculator, "empty_net_goal");
mechanic_goal_tag_node!(
    FlickGoalNode,
    FlickGoalCalculator,
    "flick_goal",
    flick_dependency,
    FlickCalculator
);
mechanic_goal_tag_node!(
    OneTimerGoalNode,
    OneTimerGoalCalculator,
    "one_timer_goal",
    one_timer_dependency,
    OneTimerCalculator
);
mechanic_goal_tag_node!(
    AirDribbleGoalNode,
    AirDribbleGoalCalculator,
    "air_dribble_goal",
    ball_carry_dependency,
    BallCarryCalculator
);
mechanic_goal_tag_node!(
    FlipResetGoalNode,
    FlipResetGoalCalculator,
    "flip_reset_goal",
    dodge_reset_dependency,
    DodgeResetCalculator
);

pub struct HalfVolleyGoalNode {
    calculator: HalfVolleyGoalCalculator,
}

impl HalfVolleyGoalNode {
    pub fn new() -> Self {
        Self {
            calculator: HalfVolleyGoalCalculator::new(),
        }
    }
}

impl Default for HalfVolleyGoalNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for HalfVolleyGoalNode {
    type State = HalfVolleyGoalCalculator;

    fn name(&self) -> &'static str {
        "half_volley_goal"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![match_stats_dependency(), half_volley_dependency()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<MatchStatsCalculator>()?,
            ctx.get::<HalfVolleyCalculator>()?,
        )
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

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

pub(crate) fn boxed_flick_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(FlickGoalNode::new())
}

pub(crate) fn boxed_one_timer_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(OneTimerGoalNode::new())
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
