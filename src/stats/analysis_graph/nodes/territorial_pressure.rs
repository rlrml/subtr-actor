use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct TerritorialPressureNode {
    calculator: TerritorialPressureCalculator,
}

impl TerritorialPressureNode {
    pub fn new() -> Self {
        Self::with_config(TerritorialPressureCalculatorConfig::default())
    }

    pub fn with_config(config: TerritorialPressureCalculatorConfig) -> Self {
        Self {
            calculator: TerritorialPressureCalculator::with_config(config),
        }
    }
}

impl_analysis_node! {
    node = TerritorialPressureNode,
    state = TerritorialPressureCalculator,
    name = "territorial_pressure",
    emitted_events = crate::stats::calculators::TERRITORIAL_BALL_HALF_EMITTED_EVENTS,
    dependencies = [
        frame_info_dependency() => FrameInfo,
        ball_frame_state_dependency() => BallFrameState,
        possession_state_dependency() => PossessionState,
        live_play_dependency() => LivePlayState,
    ],
    call = calculator.update,
    finish = calculator.finish,
}
