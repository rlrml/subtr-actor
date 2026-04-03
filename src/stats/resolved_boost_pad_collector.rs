use crate::stats::analysis_graph::{self, AnalysisNodeCollector};
use crate::*;

pub struct ResolvedBoostPadCollector {
    collector: AnalysisNodeCollector,
}

impl Default for ResolvedBoostPadCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl ResolvedBoostPadCollector {
    pub fn new() -> Self {
        let graph = analysis_graph::graph_with_builtin_analysis_nodes(["boost"])
            .expect("builtin boost analysis graph should be valid");
        Self {
            collector: AnalysisNodeCollector::new(graph),
        }
    }

    pub fn resolved_boost_pads(&self) -> Vec<ResolvedBoostPad> {
        self.collector
            .graph()
            .state::<BoostCalculator>()
            .map(BoostCalculator::resolved_boost_pads)
            .unwrap_or_default()
    }

    pub fn into_resolved_boost_pads(self) -> Vec<ResolvedBoostPad> {
        let graph = self.collector.into_graph();
        graph
            .state::<BoostCalculator>()
            .map(BoostCalculator::resolved_boost_pads)
            .unwrap_or_default()
    }
}

impl Collector for ResolvedBoostPadCollector {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        frame: &boxcars::Frame,
        frame_number: usize,
        current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        self.collector
            .process_frame(processor, frame, frame_number, current_time)
    }

    fn finish_replay(&mut self, processor: &ReplayProcessor) -> SubtrActorResult<()> {
        self.collector.finish_replay(processor)
    }
}
