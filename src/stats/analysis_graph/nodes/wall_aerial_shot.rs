use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

/// Detects wall-aerial shots from player frame state and frame events during live play.
pub struct WallAerialShotNode {
    calculator: WallAerialShotCalculator,
}

impl WallAerialShotNode {
    pub fn new() -> Self {
        Self {
            calculator: WallAerialShotCalculator::new(),
        }
    }
}

impl Default for WallAerialShotNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for WallAerialShotNode {
    type State = WallAerialShotCalculator;

    fn name(&self) -> &'static str {
        "wall_aerial_shot"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::WALL_AERIAL_SHOT_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            player_frame_state_dependency(),
            frame_events_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let frame = ctx.get::<FrameInfo>()?;
        let players = ctx.get::<PlayerFrameState>()?;
        let frame_events = ctx.get::<FrameEventsState>()?;
        let live_play_state = ctx.get::<LivePlayState>()?;
        self.calculator
            .update(frame, players, frame_events, live_play_state)
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
fn projected_timeline_events(calculator: &WallAerialShotCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    for event in calculator.events() {
        assembler.push(
            "wall_aerial_shot",
            event.takeoff_frame,
            EventLifecycle::Finalized,
            span(
                event.takeoff_frame,
                event.frame,
                event.takeoff_time,
                event.time,
            ),
            EventPayload::WallAerialShot(event.clone()),
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
    Box::new(WallAerialShotNode::new())
}
