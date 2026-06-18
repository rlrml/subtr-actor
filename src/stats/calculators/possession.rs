use super::*;

const PENDING_TURNOVER_CONFIRMATION_WINDOW_SECONDS: f32 = 1.25;
const LOOSE_BALL_TIMEOUT_SECONDS: f32 = 3.0;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum PossessionStateLabel {
    TeamZero,
    TeamOne,
    #[default]
    Neutral,
}

impl PossessionStateLabel {
    fn as_label_value(self) -> &'static str {
        match self {
            Self::TeamZero => "team_zero",
            Self::TeamOne => "team_one",
            Self::Neutral => "neutral",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PossessionEvent {
    pub time: f32,
    pub frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub active: bool,
    pub duration: f32,
    pub possession_state: String,
    #[ts(as = "Option<crate::interop::ts_bindings::RemoteIdTs>")]
    pub player_id: Option<PlayerId>,
}

impl PossessionEvent {
    fn absorb_duration(&mut self, frame: &FrameInfo, duration: f32) {
        self.end_time = frame.time;
        self.end_frame = frame.frame_number;
        self.duration += duration;
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct PossessionTracker {
    current_team_is_team_0: Option<bool>,
    current_player: Option<PlayerId>,
    last_possession_touch_time: Option<f32>,
    pending_turnover_team_is_team_0: Option<bool>,
    pending_turnover_touch_time: Option<f32>,
}

impl PossessionTracker {
    fn clear_pending_turnover(&mut self) {
        self.pending_turnover_team_is_team_0 = None;
        self.pending_turnover_touch_time = None;
    }

    pub(crate) fn reset(&mut self) {
        self.current_team_is_team_0 = None;
        self.current_player = None;
        self.last_possession_touch_time = None;
        self.clear_pending_turnover();
    }

    fn expire_pending_turnover(&mut self, time: f32) {
        let Some(pending_time) = self.pending_turnover_touch_time else {
            return;
        };
        if time - pending_time < PENDING_TURNOVER_CONFIRMATION_WINDOW_SECONDS {
            return;
        }

        self.current_team_is_team_0 = None;
        self.current_player = None;
        self.last_possession_touch_time = None;
        self.clear_pending_turnover();
    }

    fn expire_loose_ball(&mut self, time: f32) {
        if self.pending_turnover_team_is_team_0.is_some() {
            return;
        }
        let Some(last_touch_time) = self.last_possession_touch_time else {
            return;
        };
        if time - last_touch_time < LOOSE_BALL_TIMEOUT_SECONDS {
            return;
        }

        self.current_team_is_team_0 = None;
        self.current_player = None;
        self.last_possession_touch_time = None;
    }

    fn register_single_team_touch(&mut self, team_is_team_0: bool, time: f32) {
        if self.current_team_is_team_0 == Some(team_is_team_0) {
            self.last_possession_touch_time = Some(time);
            self.clear_pending_turnover();
            return;
        }

        if self.current_team_is_team_0.is_none() {
            self.current_team_is_team_0 = Some(team_is_team_0);
            self.last_possession_touch_time = Some(time);
            self.clear_pending_turnover();
            return;
        }

        if self.pending_turnover_team_is_team_0 == Some(team_is_team_0) {
            self.current_team_is_team_0 = Some(team_is_team_0);
            self.last_possession_touch_time = Some(time);
            self.clear_pending_turnover();
            return;
        }

        self.pending_turnover_team_is_team_0 = Some(team_is_team_0);
        self.pending_turnover_touch_time = Some(time);
    }

    fn register_contested_touch(&mut self, time: f32) {
        let Some(current_team_is_team_0) = self.current_team_is_team_0 else {
            self.clear_pending_turnover();
            return;
        };

        self.last_possession_touch_time = Some(time);
        self.pending_turnover_team_is_team_0 = Some(!current_team_is_team_0);
        self.pending_turnover_touch_time = Some(time);
    }

    fn update_player_control(
        &mut self,
        active_team_before_sample: Option<bool>,
        touched_team_zero_player: Option<&PlayerId>,
        touched_team_one_player: Option<&PlayerId>,
    ) {
        let Some(current_team_is_team_0) = self.current_team_is_team_0 else {
            self.current_player = None;
            return;
        };

        if self.pending_turnover_team_is_team_0.is_some() {
            self.current_player = None;
            return;
        }

        let controlling_touch_player = if current_team_is_team_0 {
            touched_team_zero_player
        } else {
            touched_team_one_player
        };
        if let Some(player) = controlling_touch_player {
            self.current_player = Some(player.clone());
            return;
        }

        if active_team_before_sample != self.current_team_is_team_0 {
            self.current_player = None;
        }
    }

    fn latest_touch_player_for_team(
        touch_events: &[TouchEvent],
        team_is_team_0: bool,
    ) -> Option<PlayerId> {
        touch_events
            .iter()
            .filter(|touch| touch.team_is_team_0 == team_is_team_0)
            .max_by(|left, right| TouchEvent::timestamp_ordering(left, right))
            .and_then(|touch| touch.player.clone())
    }

    pub(crate) fn update(&mut self, time: f32, touch_events: &[TouchEvent]) -> PossessionState {
        self.expire_pending_turnover(time);
        self.expire_loose_ball(time);

        let active_team_before_sample = self.current_team_is_team_0;
        let active_player_before_sample = self.current_player.clone();
        let touched_team_zero = touch_events.iter().any(|touch| touch.team_is_team_0);
        let touched_team_one = touch_events.iter().any(|touch| !touch.team_is_team_0);
        let touched_team_zero_player = Self::latest_touch_player_for_team(touch_events, true);
        let touched_team_one_player = Self::latest_touch_player_for_team(touch_events, false);

        match (touched_team_zero, touched_team_one) {
            (true, true) => self.register_contested_touch(time),
            (true, false) => self.register_single_team_touch(true, time),
            (false, true) => self.register_single_team_touch(false, time),
            (false, false) => {}
        }
        self.update_player_control(
            active_team_before_sample,
            touched_team_zero_player.as_ref(),
            touched_team_one_player.as_ref(),
        );

        PossessionState {
            active_team_before_sample,
            current_team_is_team_0: self.current_team_is_team_0,
            active_player_before_sample,
            current_player: self.current_player.clone(),
        }
    }
}

#[cfg(test)]
#[path = "possession_tests.rs"]
mod tests;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PossessionCalculator {
    tracker: PossessionTracker,
    events: EventStream<PossessionEvent>,
    last_emitted_event_state: Option<PossessionEventState>,
    pending_event: Option<PendingPossessionEvent>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct PossessionEventState {
    active: bool,
    possession_state: PossessionStateLabel,
    player_id: Option<PlayerId>,
}

#[derive(Debug, Clone, PartialEq)]
struct PendingPossessionEvent {
    state: PossessionEventState,
    event: PossessionEvent,
}

impl PossessionCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[PossessionEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[PossessionEvent] {
        self.events.new_events()
    }

    pub fn projected_events(&self) -> Vec<PossessionEvent> {
        let mut events = self.events.all().to_vec();
        if let Some(pending) = &self.pending_event {
            events.push(pending.event.clone());
        }
        events
    }

    pub fn flush_pending_event(&mut self) {
        let Some(pending) = self.pending_event.take() else {
            return;
        };
        self.events.push(pending.event);
    }

    /// The event covering the most recently processed frame (the in-progress
    /// pending span, or the last committed event once flushed). Used by the
    /// projection to read the current possession state per frame without
    /// rebuilding from the full event stream.
    pub fn current_event(&self) -> Option<&PossessionEvent> {
        self.pending_event
            .as_ref()
            .map(|pending| &pending.event)
            .or_else(|| self.events.all().last())
    }

    fn emit_event_if_changed(
        &mut self,
        frame: &FrameInfo,
        active: bool,
        duration: f32,
        possession_state: PossessionStateLabel,
        player_id: Option<PlayerId>,
    ) {
        let event_state = PossessionEventState {
            active,
            possession_state,
            player_id: player_id.clone(),
        };
        if self.last_emitted_event_state.as_ref() == Some(&event_state) && duration == 0.0 {
            return;
        }
        let event = PossessionEvent {
            time: frame.time,
            frame: frame.frame_number,
            end_time: frame.time,
            end_frame: frame.frame_number,
            active,
            duration,
            possession_state: possession_state.as_label_value().to_owned(),
            player_id,
        };
        self.record_event(event_state.clone(), frame, event);
        self.last_emitted_event_state = Some(event_state);
    }

    fn record_event(
        &mut self,
        state: PossessionEventState,
        frame: &FrameInfo,
        event: PossessionEvent,
    ) {
        let Some(pending) = self.pending_event.as_mut() else {
            self.pending_event = Some(PendingPossessionEvent { state, event });
            return;
        };

        if pending.state == state {
            pending.event.absorb_duration(frame, event.duration);
        } else {
            let previous = self
                .pending_event
                .replace(PendingPossessionEvent { state, event });
            let Some(previous) = previous else {
                return;
            };
            self.events.push(previous.event);
        }
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        possession_state: &PossessionState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        if !live_play_state.is_live_play {
            self.emit_event_if_changed(frame, false, 0.0, PossessionStateLabel::Neutral, None);
            return Ok(());
        }

        let (state, player_id) =
            if let Some(possession_team_is_team_0) = possession_state.active_team_before_sample {
                if possession_team_is_team_0 {
                    (
                        PossessionStateLabel::TeamZero,
                        possession_state.active_player_before_sample.clone(),
                    )
                } else {
                    (
                        PossessionStateLabel::TeamOne,
                        possession_state.active_player_before_sample.clone(),
                    )
                }
            } else {
                (PossessionStateLabel::Neutral, None)
            };
        self.emit_event_if_changed(frame, true, frame.dt, state, player_id);
        Ok(())
    }
}
