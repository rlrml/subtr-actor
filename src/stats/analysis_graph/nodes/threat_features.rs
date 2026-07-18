use super::*;
use crate::stats::calculators::*;
use crate::*;
use std::collections::HashSet;

/// Publishes the canonical two team-relative threat feature rows for the
/// current live frame. This is intentionally separate from model evaluation:
/// ndarray consumers can request the numeric inputs without running the
/// expected-goals state machine.
pub struct ThreatFeaturesNode {
    state: ThreatFeaturesState,
    is_doubles: bool,
    dodge_trackers: HashMap<PlayerId, DodgeAvailabilityTracker>,
}

#[derive(Debug, Clone, Default)]
struct DodgeAvailabilityTracker {
    available: bool,
    was_grounded: bool,
    takeoff_time: Option<f32>,
    unlimited_reset: bool,
    previous_double_jump_active: bool,
    previous_dodge_active: bool,
}

const STANDARD_DODGE_WINDOW_SECONDS: f32 = 1.25;

impl ThreatFeaturesNode {
    pub fn new() -> Self {
        Self {
            state: ThreatFeaturesState::default(),
            is_doubles: false,
            dodge_trackers: HashMap::new(),
        }
    }

    fn update_dodge_availability(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        controls: &PlayerControlState,
        events: &FrameEventsState,
    ) -> HashMap<PlayerId, bool> {
        let refreshes = events
            .dodge_refreshed_events
            .iter()
            .map(|event| event.player.clone())
            .collect::<HashSet<_>>();
        let mut availability = HashMap::new();
        for player in &players.players {
            let grounded = player
                .position()
                .is_some_and(|position| position.z <= PLAYER_GROUND_Z_THRESHOLD);
            let control = controls.sample(&player.player_id);
            let tracker = self
                .dodge_trackers
                .entry(player.player_id.clone())
                .or_default();

            if grounded {
                tracker.available = true;
                tracker.takeoff_time = None;
                tracker.unlimited_reset = false;
            } else if tracker.was_grounded {
                tracker.available = true;
                tracker.takeoff_time = Some(frame.time);
                tracker.unlimited_reset = false;
            }

            if !grounded
                && !tracker.unlimited_reset
                && tracker
                    .takeoff_time
                    .is_some_and(|time| frame.time - time > STANDARD_DODGE_WINDOW_SECONDS)
            {
                tracker.available = false;
            }

            let consumed = (control.dodge_active && !tracker.previous_dodge_active)
                || (control.double_jump_active && !tracker.previous_double_jump_active);
            if consumed {
                tracker.available = false;
                tracker.unlimited_reset = false;
            }
            if refreshes.contains(&player.player_id) {
                tracker.available = true;
                tracker.unlimited_reset = true;
                tracker.takeoff_time = None;
            }

            tracker.was_grounded = grounded;
            tracker.previous_dodge_active = control.dodge_active;
            tracker.previous_double_jump_active = control.double_jump_active;
            availability.insert(player.player_id.clone(), tracker.available);
        }
        availability
    }
}

impl Default for ThreatFeaturesNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for ThreatFeaturesNode {
    type State = ThreatFeaturesState;

    fn name(&self) -> &'static str {
        "threat_features"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        vec![
            ball_frame_state_dependency(),
            player_frame_state_dependency(),
            player_control_state_dependency(),
            frame_info_dependency(),
            frame_events_state_dependency(),
            live_play_dependency(),
        ]
    }

    fn on_replay_meta(&mut self, meta: &ReplayMeta) -> SubtrActorResult<()> {
        self.is_doubles = meta.team_zero.len() == 2 && meta.team_one.len() == 2;
        self.dodge_trackers.clear();
        self.state = ThreatFeaturesState::default();
        Ok(())
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        if !self.is_doubles {
            self.state.clear();
            return Ok(());
        }
        let frame = ctx.get::<FrameInfo>()?;
        let availability = self.update_dodge_availability(
            frame,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<PlayerControlState>()?,
            ctx.get::<FrameEventsState>()?,
        );
        self.state.update(
            frame.time,
            ctx.get::<BallFrameState>()?,
            ctx.get::<PlayerFrameState>()?,
            ctx.get::<FrameEventsState>()?,
            &availability,
            ctx.get::<LivePlayState>()?,
        );
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

pub(super) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(ThreatFeaturesNode::new())
}

#[cfg(test)]
#[path = "threat_features_tests.rs"]
mod tests;
