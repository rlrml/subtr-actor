use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

/// Tracks territorial pressure sessions from ball/possession state during live play.
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
    project_events = |node| { projected_timeline_events(&node.calculator) },
    call = calculator.update,
    finish = calculator.finish,
}

/// Projects this node's committed events for the stats timeline (see
/// `AnalysisNode::project_events`). The inline comments state the stream's
/// interim lifecycle rule.
fn projected_timeline_events(calculator: &TerritorialPressureCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    // Pressure sessions commit at session end; the active session is not part
    // of `events()`.
    for event in calculator.events() {
        assembler.push(
            "territorial_pressure",
            event.start_frame,
            EventLifecycle::Finalized,
            span(
                event.start_frame,
                event.end_frame,
                event.start_time,
                event.end_time,
            ),
            EventPayload::TerritorialPressure(event.clone()),
            None,
            None,
            Some(event.team_is_team_0),
            None,
            None,
            None,
        );
    }
    assembler.into_events()
}
