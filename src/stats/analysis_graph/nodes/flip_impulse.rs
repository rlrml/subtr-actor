use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

/// Detects dodges/flip impulses from player frame state (graph node name "dodge").
pub struct FlipImpulseNode {
    calculator: FlipImpulseCalculator,
}

impl FlipImpulseNode {
    pub fn new() -> Self {
        Self {
            calculator: FlipImpulseCalculator::new(),
        }
    }
}

impl Default for FlipImpulseNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for FlipImpulseNode {
    type State = FlipImpulseCalculator;

    fn name(&self) -> &'static str {
        "dodge"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::DODGE_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            player_frame_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update_parts(
            ctx.get::<FrameInfo>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<LivePlayState>()?,
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
fn projected_timeline_events(calculator: &FlipImpulseCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    // Dodges commit once resolved (the resolved fields are part of the
    // committed payload).
    for event in calculator.events() {
        assembler.push(
            "dodge",
            event.frame,
            EventLifecycle::Finalized,
            span(
                event.frame,
                event.resolved_frame,
                event.time,
                event.resolved_time,
            ),
            EventPayload::Dodge(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event
                .dodge_impulse
                .as_ref()
                .map(|dodge_impulse| dodge_impulse.end_position),
            None,
            event
                .dodge_impulse
                .as_ref()
                .map(|dodge_impulse| dodge_impulse.confidence),
        );
    }
    assembler.into_events()
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(FlipImpulseNode::new())
}
