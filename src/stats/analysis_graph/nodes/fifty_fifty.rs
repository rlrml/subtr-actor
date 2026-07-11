use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

/// Derives 50/50 stats and events from the shared fifty-fifty state node.
pub struct FiftyFiftyNode {
    calculator: FiftyFiftyCalculator,
}

impl FiftyFiftyNode {
    pub fn new() -> Self {
        Self {
            calculator: FiftyFiftyCalculator::new(),
        }
    }
}

impl Default for FiftyFiftyNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for FiftyFiftyNode {
    type State = FiftyFiftyCalculator;

    fn name(&self) -> &'static str {
        "fifty_fifty"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::FIFTY_FIFTY_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![fifty_fifty_state_dependency()]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let fifty_fifty_state = ctx.get::<FiftyFiftyState>()?;
        self.calculator.update(fifty_fifty_state)
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
fn projected_timeline_events(calculator: &FiftyFiftyCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    // Contests commit at resolution.
    for event in calculator.events() {
        assembler.push(
            "fifty_fifty",
            event.start_frame,
            EventLifecycle::Finalized,
            span(
                event.start_frame,
                event.resolve_frame,
                event.start_time,
                event.resolve_time,
            ),
            EventPayload::FiftyFifty(event.clone()),
            event
                .team_zero_player
                .clone()
                .or_else(|| event.team_one_player.clone()),
            None,
            event.winning_team_is_team_0,
            None,
            Some(event.midpoint),
            None,
        );
    }
    assembler.into_events()
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(FiftyFiftyNode::new())
}
