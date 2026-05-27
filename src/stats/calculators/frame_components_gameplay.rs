#[derive(Debug, Clone, Default)]
pub struct GameplayState {
    pub game_state: Option<i32>,
    pub ball_has_been_hit: Option<bool>,
    pub kickoff_countdown_time: Option<i32>,
    pub team_zero_score: Option<i32>,
    pub team_one_score: Option<i32>,
    pub possession_team_is_team_0: Option<bool>,
    pub scored_on_team_is_team_0: Option<bool>,
    pub current_in_game_team_player_counts: [usize; 2],
}

impl GameplayState {
    pub fn is_live_play(&self) -> bool {
        !self.kickoff_phase_active() && self.game_state != Some(GAME_STATE_GOAL_SCORED_REPLAY)
    }

    pub fn current_score(&self) -> Option<(i32, i32)> {
        Some((self.team_zero_score?, self.team_one_score?))
    }

    pub fn kickoff_phase_active(&self) -> bool {
        self.game_state == Some(GAME_STATE_KICKOFF_COUNTDOWN)
            || self.kickoff_countdown_time.is_some_and(|time| time > 0)
            || self.ball_has_been_hit == Some(false)
    }

    pub fn current_in_game_team_player_count(&self, is_team_0: bool) -> usize {
        self.current_in_game_team_player_counts[usize::from(!is_team_0)]
    }
}

pub(crate) const GAME_STATE_KICKOFF_COUNTDOWN: i32 = 55;
pub(crate) const GAME_STATE_GOAL_SCORED_REPLAY: i32 = 86;
