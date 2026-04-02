use super::analysis_graph::AnalysisGraph;
use crate::stats::calculators::FrameInput;
use crate::*;

#[allow(dead_code)]
pub struct AnalysisNodeCollector {
    graph: AnalysisGraph,
    last_sample_time: Option<f32>,
    replay_meta_initialized: bool,
    last_demolish_count: usize,
    last_boost_pad_event_count: usize,
    last_touch_event_count: usize,
    last_player_stat_event_count: usize,
    last_goal_event_count: usize,
}

#[allow(dead_code)]
impl AnalysisNodeCollector {
    pub fn new(mut graph: AnalysisGraph) -> Self {
        graph.register_input_state::<FrameInput>();
        Self {
            graph,
            last_sample_time: None,
            replay_meta_initialized: false,
            last_demolish_count: 0,
            last_boost_pad_event_count: 0,
            last_touch_event_count: 0,
            last_player_stat_event_count: 0,
            last_goal_event_count: 0,
        }
    }

    pub fn graph(&self) -> &AnalysisGraph {
        &self.graph
    }

    pub fn graph_mut(&mut self) -> &mut AnalysisGraph {
        &mut self.graph
    }

    pub fn into_graph(self) -> AnalysisGraph {
        self.graph
    }
}

impl Collector for AnalysisNodeCollector {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        _frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        if !self.replay_meta_initialized {
            self.graph.on_replay_meta(&processor.get_replay_meta()?)?;
            self.replay_meta_initialized = true;
        }

        let dt = self
            .last_sample_time
            .map(|last_time| (current_time - last_time).max(0.0))
            .unwrap_or(0.0);
        let frame_input = FrameInput::aggregate(
            processor,
            frame_number,
            current_time,
            dt,
            self.last_demolish_count,
            self.last_boost_pad_event_count,
            self.last_touch_event_count,
            self.last_player_stat_event_count,
            self.last_goal_event_count,
        );
        self.graph.evaluate_with_state(&frame_input)?;
        self.last_sample_time = Some(current_time);
        self.last_demolish_count = processor.demolishes.len();
        self.last_boost_pad_event_count = processor.boost_pad_events.len();
        self.last_touch_event_count = processor.touch_events.len();
        self.last_player_stat_event_count = processor.player_stat_events.len();
        self.last_goal_event_count = processor.goal_events.len();

        Ok(TimeAdvance::NextFrame)
    }

    fn finish_replay(&mut self, _processor: &ReplayProcessor) -> SubtrActorResult<()> {
        self.graph.finish()
    }
}
