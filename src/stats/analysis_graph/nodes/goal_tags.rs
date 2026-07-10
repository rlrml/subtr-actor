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
    (
        $node:ident,
        $calculator:ident,
        $name:literal,
        $dependency:ident,
        $dependency_type:ty
        $(, $extra_dependency:ident, $extra_dependency_type:ty)?
    ) => {
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
                vec![
                    match_stats_dependency(),
                    $dependency(),
                    $($extra_dependency(),)?
                ]
            }

            fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
                self.calculator.update(
                    ctx.get::<MatchStatsCalculator>()?,
                    ctx.get::<$dependency_type>()?,
                    $(ctx.get::<$extra_dependency_type>()?,)?
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
    LongDistanceGoalNode,
    LongDistanceGoalCalculator,
    "long_distance_goal"
);
goal_tag_node!(OwnHalfGoalNode, OwnHalfGoalCalculator, "own_half_goal");
goal_tag_node!(EmptyNetGoalNode, EmptyNetGoalCalculator, "empty_net_goal");
goal_tag_node!(
    CounterAttackGoalNode,
    CounterAttackGoalCalculator,
    "counter_attack_goal"
);
goal_tag_node!(
    SustainedPressureGoalNode,
    SustainedPressureGoalCalculator,
    "sustained_pressure_goal"
);
mechanic_goal_tag_node!(
    KickoffGoalNode,
    KickoffGoalCalculator,
    "kickoff_goal",
    kickoff_dependency,
    KickoffCalculator
);
mechanic_goal_tag_node!(
    FlickGoalNode,
    FlickGoalCalculator,
    "flick_goal",
    flick_dependency,
    FlickCalculator,
    touch_dependency,
    TouchCalculator
);
mechanic_goal_tag_node!(
    CeilingShotGoalNode,
    CeilingShotGoalCalculator,
    "ceiling_shot_goal",
    ceiling_shot_dependency,
    CeilingShotCalculator,
    touch_dependency,
    TouchCalculator
);
mechanic_goal_tag_node!(
    DoubleTapGoalNode,
    DoubleTapGoalCalculator,
    "double_tap_goal",
    double_tap_dependency,
    DoubleTapCalculator,
    touch_dependency,
    TouchCalculator
);
mechanic_goal_tag_node!(
    OneTimerGoalNode,
    OneTimerGoalCalculator,
    "one_timer_goal",
    one_timer_dependency,
    OneTimerCalculator,
    touch_dependency,
    TouchCalculator
);
mechanic_goal_tag_node!(
    PassingGoalNode,
    PassingGoalCalculator,
    "passing_goal",
    pass_dependency,
    PassCalculator
);
mechanic_goal_tag_node!(
    AirDribbleGoalNode,
    AirDribbleGoalCalculator,
    "air_dribble_goal",
    air_dribble_dependency,
    AirDribbleCalculator
);
mechanic_goal_tag_node!(
    FlipIntoBallGoalNode,
    FlipIntoBallGoalCalculator,
    "flip_into_ball_goal",
    touch_dependency,
    TouchCalculator
);
mechanic_goal_tag_node!(
    BumpGoalNode,
    BumpGoalCalculator,
    "bump_goal",
    bump_dependency,
    BumpCalculator
);
mechanic_goal_tag_node!(
    DemoGoalNode,
    DemoGoalCalculator,
    "demo_goal",
    demo_dependency,
    DemoCalculator
);

pub struct HighAerialGoalNode {
    calculator: HighAerialGoalCalculator,
}

impl HighAerialGoalNode {
    pub fn new() -> Self {
        Self {
            calculator: HighAerialGoalCalculator::new(),
        }
    }
}

impl Default for HighAerialGoalNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for HighAerialGoalNode {
    type State = HighAerialGoalCalculator;

    fn name(&self) -> &'static str {
        "high_aerial_goal"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            match_stats_dependency(),
            touch_dependency(),
            possession_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<MatchStatsCalculator>()?,
            ctx.get::<TouchCalculator>()?,
            ctx.get::<PossessionCalculator>()?,
        )
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub struct FlipResetGoalNode {
    calculator: FlipResetGoalCalculator,
}

impl FlipResetGoalNode {
    pub fn new() -> Self {
        Self {
            calculator: FlipResetGoalCalculator::new(),
        }
    }
}

impl Default for FlipResetGoalNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for FlipResetGoalNode {
    type State = FlipResetGoalCalculator;

    fn name(&self) -> &'static str {
        "flip_reset_goal"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            match_stats_dependency(),
            dodge_reset_dependency(),
            touch_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<MatchStatsCalculator>()?,
            ctx.get::<DodgeResetCalculator>()?,
            ctx.get::<TouchCalculator>()?,
        )
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

/// Tags goals scored via a half-volley by joining match-stats goals with half-volley events.
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

pub(crate) fn boxed_counter_attack_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(CounterAttackGoalNode::new())
}

pub(crate) fn boxed_sustained_pressure_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(SustainedPressureGoalNode::new())
}

pub(crate) fn boxed_kickoff_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(KickoffGoalNode::new())
}

pub(crate) fn boxed_flick_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(FlickGoalNode::new())
}

pub(crate) fn boxed_ceiling_shot_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(CeilingShotGoalNode::new())
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

pub(crate) fn boxed_flip_into_ball_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(FlipIntoBallGoalNode::new())
}

pub(crate) fn boxed_bump_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(BumpGoalNode::new())
}

pub(crate) fn boxed_demo_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(DemoGoalNode::new())
}

pub(crate) fn boxed_half_volley_goal() -> Box<dyn AnalysisNodeDyn> {
    Box::new(HalfVolleyGoalNode::new())
}
