use super::{FrameEventsState, GameplayState};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LivePlayState {
    pub gameplay_phase: GameplayPhase,
    pub is_live_play: bool,
}

impl LivePlayState {
    pub fn counts_toward_player_motion(&self) -> bool {
        self.gameplay_phase.counts_toward_player_motion()
    }

    pub fn counts_toward_ball_position_stats(&self) -> bool {
        self.gameplay_phase.counts_toward_ball_position_stats()
    }
}

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
mod tests {
    use super::*;
    use crate::GoalEvent;

    #[test]
    fn kickoff_waiting_for_first_touch_is_not_live_play() {
        let mut tracker = LivePlayTracker::default();
        let gameplay = GameplayState {
            ball_has_been_hit: Some(false),
            ..Default::default()
        };
        let state = tracker.state_parts(&gameplay, &FrameEventsState::default());

        assert_eq!(state.gameplay_phase, GameplayPhase::KickoffWaitingForTouch);
        assert!(!state.is_live_play);
        assert!(state.gameplay_phase.counts_toward_player_motion());
    }

    #[test]
    fn goal_event_enters_post_goal_phase() {
        let mut tracker = LivePlayTracker::default();
        let gameplay = GameplayState::default();
        let events = FrameEventsState {
            goal_events: vec![GoalEvent {
                time: 10.0,
                frame: 1,
                scoring_team_is_team_0: true,
                player: None,
                team_zero_score: None,
                team_one_score: None,
            }],
            ..Default::default()
        };

        let state = tracker.state_parts(&gameplay, &events);

        assert_eq!(state.gameplay_phase, GameplayPhase::PostGoal);
        assert!(!state.is_live_play);
    }
}
