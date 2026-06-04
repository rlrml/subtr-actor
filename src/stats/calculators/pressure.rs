use super::*;

const DEFAULT_PRESSURE_NEUTRAL_ZONE_HALF_WIDTH_Y: f32 = 200.0;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum PressureHalfLabel {
    TeamZeroSide,
    TeamOneSide,
    #[default]
    Neutral,
}

impl PressureHalfLabel {
    fn as_label_value(self) -> &'static str {
        match self {
            Self::TeamZeroSide => "team_zero_side",
            Self::TeamOneSide => "team_one_side",
            Self::Neutral => "neutral",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PressureEvent {
    pub time: f32,
    pub frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub active: bool,
    pub duration: f32,
    pub field_half: String,
}

impl PressureEvent {
    fn absorb_duration(&mut self, frame: &FrameInfo, duration: f32) {
        self.end_time = frame.time;
        self.end_frame = frame.frame_number;
        self.duration += duration;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PressureCalculatorConfig {
    pub neutral_zone_half_width_y: f32,
}

impl Default for PressureCalculatorConfig {
    fn default() -> Self {
        Self {
            neutral_zone_half_width_y: DEFAULT_PRESSURE_NEUTRAL_ZONE_HALF_WIDTH_Y,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PressureCalculator {
    config: PressureCalculatorConfig,
    events: EventStream<PressureEvent>,
    last_emitted_event_state: Option<PressureEventState>,
    pending_event: Option<PendingPressureEvent>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct PressureEventState {
    active: bool,
    field_half: PressureHalfLabel,
}

#[derive(Debug, Clone, PartialEq)]
struct PendingPressureEvent {
    state: PressureEventState,
    event: PressureEvent,
}

impl PressureCalculator {
    pub fn new() -> Self {
        Self::with_config(PressureCalculatorConfig::default())
    }

    pub fn with_config(config: PressureCalculatorConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn events(&self) -> &[PressureEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[PressureEvent] {
        self.events.new_events()
    }

    pub fn projected_events(&self) -> Vec<PressureEvent> {
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

    pub fn config(&self) -> &PressureCalculatorConfig {
        &self.config
    }

    fn emit_event_if_changed(
        &mut self,
        frame: &FrameInfo,
        active: bool,
        duration: f32,
        field_half: PressureHalfLabel,
    ) {
        let event_state = PressureEventState { active, field_half };
        if self.last_emitted_event_state == Some(event_state) && duration == 0.0 {
            return;
        }
        let event = PressureEvent {
            time: frame.time,
            frame: frame.frame_number,
            end_time: frame.time,
            end_frame: frame.frame_number,
            active,
            duration,
            field_half: field_half.as_label_value().to_owned(),
        };
        self.record_event(event_state, frame, event);
        self.last_emitted_event_state = Some(event_state);
    }

    fn record_event(&mut self, state: PressureEventState, frame: &FrameInfo, event: PressureEvent) {
        let Some(pending) = self.pending_event.as_mut() else {
            self.pending_event = Some(PendingPressureEvent { state, event });
            return;
        };

        if pending.state == state {
            pending.event.absorb_duration(frame, event.duration);
        } else {
            let previous = self
                .pending_event
                .replace(PendingPressureEvent { state, event });
            let Some(previous) = previous else {
                return;
            };
            self.events.push(previous.event);
        }
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        if !live_play_state.is_live_play {
            self.emit_event_if_changed(frame, false, 0.0, PressureHalfLabel::Neutral);
            return Ok(());
        }
        if let Some(ball) = ball.sample() {
            let ball_y = ball.position().y;
            let half = if ball_y.abs() <= self.config.neutral_zone_half_width_y {
                PressureHalfLabel::Neutral
            } else if ball_y < 0.0 {
                PressureHalfLabel::TeamZeroSide
            } else {
                PressureHalfLabel::TeamOneSide
            };
            self.emit_event_if_changed(frame, true, frame.dt, half);
        } else {
            self.emit_event_if_changed(frame, false, 0.0, PressureHalfLabel::Neutral);
        }
        Ok(())
    }
}
