pub use crate::stats::calculators::musty_flick::*;
pub type MustyFlickReducer = MustyFlickCalculator;

use super::*;

impl StatsReducer for MustyFlickReducer {
    fn required_derived_signals(&self) -> Vec<DerivedSignalId> {
        vec![TOUCH_STATE_SIGNAL_ID]
    }

    fn on_sample(&mut self, sample: &CoreSample) -> SubtrActorResult<()> {
        self.update(sample, &sample.touch_events)
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
        self.update(sample, &touch_state.touch_events)
    }
}
