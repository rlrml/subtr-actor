pub use crate::stats::calculators::ball_carry::*;
pub type BallCarryReducer = BallCarryCalculator;

use super::*;

impl StatsReducer for BallCarryReducer {
    fn required_derived_signals(&self) -> Vec<DerivedSignalId> {
        vec![TOUCH_STATE_SIGNAL_ID]
    }

    fn on_sample(&mut self, sample: &FrameState) -> SubtrActorResult<()> {
        self.update_from_sample_touch_events(sample)
    }

    fn on_sample_with_context(
        &mut self,
        sample: &FrameState,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<()> {
        let controlling_player = ctx
            .get::<TouchState>(TOUCH_STATE_SIGNAL_ID)
            .and_then(|state| state.last_touch_player.clone());
        self.update(sample, controlling_player)
    }

    fn finish(&mut self) -> SubtrActorResult<()> {
        self.finish_calculation()
    }
}
