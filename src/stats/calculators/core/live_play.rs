use super::{FrameEventsState, FrameState, GameplayState};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LivePlayState {
    pub is_live_play: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LivePlayTracker {
    post_goal_phase_active: bool,
    last_score: Option<(i32, i32)>,
}

impl LivePlayTracker {
    fn live_play_internal(&mut self, gameplay: &GameplayState, events: &FrameEventsState) -> bool {
        let kickoff_phase_active = gameplay.kickoff_phase_active();
        let score_changed = gameplay.current_score().zip(self.last_score).is_some_and(
            |((team_zero_score, team_one_score), (last_team_zero, last_team_one))| {
                team_zero_score > last_team_zero || team_one_score > last_team_one
            },
        );

        if !events.goal_events.is_empty() || score_changed {
            self.post_goal_phase_active = true;
        }

        let live_play = gameplay.is_live_play() && !self.post_goal_phase_active;

        if kickoff_phase_active {
            self.post_goal_phase_active = false;
        }

        if let Some(score) = gameplay.current_score() {
            self.last_score = Some(score);
        }

        live_play
    }

    pub fn is_live_play_parts(
        &mut self,
        gameplay: &GameplayState,
        events: &FrameEventsState,
    ) -> bool {
        self.live_play_internal(gameplay, events)
    }

    pub fn is_live_play(&mut self, sample: &FrameState) -> bool {
        self.live_play_internal(
            &GameplayState {
                game_state: sample.game_state,
                ball_has_been_hit: sample.ball_has_been_hit,
                kickoff_countdown_time: sample.kickoff_countdown_time,
                team_zero_score: sample.team_zero_score,
                team_one_score: sample.team_one_score,
                possession_team_is_team_0: sample.possession_team_is_team_0,
                scored_on_team_is_team_0: sample.scored_on_team_is_team_0,
                current_in_game_team_player_counts: sample
                    .current_in_game_team_player_counts
                    .unwrap_or_default(),
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
