pub use crate::stats::calculators::touch::*;
pub type TouchReducer = TouchCalculator;

use super::*;

impl StatsReducer for TouchReducer {
    fn required_derived_signals(&self) -> Vec<DerivedSignalId> {
        vec![TOUCH_STATE_SIGNAL_ID]
    }

    fn on_sample(&mut self, sample: &CoreSample) -> SubtrActorResult<()> {
        self.update_from_sample_touch_events(sample)
    }

    fn on_sample_with_context(
        &mut self,
        sample: &CoreSample,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<()> {
        let default_state = TouchState::default();
        let touch_state = ctx
            .get::<TouchState>(TOUCH_STATE_SIGNAL_ID)
            .unwrap_or(&default_state);
        self.update(sample, touch_state)
    }
}
