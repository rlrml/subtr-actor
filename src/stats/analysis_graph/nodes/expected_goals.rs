use super::*;
use crate::stats::calculators::*;
use crate::*;

/// Evaluates the continuous threat value (expected-goals state value) for both
/// teams each live-play frame and derives touch threat deltas and threat
/// episodes.
pub struct ExpectedGoalsNode {
    calculator: ExpectedGoalsCalculator,
}

impl ExpectedGoalsNode {
    pub fn new() -> Self {
        Self::with_config(ExpectedGoalsCalculatorConfig::default())
    }

    pub fn with_config(config: ExpectedGoalsCalculatorConfig) -> Self {
        Self {
            calculator: ExpectedGoalsCalculator::with_config(config),
        }
    }
}

impl_analysis_node! {
    node = ExpectedGoalsNode,
    state = ExpectedGoalsCalculator,
    name = "expected_goals",
    emitted_events = crate::stats::calculators::EXPECTED_GOALS_EMITTED_EVENTS,
    dependencies = [
        frame_info_dependency() => FrameInfo,
        gameplay_state_dependency() => GameplayState,
        ball_frame_state_dependency() => BallFrameState,
        player_frame_state_dependency() => PlayerFrameState,
        frame_events_state_dependency() => FrameEventsState,
        touch_state_dependency() => TouchState,
        live_play_dependency() => LivePlayState,
    ],
    call = calculator.update_parts,
    finish = calculator.finish_calculation,
}
