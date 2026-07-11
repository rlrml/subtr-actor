use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

/// Detects one-timers from ball state and upstream pass detection during live play.
pub struct OneTimerNode {
    calculator: OneTimerCalculator,
}

impl OneTimerNode {
    pub fn new() -> Self {
        Self {
            calculator: OneTimerCalculator::new(),
        }
    }
}

impl Default for OneTimerNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for OneTimerNode {
    type State = OneTimerCalculator;

    fn name(&self) -> &'static str {
        "one_timer"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::ONE_TIMER_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            pass_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PassCalculator>()?,
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
fn projected_timeline_events(calculator: &OneTimerCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    for event in calculator.events() {
        assembler.push(
            "one_timer",
            event.pass_start_frame,
            EventLifecycle::Finalized,
            span(
                event.pass_start_frame,
                event.frame,
                event.pass_start_time,
                event.time,
            ),
            EventPayload::OneTimer(event.clone()),
            Some(event.player.clone()),
            Some(event.passer.clone()),
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        );
    }
    assembler.into_events()
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(OneTimerNode::new())
}
