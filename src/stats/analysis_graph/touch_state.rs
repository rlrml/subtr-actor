use super::graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct TouchStateNode {
    calculator: TouchStateCalculator,
    state: TouchState,
}

impl TouchStateNode {
    pub fn new() -> Self {
        Self {
            calculator: TouchStateCalculator::new(),
            state: TouchState::default(),
        }
    }
}

impl_analysis_node! {
    node = TouchStateNode,
    state = TouchState,
    name = "touch_state",
    dependencies = [
        frame_info_dependency() => FrameInfo,
        ball_frame_state_dependency() => BallFrameState,
        player_frame_state_dependency() => PlayerFrameState,
        frame_events_state_dependency() => FrameEventsState,
        live_play_dependency() => LivePlayState,
    ],
    update_state = calculator.update,
}
