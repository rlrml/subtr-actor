pub use crate::stats::calculators::dodge_reset::*;
pub type DodgeResetReducer = DodgeResetCalculator;

use super::*;

impl StatsReducer for DodgeResetReducer {
    fn on_sample(&mut self, sample: &FrameState) -> SubtrActorResult<()> {
        self.update(
            &BallFrameState {
                ball: sample.ball.clone(),
            },
            &PlayerFrameState {
                players: sample.players.clone(),
            },
            &FrameEventsState {
                active_demos: sample.active_demos.clone(),
                demo_events: sample.demo_events.clone(),
                boost_pad_events: sample.boost_pad_events.clone(),
                touch_events: sample.touch_events.clone(),
                dodge_refreshed_events: sample.dodge_refreshed_events.clone(),
                player_stat_events: sample.player_stat_events.clone(),
                goal_events: sample.goal_events.clone(),
            },
        )
    }
}
