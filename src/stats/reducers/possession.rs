pub use crate::stats::calculators::possession::*;
pub type PossessionReducer = PossessionCalculator;

use super::*;

impl StatsReducer for PossessionReducer {
    fn required_derived_signals(&self) -> Vec<DerivedSignalId> {
        vec![POSSESSION_STATE_SIGNAL_ID]
    }

    fn on_sample(&mut self, sample: &FrameState) -> SubtrActorResult<()> {
        self.update_from_sample_touch_events(sample)
    }

    fn on_sample_with_context(
        &mut self,
        sample: &FrameState,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<()> {
        let active_team_before_sample = ctx
            .get::<PossessionState>(POSSESSION_STATE_SIGNAL_ID)
            .and_then(|state| state.active_team_before_sample);
        self.update(
            &FrameInfo {
                frame_number: sample.frame_number,
                time: sample.time,
                dt: sample.dt,
                seconds_remaining: sample.seconds_remaining,
            },
            &BallFrameState {
                ball: sample.ball.clone(),
            },
            sample.is_live_play(),
            active_team_before_sample,
        )
    }
}
