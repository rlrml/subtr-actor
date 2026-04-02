use super::*;

#[derive(Debug, Clone, Default)]
pub struct PossessionState {
    pub active_team_before_sample: Option<bool>,
    pub current_team_is_team_0: Option<bool>,
    pub active_player_before_sample: Option<PlayerId>,
    pub current_player: Option<PlayerId>,
}

#[derive(Default)]
pub struct PossessionStateCalculator {
    tracker: PossessionTracker,
    live_play_tracker: LivePlayTracker,
}

impl PossessionStateCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, sample: &CoreSample, touch_state: &TouchState) -> PossessionState {
        let live_play = self.live_play_tracker.is_live_play(sample);
        if !live_play {
            self.tracker.reset();
            return PossessionState::default();
        }

        self.tracker.update(sample, &touch_state.touch_events)
    }
}
