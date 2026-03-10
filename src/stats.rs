use std::collections::HashMap;

use boxcars;

use crate::*;

#[derive(Debug, Clone)]
pub struct BallSample {
    pub rigid_body: boxcars::RigidBody,
}

impl BallSample {
    pub fn position(&self) -> glam::Vec3 {
        vec_to_glam(&self.rigid_body.location)
    }

    pub fn velocity(&self) -> glam::Vec3 {
        self.rigid_body
            .linear_velocity
            .as_ref()
            .map(vec_to_glam)
            .unwrap_or(glam::Vec3::ZERO)
    }
}

#[derive(Debug, Clone)]
pub struct PlayerSample {
    pub player_id: PlayerId,
    pub is_team_0: bool,
    pub rigid_body: Option<boxcars::RigidBody>,
    pub boost_amount: Option<f32>,
    pub boost_active: bool,
    pub powerslide_active: bool,
    pub match_goals: Option<i32>,
    pub match_assists: Option<i32>,
    pub match_saves: Option<i32>,
    pub match_shots: Option<i32>,
    pub match_score: Option<i32>,
}

impl PlayerSample {
    pub fn position(&self) -> Option<glam::Vec3> {
        self.rigid_body.as_ref().map(|rb| vec_to_glam(&rb.location))
    }

    pub fn velocity(&self) -> Option<glam::Vec3> {
        self.rigid_body
            .as_ref()
            .and_then(|rb| rb.linear_velocity.as_ref().map(vec_to_glam))
    }

    pub fn speed(&self) -> Option<f32> {
        self.velocity().map(|velocity| velocity.length())
    }
}

#[derive(Debug, Clone)]
pub struct StatsSample {
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: Option<i32>,
    pub game_state: Option<i32>,
    pub team_zero_score: Option<i32>,
    pub team_one_score: Option<i32>,
    pub possession_team_is_team_0: Option<bool>,
    pub scored_on_team_is_team_0: Option<bool>,
    pub ball: Option<BallSample>,
    pub players: Vec<PlayerSample>,
}

impl StatsSample {
    fn from_processor(
        processor: &ReplayProcessor,
        current_time: f32,
        dt: f32,
    ) -> SubtrActorResult<Self> {
        let ball = processor
            .get_interpolated_ball_rigid_body(current_time, 0.0)
            .ok()
            .filter(|rigid_body| !rigid_body.sleeping)
            .map(|rigid_body| BallSample { rigid_body });

        let players: SubtrActorResult<Vec<_>> = processor
            .iter_player_ids_in_order()
            .map(|player_id| {
                Ok(PlayerSample {
                    player_id: player_id.clone(),
                    is_team_0: processor.get_player_is_team_0(player_id)?,
                    rigid_body: processor
                        .get_interpolated_player_rigid_body(player_id, current_time, 0.0)
                        .ok()
                        .filter(|rigid_body| !rigid_body.sleeping),
                    boost_amount: processor.get_player_boost_level(player_id).ok(),
                    boost_active: processor.get_boost_active(player_id).unwrap_or(0) % 2 == 1,
                    powerslide_active: processor.get_powerslide_active(player_id).unwrap_or(false),
                    match_goals: processor.get_player_match_goals(player_id).ok(),
                    match_assists: processor.get_player_match_assists(player_id).ok(),
                    match_saves: processor.get_player_match_saves(player_id).ok(),
                    match_shots: processor.get_player_match_shots(player_id).ok(),
                    match_score: processor.get_player_match_score(player_id).ok(),
                })
            })
            .collect();

        let team_scores = processor.get_team_scores().ok();
        let possession_team_is_team_0 = processor.get_ball_hit_team_num().ok().and_then(|team_num| {
            match team_num {
                0 => Some(true),
                1 => Some(false),
                _ => None,
            }
        });
        let scored_on_team_is_team_0 = processor
            .get_scored_on_team_num()
            .ok()
            .and_then(|team_num| match team_num {
                0 => Some(true),
                1 => Some(false),
                _ => None,
            });

        Ok(Self {
            time: current_time,
            dt,
            seconds_remaining: processor.get_seconds_remaining().ok(),
            game_state: processor.get_replicated_state_name().ok(),
            team_zero_score: team_scores.map(|scores| scores.0),
            team_one_score: team_scores.map(|scores| scores.1),
            possession_team_is_team_0,
            scored_on_team_is_team_0,
            ball,
            players: players?,
        })
    }
}

pub trait StatsReducer {
    fn on_replay_meta(&mut self, _meta: &ReplayMeta) -> SubtrActorResult<()> {
        Ok(())
    }

    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()>;
}

