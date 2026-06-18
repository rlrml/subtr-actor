use crate::collector::frame_resolution::{
    FinalStatsFrameAction, StatsFramePersistenceController, StatsFrameResolution,
};
use crate::stats::analysis_graph::{
    AnalysisGraph, StatsTimelineEventsNode, StatsTimelineEventsState, StatsTimelineFrameNode,
    StatsTimelineFrameState,
};
use crate::stats::calculators::ReplayFrameInputBuilder;
use crate::*;
use std::collections::BTreeMap;

pub fn build_legacy_timeline_graph() -> AnalysisGraph {
    let mut graph = AnalysisGraph::new().with_input_state_type::<FrameInput>();
    graph.push_boxed_node(Box::new(StatsTimelineFrameNode::new()));
    graph.push_boxed_node(Box::new(StatsTimelineEventsNode::new()));
    graph
}

pub fn build_timeline_event_graph() -> AnalysisGraph {
    let mut graph = AnalysisGraph::new().with_input_state_type::<FrameInput>();
    graph.push_boxed_node(Box::new(StatsTimelineEventsNode::new()));
    graph
}

pub fn default_stats_timeline_config() -> StatsTimelineConfig {
    let rotation_defaults = RotationCalculatorConfig::default();
    let territorial_pressure_defaults = TerritorialPressureCalculatorConfig::default();
    StatsTimelineConfig {
        most_back_forward_threshold_y: PositioningCalculatorConfig::default()
            .most_back_forward_threshold_y,
        level_ball_depth_margin: PositioningCalculatorConfig::default().level_ball_depth_margin,
        closest_to_ball_switch_margin: PositioningCalculatorConfig::default()
            .closest_to_ball_switch_margin,
        closest_to_ball_switch_min_seconds: PositioningCalculatorConfig::default()
            .closest_to_ball_switch_min_seconds,
        ball_half_neutral_zone_half_width_y: BallHalfCalculatorConfig::default()
            .neutral_zone_half_width_y,
        ball_third_boundary_y: BallThirdCalculatorConfig::default().boundary_y,
        territorial_pressure_neutral_zone_half_width_y: territorial_pressure_defaults
            .neutral_zone_half_width_y,
        territorial_pressure_min_establish_seconds: territorial_pressure_defaults
            .min_establish_seconds,
        territorial_pressure_min_establish_third_seconds: territorial_pressure_defaults
            .min_establish_third_seconds,
        territorial_pressure_relief_grace_seconds: territorial_pressure_defaults
            .relief_grace_seconds,
        territorial_pressure_confirmed_relief_grace_seconds: territorial_pressure_defaults
            .confirmed_relief_grace_seconds,
        rotation_role_depth_margin: rotation_defaults.role_depth_margin,
        rotation_first_man_ambiguity_margin: rotation_defaults.first_man_ambiguity_margin,
        rotation_first_man_debounce_seconds: rotation_defaults.first_man_debounce_seconds,
        rotation_first_man_stint_end_grace_seconds: rotation_defaults
            .first_man_stint_end_grace_seconds,
        rush_max_start_y: RushCalculatorConfig::default().max_start_y,
        rush_attack_support_distance_y: RushCalculatorConfig::default().attack_support_distance_y,
        rush_defender_distance_y: RushCalculatorConfig::default().defender_distance_y,
        rush_min_possession_retained_seconds: RushCalculatorConfig::default()
            .min_possession_retained_seconds,
        aerial_goal_min_ball_z: AerialGoalCalculatorConfig::default().min_ball_z,
        high_aerial_goal_min_ball_z: HighAerialGoalCalculatorConfig::default().min_ball_z,
        long_distance_goal_max_attacking_y: LongDistanceGoalCalculatorConfig::default()
            .max_attacking_y,
        own_half_goal_max_attacking_y: OwnHalfGoalCalculatorConfig::default().max_attacking_y,
        empty_net_min_defender_y_margin: EmptyNetGoalCalculatorConfig::default()
            .min_defender_y_margin,
        empty_net_min_defender_distance: EmptyNetGoalCalculatorConfig::default()
            .min_defender_distance,
        empty_net_max_touch_attacking_y: EmptyNetGoalCalculatorConfig::default()
            .max_touch_attacking_y,
        flick_goal_max_event_to_goal_seconds: FlickGoalCalculatorConfig::default()
            .max_event_to_goal_seconds,
        ceiling_shot_goal_max_event_to_goal_seconds: CeilingShotGoalCalculatorConfig::default()
            .max_event_to_goal_seconds,
        double_tap_goal_max_event_to_goal_seconds: DoubleTapGoalCalculatorConfig::default()
            .max_event_to_goal_seconds,
        one_timer_goal_max_event_to_goal_seconds: OneTimerGoalCalculatorConfig::default()
            .max_event_to_goal_seconds,
        air_dribble_goal_max_end_to_goal_seconds: AirDribbleGoalCalculatorConfig::default()
            .max_end_to_goal_seconds,
        flip_reset_goal_max_event_to_goal_seconds: FlipResetGoalCalculatorConfig::default()
            .max_event_to_goal_seconds,
        flip_into_ball_goal_max_touch_to_goal_seconds: FlipIntoBallGoalCalculatorConfig::default()
            .max_touch_to_goal_seconds,
        bump_goal_max_event_to_goal_seconds: BumpGoalCalculatorConfig::default()
            .max_event_to_goal_seconds,
        demo_goal_max_event_to_goal_seconds: DemoGoalCalculatorConfig::default()
            .max_event_to_goal_seconds,
        half_volley_max_bounce_to_touch_seconds: HalfVolleyCalculatorConfig::default()
            .max_bounce_to_touch_seconds,
        half_volley_min_ball_speed: HalfVolleyCalculatorConfig::default().min_ball_speed,
        half_volley_goal_max_touch_to_goal_seconds: HalfVolleyGoalCalculatorConfig::default()
            .max_touch_to_goal_seconds,
        half_volley_goal_min_goal_alignment: HalfVolleyGoalCalculatorConfig::default()
            .min_goal_alignment,
    }
}

