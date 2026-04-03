use crate::collector::frame_resolution::{
    FinalStatsFrameAction, StatsFramePersistenceController, StatsFrameResolution,
};
use crate::stats::analysis_graph::{
    AnalysisGraph,
    StatsTimelineEventsNode, StatsTimelineEventsState, StatsTimelineFrameNode,
    StatsTimelineFrameState,
};
use crate::*;

pub fn build_timeline_graph() -> AnalysisGraph {
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
