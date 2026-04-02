use super::{FrameEventsState, GameplayState};

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
}
