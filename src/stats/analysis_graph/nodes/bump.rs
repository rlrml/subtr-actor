use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, moment};
use crate::*;

/// Detects player-on-player bumps from player frame/events and 50/50 state.
pub struct BumpNode {
    calculator: BumpCalculator,
}

impl BumpNode {
    pub fn new() -> Self {
        Self {
            calculator: BumpCalculator::new(),
        }
    }
}

impl Default for BumpNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for BumpNode {
    type State = BumpCalculator;

    fn name(&self) -> &'static str {
        "bump"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::BUMP_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            player_frame_state_dependency(),
            frame_events_state_dependency(),
            fifty_fifty_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update_with_fifty_fifty_state(
            ctx.get::<FrameInfo>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<FrameEventsState>()?,
            ctx.get::<FiftyFiftyState>()?,
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
fn projected_timeline_events(calculator: &BumpCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    for event in calculator.events() {
        assembler.push(
            "bump",
            event.frame,
            EventLifecycle::Finalized,
            moment(event.frame, event.time),
            EventPayload::Bump(event.clone()),
            Some(event.initiator.clone()),
            Some(event.victim.clone()),
            Some(event.initiator_is_team_0),
            Some(event.initiator_position),
            None,
            Some(event.confidence),
        );
    }
    assembler.into_events()
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(BumpNode::new())
}
