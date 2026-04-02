pub use crate::stats::calculators::backboard::*;
pub type BackboardReducer = BackboardCalculator;

use super::*;

impl StatsReducer for BackboardReducer {
    fn required_derived_signals(&self) -> Vec<DerivedSignalId> {
        vec![BACKBOARD_BOUNCE_STATE_SIGNAL_ID]
    }

    fn on_sample_with_context(
        &mut self,
        sample: &CoreSample,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<()> {
        let default_state = BackboardBounceState::default();
        let backboard_bounce_state = ctx
            .get::<BackboardBounceState>(BACKBOARD_BOUNCE_STATE_SIGNAL_ID)
            .unwrap_or(&default_state);
        self.update(sample, backboard_bounce_state)
    }
}
