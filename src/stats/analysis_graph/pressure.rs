use super::graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct PressureNode {
    calculator: PressureCalculator,
}

impl PressureNode {
    pub fn new() -> Self {
        Self::with_config(PressureCalculatorConfig::default())
    }

    pub fn with_config(config: PressureCalculatorConfig) -> Self {
        Self {
            calculator: PressureCalculator::with_config(config),
        }
    }
}

impl_analysis_node! {
    node = PressureNode,
    state = PressureCalculator,
    name = "pressure",
    dependencies = [
        frame_info_dependency() => FrameInfo,
        ball_frame_state_dependency() => BallFrameState,
        live_play_dependency() => LivePlayState,
    ],
    call = calculator.update,
}
