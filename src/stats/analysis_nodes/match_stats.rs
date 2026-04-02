use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct MatchStatsNode {
    calculator: MatchStatsCalculator,
}

impl MatchStatsNode {
    pub fn new() -> Self {
        Self {
            calculator: MatchStatsCalculator::new(),
        }
    }
}

impl Default for MatchStatsNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for MatchStatsNode {
    type State = MatchStatsCalculator;

    fn name(&self) -> &'static str {
        "match_stats"
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![core_sample_dependency()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let sample = ctx.get::<CoreSample>()?;
        self.calculator.update(sample)
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(MatchStatsNode::new())
}
