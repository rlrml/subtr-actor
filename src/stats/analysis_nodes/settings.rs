use super::analysis_graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct SettingsNode {
    calculator: SettingsCalculator,
}

impl SettingsNode {
    pub fn new() -> Self {
        Self {
            calculator: SettingsCalculator::new(),
        }
    }
}

impl Default for SettingsNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for SettingsNode {
    type State = SettingsCalculator;

    fn name(&self) -> &'static str {
        "settings"
    }

    fn on_replay_meta(&mut self, meta: &ReplayMeta) -> SubtrActorResult<()> {
        self.calculator.apply_replay_meta(meta)
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
    Box::new(SettingsNode::new())
}
