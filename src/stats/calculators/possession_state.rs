use super::*;

/// Shared current ball-possession state.
#[derive(Debug, Clone, Default)]
pub struct PossessionState {
    pub active_team_before_sample: Option<bool>,
    pub current_team_is_team_0: Option<bool>,
    pub active_player_before_sample: Option<PlayerId>,
    pub current_player: Option<PlayerId>,
    /// Team-possession segments finalized by the backdating resolver this frame.
    pub(crate) newly_resolved: Vec<ResolvedPossession>,
    /// The resolver's current open (unresolved) segment, if live.
    pub(crate) open_possession: Option<OpenPossession>,
}

/// Maintains shared ball-possession state from touches and live play.
#[derive(Default)]
pub struct PossessionStateCalculator {
    tracker: PossessionTracker,
    was_live: bool,
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
            // On the falling edge, flush the open segment so the just-ended live
            // stretch is fully resolved (the trailing held tail goes neutral).
            let newly_resolved = if self.was_live {
                self.tracker.flush_resolver(frame)
            } else {
                Vec::new()
            };
            self.tracker.reset();
            self.was_live = false;
            return PossessionState {
                newly_resolved,
                ..PossessionState::default()
            };
        }

        if !self.was_live {
            self.tracker.begin_resolver(frame);
            self.was_live = true;
        }

        self.tracker.update(frame, &touch_state.touch_events)
    }
}
