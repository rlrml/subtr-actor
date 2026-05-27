#[path = "live_play_phase.rs"]
mod phase;
#[path = "live_play_state.rs"]
mod state;

pub use phase::*;
pub use state::*;

use super::{FrameEventsState, GameplayState};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LivePlayTracker {
    post_goal_phase_active: bool,
    last_score: Option<(i32, i32)>,
}

impl LivePlayTracker {
    fn gameplay_phase_internal(
        &mut self,
        gameplay: &GameplayState,
        events: &FrameEventsState,
    ) -> GameplayPhase {
        let kickoff_phase_active = gameplay.kickoff_phase_active();
        let score_changed = gameplay.current_score().zip(self.last_score).is_some_and(
            |((team_zero_score, team_one_score), (last_team_zero, last_team_one))| {
                team_zero_score > last_team_zero || team_one_score > last_team_one
            },
        );

        if !events.goal_events.is_empty() || score_changed {
            self.post_goal_phase_active = true;
        }

        if kickoff_phase_active {
            self.post_goal_phase_active = false;
        }

        if let Some(score) = gameplay.current_score() {
            self.last_score = Some(score);
        }

        if gameplay.game_state == Some(crate::stats::calculators::GAME_STATE_KICKOFF_COUNTDOWN)
            || gameplay.kickoff_countdown_time.is_some_and(|time| time > 0)
        {
            GameplayPhase::KickoffCountdown
        } else if gameplay.game_state
            == Some(crate::stats::calculators::GAME_STATE_GOAL_SCORED_REPLAY)
            || self.post_goal_phase_active
        {
            GameplayPhase::PostGoal
        } else if gameplay.ball_has_been_hit == Some(false) {
            GameplayPhase::KickoffWaitingForTouch
        } else if gameplay.is_live_play() {
            GameplayPhase::ActivePlay
        } else {
            GameplayPhase::Unknown
        }
    }

    pub fn state_parts(
        &mut self,
        gameplay: &GameplayState,
        events: &FrameEventsState,
    ) -> LivePlayState {
        let gameplay_phase = self.gameplay_phase_internal(gameplay, events);
        LivePlayState {
            gameplay_phase,
            is_live_play: gameplay_phase.is_live_play(),
        }
    }
}

#[cfg(test)]
#[path = "live_play_tests.rs"]
mod tests;
