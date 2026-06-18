use super::*;

const DEFAULT_BALL_THIRD_BOUNDARY_Y: f32 = FIELD_ZONE_BOUNDARY_Y;

/// Canonical ball-third classification, shared by the `ball_third` stream and
/// the possession cross-tab so there is a single definition of the field-third
/// boundaries.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum BallThirdLabel {
    TeamZeroThird,
    #[default]
    NeutralThird,
    TeamOneThird,
}

impl BallThirdLabel {
    pub(crate) fn from_ball(ball: &BallSample) -> Self {
        Self::from_y(ball.position().y, DEFAULT_BALL_THIRD_BOUNDARY_Y)
    }

    pub(crate) fn from_y(ball_y: f32, boundary_y: f32) -> Self {
        if ball_y < -boundary_y {
            Self::TeamZeroThird
        } else if ball_y > boundary_y {
            Self::TeamOneThird
        } else {
            Self::NeutralThird
        }
    }

    pub(crate) fn as_label_value(self) -> &'static str {
        match self {
            Self::TeamZeroThird => "team_zero_third",
            Self::NeutralThird => "neutral_third",
            Self::TeamOneThird => "team_one_third",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BallThirdEvent {
    pub time: f32,
    pub frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub active: bool,
    pub duration: f32,
    pub field_third: String,
}

impl BallThirdEvent {
    fn absorb_duration(&mut self, frame: &FrameInfo, duration: f32) {
        self.end_time = frame.time;
        self.end_frame = frame.frame_number;
        self.duration += duration;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BallThirdCalculatorConfig {
    pub boundary_y: f32,
}

impl Default for BallThirdCalculatorConfig {
    fn default() -> Self {
        Self {
            boundary_y: DEFAULT_BALL_THIRD_BOUNDARY_Y,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BallThirdCalculator {
    config: BallThirdCalculatorConfig,
    events: EventStream<BallThirdEvent>,
    last_emitted_event_state: Option<BallThirdEventState>,
    pending_event: Option<PendingBallThirdEvent>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct BallThirdEventState {
    active: bool,
    field_third: BallThirdLabel,
}

#[derive(Debug, Clone, PartialEq)]
struct PendingBallThirdEvent {
    state: BallThirdEventState,
    event: BallThirdEvent,
}

impl BallThirdCalculator {
    pub fn new() -> Self {
        Self::with_config(BallThirdCalculatorConfig::default())
    }

    pub fn with_config(config: BallThirdCalculatorConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn events(&self) -> &[BallThirdEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[BallThirdEvent] {
        self.events.new_events()
    }

    pub fn projected_events(&self) -> Vec<BallThirdEvent> {
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

    /// The event covering the most recently processed frame (in-progress
    /// pending span, or the last committed event once flushed).
    pub fn current_event(&self) -> Option<&BallThirdEvent> {
        self.pending_event
            .as_ref()
            .map(|pending| &pending.event)
            .or_else(|| self.events.all().last())
    }

    pub fn config(&self) -> &BallThirdCalculatorConfig {
        &self.config
    }

    fn emit_event_if_changed(
        &mut self,
        frame: &FrameInfo,
        active: bool,
        duration: f32,
        field_third: BallThirdLabel,
    ) {
        let event_state = BallThirdEventState {
            active,
            field_third,
        };
        if self.last_emitted_event_state == Some(event_state) && duration == 0.0 {
            return;
        }
        let event = BallThirdEvent {
            time: frame.time,
            frame: frame.frame_number,
            end_time: frame.time,
            end_frame: frame.frame_number,
            active,
            duration,
            field_third: field_third.as_label_value().to_owned(),
        };
        self.record_event(event_state, frame, event);
        self.last_emitted_event_state = Some(event_state);
    }

    fn record_event(
        &mut self,
        state: BallThirdEventState,
        frame: &FrameInfo,
        event: BallThirdEvent,
    ) {
        let Some(pending) = self.pending_event.as_mut() else {
            self.pending_event = Some(PendingBallThirdEvent { state, event });
            return;
        };

        if pending.state == state {
            pending.event.absorb_duration(frame, event.duration);
        } else {
            let previous = self
                .pending_event
                .replace(PendingBallThirdEvent { state, event });
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
            self.emit_event_if_changed(frame, false, 0.0, BallThirdLabel::NeutralThird);
            return Ok(());
        }
        if let Some(ball) = ball.sample() {
            let third = BallThirdLabel::from_y(ball.position().y, self.config.boundary_y);
            self.emit_event_if_changed(frame, true, frame.dt, third);
        } else {
            self.emit_event_if_changed(frame, false, 0.0, BallThirdLabel::NeutralThird);
        }
        Ok(())
    }
}
