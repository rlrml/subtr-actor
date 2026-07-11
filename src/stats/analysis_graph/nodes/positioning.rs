use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span, span_lifecycle};
use crate::*;

/// Tracks per-player field positioning (thirds/halves/roles/proximity) from frame and possession state.
pub struct PositioningNode {
    calculator: PositioningCalculator,
}

impl PositioningNode {
    pub fn new() -> Self {
        Self::with_config(PositioningCalculatorConfig::default())
    }

    pub fn with_config(config: PositioningCalculatorConfig) -> Self {
        Self {
            calculator: PositioningCalculator::with_config(config),
        }
    }
}

impl Default for PositioningNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for PositioningNode {
    type State = PositioningCalculator;

    fn name(&self) -> &'static str {
        "positioning"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::POSITIONING_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            gameplay_state_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            frame_events_state_dependency(),
            possession_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let possession_state = ctx.get::<PossessionState>()?;
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<GameplayState>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<FrameEventsState>()?,
            ctx.get::<LivePlayState>()?,
            possession_state.active_player_before_sample.as_ref(),
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
fn projected_timeline_events(calculator: &PositioningCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    // The positioning facets project *open* spans too (their end advances
    // every frame until the state changes), so the open span is Confirmed and
    // a closed span is Finalized. The `_by_player` accessors provide the
    // cadence-invariant visiting order (see `EventAssembler`).
    for (event, closed) in calculator.activity_events_by_player() {
        assembler.push(
            "player_activity",
            event.frame,
            span_lifecycle(closed),
            span(event.frame, event.end_frame, event.time, event.end_time),
            EventPayload::PlayerActivity(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        );
    }

    for (event, closed) in calculator.field_third_events_by_player() {
        assembler.push(
            "field_third",
            event.frame,
            span_lifecycle(closed),
            span(event.frame, event.end_frame, event.time, event.end_time),
            EventPayload::FieldThird(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        );
    }

    for (event, closed) in calculator.field_half_events_by_player() {
        assembler.push(
            "field_half",
            event.frame,
            span_lifecycle(closed),
            span(event.frame, event.end_frame, event.time, event.end_time),
            EventPayload::FieldHalf(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        );
    }

    for (event, closed) in calculator.ball_depth_events_by_player() {
        assembler.push(
            "ball_depth",
            event.frame,
            span_lifecycle(closed),
            span(event.frame, event.end_frame, event.time, event.end_time),
            EventPayload::BallDepth(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        );
    }

    for (event, closed) in calculator.depth_role_events_by_player() {
        assembler.push(
            "depth_role",
            event.frame,
            span_lifecycle(closed),
            span(event.frame, event.end_frame, event.time, event.end_time),
            EventPayload::DepthRole(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        );
    }

    for (event, closed) in calculator.ball_proximity_events_by_player() {
        assembler.push(
            "ball_proximity",
            event.frame,
            span_lifecycle(closed),
            span(event.frame, event.end_frame, event.time, event.end_time),
            EventPayload::BallProximity(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        );
    }

    for (event, closed) in calculator.shadow_defense_events_by_player() {
        assembler.push(
            "shadow_defense",
            event.frame,
            span_lifecycle(closed),
            span(event.frame, event.end_frame, event.time, event.end_time),
            EventPayload::ShadowDefense(event.clone()),
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
    Box::new(PositioningNode::new())
}
