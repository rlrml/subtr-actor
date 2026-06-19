use super::*;

/// Maintains shared 50/50 contest state for downstream consumers.
#[derive(Default)]
pub struct FiftyFiftyStateCalculator {
    active_event: Option<ActiveFiftyFifty>,
    last_resolved_event: Option<FiftyFiftyEvent>,
    pending_initial_touch: Option<PendingFiftyFiftyTouch>,
    kickoff_touch_window_open: bool,
}

#[derive(Clone)]
struct PendingFiftyFiftyTouch {
    touch: TouchEvent,
    is_kickoff: bool,
}

impl FiftyFiftyStateCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    fn reset(&mut self) {
        self.active_event = None;
        self.pending_initial_touch = None;
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
            team_zero_touch_time: active.team_zero_touch_time,
            team_zero_touch_frame: active.team_zero_touch_frame,
            team_zero_dodge_contact: active.team_zero_dodge_contact,
            team_one_touch_time: active.team_one_touch_time,
            team_one_touch_frame: active.team_one_touch_frame,
            team_one_dodge_contact: active.team_one_dodge_contact,
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

    fn update_active_last_touch_from_continuation(
        frame: &FrameInfo,
        active_event: &mut ActiveFiftyFifty,
        touch_events: &[TouchEvent],
    ) {
        let age = (frame.time - active_event.last_touch_time).max(0.0);
        if age > FIFTY_FIFTY_CONTINUATION_TOUCH_WINDOW_SECONDS {
            return;
        }
        let Some(touch) = active_event.latest_continuation_touch(touch_events) else {
            return;
        };
        active_event.last_touch_time = touch.time;
        active_event.last_touch_frame = touch.frame;
    }

    fn touch_pair_in_initial_window(previous: &TouchEvent, current: &TouchEvent) -> bool {
        current.frame >= previous.frame
            && current.time >= previous.time
            && current.time - previous.time <= FIFTY_FIFTY_CONTINUATION_TOUCH_WINDOW_SECONDS
    }

    fn contested_touch_from_pending(
        frame: &FrameInfo,
        players: &PlayerFrameState,
        pending: &PendingFiftyFiftyTouch,
        touch_events: &[TouchEvent],
        is_kickoff: bool,
    ) -> Option<ActiveFiftyFifty> {
        let latest_opposing_touch = touch_events
            .iter()
            .filter(|touch| touch.team_is_team_0 != pending.touch.team_is_team_0)
            .filter(|touch| Self::touch_pair_in_initial_window(&pending.touch, touch))
            .max_by(|left, right| TouchEvent::timestamp_ordering(left, right))?;

        let combined_touches = vec![pending.touch.clone(), latest_opposing_touch.clone()];
        let mut active = FiftyFiftyCalculator::contested_touch(
            frame,
            players,
            &combined_touches,
            pending.is_kickoff || is_kickoff,
        )?;
        active.start_time = pending.touch.time.min(latest_opposing_touch.time);
        active.start_frame = pending.touch.frame.min(latest_opposing_touch.frame);
        Some(active)
    }

    fn pending_touch_from_events(
        touch_events: &[TouchEvent],
        is_kickoff: bool,
    ) -> Option<PendingFiftyFiftyTouch> {
        touch_events
            .iter()
            .max_by(|left, right| TouchEvent::timestamp_ordering(left, right))
            .cloned()
            .map(|touch| PendingFiftyFiftyTouch { touch, is_kickoff })
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

        if !live_play_state.counts_toward_player_motion() {
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
            Self::update_active_last_touch_from_continuation(
                frame,
                active_event,
                &touch_state.touch_events,
            );
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
                if self.active_event.is_some() {
                    self.pending_initial_touch = None;
                }
            }
        } else if has_touch {
            if self.active_event.is_none() {
                if let Some(pending) = self.pending_initial_touch.as_ref() {
                    self.active_event = Self::contested_touch_from_pending(
                        frame,
                        players,
                        pending,
                        &touch_state.touch_events,
                        self.kickoff_touch_window_open,
                    );
                }
                if self.active_event.is_some() {
                    self.pending_initial_touch = None;
                } else {
                    self.pending_initial_touch = Self::pending_touch_from_events(
                        &touch_state.touch_events,
                        self.kickoff_touch_window_open,
                    );
                }
            }
            if let Some(active_event) = self.active_event.as_mut() {
                Self::update_active_last_touch_from_continuation(
                    frame,
                    active_event,
                    &touch_state.touch_events,
                );
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

#[cfg(test)]
#[path = "fifty_fifty_state_tests.rs"]
mod tests;
