use crate::collector::frame_resolution::{
    FinalStatsFrameAction, StatsFramePersistenceController, StatsFrameResolution,
};
use crate::stats::analysis_nodes::analysis_graph::{AnalysisGraph, AnalysisNodeDyn};
use crate::stats::analysis_nodes::{
    boxed_analysis_node_by_name, LivePlayNode, PositioningNode, PressureNode, RushNode,
    StatsTimelineFrameNode, StatsTimelineFrameState,
};
use crate::*;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StatsTimelineConfig {
    pub most_back_forward_threshold_y: f32,
    pub pressure_neutral_zone_half_width_y: f32,
    pub rush_max_start_y: f32,
    pub rush_attack_support_distance_y: f32,
    pub rush_defender_distance_y: f32,
    pub rush_min_possession_retained_seconds: f32,
}

const PRESSURE_MODULE: &str = "pressure";
const RUSH_MODULE: &str = "rush";
const POSITIONING_MODULE: &str = "positioning";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReplayStatsTimeline {
    pub config: StatsTimelineConfig,
    pub replay_meta: ReplayMeta,
    pub timeline_events: Vec<TimelineEvent>,
    pub backboard_events: Vec<BackboardBounceEvent>,
    pub ceiling_shot_events: Vec<CeilingShotEvent>,
    pub double_tap_events: Vec<DoubleTapEvent>,
    pub fifty_fifty_events: Vec<FiftyFiftyEvent>,
    pub rush_events: Vec<RushEvent>,
    pub speed_flip_events: Vec<SpeedFlipEvent>,
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
pub struct ReplayStatsFrame {
    pub frame_number: usize,
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: Option<i32>,
    pub game_state: Option<i32>,
    pub is_live_play: bool,
    pub fifty_fifty: FiftyFiftyStats,
    pub possession: PossessionStats,
    pub pressure: PressureStats,
    pub rush: RushStats,
    pub team_zero: TeamStatsSnapshot,
    pub team_one: TeamStatsSnapshot,
    pub players: Vec<PlayerStatsSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TeamStatsSnapshot {
    pub core: CoreTeamStats,
    pub backboard: BackboardTeamStats,
    pub double_tap: DoubleTapTeamStats,
    pub ball_carry: BallCarryStats,
    pub boost: BoostStats,
    pub movement: MovementStats,
    pub powerslide: PowerslideStats,
    pub demo: DemoTeamStats,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlayerStatsSnapshot {
    pub player_id: PlayerId,
    pub name: String,
    pub is_team_0: bool,
    pub core: CorePlayerStats,
    pub backboard: BackboardPlayerStats,
    pub ceiling_shot: CeilingShotStats,
    pub double_tap: DoubleTapPlayerStats,
    pub fifty_fifty: FiftyFiftyPlayerStats,
    pub speed_flip: SpeedFlipStats,
    pub touch: TouchStats,
    pub musty_flick: MustyFlickStats,
    pub dodge_reset: DodgeResetStats,
    pub ball_carry: BallCarryStats,
    pub boost: BoostStats,
    pub movement: MovementStats,
    pub positioning: PositioningStats,
    pub powerslide: PowerslideStats,
    pub demo: DemoPlayerStats,
}

fn timeline_node_for_module(
    module_name: &str,
    positioning_config: &PositioningCalculatorConfig,
    pressure_config: &PressureCalculatorConfig,
    rush_config: &RushCalculatorConfig,
) -> SubtrActorResult<Box<dyn AnalysisNodeDyn>> {
    match module_name {
        POSITIONING_MODULE => Ok(Box::new(PositioningNode::with_config(
            positioning_config.clone(),
        ))),
        PRESSURE_MODULE => Ok(Box::new(PressureNode::with_config(pressure_config.clone()))),
        RUSH_MODULE => Ok(Box::new(RushNode::with_config(rush_config.clone()))),
        _ => boxed_analysis_node_by_name(module_name).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::UnknownStatsModuleName(
                module_name.to_owned(),
            ))
        }),
    }
}

fn build_timeline_graph(
    positioning_config: &PositioningCalculatorConfig,
    pressure_config: &PressureCalculatorConfig,
    rush_config: &RushCalculatorConfig,
) -> SubtrActorResult<AnalysisGraph> {
    let mut graph = AnalysisGraph::new().with_input_state_type::<FrameInput>();
    graph.push_boxed_node(Box::new(LivePlayNode::new()));
    for module_name in builtin_stats_module_names() {
        graph.push_boxed_node(timeline_node_for_module(
            module_name,
            positioning_config,
            pressure_config,
            rush_config,
        )?);
    }
    graph.push_boxed_node(Box::new(StatsTimelineFrameNode::new()));
    Ok(graph)
}

pub struct StatsTimelineCollector {
    graph: AnalysisGraph,
    positioning_config: PositioningCalculatorConfig,
    pressure_config: PressureCalculatorConfig,
    rush_config: RushCalculatorConfig,
    replay_meta: Option<ReplayMeta>,
    frames: Vec<ReplayStatsFrame>,
    last_sample_time: Option<f32>,
    frame_persistence: StatsFramePersistenceController,
}

impl Default for StatsTimelineCollector {
    fn default() -> Self {
        Self::with_configs(
            PositioningCalculatorConfig::default(),
            PressureCalculatorConfig::default(),
            RushCalculatorConfig::default(),
        )
    }
}

