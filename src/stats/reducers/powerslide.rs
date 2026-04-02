pub use crate::stats::calculators::powerslide::*;
pub type PowerslideReducer = PowerslideCalculator;

use super::*;

impl StatsReducer for PowerslideReducer {
    fn on_sample(&mut self, sample: &FrameState) -> SubtrActorResult<()> {
        self.update(
            &FrameInfo {
                frame_number: sample.frame_number,
                time: sample.time,
                dt: sample.dt,
                seconds_remaining: sample.seconds_remaining,
            },
            &PlayerFrameState {
                players: sample.players.clone(),
            },
            sample.is_live_play(),
        )
    }
}