pub struct StatsTimelineCollector {
    graph: AnalysisGraph,
    replay_meta: Option<ReplayMeta>,
    frames: Vec<ReplayStatsFrame>,
    frame_input_builder: ReplayFrameInputBuilder,
    last_replay_meta_player_count: Option<usize>,
    last_sample_time: Option<f32>,
    frame_persistence: StatsFramePersistenceController,
}

impl Default for StatsTimelineCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsTimelineCollector {
    /// Create the legacy full-snapshot timeline collector.
    ///
    /// This evaluates and stores cumulative team/player stat modules for every
    /// captured frame. Prefer [`StatsTimelineEventCollector`] for compact
    /// event-backed transfer.
    pub fn new() -> Self {
        let graph = build_legacy_timeline_graph();
        Self {
            graph,
            replay_meta: None,
            frames: Vec::new(),
            frame_input_builder: ReplayFrameInputBuilder::default(),
            last_replay_meta_player_count: None,
            last_sample_time: None,
            frame_persistence: StatsFramePersistenceController::new(StatsFrameResolution::default()),
        }
    }

    fn timeline_config(&self) -> StatsTimelineConfig {
        default_stats_timeline_config()
    }

    fn snapshot_frame(&self) -> SubtrActorResult<ReplayStatsFrame> {
        self.graph
            .state::<StatsTimelineFrameState>()
            .and_then(|state| state.frame.clone())
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
                    "missing StatsTimelineFrame state while building timeline frame".to_owned(),
                ))
            })
    }

    pub fn into_legacy_replay_stats_timeline(self) -> SubtrActorResult<ReplayStatsTimeline> {
        let replay_meta = self
            .replay_meta
            .clone()
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::CouldNotBuildReplayMeta))?;
        let events = self
            .graph
            .state::<StatsTimelineEventsState>()
            .map(|state| state.events.clone())
            .unwrap_or_default();
        Ok(ReplayStatsTimeline {
            config: self.timeline_config(),
            replay_meta,
            events,
            frames: self.frames,
        })
    }

    pub fn with_frame_resolution(mut self, resolution: StatsFrameResolution) -> Self {
        self.frame_persistence = StatsFramePersistenceController::new(resolution);
        self
    }

    pub fn get_legacy_replay_stats_timeline(
        mut self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<ReplayStatsTimeline> {
        let mut processor = ReplayProcessor::new(replay)?;
        processor.process(&mut self)?;
        self.into_legacy_replay_stats_timeline()
    }

    pub fn into_legacy_timeline(self) -> ReplayStatsTimeline {
        self.into_legacy_replay_stats_timeline()
            .expect("analysis-node timeline collector should build typed stats frames")
    }
}

