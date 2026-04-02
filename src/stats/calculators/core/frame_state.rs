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
pub struct BallFrameState {
    pub ball: Option<BallSample>,
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

#[derive(Debug, Clone, Default)]
pub struct FrameState {
    pub frame_number: usize,
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: Option<i32>,
    pub game_state: Option<i32>,
    pub ball_has_been_hit: Option<bool>,
    pub kickoff_countdown_time: Option<i32>,
    pub team_zero_score: Option<i32>,
    pub team_one_score: Option<i32>,
    pub possession_team_is_team_0: Option<bool>,
    pub scored_on_team_is_team_0: Option<bool>,
    pub current_in_game_team_player_counts: Option<[usize; 2]>,
    pub ball: Option<BallSample>,
    pub players: Vec<PlayerSample>,
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

impl FrameState {
    pub(crate) fn from_processor(
        processor: &ReplayProcessor,
        frame_number: usize,
        current_time: f32,
        dt: f32,
    ) -> SubtrActorResult<Self> {
        super::FrameInput::timeline(processor, frame_number, current_time, dt).frame_state()
    }

    pub fn from_parts(
        frame: FrameInfo,
        gameplay: GameplayState,
        ball: BallFrameState,
        players: PlayerFrameState,
        events: FrameEventsState,
    ) -> Self {
        Self {
            frame_number: frame.frame_number,
            time: frame.time,
            dt: frame.dt,
            seconds_remaining: frame.seconds_remaining,
            game_state: gameplay.game_state,
            ball_has_been_hit: gameplay.ball_has_been_hit,
            kickoff_countdown_time: gameplay.kickoff_countdown_time,
            team_zero_score: gameplay.team_zero_score,
            team_one_score: gameplay.team_one_score,
            possession_team_is_team_0: gameplay.possession_team_is_team_0,
            scored_on_team_is_team_0: gameplay.scored_on_team_is_team_0,
            current_in_game_team_player_counts: Some(gameplay.current_in_game_team_player_counts),
            ball: ball.ball,
            players: players.players,
            active_demos: events.active_demos,
            demo_events: events.demo_events,
            boost_pad_events: events.boost_pad_events,
            touch_events: events.touch_events,
            dodge_refreshed_events: events.dodge_refreshed_events,
            player_stat_events: events.player_stat_events,
            goal_events: events.goal_events,
        }
    }

    pub fn is_live_play(&self) -> bool {
        !matches!(
            self.game_state,
            Some(GAME_STATE_KICKOFF_COUNTDOWN | GAME_STATE_GOAL_SCORED_REPLAY)
        )
    }

    pub fn current_in_game_team_player_count(&self, is_team_0: bool) -> usize {
        self.current_in_game_team_player_counts
            .map(|counts| counts[usize::from(!is_team_0)])
            .unwrap_or_else(|| {
                self.players
                    .iter()
                    .filter(|player| player.is_team_0 == is_team_0)
                    .count()
            })
    }
}
