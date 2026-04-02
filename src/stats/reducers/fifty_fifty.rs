pub use crate::stats::calculators::fifty_fifty::*;
pub type FiftyFiftyReducer = FiftyFiftyCalculator;

use super::*;

impl StatsReducer for FiftyFiftyReducer {
    fn required_derived_signals(&self) -> Vec<DerivedSignalId> {
        vec![FIFTY_FIFTY_STATE_SIGNAL_ID]
    }

    fn on_sample_with_context(
        &mut self,
        _sample: &CoreSample,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<()> {
        let default_state = FiftyFiftyState::default();
        let fifty_fifty_state = ctx
            .get::<FiftyFiftyState>(FIFTY_FIFTY_STATE_SIGNAL_ID)
            .unwrap_or(&default_state);
        self.update(fifty_fifty_state)
    }
}
