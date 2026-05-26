use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct LivePlayNode {
    tracker: LivePlayTracker,
    state: LivePlayState,
}

impl LivePlayNode {
    pub fn new() -> Self {
        Self {
            tracker: LivePlayTracker::default(),
            state: LivePlayState::default(),
        }
    }
}

impl_analysis_node! {
    node = LivePlayNode,
    state = LivePlayState,
    name = "live_play",
    dependencies = [
        AnalysisDependency::required::<FrameInput>(),
        gameplay_state_dependency(),
        frame_events_state_dependency(),
    ],
    inputs = {
        frame_input: FrameInput,
        gameplay: GameplayState,
        events: FrameEventsState,
    },
    evaluate = |node| {
        node.state = frame_input
            .live_play_state()
            .unwrap_or_else(|| node.tracker.state_parts(gameplay, events));
        Ok(())
    },
    state_ref = |node| &node.state,
}
