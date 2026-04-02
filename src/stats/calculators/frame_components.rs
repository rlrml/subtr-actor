use crate::*;

use super::{BallSample, DemoEventSample, PlayerSample};

#[derive(Debug, Clone, Default)]
pub struct FrameInfo {
    pub frame_number: usize,
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: Option<i32>,
}

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
        !matches!(
            self.game_state,
            Some(GAME_STATE_KICKOFF_COUNTDOWN | GAME_STATE_GOAL_SCORED_REPLAY)
        )
    }

    pub fn current_score(&self) -> Option<(i32, i32)> {
        Some((self.team_zero_score?, self.team_one_score?))
    }

    pub fn kickoff_phase_active(&self) -> bool {
        self.game_state == Some(GAME_STATE_KICKOFF_COUNTDOWN)
            || self.kickoff_countdown_time.is_some_and(|time| time > 0)
    }

    pub fn current_in_game_team_player_count(&self, is_team_0: bool) -> usize {
        self.current_in_game_team_player_counts[usize::from(!is_team_0)]
    }
}

#[derive(Debug, Clone, Default)]
pub enum BallFrameState {
    #[default]
    Missing,
    Present(BallSample),
}

impl BallFrameState {
    pub fn sample(&self) -> Option<&BallSample> {
        match self {
            Self::Missing => None,
            Self::Present(ball) => Some(ball),
        }
    }

    pub fn into_sample(self) -> Option<BallSample> {
        match self {
            Self::Missing => None,
            Self::Present(ball) => Some(ball),
        }
    }

    pub fn position(&self) -> Option<glam::Vec3> {
        self.sample().map(BallSample::position)
    }

    pub fn velocity(&self) -> Option<glam::Vec3> {
        self.sample().map(BallSample::velocity)
    }
}

impl From<BallSample> for BallFrameState {
    fn from(ball: BallSample) -> Self {
        Self::Present(ball)
    }
}

impl From<Option<BallSample>> for BallFrameState {
    fn from(ball: Option<BallSample>) -> Self {
        match ball {
            Some(ball) => Self::Present(ball),
            None => Self::Missing,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PlayerFrameState {
    pub players: Vec<PlayerSample>,
}

#[derive(Debug, Clone, Default)]
pub struct FrameEventsState {
    pub active_demos: Vec<DemoEventSample>,
    pub demo_events: Vec<DemolishInfo>,
    pub boost_pad_events: Vec<BoostPadEvent>,
    pub touch_events: Vec<TouchEvent>,
    pub dodge_refreshed_events: Vec<DodgeRefreshedEvent>,
    pub player_stat_events: Vec<PlayerStatEvent>,
    pub goal_events: Vec<GoalEvent>,
}

pub(crate) const GAME_STATE_KICKOFF_COUNTDOWN: i32 = 55;
pub(crate) const GAME_STATE_GOAL_SCORED_REPLAY: i32 = 86;
