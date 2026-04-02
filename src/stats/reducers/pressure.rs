pub use crate::stats::calculators::pressure::*;
pub type PressureReducer = PressureCalculator;
pub type PressureReducerConfig = PressureCalculatorConfig;

use super::*;

impl StatsReducer for PressureReducer {
    fn on_sample(&mut self, sample: &FrameState) -> SubtrActorResult<()> {
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
        )
    }
}
