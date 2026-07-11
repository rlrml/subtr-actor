use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

/// Detects wall aerials from ball/player positions and touches during live play.
pub struct WallAerialNode {
    calculator: WallAerialCalculator,
}

impl WallAerialNode {
    pub fn new() -> Self {
        Self {
            calculator: WallAerialCalculator::new(),
        }
    }
}

impl Default for WallAerialNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for WallAerialNode {
    type State = WallAerialCalculator;

    fn name(&self) -> &'static str {
        "wall_aerial"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::WALL_AERIAL_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            touch_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let frame = ctx.get::<FrameInfo>()?;
        let ball = ctx.get::<BallFrameState>()?;
        let players = ctx.get::<PlayerFrameState>()?;
        let touch_state = ctx.get::<TouchState>()?;
        let live_play_state = ctx.get::<LivePlayState>()?;
        self.calculator
            .update(frame, ball, players, touch_state, live_play_state)
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
fn projected_timeline_events(calculator: &WallAerialCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    for event in calculator.events() {
        assembler.push(
            "wall_aerial",
            event.wall_contact_frame,
            EventLifecycle::Finalized,
            span(
                event.wall_contact_frame,
                event.frame,
                event.wall_contact_time,
                event.time,
            ),
            EventPayload::WallAerial(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            Some(event.player_position),
            Some(event.ball_position),
            Some(event.confidence),
        );
    }
    assembler.into_events()
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(WallAerialNode::new())
}
