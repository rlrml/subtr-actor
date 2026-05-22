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
