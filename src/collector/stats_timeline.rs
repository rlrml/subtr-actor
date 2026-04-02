use crate::stats::analysis_nodes::analysis_graph::{AnalysisGraph, AnalysisNodeDyn};
use crate::stats::analysis_nodes::{
    boxed_analysis_node_by_name, LivePlayNode, PositioningNode, PressureNode, RushNode,
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

    pub fn into_dynamic(self) -> DynamicReplayStatsTimeline {
        DynamicReplayStatsTimeline {
            config: self.config,
            replay_meta: self.replay_meta,
            timeline_events: self.timeline_events,
            backboard_events: self.backboard_events,
            ceiling_shot_events: self.ceiling_shot_events,
            double_tap_events: self.double_tap_events,
            fifty_fifty_events: self.fifty_fifty_events,
            rush_events: self.rush_events,
            speed_flip_events: self.speed_flip_events,
            frames: self
                .frames
                .into_iter()
                .map(ReplayStatsFrame::into_dynamic)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DynamicReplayStatsTimeline {
    pub config: StatsTimelineConfig,
    pub replay_meta: ReplayMeta,
    pub timeline_events: Vec<TimelineEvent>,
    pub backboard_events: Vec<BackboardBounceEvent>,
    pub ceiling_shot_events: Vec<CeilingShotEvent>,
    pub double_tap_events: Vec<DoubleTapEvent>,
    pub fifty_fifty_events: Vec<FiftyFiftyEvent>,
    pub rush_events: Vec<RushEvent>,
    pub speed_flip_events: Vec<SpeedFlipEvent>,
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
    pub fifty_fifty: FiftyFiftyStats,
    pub possession: PossessionStats,
    pub pressure: PressureStats,
    pub rush: RushStats,
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
    pub fifty_fifty: Vec<ExportedStat>,
    pub possession: Vec<ExportedStat>,
    pub pressure: Vec<ExportedStat>,
    pub rush: Vec<ExportedStat>,
    pub team_zero: DynamicTeamStatsSnapshot,
    pub team_one: DynamicTeamStatsSnapshot,
    pub players: Vec<DynamicPlayerStatsSnapshot>,
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
pub struct DynamicTeamStatsSnapshot {
    pub stats: Vec<ExportedStat>,
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
        self.backboard.visit_stat_fields(visitor);
        self.double_tap.visit_stat_fields(visitor);
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
        self.backboard.visit_stat_fields(visitor);
        self.ceiling_shot.visit_stat_fields(visitor);
        self.double_tap.visit_stat_fields(visitor);
        self.fifty_fifty.visit_stat_fields(visitor);
        self.speed_flip.visit_stat_fields(visitor);
        self.touch.visit_stat_fields(visitor);
        self.musty_flick.visit_stat_fields(visitor);
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
            fifty_fifty: self.fifty_fifty.stat_fields(),
            possession: self.possession.stat_fields(),
            pressure: self.pressure.stat_fields(),
            rush: self.rush.stat_fields(),
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
                        name: player.name.clone(),
                        is_team_0: player.is_team_0,
                        stats,
                    }
                })
                .collect(),
        }
    }
}
fn timeline_node_for_module(
    module_name: &str,
    positioning_config: &PositioningReducerConfig,
    pressure_config: &PressureReducerConfig,
    rush_config: &RushReducerConfig,
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
    positioning_config: &PositioningReducerConfig,
    pressure_config: &PressureReducerConfig,
    rush_config: &RushReducerConfig,
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
    Ok(graph)
}

pub struct StatsTimelineCollector {
    graph: AnalysisGraph,
    positioning_config: PositioningReducerConfig,
    pressure_config: PressureReducerConfig,
    rush_config: RushReducerConfig,
    replay_meta: Option<ReplayMeta>,
    frames: Vec<ReplayStatsFrame>,
    last_sample_time: Option<f32>,
}

impl Default for StatsTimelineCollector {
    fn default() -> Self {
        Self::with_configs(
            PositioningReducerConfig::default(),
            PressureReducerConfig::default(),
            RushReducerConfig::default(),
        )
    }
}

