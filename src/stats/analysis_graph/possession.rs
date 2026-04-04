use super::graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct PossessionNode {
    calculator: PossessionCalculator,
}

impl PossessionNode {
    pub fn new() -> Self {
        Self {
            calculator: PossessionCalculator::new(),
        }
    }
}

impl_analysis_node! {
    node = PossessionNode,
    state = PossessionCalculator,
    name = "possession",
    dependencies = [
        frame_info_dependency() => FrameInfo,
        ball_frame_state_dependency() => BallFrameState,
        possession_state_dependency() => PossessionState,
        live_play_dependency() => LivePlayState,
    ],
    call = calculator.update,
}
