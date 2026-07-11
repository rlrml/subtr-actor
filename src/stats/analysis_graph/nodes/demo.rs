use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, moment};
use crate::*;

/// Detects demolitions from player frame state and frame events.
pub struct DemoNode {
    calculator: DemoCalculator,
}

impl DemoNode {
    pub fn new() -> Self {
        Self {
            calculator: DemoCalculator::new(),
        }
    }
}

impl Default for DemoNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for DemoNode {
    type State = DemoCalculator;

    fn name(&self) -> &'static str {
        "demo"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::DEMO_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            player_frame_state_dependency(),
            frame_events_state_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<FrameEventsState>()?,
        )
    }

    fn project_events(&self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<Vec<Event>> {
        Ok(projected_timeline_events(&self.calculator))
    }

    fn state(&self) -> &Self::State {
        &self.calculator
    }
}

/// Projects this node's committed events for the stats timeline (see
/// `AnalysisNode::project_events`). The inline comments state the stream's
/// interim lifecycle rule.
fn projected_timeline_events(calculator: &DemoCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    for event in calculator.events() {
        assembler.push(
            "demolition",
            event.frame,
            EventLifecycle::Finalized,
            moment(event.frame, event.time),
            EventPayload::Demolition(event.clone()),
            Some(event.attacker.clone()),
            Some(event.victim.clone()),
            event.attacker_is_team_0,
            event.attacker_position,
            None,
            None,
        );
    }
    assembler.into_events()
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(DemoNode::new())
}
