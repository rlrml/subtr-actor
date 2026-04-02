pub use crate::stats::calculators::positioning::*;
pub type PositioningReducer = PositioningCalculator;
pub type PositioningReducerConfig = PositioningCalculatorConfig;

use super::*;

impl StatsReducer for PositioningReducer {
    fn required_derived_signals(&self) -> Vec<DerivedSignalId> {
        vec![POSSESSION_STATE_SIGNAL_ID]
    }

    fn on_sample(&mut self, sample: &CoreSample) -> SubtrActorResult<()> {
        self.update_from_sample_touch_events(sample)
    }

    fn on_sample_with_context(
        &mut self,
        sample: &CoreSample,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<()> {
        let possession_player_before_sample = ctx
            .get::<PossessionState>(POSSESSION_STATE_SIGNAL_ID)
            .and_then(|state| state.active_player_before_sample.as_ref());
        self.update(sample, possession_player_before_sample)
    }
}
