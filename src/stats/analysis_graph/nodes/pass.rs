use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

/// Detects passes from touches, backboard-bounce, and 50/50 state during live play.
pub struct PassNode {
    calculator: PassCalculator,
}

impl PassNode {
    pub fn new() -> Self {
        Self {
            calculator: PassCalculator::new(),
        }
    }
}

impl Default for PassNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for PassNode {
    type State = PassCalculator;

    fn name(&self) -> &'static str {
        "pass"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::PASS_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            touch_state_dependency(),
            backboard_bounce_state_dependency(),
            fifty_fifty_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<TouchState>()?,
            ctx.get::<BackboardBounceState>()?,
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
fn projected_timeline_events(calculator: &PassCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    for event in calculator.events() {
        assembler.push(
            "pass",
            event.start_frame,
            EventLifecycle::Finalized,
            span(event.start_frame, event.frame, event.start_time, event.time),
            EventPayload::Pass(event.clone()),
            Some(event.passer.clone()),
            Some(event.receiver.clone()),
            Some(event.is_team_0),
            event.passer_position,
            None,
            None,
        );
    }
    assembler.into_events()
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(PassNode::new())
}
