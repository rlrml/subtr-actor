use super::{FrameEventsState, GameplayState};
use serde::{Deserialize, Serialize};

/// The current phase of gameplay (kickoff, active, replay, etc.).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GameplayPhase {
    #[default]
    Unknown,
    KickoffCountdown,
    KickoffWaitingForTouch,
    ActivePlay,
    PostGoal,
}

impl GameplayPhase {
    pub fn is_live_play(self) -> bool {
        matches!(self, Self::ActivePlay)
    }

    pub fn counts_toward_player_motion(self) -> bool {
        matches!(self, Self::ActivePlay | Self::KickoffWaitingForTouch)
    }

    pub fn counts_toward_ball_position_stats(self) -> bool {
        self.is_live_play()
    }
}

/// Shared state describing whether the current frame is live play.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LivePlayState {
    pub gameplay_phase: GameplayPhase,
    pub is_live_play: bool,
}

impl LivePlayState {
    pub fn new(gameplay_phase: GameplayPhase) -> Self {
        Self {
            gameplay_phase,
            is_live_play: gameplay_phase.is_live_play(),
        }
    }

    pub fn active_play() -> Self {
        Self::new(GameplayPhase::ActivePlay)
    }

    pub fn counts_toward_player_motion(&self) -> bool {
        self.gameplay_phase.counts_toward_player_motion()
    }

    pub fn counts_toward_ball_position_stats(&self) -> bool {
        self.gameplay_phase.counts_toward_ball_position_stats()
    }
}

/// Determines whether each frame is live play and its gameplay phase.
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

        if gameplay.kickoff_countdown_active() {
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
        LivePlayState::new(gameplay_phase)
    }
}

#[cfg(test)]
#[path = "live_play_tests.rs"]
mod tests;
