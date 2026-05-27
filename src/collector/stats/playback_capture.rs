use serde::Serialize;
use serde_json::{Map, Value};

use crate::*;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CapturedStatsFrame<Modules> {
    pub frame_number: usize,
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: Option<i32>,
    pub game_state: Option<i32>,
    pub ball_has_been_hit: Option<bool>,
    pub kickoff_countdown_time: Option<i32>,
    pub gameplay_phase: GameplayPhase,
    pub is_live_play: bool,
    pub modules: Modules,
}

pub type StatsSnapshotFrame = CapturedStatsFrame<Map<String, Value>>;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CapturedStatsData<Frame> {
    pub replay_meta: ReplayMeta,
    pub config: Map<String, Value>,
    pub modules: Map<String, Value>,
    pub frames: Vec<Frame>,
}

pub type StatsSnapshotData = CapturedStatsData<StatsSnapshotFrame>;

impl<Modules> CapturedStatsFrame<Modules> {
    pub fn map_modules<Mapped, F>(
        self,
        transform: F,
    ) -> SubtrActorResult<CapturedStatsFrame<Mapped>>
    where
        F: FnOnce(Modules) -> SubtrActorResult<Mapped>,
    {
        Ok(CapturedStatsFrame {
            frame_number: self.frame_number,
            time: self.time,
            dt: self.dt,
            seconds_remaining: self.seconds_remaining,
            game_state: self.game_state,
            ball_has_been_hit: self.ball_has_been_hit,
            kickoff_countdown_time: self.kickoff_countdown_time,
            gameplay_phase: self.gameplay_phase,
            is_live_play: self.is_live_play,
            modules: transform(self.modules)?,
        })
    }
}
