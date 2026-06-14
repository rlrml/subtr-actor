use super::*;

/// Shared current ball-possession state.
#[derive(Debug, Clone, Default)]
pub struct PossessionState {
    pub active_team_before_sample: Option<bool>,
    pub current_team_is_team_0: Option<bool>,
    pub active_player_before_sample: Option<PlayerId>,
    pub current_player: Option<PlayerId>,
}

/// Maintains shared ball-possession state from touches and live play.
#[derive(Default)]
pub struct PossessionStateCalculator {
    tracker: PossessionTracker,
}

impl PossessionStateCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        touch_state: &TouchState,
        live_play_state: &LivePlayState,
    ) -> PossessionState {
        if !live_play_state.is_live_play {
            self.tracker.reset();
            return PossessionState::default();
        }

        self.tracker.update(frame.time, &touch_state.touch_events)
    }
}
