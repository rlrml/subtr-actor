use super::graph::AnalysisGraph;
use crate::stats::calculators::{FrameInput, ReplayFrameInputBuilder};
use crate::*;

pub struct AnalysisNodeCollector {
    graph: AnalysisGraph,
    frame_input_builder: ReplayFrameInputBuilder,
    last_sample_time: Option<f32>,
    last_replay_meta_player_count: Option<usize>,
}

impl AnalysisNodeCollector {
    pub fn new(mut graph: AnalysisGraph) -> Self {
        graph.register_input_state::<FrameInput>();
        Self {
            graph,
            frame_input_builder: ReplayFrameInputBuilder::default(),
            last_sample_time: None,
            last_replay_meta_player_count: None,
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
        processor: &dyn ProcessorView,
        _frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        let player_count = processor.player_count();
        if self.last_replay_meta_player_count != Some(player_count) {
            self.graph.on_replay_meta(&processor.get_replay_meta()?)?;
            self.last_replay_meta_player_count = Some(player_count);
        }

        let dt = self
            .last_sample_time
            .map(|last_time| (current_time - last_time).max(0.0))
            .unwrap_or(0.0);
        let frame_input =
            self.frame_input_builder
                .aggregate(processor, frame_number, current_time, dt);
        self.graph.evaluate_with_state(&frame_input)?;
        self.last_sample_time = Some(current_time);

        Ok(TimeAdvance::NextFrame)
    }

    fn finish_replay(&mut self, _processor: &dyn ProcessorView) -> SubtrActorResult<()> {
        self.graph.finish()
    }
}