pub struct StatsTimelineEventCollector {
    graph: AnalysisGraph,
    replay_meta: Option<ReplayMeta>,
    frames: Vec<ReplayStatsFrameScaffold>,
    frame_input_builder: ReplayFrameInputBuilder,
    last_replay_meta_player_count: Option<usize>,
    last_sample_time: Option<f32>,
    frame_persistence: StatsFramePersistenceController,
}

impl Default for StatsTimelineEventCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsTimelineEventCollector {
    pub fn new() -> Self {
        Self {
            graph: build_timeline_event_graph(),
            replay_meta: None,
            frames: Vec::new(),
            frame_input_builder: ReplayFrameInputBuilder::default(),
            last_replay_meta_player_count: None,
            last_sample_time: None,
            frame_persistence: StatsFramePersistenceController::new(StatsFrameResolution::default()),
        }
    }

    pub fn with_frame_resolution(mut self, resolution: StatsFrameResolution) -> Self {
        self.frame_persistence = StatsFramePersistenceController::new(resolution);
        self
    }

    fn replay_meta(&self) -> SubtrActorResult<&ReplayMeta> {
        self.replay_meta
            .as_ref()
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::CouldNotBuildReplayMeta))
    }

    fn is_team_zero_player(replay_meta: &ReplayMeta, player: &PlayerInfo) -> bool {
        replay_meta
            .team_zero
            .iter()
            .any(|team_player| team_player.remote_id == player.remote_id)
    }

    fn snapshot_frame_scaffold(&self) -> SubtrActorResult<ReplayStatsFrameScaffold> {
        let replay_meta = self.replay_meta()?;
        let frame = self.graph.state::<FrameInfo>().ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
                "missing FrameInfo state while building stats timeline frame scaffold".to_owned(),
            ))
        })?;
        let gameplay = self.graph.state::<GameplayState>().ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
                "missing GameplayState state while building stats timeline frame scaffold"
                    .to_owned(),
            ))
        })?;
        let live_play_state = self.graph.state::<LivePlayState>().ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
                "missing LivePlayState state while building stats timeline frame scaffold"
                    .to_owned(),
            ))
        })?;

        Ok(ReplayStatsFrameScaffold {
            frame_number: frame.frame_number,
            time: frame.time,
            dt: frame.dt,
            seconds_remaining: frame.seconds_remaining,
            game_state: gameplay.game_state,
            ball_has_been_hit: gameplay.ball_has_been_hit,
            kickoff_countdown_time: gameplay.kickoff_countdown_time,
            gameplay_phase: live_play_state.gameplay_phase,
            is_live_play: live_play_state.is_live_play,
            team_zero: BTreeMap::new(),
            team_one: BTreeMap::new(),
            players: replay_meta
                .player_order()
                .map(|player| ReplayStatsPlayerIdentity {
                    player_id: player.remote_id.clone(),
                    name: player.name.clone(),
                    is_team_0: Self::is_team_zero_player(replay_meta, player),
                })
                .collect(),
        })
    }

    pub fn into_replay_stats_timeline_scaffold(
        self,
    ) -> SubtrActorResult<ReplayStatsTimelineScaffold> {
        let replay_meta = self
            .replay_meta
            .clone()
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::CouldNotBuildReplayMeta))?;
        let events = self
            .graph
            .state::<StatsTimelineEventsState>()
            .map(|state| state.events.clone())
            .unwrap_or_default();
        let positioning = self.graph.state::<PositioningCalculator>();
        let positioning_summary = replay_meta
            .player_order()
            .map(|player| ReplayStatsPositioningSummary {
                player_id: player.remote_id.clone(),
                is_team_0: Self::is_team_zero_player(&replay_meta, player),
                distance: positioning
                    .map(|calculator| calculator.player_signal(&player.remote_id))
                    .unwrap_or_default(),
            })
            .collect();
        let accumulation_tracks = self
            .graph
            .state::<BoostCalculator>()
            .map(|calculator| calculator.accumulation_tracks())
            .unwrap_or_default();
        Ok(ReplayStatsTimelineScaffold {
            config: default_stats_timeline_config(),
            replay_meta,
            events,
            frames: self.frames,
            positioning_summary,
            accumulation_tracks,
        })
    }

    pub fn get_replay_stats_timeline_scaffold(
        mut self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<ReplayStatsTimelineScaffold> {
        let mut processor = ReplayProcessor::new(replay)?;
        processor.process(&mut self)?;
        self.into_replay_stats_timeline_scaffold()
    }
}