impl StatsTimelineCollector {
    fn with_configs(
        positioning_config: PositioningCalculatorConfig,
        pressure_config: PressureCalculatorConfig,
        rush_config: RushCalculatorConfig,
    ) -> Self {
        let graph = build_timeline_graph(&positioning_config, &pressure_config, &rush_config)
            .expect("builtin stats timeline modules should resolve without conflicts");
        Self {
            graph,
            positioning_config,
            pressure_config,
            rush_config,
            replay_meta: None,
            frames: Vec::new(),
            last_sample_time: None,
            frame_persistence: StatsFramePersistenceController::new(StatsFrameResolution::default()),
        }
    }

    fn timeline_config(&self) -> StatsTimelineConfig {
        StatsTimelineConfig {
            most_back_forward_threshold_y: self.positioning_config.most_back_forward_threshold_y,
            pressure_neutral_zone_half_width_y: self.pressure_config.neutral_zone_half_width_y,
            rush_max_start_y: self.rush_config.max_start_y,
            rush_attack_support_distance_y: self.rush_config.attack_support_distance_y,
            rush_defender_distance_y: self.rush_config.defender_distance_y,
            rush_min_possession_retained_seconds: self.rush_config.min_possession_retained_seconds,
        }
    }

    fn graph_value_or_default<T, S, F>(&self, project: F) -> T
    where
        T: Default,
        S: 'static,
        F: FnOnce(&S) -> T,
    {
        self.graph.state::<S>().map(project).unwrap_or_default()
    }

    fn timeline_events(&self) -> Vec<TimelineEvent> {
        let mut events = self
            .graph_value_or_default::<Vec<TimelineEvent>, MatchStatsCalculator, _>(|calculator| {
                calculator.timeline().to_vec()
            });
        events.extend(
            self.graph_value_or_default::<Vec<TimelineEvent>, DemoCalculator, _>(|calculator| {
                calculator.timeline().to_vec()
            }),
        );
        events.sort_by(|left, right| left.time.total_cmp(&right.time));
        events
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

    fn into_timeline_result(self) -> SubtrActorResult<ReplayStatsTimeline> {
        let replay_meta = self
            .replay_meta
            .clone()
            .ok_or_else(|| SubtrActorError::new(SubtrActorErrorVariant::CouldNotBuildReplayMeta))?;
        Ok(ReplayStatsTimeline {
            config: self.timeline_config(),
            replay_meta,
            timeline_events: self.timeline_events(),
            backboard_events: self
                .graph_value_or_default::<Vec<BackboardBounceEvent>, BackboardCalculator, _>(
                    |calculator| calculator.events().to_vec(),
                ),
            ceiling_shot_events: self
                .graph_value_or_default::<Vec<CeilingShotEvent>, CeilingShotCalculator, _>(
                    |calculator| calculator.events().to_vec(),
                ),
            double_tap_events: self
                .graph_value_or_default::<Vec<DoubleTapEvent>, DoubleTapCalculator, _>(
                    |calculator| calculator.events().to_vec(),
                ),
            fifty_fifty_events: self
                .graph_value_or_default::<Vec<FiftyFiftyEvent>, FiftyFiftyCalculator, _>(
                    |calculator| calculator.events().to_vec(),
                ),
            rush_events: self.graph_value_or_default::<Vec<RushEvent>, RushCalculator, _>(
                |calculator| calculator.events().to_vec(),
            ),
            speed_flip_events: self
                .graph_value_or_default::<Vec<SpeedFlipEvent>, SpeedFlipCalculator, _>(
                    |calculator| calculator.events().to_vec(),
                ),
            frames: self.frames,
        })
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_positioning_config(config: PositioningCalculatorConfig) -> Self {
        Self::with_configs(
            config,
            PressureCalculatorConfig::default(),
            RushCalculatorConfig::default(),
        )
    }

    pub fn with_pressure_config(config: PressureCalculatorConfig) -> Self {
        Self::with_configs(
            PositioningCalculatorConfig::default(),
            config,
            RushCalculatorConfig::default(),
        )
    }

    pub fn with_rush_config(config: RushCalculatorConfig) -> Self {
        Self::with_configs(
            PositioningCalculatorConfig::default(),
            PressureCalculatorConfig::default(),
            config,
        )
    }

    pub fn with_frame_resolution(mut self, resolution: StatsFrameResolution) -> Self {
        self.frame_persistence = StatsFramePersistenceController::new(resolution);
        self
    }

    pub fn get_replay_data(
        mut self,
        replay: &boxcars::Replay,
    ) -> SubtrActorResult<ReplayStatsTimeline> {
        let mut processor = ReplayProcessor::new(replay)?;
        processor.process(&mut self)?;
        self.into_timeline_result()
    }

    pub fn into_timeline(self) -> ReplayStatsTimeline {
        self.into_timeline_result()
            .expect("analysis-node timeline collector should build typed stats frames")
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
            self.graph.on_replay_meta(&replay_meta)?;
            self.replay_meta = Some(replay_meta);
        }

        let dt = self
            .last_sample_time
            .map(|last_time| (current_time - last_time).max(0.0))
            .unwrap_or(0.0);
        let frame_input = FrameInput::timeline(processor, frame_number, current_time, dt);
        self.graph.evaluate_with_state(&frame_input)?;
        self.last_sample_time = Some(current_time);

        if let Some(emitted_dt) = self.frame_persistence.on_frame(frame_number, current_time) {
            let mut frame = self.snapshot_frame()?;
            frame.dt = emitted_dt;
            self.frames.push(frame);
        }

        Ok(TimeAdvance::NextFrame)
    }

    fn finish_replay(&mut self, _processor: &ReplayProcessor) -> SubtrActorResult<()> {
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
