use super::graph::*;
use super::nodes::*;
use crate::stats::calculators::*;
use crate::*;

pub struct RushNode {
    calculator: RushCalculator,
}

impl RushNode {
    pub fn new() -> Self {
        Self::with_config(RushCalculatorConfig::default())
    }

    pub fn with_config(config: RushCalculatorConfig) -> Self {
        Self {
            calculator: RushCalculator::with_config(config),
        }
    }
}

impl_analysis_node! {
    node = RushNode,
    state = RushCalculator,
    name = "rush",
    dependencies = [
        frame_info_dependency() => FrameInfo,
        gameplay_state_dependency() => GameplayState,
        ball_frame_state_dependency() => BallFrameState,
        player_frame_state_dependency() => PlayerFrameState,
        frame_events_state_dependency() => FrameEventsState,
        possession_state_dependency() => PossessionState,
        live_play_dependency() => LivePlayState,
    ],
    call = calculator.update_parts,
    finish = calculator.finish_calculation,
}