impl Collector for StatsTimelineCollector {
    fn process_frame(
        &mut self,
        processor: &dyn ProcessorView,
        _frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        let player_count = processor.player_count();
        if self.last_replay_meta_player_count != Some(player_count) {
            let replay_meta = processor.get_replay_meta()?;
            self.graph.on_replay_meta(&replay_meta)?;
            self.replay_meta = Some(replay_meta);
            self.last_replay_meta_player_count = Some(player_count);
        }

        let dt = self
            .last_sample_time
            .map(|last_time| (current_time - last_time).max(0.0))
            .unwrap_or(0.0);
        let frame_input =
            self.frame_input_builder
                .timeline(processor, frame_number, current_time, dt);
        self.graph.evaluate_with_state(&frame_input)?;
        self.last_sample_time = Some(current_time);

        if let Some(emitted_dt) = self.frame_persistence.on_frame(frame_number, current_time) {
            let mut frame = self.snapshot_frame()?;
            frame.dt = emitted_dt;
            self.frames.push(frame);
        }

        Ok(TimeAdvance::NextFrame)
    }

    fn finish_replay(&mut self, _processor: &dyn ProcessorView) -> SubtrActorResult<()> {
        self.graph.finish()?;
        let Some(_) = self.replay_meta.as_ref() else {
            return Ok(());
        };
        let Some(_) = self.graph.state::<StatsTimelineFrameState>() else {
            return Ok(());
        };
        let mut final_snapshot = self.snapshot_frame()?;
        match self
            .frame_persistence
            .final_frame_action(final_snapshot.frame_number, final_snapshot.time)
        {
            Some(FinalStatsFrameAction::Append { dt }) => {
                final_snapshot.dt = dt;
                self.frames.push(final_snapshot);
            }
            Some(FinalStatsFrameAction::ReplaceLast { dt }) => {
                final_snapshot.dt = dt;
                if let Some(last_frame) = self.frames.last_mut() {
                    *last_frame = final_snapshot;
                }
            }
            None => {}
        }
        Ok(())
    }
}

impl Collector for StatsTimelineEventCollector {
    fn process_frame(
        &mut self,
        processor: &dyn ProcessorView,
        _frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        let player_count = processor.player_count();
        if self.last_replay_meta_player_count != Some(player_count) {
            let replay_meta = processor.get_replay_meta()?;
            self.graph.on_replay_meta(&replay_meta)?;
            self.replay_meta = Some(replay_meta);
            self.last_replay_meta_player_count = Some(player_count);
        }

        let dt = self
            .last_sample_time
            .map(|last_time| (current_time - last_time).max(0.0))
            .unwrap_or(0.0);
        let frame_input =
            self.frame_input_builder
                .timeline(processor, frame_number, current_time, dt);
        self.graph.evaluate_with_state(&frame_input)?;
        self.last_sample_time = Some(current_time);

        if let Some(emitted_dt) = self.frame_persistence.on_frame(frame_number, current_time) {
            let mut frame = self.snapshot_frame_scaffold()?;
            frame.dt = emitted_dt;
            self.frames.push(frame);
        }

        Ok(TimeAdvance::NextFrame)
    }

    fn finish_replay(&mut self, _processor: &dyn ProcessorView) -> SubtrActorResult<()> {
        self.graph.finish()?;
        let Some(_) = self.replay_meta.as_ref() else {
            return Ok(());
        };
        let Some(_) = self.graph.state::<FrameInfo>() else {
            return Ok(());
        };
        let mut final_snapshot = self.snapshot_frame_scaffold()?;
        match self
            .frame_persistence
            .final_frame_action(final_snapshot.frame_number, final_snapshot.time)
        {
            Some(FinalStatsFrameAction::Append { dt }) => {
                final_snapshot.dt = dt;
                self.frames.push(final_snapshot);
            }
            Some(FinalStatsFrameAction::ReplaceLast { dt }) => {
                final_snapshot.dt = dt;
                if let Some(last_frame) = self.frames.last_mut() {
                    *last_frame = final_snapshot;
                }
            }
            None => {}
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "collector_tests.rs"]
mod collector_tests;
