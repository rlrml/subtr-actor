use super::*;

pub(crate) const DEFAULT_BALL_HALF_NEUTRAL_ZONE_HALF_WIDTH_Y: f32 = 200.0;

/// Canonical ball-half classification, shared by the `ball_half` stream and the
/// possession cross-tab so there is a single definition of the midfield split
/// (and its neutral deadzone).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum BallHalfLabel {
    TeamZeroSide,
    TeamOneSide,
    #[default]
    Neutral,
}

impl BallHalfLabel {
    pub(crate) fn from_y(ball_y: f32, neutral_zone_half_width_y: f32) -> Self {
        if ball_y.abs() <= neutral_zone_half_width_y {
            Self::Neutral
        } else if ball_y < 0.0 {
            Self::TeamZeroSide
        } else {
            Self::TeamOneSide
        }
    }

    pub(crate) fn as_label_value(self) -> &'static str {
        match self {
            Self::TeamZeroSide => "team_zero_side",
            Self::TeamOneSide => "team_one_side",
            Self::Neutral => "neutral",
        }
    }
}

/// A change in which half of the field the ball occupies.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BallHalfEvent {
    pub time: f32,
    pub frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub active: bool,
    pub duration: f32,
    pub field_half: String,
}

impl BallHalfEvent {
    fn absorb_duration(&mut self, frame: &FrameInfo, duration: f32) {
        self.end_time = frame.time;
        self.end_frame = frame.frame_number;
        self.duration += duration;
    }
}

/// Configuration thresholds for ball-half classification.
#[derive(Debug, Clone, PartialEq)]
pub struct BallHalfCalculatorConfig {
    pub neutral_zone_half_width_y: f32,
}

impl Default for BallHalfCalculatorConfig {
    fn default() -> Self {
        Self {
            neutral_zone_half_width_y: DEFAULT_BALL_HALF_NEUTRAL_ZONE_HALF_WIDTH_Y,
        }
    }
}

/// Tracks which half of the field the ball is in over time.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BallHalfCalculator {
    config: BallHalfCalculatorConfig,
    events: EventStream<BallHalfEvent>,
    last_emitted_event_state: Option<BallHalfEventState>,
    pending_event: Option<PendingBallHalfEvent>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct BallHalfEventState {
    active: bool,
    field_half: BallHalfLabel,
}

#[derive(Debug, Clone, PartialEq)]
struct PendingBallHalfEvent {
    state: BallHalfEventState,
    event: BallHalfEvent,
}

impl BallHalfCalculator {
    pub fn new() -> Self {
        Self::with_config(BallHalfCalculatorConfig::default())
    }

    pub fn with_config(config: BallHalfCalculatorConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn events(&self) -> &[BallHalfEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[BallHalfEvent] {
        self.events.new_events()
    }

    pub fn projected_events(&self) -> Vec<BallHalfEvent> {
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
    pub fn current_event(&self) -> Option<&BallHalfEvent> {
        self.pending_event
            .as_ref()
            .map(|pending| &pending.event)
            .or_else(|| self.events.all().last())
    }

    pub fn config(&self) -> &BallHalfCalculatorConfig {
        &self.config
    }

    fn emit_event_if_changed(
        &mut self,
        frame: &FrameInfo,
        active: bool,
        duration: f32,
        field_half: BallHalfLabel,
    ) {
        let event_state = BallHalfEventState { active, field_half };
        if self.last_emitted_event_state == Some(event_state) && duration == 0.0 {
            return;
        }
        let event = BallHalfEvent {
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

    fn record_event(&mut self, state: BallHalfEventState, frame: &FrameInfo, event: BallHalfEvent) {
        let Some(pending) = self.pending_event.as_mut() else {
            self.pending_event = Some(PendingBallHalfEvent { state, event });
            return;
        };

        if pending.state == state {
            pending.event.absorb_duration(frame, event.duration);
        } else {
            let previous = self
                .pending_event
                .replace(PendingBallHalfEvent { state, event });
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
            self.emit_event_if_changed(frame, false, 0.0, BallHalfLabel::Neutral);
            return Ok(());
        }
        if let Some(ball) = ball.sample() {
            let half =
                BallHalfLabel::from_y(ball.position().y, self.config.neutral_zone_half_width_y);
            self.emit_event_if_changed(frame, true, frame.dt, half);
        } else {
            self.emit_event_if_changed(frame, false, 0.0, BallHalfLabel::Neutral);
        }
        Ok(())
    }
}