impl StatsTimelineCollector {
    fn with_configs(
        positioning_config: PositioningReducerConfig,
        pressure_config: PressureReducerConfig,
        rush_config: RushReducerConfig,
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

    fn is_team_zero_player(replay_meta: &ReplayMeta, player: &PlayerInfo) -> bool {
        replay_meta
            .team_zero
            .iter()
            .any(|team_player| team_player.remote_id == player.remote_id)
    }

    fn team_snapshot(&self, is_team_zero: bool) -> TeamStatsSnapshot {
        TeamStatsSnapshot {
            core: if is_team_zero {
                self.graph_value_or_default::<CoreTeamStats, MatchStatsCalculator, _>(
                    |calculator| calculator.team_zero_stats(),
                )
            } else {
                self.graph_value_or_default::<CoreTeamStats, MatchStatsCalculator, _>(
                    |calculator| calculator.team_one_stats(),
                )
            },
            backboard: if is_team_zero {
                self.graph_value_or_default::<BackboardTeamStats, BackboardCalculator, _>(
                    |calculator| calculator.team_zero_stats().clone(),
                )
            } else {
                self.graph_value_or_default::<BackboardTeamStats, BackboardCalculator, _>(
                    |calculator| calculator.team_one_stats().clone(),
                )
            },
            double_tap: if is_team_zero {
                self.graph_value_or_default::<DoubleTapTeamStats, DoubleTapCalculator, _>(
                    |calculator| calculator.team_zero_stats().clone(),
                )
            } else {
                self.graph_value_or_default::<DoubleTapTeamStats, DoubleTapCalculator, _>(
                    |calculator| calculator.team_one_stats().clone(),
                )
            },
            ball_carry: if is_team_zero {
                self.graph_value_or_default::<BallCarryStats, BallCarryCalculator, _>(
                    |calculator| calculator.team_zero_stats().clone(),
                )
            } else {
                self.graph_value_or_default::<BallCarryStats, BallCarryCalculator, _>(
                    |calculator| calculator.team_one_stats().clone(),
                )
            },
            boost: if is_team_zero {
                self.graph_value_or_default::<BoostStats, BoostCalculator, _>(|calculator| {
                    calculator.team_zero_stats().clone()
                })
            } else {
                self.graph_value_or_default::<BoostStats, BoostCalculator, _>(|calculator| {
                    calculator.team_one_stats().clone()
                })
            },
            movement: if is_team_zero {
                self.graph_value_or_default::<MovementStats, MovementCalculator, _>(|calculator| {
                    calculator.team_zero_stats().clone()
                })
            } else {
                self.graph_value_or_default::<MovementStats, MovementCalculator, _>(|calculator| {
                    calculator.team_one_stats().clone()
                })
            },
            powerslide: if is_team_zero {
                self.graph_value_or_default::<PowerslideStats, PowerslideCalculator, _>(
                    |calculator| calculator.team_zero_stats().clone(),
                )
            } else {
                self.graph_value_or_default::<PowerslideStats, PowerslideCalculator, _>(
                    |calculator| calculator.team_one_stats().clone(),
                )
            },
            demo: if is_team_zero {
                self.graph_value_or_default::<DemoTeamStats, DemoCalculator, _>(|calculator| {
                    calculator.team_zero_stats().clone()
                })
            } else {
                self.graph_value_or_default::<DemoTeamStats, DemoCalculator, _>(|calculator| {
                    calculator.team_one_stats().clone()
                })
            },
        }
    }

    fn player_snapshot(
        &self,
        replay_meta: &ReplayMeta,
        player: &PlayerInfo,
    ) -> PlayerStatsSnapshot {
        let player_id = &player.remote_id;
        PlayerStatsSnapshot {
            player_id: player.remote_id.clone(),
            name: player.name.clone(),
            is_team_0: Self::is_team_zero_player(replay_meta, player),
            core: self.graph_value_or_default::<CorePlayerStats, MatchStatsCalculator, _>(
                |calculator| {
                    calculator
                        .player_stats()
                        .get(player_id)
                        .cloned()
                        .unwrap_or_default()
                },
            ),
            backboard: self.graph_value_or_default::<BackboardPlayerStats, BackboardCalculator, _>(
                |calculator| {
                    calculator
                        .player_stats()
                        .get(player_id)
                        .cloned()
                        .unwrap_or_default()
                },
            ),
            ceiling_shot: self
                .graph_value_or_default::<CeilingShotStats, CeilingShotCalculator, _>(
                    |calculator| {
                        calculator
                            .player_stats()
                            .get(player_id)
                            .cloned()
                            .unwrap_or_default()
                    },
                ),
            double_tap: self
                .graph_value_or_default::<DoubleTapPlayerStats, DoubleTapCalculator, _>(
                    |calculator| {
                        calculator
                            .player_stats()
                            .get(player_id)
                            .cloned()
                            .unwrap_or_default()
                    },
                ),
            fifty_fifty: self
                .graph_value_or_default::<FiftyFiftyPlayerStats, FiftyFiftyCalculator, _>(
                    |calculator| {
                        calculator
                            .player_stats()
                            .get(player_id)
                            .cloned()
                            .unwrap_or_default()
                    },
                ),
            speed_flip: self.graph_value_or_default::<SpeedFlipStats, SpeedFlipCalculator, _>(
                |calculator| {
                    calculator
                        .player_stats()
                        .get(player_id)
                        .cloned()
                        .unwrap_or_default()
                },
            ),
            touch: self.graph_value_or_default::<TouchStats, TouchCalculator, _>(|calculator| {
                calculator
                    .player_stats()
                    .get(player_id)
                    .cloned()
                    .unwrap_or_default()
                    .with_complete_labeled_touch_counts()
            }),
            musty_flick: self.graph_value_or_default::<MustyFlickStats, MustyFlickCalculator, _>(
                |calculator| {
                    calculator
                        .player_stats()
                        .get(player_id)
                        .cloned()
                        .unwrap_or_default()
                },
            ),
            dodge_reset: self.graph_value_or_default::<DodgeResetStats, DodgeResetCalculator, _>(
                |calculator| {
                    calculator
                        .player_stats()
                        .get(player_id)
                        .cloned()
                        .unwrap_or_default()
                },
            ),
            ball_carry: self.graph_value_or_default::<BallCarryStats, BallCarryCalculator, _>(
                |calculator| {
                    calculator
                        .player_stats()
                        .get(player_id)
                        .cloned()
                        .unwrap_or_default()
                },
            ),
            boost: self.graph_value_or_default::<BoostStats, BoostCalculator, _>(|calculator| {
                calculator
                    .player_stats()
                    .get(player_id)
                    .cloned()
                    .unwrap_or_default()
            }),
            movement: self.graph_value_or_default::<MovementStats, MovementCalculator, _>(
                |calculator| {
                    calculator
                        .player_stats()
                        .get(player_id)
                        .cloned()
                        .unwrap_or_default()
                        .with_complete_labeled_tracked_time()
                },
            ),
            positioning: self.graph_value_or_default::<PositioningStats, PositioningCalculator, _>(
                |calculator| {
                    calculator
                        .player_stats()
                        .get(player_id)
                        .cloned()
                        .unwrap_or_default()
                },
            ),
            powerslide: self.graph_value_or_default::<PowerslideStats, PowerslideCalculator, _>(
                |calculator| {
                    calculator
                        .player_stats()
                        .get(player_id)
                        .cloned()
                        .unwrap_or_default()
                },
            ),
            demo: self.graph_value_or_default::<DemoPlayerStats, DemoCalculator, _>(|calculator| {
                calculator
                    .player_stats()
                    .get(player_id)
                    .cloned()
                    .unwrap_or_default()
            }),
        }
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

    fn snapshot_frame(&self, replay_meta: &ReplayMeta) -> SubtrActorResult<ReplayStatsFrame> {
        let frame = self.graph.state::<FrameInfo>().ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
                "missing FrameInfo state while building timeline frame".to_owned(),
            ))
        })?;
        let gameplay = self.graph.state::<GameplayState>().ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::CallbackError(
                "missing GameplayState state while building timeline frame".to_owned(),
            ))
        })?;
        let live_play = self
            .graph
            .state::<LivePlayState>()
            .map(|state| state.is_live_play)
            .unwrap_or(false);
        Ok(ReplayStatsFrame {
            frame_number: frame.frame_number,
            time: frame.time,
            dt: frame.dt,
            seconds_remaining: frame.seconds_remaining,
            game_state: gameplay.game_state,
            is_live_play: live_play,
            fifty_fifty: self.graph_value_or_default::<FiftyFiftyStats, FiftyFiftyCalculator, _>(
                |calculator| calculator.stats().clone(),
            ),
            possession: self.graph_value_or_default::<PossessionStats, PossessionCalculator, _>(
                |calculator| calculator.stats().clone(),
            ),
            pressure: self.graph_value_or_default::<PressureStats, PressureCalculator, _>(
                |calculator| calculator.stats().clone(),
            ),
            rush: self.graph_value_or_default::<RushStats, RushCalculator, _>(|calculator| {
                calculator.stats().clone()
            }),
            team_zero: self.team_snapshot(true),
            team_one: self.team_snapshot(false),
            players: replay_meta
                .player_order()
                .map(|player| self.player_snapshot(replay_meta, player))
                .collect(),
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

    pub fn with_positioning_config(config: PositioningReducerConfig) -> Self {
        Self::with_configs(
            config,
            PressureReducerConfig::default(),
            RushReducerConfig::default(),
        )
    }

    pub fn with_pressure_config(config: PressureReducerConfig) -> Self {
        Self::with_configs(
            PositioningReducerConfig::default(),
            config,
            RushReducerConfig::default(),
        )
    }

    pub fn with_rush_config(config: RushReducerConfig) -> Self {
        Self::with_configs(
            PositioningReducerConfig::default(),
            PressureReducerConfig::default(),
            config,
        )
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

        let replay_meta = self
            .replay_meta
            .as_ref()
            .expect("replay metadata should be initialized before snapshotting");
        self.frames.push(self.snapshot_frame(replay_meta)?);

        Ok(TimeAdvance::NextFrame)
    }

    fn finish_replay(&mut self, _processor: &ReplayProcessor) -> SubtrActorResult<()> {
        self.graph.finish()?;
        let Some(replay_meta) = self.replay_meta.as_ref() else {
            return Ok(());
        };
        let Some(_) = self.graph.state::<FrameInfo>() else {
            return Ok(());
        };
        let final_snapshot = self.snapshot_frame(replay_meta)?;
        if let Some(last_frame) = self.frames.last_mut() {
            *last_frame = final_snapshot;
        }
        Ok(())
    }
}
