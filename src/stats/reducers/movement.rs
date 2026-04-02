pub use crate::stats::calculators::movement::*;
pub type MovementReducer = MovementCalculator;

use super::*;

impl StatsReducer for MovementReducer {
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
