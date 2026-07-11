use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

/// Detects double taps from touches plus backboard-bounce state during live play.
pub struct DoubleTapNode {
    calculator: DoubleTapCalculator,
}

impl DoubleTapNode {
    pub fn new() -> Self {
        Self {
            calculator: DoubleTapCalculator::new(),
        }
    }
}

impl Default for DoubleTapNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for DoubleTapNode {
    type State = DoubleTapCalculator;

    fn name(&self) -> &'static str {
        "double_tap"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::DOUBLE_TAP_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            touch_state_dependency(),
            backboard_bounce_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<TouchState>()?,
            ctx.get::<BackboardBounceState>()?,
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
fn projected_timeline_events(calculator: &DoubleTapCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    for event in calculator.events() {
        assembler.push(
            "double_tap",
            event.backboard_frame,
            EventLifecycle::Finalized,
            span(
                event.backboard_frame,
                event.frame,
                event.backboard_time,
                event.time,
            ),
            EventPayload::DoubleTap(event.clone()),
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

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(DoubleTapNode::new())
}
