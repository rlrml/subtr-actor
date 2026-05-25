use super::*;
use crate::stats::calculators::*;
use crate::*;

pub struct ContinuousBallControlNode {
    tracker: ContinuousBallControlTracker<BallCarryKind>,
    state: ContinuousBallControlState,
}

impl ContinuousBallControlNode {
    pub fn new() -> Self {
        Self {
            tracker: ContinuousBallControlTracker::default(),
            state: ContinuousBallControlState::default(),
        }
    }
}

impl Default for ContinuousBallControlNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for ContinuousBallControlNode {
    type State = ContinuousBallControlState;

    fn name(&self) -> &'static str {
        "continuous_ball_control"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
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
        let touch_state = ctx.get::<TouchState>()?;
        let players = ctx.get::<PlayerFrameState>()?;
        let candidate = if frame.dt > 0.0 {
            BallCarryCalculator::control_candidate(
                ctx.get::<BallFrameState>()?,
                players,
                ctx.get::<LivePlayState>()?.is_live_play,
                touch_state,
            )
        } else {
            None
        };
        let player_statuses = BallCarryCalculator::control_player_statuses(players);
        let touches = BallCarryCalculator::control_touches(touch_state, players);
        self.state.completed_sequences.extend(self.tracker.update(
            frame,
            candidate,
            &player_statuses,
            &touches,
            BallCarryCalculator::min_duration_for_kind,
            BallCarryCalculator::kind_requires_airborne,
        ));
        Ok(())
    }

    fn finish(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        if let Some(sequence) = self
            .tracker
            .finish(BallCarryCalculator::min_duration_for_kind)
        {
            self.state.completed_sequences.push(sequence);
        }
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(ContinuousBallControlNode::new())
}
