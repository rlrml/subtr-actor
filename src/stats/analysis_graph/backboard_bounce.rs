use super::graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct BackboardBounceStateNode {
    calculator: BackboardBounceCalculator,
    state: BackboardBounceState,
}

impl BackboardBounceStateNode {
    pub fn new() -> Self {
        Self {
            calculator: BackboardBounceCalculator::new(),
            state: BackboardBounceState::default(),
        }
    }
}

impl_analysis_node! {
    node = BackboardBounceStateNode,
    state = BackboardBounceState,
    name = "backboard_bounce_state",
    dependencies = [
        frame_info_dependency() => FrameInfo,
        ball_frame_state_dependency() => BallFrameState,
        frame_events_state_dependency() => FrameEventsState,
        live_play_dependency() => LivePlayState,
    ],
    update_state = calculator.update,
}
