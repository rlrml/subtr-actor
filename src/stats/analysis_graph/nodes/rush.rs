use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

/// Detects rushes/over-commits from ball/player/possession state during live play.
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
    emitted_events = crate::stats::calculators::RUSH_EMITTED_EVENTS,
    dependencies = [
        frame_info_dependency() => FrameInfo,
        gameplay_state_dependency() => GameplayState,
        ball_frame_state_dependency() => BallFrameState,
        player_frame_state_dependency() => PlayerFrameState,
        frame_events_state_dependency() => FrameEventsState,
        possession_state_dependency() => PossessionState,
        live_play_dependency() => LivePlayState,
    ],
    project_events = |node| { projected_timeline_events(&node.calculator) },
    call = calculator.update_parts,
    finish = calculator.finish_calculation,
}

/// Projects this node's committed events for the stats timeline (see
/// `AnalysisNode::project_events`). The inline comments state the stream's
/// interim lifecycle rule.
fn projected_timeline_events(calculator: &RushCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    // Rushes commit (with a frozen end) once retained possession crosses the
    // counting threshold, and are not revised afterwards.
    for event in calculator.events() {
        assembler.push(
            "rush",
            event.start_frame,
            EventLifecycle::Finalized,
            span(
                event.start_frame,
                event.end_frame,
                event.start_time,
                event.end_time,
            ),
            EventPayload::Rush(event.clone()),
            None,
            None,
            Some(event.is_team_0),
            None,
            None,
            None,
        );
    }
    assembler.into_events()
}
