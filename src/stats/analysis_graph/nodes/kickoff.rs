use super::*;
use crate::stats::calculators::*;
use crate::stats::timeline::projection::{EventAssembler, span};
use crate::*;

/// Detects and classifies kickoffs from gameplay/ball/player state, touches, speed-flips, and boost pickups.
pub struct KickoffNode {
    calculator: KickoffCalculator,
}

impl KickoffNode {
    pub fn new() -> Self {
        Self {
            calculator: KickoffCalculator::new(),
        }
    }
}

impl Default for KickoffNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for KickoffNode {
    type State = KickoffCalculator;

    fn name(&self) -> &'static str {
        "kickoff"
    }

    fn emitted_events(&self) -> &'static [crate::stats::calculators::EmittedEvent] {
        crate::stats::calculators::KICKOFF_EMITTED_EVENTS
    }

    fn dependencies(&self) -> NodeDependencies {
        vec![
            frame_info_dependency(),
            gameplay_state_dependency(),
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            touch_state_dependency(),
            frame_events_state_dependency(),
            speed_flip_dependency(),
            boost_dependency(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let speed_flip = ctx.get::<SpeedFlipCalculator>()?;
        let boost = ctx.get::<BoostCalculator>()?;
        self.calculator
            .update_with_speed_flips(KickoffUpdateContext {
                frame: ctx.get::<FrameInfo>()?,
                gameplay: ctx.get::<GameplayState>()?,
                ball: ctx.get::<BallFrameState>()?,
                players: ctx.get::<PlayerFrameState>()?,
                touch_state: ctx.get::<TouchState>()?,
                events: ctx.get::<FrameEventsState>()?,
                speed_flip_events: speed_flip.events(),
                boost_pickups: boost.new_pickup_events(),
            })
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
fn projected_timeline_events(calculator: &KickoffCalculator) -> Vec<Event> {
    let mut assembler = EventAssembler::new();
    // Kickoffs commit only when their in-flight ledger finalizes (goal
    // attribution window closes or the next kickoff begins).
    for event in calculator.events() {
        assembler.push(
            "kickoff",
            event.start_frame,
            EventLifecycle::Finalized,
            span(
                event.start_frame,
                event.end_frame,
                event.start_time,
                event.end_time,
            ),
            EventPayload::Kickoff(Box::new(event.clone())),
            event.first_touch_player.clone(),
            None,
            event.first_touch_team_is_team_0,
            None,
            None,
            None,
        );
    }
    assembler.into_events()
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(KickoffNode::new())
}