impl<A: StatsReducer, B: StatsReducer> StatsReducer for (A, B) {
    fn on_replay_meta(&mut self, meta: &ReplayMeta) -> SubtrActorResult<()> {
        self.0.on_replay_meta(meta)?;
        self.1.on_replay_meta(meta)?;
        Ok(())
    }

    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        self.0.on_sample(sample)?;
        self.1.on_sample(sample)?;
        Ok(())
    }
}

impl<A: StatsReducer, B: StatsReducer, C: StatsReducer> StatsReducer for (A, B, C) {
    fn on_replay_meta(&mut self, meta: &ReplayMeta) -> SubtrActorResult<()> {
        self.0.on_replay_meta(meta)?;
        self.1.on_replay_meta(meta)?;
        self.2.on_replay_meta(meta)?;
        Ok(())
    }

    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        self.0.on_sample(sample)?;
        self.1.on_sample(sample)?;
        self.2.on_sample(sample)?;
        Ok(())
    }
}

pub struct ReducerCollector<R> {
    reducer: R,
    last_sample_time: Option<f32>,
    replay_meta_initialized: bool,
}

impl<R> ReducerCollector<R> {
    pub fn new(reducer: R) -> Self {
        Self {
            reducer,
            last_sample_time: None,
            replay_meta_initialized: false,
        }
    }

    pub fn into_inner(self) -> R {
        self.reducer
    }

    pub fn reducer(&self) -> &R {
        &self.reducer
    }

    pub fn reducer_mut(&mut self) -> &mut R {
        &mut self.reducer
    }
}

impl<R> From<R> for ReducerCollector<R> {
    fn from(reducer: R) -> Self {
        Self::new(reducer)
    }
}

impl<R: StatsReducer> Collector for ReducerCollector<R> {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        _frame: &boxcars::Frame,
        _frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        if !self.replay_meta_initialized {
            let replay_meta = processor.get_replay_meta()?;
            self.reducer.on_replay_meta(&replay_meta)?;
            self.replay_meta_initialized = true;
        }

        let dt = self
            .last_sample_time
            .map(|last_time| (current_time - last_time).max(0.0))
            .unwrap_or(0.0);
        let sample = StatsSample::from_processor(processor, current_time, dt)?;
        self.reducer.on_sample(&sample)?;
        self.last_sample_time = Some(current_time);

        Ok(TimeAdvance::NextFrame)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PowerslideStats {
    pub total_duration: f32,
    pub press_count: u32,
}

impl PowerslideStats {
    pub fn average_duration(&self) -> f32 {
        if self.press_count == 0 {
            0.0
        } else {
            self.total_duration / self.press_count as f32
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PowerslideReducer {
    player_stats: HashMap<PlayerId, PowerslideStats>,
    team_zero_stats: PowerslideStats,
    team_one_stats: PowerslideStats,
    last_active: HashMap<PlayerId, bool>,
}

impl PowerslideReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, PowerslideStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &PowerslideStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &PowerslideStats {
        &self.team_one_stats
    }
}

impl StatsReducer for PowerslideReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        for player in &sample.players {
            let previous_active = self
                .last_active
                .get(&player.player_id)
                .copied()
                .unwrap_or(false);
            let stats = self
                .player_stats
                .entry(player.player_id.clone())
                .or_default();
            let team_stats = if player.is_team_0 {
                &mut self.team_zero_stats
            } else {
                &mut self.team_one_stats
            };

            if player.powerslide_active {
                stats.total_duration += sample.dt;
                team_stats.total_duration += sample.dt;
            }

            if player.powerslide_active && !previous_active {
                stats.press_count += 1;
                team_stats.press_count += 1;
            }

            self.last_active
                .insert(player.player_id.clone(), player.powerslide_active);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PressureReducer {
    team_zero_side_duration: f32,
    team_one_side_duration: f32,
}

impl PressureReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn team_zero_side_duration(&self) -> f32 {
        self.team_zero_side_duration
    }

    pub fn team_one_side_duration(&self) -> f32 {
        self.team_one_side_duration
    }

    pub fn total_tracked_duration(&self) -> f32 {
        self.team_zero_side_duration + self.team_one_side_duration
    }
}

impl StatsReducer for PressureReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        if let Some(ball) = &sample.ball {
            if ball.position().y < 0.0 {
                self.team_zero_side_duration += sample.dt;
            } else {
                self.team_one_side_duration += sample.dt;
            }
        }
        Ok(())
    }
}
