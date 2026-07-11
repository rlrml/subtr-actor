use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, moment, span};
use crate::*;

/// Detects flip/dodge resets and their outcomes from player, ball, touch, and event state.
pub struct DodgeResetNode {
    calculator: DodgeResetCalculator,
}

impl DodgeResetNode {
    pub fn new() -> Self {
        Self {
            calculator: DodgeResetCalculator::new(),
        }
    }
}

impl Default for DodgeResetNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for DodgeResetNode {
    type State = DodgeResetCalculator;

    fn name(&self) -> &'static str {
        "dodge_reset"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::DODGE_RESET_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            frame_events_state_dependency(),
            touch_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<FrameEventsState>()?,
            ctx.get::<TouchState>()?,
            ctx.get::<LivePlayState>()?,
        )
    }

    fn finish(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.finish();
        Ok(())
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
fn projected_timeline_events(calculator: &DodgeResetCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    // A dodge reset is committed at the reset moment but its outcome (used /
    // wasted, latency) is patched in once the pending reset resolves; the
    // payload's `outcome` field is the calculator's own "still pending"
    // signal.
    for event in calculator.events() {
        let lifecycle = if event.outcome.is_some() {
            EventLifecycle::Finalized
        } else {
            EventLifecycle::Confirmed
        };
        assembler.push(
            "dodge_reset",
            event.frame,
            lifecycle,
            moment(event.frame, event.time),
            EventPayload::DodgeReset(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        );
    }

    for event in calculator.confirmed_flip_reset_events() {
        assembler.push(
            "flip_reset",
            event.reset_frame,
            EventLifecycle::Finalized,
            span(event.reset_frame, event.frame, event.reset_time, event.time),
            EventPayload::FlipReset(event.clone()),
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
    Box::new(DodgeResetNode::new())
}
