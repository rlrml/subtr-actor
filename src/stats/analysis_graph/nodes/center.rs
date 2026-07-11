use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

/// Detects centering passes from ball state, touches, and frame events during live play.
pub struct CenterNode {
    calculator: CenterCalculator,
}

impl CenterNode {
    pub fn new() -> Self {
        Self {
            calculator: CenterCalculator::new(),
        }
    }
}

impl Default for CenterNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for CenterNode {
    type State = CenterCalculator;

    fn name(&self) -> &'static str {
        "center"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::CENTER_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            touch_state_dependency(),
            frame_events_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<TouchState>()?,
            ctx.get::<FrameEventsState>()?,
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
fn projected_timeline_events(calculator: &CenterCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    for event in calculator.events() {
        assembler.push(
            "center",
            event.start_frame,
            EventLifecycle::Finalized,
            span(event.start_frame, event.frame, event.start_time, event.time),
            EventPayload::Center(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            Some(event.end_ball_position),
            None,
        );
    }
    assembler.into_events()
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(CenterNode::new())
}
