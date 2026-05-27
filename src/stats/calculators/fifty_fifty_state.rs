use super::*;

#[path = "fifty_fifty_state_resolution.rs"]
mod fifty_fifty_state_resolution;

#[derive(Default)]
pub struct FiftyFiftyStateCalculator {
    active_event: Option<ActiveFiftyFifty>,
    last_resolved_event: Option<FiftyFiftyEvent>,
    kickoff_touch_window_open: bool,
}

impl FiftyFiftyStateCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_state: &TouchState,
        possession_state: &PossessionState,
        live_play_state: &LivePlayState,
    ) -> FiftyFiftyState {
        if FiftyFiftyCalculator::kickoff_phase_active(gameplay) {
            self.kickoff_touch_window_open = true;
        }

        if !live_play_state.is_live_play {
            self.reset();
            return FiftyFiftyState {
                active_event: None,
                resolved_events: Vec::new(),
                last_resolved_event: self.last_resolved_event.clone(),
            };
        }

        let has_touch = !touch_state.touch_events.is_empty();
        let has_contested_touch = touch_state
            .touch_events
            .iter()
            .any(|touch| touch.team_is_team_0)
            && touch_state
                .touch_events
                .iter()
                .any(|touch| !touch.team_is_team_0);

        if let Some(active_event) = self.active_event.as_mut() {
            let age = (frame.time - active_event.last_touch_time).max(0.0);
            if age <= FIFTY_FIFTY_CONTINUATION_TOUCH_WINDOW_SECONDS
                && active_event.contains_team_touch(&touch_state.touch_events)
            {
                active_event.last_touch_time = frame.time;
                active_event.last_touch_frame = frame.frame_number;
            }
        }

        let mut resolved_events = Vec::new();
        if let Some(event) = self.maybe_resolve_active_event(frame, ball, possession_state) {
            resolved_events.push(event);
        }

        if has_contested_touch {
            if self.active_event.is_none() {
                self.active_event = FiftyFiftyCalculator::contested_touch(
                    frame,
                    players,
                    &touch_state.touch_events,
                    self.kickoff_touch_window_open,
                );
            }
        } else if has_touch {
            if let Some(active_event) = self.active_event.as_mut() {
                let age = (frame.time - active_event.last_touch_time).max(0.0);
                if age <= FIFTY_FIFTY_CONTINUATION_TOUCH_WINDOW_SECONDS
                    && active_event.contains_team_touch(&touch_state.touch_events)
                {
                    active_event.last_touch_time = frame.time;
                    active_event.last_touch_frame = frame.frame_number;
                }
            }
        }

        if has_touch {
            self.kickoff_touch_window_open = false;
        }

        FiftyFiftyState {
            active_event: self.active_event.clone(),
            resolved_events,
            last_resolved_event: self.last_resolved_event.clone(),
        }
    }
}
