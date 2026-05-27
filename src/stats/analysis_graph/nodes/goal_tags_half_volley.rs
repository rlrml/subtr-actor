use super::*;

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
