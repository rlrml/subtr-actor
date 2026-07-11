use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

/// Tracks per-player possession from ball/player/possession/touch state.
pub struct PlayerPossessionNode {
    calculator: PlayerPossessionCalculator,
}

impl PlayerPossessionNode {
    pub fn new() -> Self {
        Self {
            calculator: PlayerPossessionCalculator::new(),
        }
    }
}

impl Default for PlayerPossessionNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for PlayerPossessionNode {
    type State = PlayerPossessionCalculator;

    fn name(&self) -> &'static str {
        "player_possession"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::PLAYER_POSSESSION_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            possession_state_dependency(),
            touch_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.calculator.update(
            ctx.get::<FrameInfo>()?,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<PossessionState>()?,
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
fn projected_timeline_events(calculator: &PlayerPossessionCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    // Player-possession spans reach the list only via `finalize` (in-progress
    // spans live outside it), immutable afterwards.
    for event in calculator.events() {
        assembler.push(
            "player_possession",
            event.start_frame,
            EventLifecycle::Finalized,
            span(
                event.start_frame,
                event.end_frame,
                event.start_time,
                event.end_time,
            ),
            EventPayload::PlayerPossession(event.clone()),
            Some(event.player_id.clone()),
            None,
            Some(event.is_team_0),
            None,
            None,
            None,
        );
    }
    assembler.into_events()
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(PlayerPossessionNode::new())
}
