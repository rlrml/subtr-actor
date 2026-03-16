use serde::Serialize;

use crate::*;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StatsTimelineConfig {
    pub most_back_forward_threshold_y: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReplayStatsTimeline {
    pub config: StatsTimelineConfig,
    pub replay_meta: ReplayMeta,
    pub timeline_events: Vec<TimelineEvent>,
    pub frames: Vec<ReplayStatsFrame>,
}

impl ReplayStatsTimeline {
    pub fn frame_by_number(&self, frame_number: usize) -> Option<&ReplayStatsFrame> {
        self.frames
            .iter()
            .find(|frame| frame.frame_number == frame_number)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DynamicReplayStatsTimeline {
    pub config: StatsTimelineConfig,
    pub replay_meta: ReplayMeta,
    pub timeline_events: Vec<TimelineEvent>,
    pub frames: Vec<DynamicReplayStatsFrame>,
}

impl DynamicReplayStatsTimeline {
    pub fn frame_by_number(&self, frame_number: usize) -> Option<&DynamicReplayStatsFrame> {
        self.frames
            .iter()
            .find(|frame| frame.frame_number == frame_number)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReplayStatsFrame {
    pub frame_number: usize,
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: Option<i32>,
    pub game_state: Option<i32>,
    pub is_live_play: bool,
    pub possession: PossessionStats,
    pub team_zero: TeamStatsSnapshot,
    pub team_one: TeamStatsSnapshot,
    pub players: Vec<PlayerStatsSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DynamicReplayStatsFrame {
    pub frame_number: usize,
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: Option<i32>,
    pub game_state: Option<i32>,
    pub is_live_play: bool,
    pub possession: Vec<ExportedStat>,
    pub team_zero: DynamicTeamStatsSnapshot,
    pub team_one: DynamicTeamStatsSnapshot,
    pub players: Vec<DynamicPlayerStatsSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TeamStatsSnapshot {
    pub core: CoreTeamStats,
    pub ball_carry: BallCarryStats,
    pub boost: BoostStats,
    pub movement: MovementStats,
    pub powerslide: PowerslideStats,
    pub demo: DemoTeamStats,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DynamicTeamStatsSnapshot {
    pub stats: Vec<ExportedStat>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlayerStatsSnapshot {
    pub player_id: PlayerId,
    pub name: String,
    pub is_team_0: bool,
    pub core: CorePlayerStats,
    pub dodge_reset: DodgeResetStats,
    pub ball_carry: BallCarryStats,
    pub boost: BoostStats,
    pub movement: MovementStats,
    pub positioning: PositioningStats,
    pub powerslide: PowerslideStats,
    pub demo: DemoPlayerStats,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DynamicPlayerStatsSnapshot {
    pub player_id: PlayerId,
    pub name: String,
    pub is_team_0: bool,
    pub stats: Vec<ExportedStat>,
}

impl StatFieldProvider for TeamStatsSnapshot {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        self.core.visit_stat_fields(visitor);
        self.ball_carry.visit_stat_fields(visitor);
        self.boost.visit_stat_fields(visitor);
        self.movement.visit_stat_fields(visitor);
        self.powerslide.visit_stat_fields(visitor);
        self.demo.visit_stat_fields(visitor);
    }
}

impl StatFieldProvider for PlayerStatsSnapshot {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        self.core.visit_stat_fields(visitor);
        self.dodge_reset.visit_stat_fields(visitor);
        self.ball_carry.visit_stat_fields(visitor);
        self.boost.visit_stat_fields(visitor);
        self.movement.visit_stat_fields(visitor);
        self.positioning.visit_stat_fields(visitor);
        self.powerslide.visit_stat_fields(visitor);
        self.demo.visit_stat_fields(visitor);
    }
}

impl ReplayStatsFrame {
    pub fn into_dynamic(self) -> DynamicReplayStatsFrame {
        DynamicReplayStatsFrame {
            frame_number: self.frame_number,
            time: self.time,
            dt: self.dt,
            seconds_remaining: self.seconds_remaining,
            game_state: self.game_state,
            is_live_play: self.is_live_play,
            possession: self.possession.stat_fields(),
            team_zero: DynamicTeamStatsSnapshot {
                stats: self.team_zero.stat_fields(),
            },
            team_one: DynamicTeamStatsSnapshot {
                stats: self.team_one.stat_fields(),
            },
            players: self
                .players
                .into_iter()
                .map(|player| {
                    let stats = player.stat_fields();
                    DynamicPlayerStatsSnapshot {
                        player_id: player.player_id,
                        name: player.name,
                        is_team_0: player.is_team_0,
                        stats,
                    }
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct StatsTimelineReducers {
    possession: PossessionReducer,
    match_stats: MatchStatsReducer,
    ball_carry: BallCarryReducer,
    boost: BoostReducer,
    movement: MovementReducer,
    positioning: PositioningReducer,
    powerslide: PowerslideReducer,
    demo: DemoReducer,
    dodge_reset: DodgeResetReducer,
}

impl StatsTimelineReducers {
    fn with_positioning_config(config: PositioningReducerConfig) -> Self {
        Self {
            positioning: PositioningReducer::with_config(config),
            ..Self::default()
        }
    }
}

impl StatsReducer for StatsTimelineReducers {
    fn on_replay_meta(&mut self, meta: &ReplayMeta) -> SubtrActorResult<()> {
        self.possession.on_replay_meta(meta)?;
        self.match_stats.on_replay_meta(meta)?;
        self.ball_carry.on_replay_meta(meta)?;
        self.boost.on_replay_meta(meta)?;
        self.movement.on_replay_meta(meta)?;
        self.positioning.on_replay_meta(meta)?;
        self.powerslide.on_replay_meta(meta)?;
        self.demo.on_replay_meta(meta)?;
        self.dodge_reset.on_replay_meta(meta)?;
        Ok(())
    }

    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        self.possession.on_sample(sample)?;
        self.match_stats.on_sample(sample)?;
        self.ball_carry.on_sample(sample)?;
        self.boost.on_sample(sample)?;
        self.movement.on_sample(sample)?;
        self.positioning.on_sample(sample)?;
        self.powerslide.on_sample(sample)?;
        self.demo.on_sample(sample)?;
        self.dodge_reset.on_sample(sample)?;
        Ok(())
    }

    fn finish(&mut self) -> SubtrActorResult<()> {
        self.possession.finish()?;
        self.match_stats.finish()?;
        self.ball_carry.finish()?;
        self.boost.finish()?;
        self.movement.finish()?;
        self.positioning.finish()?;
        self.powerslide.finish()?;
        self.demo.finish()?;
        self.dodge_reset.finish()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct StatsTimelineCollector {
    reducers: StatsTimelineReducers,
    replay_meta: Option<ReplayMeta>,
    frames: Vec<ReplayStatsFrame>,
    last_sample_time: Option<f32>,
    last_sample: Option<StatsSample>,
    last_live_play: Option<bool>,
    live_play_tracker: LivePlayTracker,
}

impl StatsTimelineCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_positioning_config(config: PositioningReducerConfig) -> Self {
        Self {
            reducers: StatsTimelineReducers::with_positioning_config(config),
            ..Self::default()
        }
    }

    pub fn get_replay_data(
        mut self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<ReplayStatsTimeline> {
        let mut processor = ReplayProcessor::new(replay)?;
        processor.process(&mut self)?;
        Ok(self.into_timeline())
    }

    pub fn get_dynamic_replay_data(
        mut self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<DynamicReplayStatsTimeline> {
        let mut processor = ReplayProcessor::new(replay)?;
        processor.process(&mut self)?;
        Ok(self.into_dynamic_timeline())
    }

    pub fn into_timeline(self) -> ReplayStatsTimeline {
        let replay_meta = self
            .replay_meta
            .expect("replay metadata should be initialized before building a stats timeline");
        let config = StatsTimelineConfig {
            most_back_forward_threshold_y: self
                .reducers
                .positioning
                .config()
                .most_back_forward_threshold_y,
        };
        let mut timeline_events = self.reducers.match_stats.timeline().to_vec();
        timeline_events.extend(self.reducers.demo.timeline().iter().cloned());
        timeline_events.sort_by(|left, right| left.time.total_cmp(&right.time));
        ReplayStatsTimeline {
            config,
            replay_meta,
            timeline_events,
            frames: self.frames,
        }
    }

    pub fn into_dynamic_timeline(self) -> DynamicReplayStatsTimeline {
        let replay_meta = self
            .replay_meta
            .expect("replay metadata should be initialized before building a stats timeline");
        let config = StatsTimelineConfig {
            most_back_forward_threshold_y: self
                .reducers
                .positioning
                .config()
                .most_back_forward_threshold_y,
        };
        let mut timeline_events = self.reducers.match_stats.timeline().to_vec();
        timeline_events.extend(self.reducers.demo.timeline().iter().cloned());
        timeline_events.sort_by(|left, right| left.time.total_cmp(&right.time));
        DynamicReplayStatsTimeline {
            config,
            replay_meta,
            timeline_events,
            frames: self
                .frames
                .into_iter()
                .map(ReplayStatsFrame::into_dynamic)
                .collect(),
        }
    }

    fn snapshot_frame(
        &self,
        sample: &StatsSample,
        replay_meta: &ReplayMeta,
        live_play: bool,
    ) -> ReplayStatsFrame {
        ReplayStatsFrame {
            frame_number: sample.frame_number,
            time: sample.time,
            dt: sample.dt,
            seconds_remaining: sample.seconds_remaining,
            game_state: sample.game_state,
            is_live_play: live_play,
            possession: self.reducers.possession.stats().clone(),
            team_zero: TeamStatsSnapshot {
                core: self.reducers.match_stats.team_zero_stats(),
                ball_carry: self.reducers.ball_carry.team_zero_stats().clone(),
                boost: self.reducers.boost.team_zero_stats().clone(),
                movement: self.reducers.movement.team_zero_stats().clone(),
                powerslide: self.reducers.powerslide.team_zero_stats().clone(),
                demo: self.reducers.demo.team_zero_stats().clone(),
            },
            team_one: TeamStatsSnapshot {
                core: self.reducers.match_stats.team_one_stats(),
                ball_carry: self.reducers.ball_carry.team_one_stats().clone(),
                boost: self.reducers.boost.team_one_stats().clone(),
                movement: self.reducers.movement.team_one_stats().clone(),
                powerslide: self.reducers.powerslide.team_one_stats().clone(),
                demo: self.reducers.demo.team_one_stats().clone(),
            },
            players: replay_meta
                .player_order()
                .map(|player| PlayerStatsSnapshot {
                    player_id: player.remote_id.clone(),
                    name: player.name.clone(),
                    is_team_0: replay_meta
                        .team_zero
                        .iter()
                        .any(|team_player| team_player.remote_id == player.remote_id),
                    core: self
                        .reducers
                        .match_stats
                        .player_stats()
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                    dodge_reset: self
                        .reducers
                        .dodge_reset
                        .player_stats()
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                    ball_carry: self
                        .reducers
                        .ball_carry
                        .player_stats()
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                    boost: self
                        .reducers
                        .boost
                        .player_stats()
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                    movement: self
                        .reducers
                        .movement
                        .player_stats()
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                    positioning: self
                        .reducers
                        .positioning
                        .player_stats()
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                    powerslide: self
                        .reducers
                        .powerslide
                        .player_stats()
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                    demo: self
                        .reducers
                        .demo
                        .player_stats()
                        .get(&player.remote_id)
                        .cloned()
                        .unwrap_or_default(),
                })
                .collect(),
        }
    }
}

impl Collector for StatsTimelineCollector {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        _frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        if self.replay_meta.is_none() {
            let replay_meta = processor.get_replay_meta()?;
            self.reducers.on_replay_meta(&replay_meta)?;
            self.replay_meta = Some(replay_meta);
        }

        let dt = self
            .last_sample_time
            .map(|last_time| (current_time - last_time).max(0.0))
            .unwrap_or(0.0);
        let sample = StatsSample::from_processor(processor, frame_number, current_time, dt)?;
        let live_play = self.live_play_tracker.is_live_play(&sample);
        self.reducers.on_sample(&sample)?;
        self.last_sample_time = Some(current_time);
        self.last_live_play = Some(live_play);

        let replay_meta = self
            .replay_meta
            .as_ref()
            .expect("replay metadata should be initialized before snapshotting");
        self.frames
            .push(self.snapshot_frame(&sample, replay_meta, live_play));
        self.last_sample = Some(sample);

        Ok(TimeAdvance::NextFrame)
    }

    fn finish_replay(&mut self, _processor: &ReplayProcessor) -> SubtrActorResult<()> {
        self.reducers.finish()?;
        let Some(last_sample) = self.last_sample.as_ref() else {
            return Ok(());
        };
        let Some(replay_meta) = self.replay_meta.as_ref() else {
            return Ok(());
        };
        let final_snapshot = self.snapshot_frame(
            last_sample,
            replay_meta,
            self.last_live_play.unwrap_or(false),
        );
        if let Some(last_frame) = self.frames.last_mut() {
            *last_frame = final_snapshot;
        }
        Ok(())
    }
}
