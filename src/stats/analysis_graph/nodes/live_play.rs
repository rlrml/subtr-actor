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
        gameplay_state_dependency() => GameplayState,
        frame_events_state_dependency() => FrameEventsState,
    ],
    update_state = tracker.state_parts,
}
