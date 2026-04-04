use super::graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct BallCarryNode {
    calculator: BallCarryCalculator,
}

impl BallCarryNode {
    pub fn new() -> Self {
        Self {
            calculator: BallCarryCalculator::new(),
        }
    }
}

impl_analysis_node! {
    node = BallCarryNode,
    state = BallCarryCalculator,
    name = "ball_carry",
    dependencies = [
        frame_info_dependency() => FrameInfo,
        ball_frame_state_dependency() => BallFrameState,
        player_frame_state_dependency() => PlayerFrameState,
        touch_state_dependency() => TouchState,
        live_play_dependency() => LivePlayState,
    ],
    call = calculator.update,
    finish = calculator.finish_calculation,
}
