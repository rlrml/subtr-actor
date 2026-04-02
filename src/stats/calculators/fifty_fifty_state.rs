use super::*;

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

    fn reset(&mut self) {
        self.active_event = None;
    }

    fn maybe_resolve_active_event(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        possession_state: &PossessionState,
    ) -> Option<FiftyFiftyEvent> {
        let active = self.active_event.as_ref()?;
        let age = (frame.time - active.last_touch_time).max(0.0);
        if age < FIFTY_FIFTY_RESOLUTION_DELAY_SECONDS {
            return None;
        }

        let winning_team_is_team_0 = FiftyFiftyCalculator::winning_team_from_ball(active, ball);
        let possession_team_is_team_0 = possession_state.current_team_is_team_0;
        let should_resolve = winning_team_is_team_0.is_some()
            || possession_team_is_team_0.is_some()
            || age >= FIFTY_FIFTY_MAX_DURATION_SECONDS;
        if !should_resolve {
            return None;
        }

        let active = self.active_event.take()?;
        let event = FiftyFiftyEvent {
            start_time: active.start_time,
            start_frame: active.start_frame,
            resolve_time: frame.time,
            resolve_frame: frame.frame_number,
            is_kickoff: active.is_kickoff,
            team_zero_player: active.team_zero_player,
            team_one_player: active.team_one_player,
            team_zero_position: active.team_zero_position,
            team_one_position: active.team_one_position,
            midpoint: active.midpoint,
            plane_normal: active.plane_normal,
            winning_team_is_team_0,
            possession_team_is_team_0,
        };
        self.last_resolved_event = Some(event.clone());
        Some(event)
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
        live_play: bool,
    ) -> FiftyFiftyState {
        if FiftyFiftyCalculator::kickoff_phase_active(gameplay) {
            self.kickoff_touch_window_open = true;
        }

        if !live_play {
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
