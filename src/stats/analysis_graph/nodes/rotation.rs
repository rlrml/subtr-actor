use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, moment, span, span_lifecycle};
use crate::*;

/// Tracks rotational roles (first/second/third man) from positions and events during live play.
pub struct RotationNode {
    calculator: RotationCalculator,
}

impl RotationNode {
    pub fn new() -> Self {
        Self::with_config(RotationCalculatorConfig::default())
    }

    pub fn with_config(config: RotationCalculatorConfig) -> Self {
        Self {
            calculator: RotationCalculator::with_config(config),
        }
    }
}

impl Default for RotationNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for RotationNode {
    type State = RotationCalculator;

    fn name(&self) -> &'static str {
        "rotation"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::ROTATION_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            gameplay_state_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            frame_events_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<GameplayState>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<FrameEventsState>()?,
            ctx.get::<LivePlayState>()?,
        )
    }

    fn finish(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.flush_pending_events();
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
fn projected_timeline_events(calculator: &RotationCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    // Rotation role spans share the positioning facets' open-span projection.
    for (event, closed) in calculator.role_events_by_player() {
        assembler.push(
            "rotation_role",
            event.frame,
            span_lifecycle(closed),
            span(event.frame, event.end_frame, event.time, event.end_time),
            EventPayload::RotationRole(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        );
    }

    // Debounced change moments: committed complete, never revised.
    for event in calculator.first_man_change_events() {
        assembler.push(
            "first_man_change",
            event.frame,
            EventLifecycle::Finalized,
            moment(event.frame, event.time),
            EventPayload::FirstManChange(event.clone()),
            Some(event.next_first_man.clone()),
            Some(event.previous_first_man.clone()),
            Some(event.is_team_0),
            None,
            None,
            None,
        );
    }
    assembler.into_events()
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(RotationNode::new())
}
