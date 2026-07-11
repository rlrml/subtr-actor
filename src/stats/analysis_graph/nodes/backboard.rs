use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, moment};
use crate::*;

/// Derives backboard-play stats from the upstream backboard-bounce state.
pub struct BackboardNode {
    calculator: BackboardCalculator,
}

impl BackboardNode {
    pub fn new() -> Self {
        Self {
            calculator: BackboardCalculator::new(),
        }
    }
}

impl_analysis_node! {
    node = BackboardNode,
    state = BackboardCalculator,
    name = "backboard",
    emitted_events = crate::stats::calculators::BACKBOARD_EMITTED_EVENTS,
    dependencies = [
        frame_info_dependency() => FrameInfo,
        backboard_bounce_state_dependency() => BackboardBounceState,
    ],
    project_events = |node| { projected_timeline_events(&node.calculator) },
    call = calculator.update,
}

/// Projects this node's committed events for the stats timeline (see
/// `AnalysisNode::project_events`). The inline comments state the stream's
/// interim lifecycle rule.
fn projected_timeline_events(calculator: &BackboardCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    // Detection moments below (backboard, dodge, mechanics, ...) are pushed
    // once, fully formed, when their detector resolves, and are never mutated
    // afterwards — Finalized as soon as they are visible.
    for event in calculator.events() {
        assembler.push(
            "backboard",
            event.frame,
            EventLifecycle::Finalized,
            moment(event.frame, event.time),
            EventPayload::Backboard(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        );
    }
    assembler.into_events()
}
