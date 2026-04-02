pub use crate::stats::calculators::rush::*;
pub type RushReducer = RushCalculator;
pub type RushReducerConfig = RushCalculatorConfig;

use super::*;

impl StatsReducer for RushReducer {
    fn required_derived_signals(&self) -> Vec<DerivedSignalId> {
        vec![POSSESSION_STATE_SIGNAL_ID]
    }

    fn on_sample_with_context(
        &mut self,
        sample: &CoreSample,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<()> {
        let default_state = PossessionState::default();
        let possession_state = ctx
            .get::<PossessionState>(POSSESSION_STATE_SIGNAL_ID)
            .unwrap_or(&default_state);
        self.update(sample, possession_state)
    }

    fn finish(&mut self) -> SubtrActorResult<()> {
        self.finish_calculation()
    }
}
