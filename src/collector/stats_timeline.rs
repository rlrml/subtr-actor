use crate::collector::frame_resolution::{
    FinalStatsFrameAction, StatsFramePersistenceController, StatsFrameResolution,
};
use crate::stats::analysis_nodes::analysis_graph::AnalysisGraph;
use crate::stats::analysis_nodes::{
    StatsTimelineEventsNode, StatsTimelineEventsState, StatsTimelineFrameNode,
    StatsTimelineFrameState,
};
use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct StatsTimelineConfig {
    pub most_back_forward_threshold_y: f32,
    pub pressure_neutral_zone_half_width_y: f32,
    pub rush_max_start_y: f32,
    pub rush_attack_support_distance_y: f32,
    pub rush_defender_distance_y: f32,
    pub rush_min_possession_retained_seconds: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayStatsTimeline {
    pub config: StatsTimelineConfig,
    pub replay_meta: ReplayMeta,
    pub events: ReplayStatsTimelineEvents,
    pub frames: Vec<ReplayStatsFrame>,
}

impl ReplayStatsTimeline {
    pub fn frame_by_number(&self, frame_number: usize) -> Option<&ReplayStatsFrame> {
        self.frames
            .iter()
            .find(|frame| frame.frame_number == frame_number)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayStatsTimelineEvents {
    pub timeline: Vec<TimelineEvent>,
    pub backboard: Vec<BackboardBounceEvent>,
    pub ceiling_shot: Vec<CeilingShotEvent>,
    pub double_tap: Vec<DoubleTapEvent>,
    pub fifty_fifty: Vec<FiftyFiftyEvent>,
    pub rush: Vec<RushEvent>,
    pub speed_flip: Vec<SpeedFlipEvent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayStatsFrame {
    pub frame_number: usize,
    pub time: f32,
    pub dt: f32,
    pub seconds_remaining: Option<i32>,
    pub game_state: Option<i32>,
    pub is_live_play: bool,
    pub team_zero: TeamStatsSnapshot,
    pub team_one: TeamStatsSnapshot,
    pub players: Vec<PlayerStatsSnapshot>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FiftyFiftyTeamStats {
    pub count: u32,
    pub wins: u32,
    pub losses: u32,
    pub neutral_outcomes: u32,
    pub kickoff_count: u32,
    pub kickoff_wins: u32,
    pub kickoff_losses: u32,
    pub kickoff_neutral_outcomes: u32,
    pub possession_after_count: u32,
    pub opponent_possession_after_count: u32,
    pub neutral_possession_after_count: u32,
    pub kickoff_possession_after_count: u32,
    pub kickoff_opponent_possession_after_count: u32,
    pub kickoff_neutral_possession_after_count: u32,
}

impl FiftyFiftyStats {
    pub fn for_team(&self, is_team_zero: bool) -> FiftyFiftyTeamStats {
        let (
            wins,
            losses,
            kickoff_wins,
            kickoff_losses,
            possession_after_count,
            opponent_possession_after_count,
            kickoff_possession_after_count,
            kickoff_opponent_possession_after_count,
        ) = if is_team_zero {
            (
                self.team_zero_wins,
                self.team_one_wins,
                self.kickoff_team_zero_wins,
                self.kickoff_team_one_wins,
                self.team_zero_possession_after_count,
                self.team_one_possession_after_count,
                self.kickoff_team_zero_possession_after_count,
                self.kickoff_team_one_possession_after_count,
            )
        } else {
            (
                self.team_one_wins,
                self.team_zero_wins,
                self.kickoff_team_one_wins,
                self.kickoff_team_zero_wins,
                self.team_one_possession_after_count,
                self.team_zero_possession_after_count,
                self.kickoff_team_one_possession_after_count,
                self.kickoff_team_zero_possession_after_count,
            )
        };

        FiftyFiftyTeamStats {
            count: self.count,
            wins,
            losses,
            neutral_outcomes: self.neutral_outcomes,
            kickoff_count: self.kickoff_count,
            kickoff_wins,
            kickoff_losses,
            kickoff_neutral_outcomes: self.kickoff_neutral_outcomes,
            possession_after_count,
            opponent_possession_after_count,
            neutral_possession_after_count: self.neutral_possession_after_count,
            kickoff_possession_after_count,
            kickoff_opponent_possession_after_count,
            kickoff_neutral_possession_after_count: self.kickoff_neutral_possession_after_count,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PossessionTeamStats {
    pub tracked_time: f32,
    pub possession_time: f32,
    pub opponent_possession_time: f32,
    pub neutral_time: f32,
    #[serde(default, skip_serializing_if = "LabeledFloatSums::is_empty")]
    pub labeled_time: LabeledFloatSums,
}

fn team_relative_possession_label(label: &StatLabel, is_team_zero: bool) -> StatLabel {
    match (label.key, label.value) {
        ("possession_state", "team_zero") => StatLabel::new(
            "possession_state",
            if is_team_zero { "own" } else { "opponent" },
        ),
        ("possession_state", "team_one") => StatLabel::new(
            "possession_state",
            if is_team_zero { "opponent" } else { "own" },
        ),
        ("field_third", "team_zero_third") => StatLabel::new(
            "field_third",
            if is_team_zero {
                "defensive_third"
            } else {
                "offensive_third"
            },
        ),
        ("field_third", "team_one_third") => StatLabel::new(
            "field_third",
            if is_team_zero {
                "offensive_third"
            } else {
                "defensive_third"
            },
        ),
        _ => label.clone(),
    }
}

impl PossessionStats {
    pub fn for_team(&self, is_team_zero: bool) -> PossessionTeamStats {
        let (possession_time, opponent_possession_time) = if is_team_zero {
            (self.team_zero_time, self.team_one_time)
        } else {
            (self.team_one_time, self.team_zero_time)
        };

        let mut labeled_time = LabeledFloatSums::default();
        for entry in &self.labeled_time.entries {
            labeled_time.add(
                entry
                    .labels
                    .iter()
                    .map(|label| team_relative_possession_label(label, is_team_zero)),
                entry.value,
            );
        }

        PossessionTeamStats {
            tracked_time: self.tracked_time,
            possession_time,
            opponent_possession_time,
            neutral_time: self.neutral_time,
            labeled_time,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PressureTeamStats {
    pub tracked_time: f32,
    pub defensive_half_time: f32,
    pub offensive_half_time: f32,
    pub neutral_time: f32,
    #[serde(default, skip_serializing_if = "LabeledFloatSums::is_empty")]
    pub labeled_time: LabeledFloatSums,
}

fn team_relative_pressure_label(label: &StatLabel, is_team_zero: bool) -> StatLabel {
    match (label.key, label.value) {
        ("field_half", "team_zero_side") => StatLabel::new(
            "field_half",
            if is_team_zero {
                "defensive_half"
            } else {
                "offensive_half"
            },
        ),
        ("field_half", "team_one_side") => StatLabel::new(
            "field_half",
            if is_team_zero {
                "offensive_half"
            } else {
                "defensive_half"
            },
        ),
        _ => label.clone(),
    }
}

impl PressureStats {
    pub fn for_team(&self, is_team_zero: bool) -> PressureTeamStats {
        let (defensive_half_time, offensive_half_time) = if is_team_zero {
            (self.team_zero_side_time, self.team_one_side_time)
        } else {
            (self.team_one_side_time, self.team_zero_side_time)
        };

        let mut labeled_time = LabeledFloatSums::default();
        for entry in &self.labeled_time.entries {
            labeled_time.add(
                entry
                    .labels
                    .iter()
                    .map(|label| team_relative_pressure_label(label, is_team_zero)),
                entry.value,
            );
        }

        PressureTeamStats {
            tracked_time: self.tracked_time,
            defensive_half_time,
            offensive_half_time,
            neutral_time: self.neutral_time,
            labeled_time,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct RushTeamStats {
    pub count: u32,
    pub two_v_one_count: u32,
    pub two_v_two_count: u32,
    pub two_v_three_count: u32,
    pub three_v_one_count: u32,
    pub three_v_two_count: u32,
    pub three_v_three_count: u32,
}

impl RushStats {
    pub fn for_team(&self, is_team_zero: bool) -> RushTeamStats {
        if is_team_zero {
            RushTeamStats {
                count: self.team_zero_count,
                two_v_one_count: self.team_zero_two_v_one_count,
                two_v_two_count: self.team_zero_two_v_two_count,
                two_v_three_count: self.team_zero_two_v_three_count,
                three_v_one_count: self.team_zero_three_v_one_count,
                three_v_two_count: self.team_zero_three_v_two_count,
                three_v_three_count: self.team_zero_three_v_three_count,
            }
        } else {
            RushTeamStats {
                count: self.team_one_count,
                two_v_one_count: self.team_one_two_v_one_count,
                two_v_two_count: self.team_one_two_v_two_count,
                two_v_three_count: self.team_one_two_v_three_count,
                three_v_one_count: self.team_one_three_v_one_count,
                three_v_two_count: self.team_one_three_v_two_count,
                three_v_three_count: self.team_one_three_v_three_count,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct TeamStatsSnapshot {
    pub fifty_fifty: FiftyFiftyTeamStats,
    pub possession: PossessionTeamStats,
    pub pressure: PressureTeamStats,
    pub rush: RushTeamStats,
    pub core: CoreTeamStats,
    pub backboard: BackboardTeamStats,
    pub double_tap: DoubleTapTeamStats,
    pub ball_carry: BallCarryStats,
    pub boost: BoostStats,
    pub movement: MovementStats,
    pub powerslide: PowerslideStats,
    pub demo: DemoTeamStats,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PlayerStatsSnapshot {
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
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

fn build_timeline_graph() -> AnalysisGraph {
    let mut graph = AnalysisGraph::new().with_input_state_type::<FrameInput>();
    graph.push_boxed_node(Box::new(StatsTimelineFrameNode::new()));
    graph.push_boxed_node(Box::new(StatsTimelineEventsNode::new()));
    graph
}

pub struct StatsTimelineCollector {
    graph: AnalysisGraph,
    replay_meta: Option<ReplayMeta>,
    frames: Vec<ReplayStatsFrame>,
    last_sample_time: Option<f32>,
    frame_persistence: StatsFramePersistenceController,
}

impl Default for StatsTimelineCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsTimelineCollector {
    pub fn new() -> Self {
        let graph = build_timeline_graph();
        Self {
            graph,
            replay_meta: None,
            frames: Vec::new(),
            last_sample_time: None,
            frame_persistence: StatsFramePersistenceController::new(StatsFrameResolution::default()),
        }
    }

    fn timeline_config(&self) -> StatsTimelineConfig {
        StatsTimelineConfig {
            most_back_forward_threshold_y: PositioningCalculatorConfig::default()
                .most_back_forward_threshold_y,
            pressure_neutral_zone_half_width_y: PressureCalculatorConfig::default()
                .neutral_zone_half_width_y,
            rush_max_start_y: RushCalculatorConfig::default().max_start_y,
            rush_attack_support_distance_y: RushCalculatorConfig::default()
                .attack_support_distance_y,
            rush_defender_distance_y: RushCalculatorConfig::default().defender_distance_y,
            rush_min_possession_retained_seconds: RushCalculatorConfig::default()
                .min_possession_retained_seconds,
        }
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
