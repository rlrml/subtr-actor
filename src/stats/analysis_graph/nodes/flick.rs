use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct FlickNode {
    calculator: FlickCalculator,
}

impl FlickNode {
    pub fn new() -> Self {
        Self {
            calculator: FlickCalculator::new(),
        }
    }
}

impl_analysis_node! {
    node = FlickNode,
    state = FlickCalculator,
    name = "flick",
    dependencies = [
        frame_info_dependency() => FrameInfo,
        ball_frame_state_dependency() => BallFrameState,
        player_frame_state_dependency() => PlayerFrameState,
        touch_state_dependency() => TouchState,
        live_play_dependency() => LivePlayState,
    ],
    call = calculator.update,
}
