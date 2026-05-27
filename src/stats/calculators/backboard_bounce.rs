use super::*;

#[path = "backboard_bounce_detection.rs"]
mod detection;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BackboardBounceEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BackboardBounceState {
    pub bounce_events: Vec<BackboardBounceEvent>,
    pub last_bounce_event: Option<BackboardBounceEvent>,
}

#[derive(Default)]
pub struct BackboardBounceCalculator {
    previous_ball_velocity: Option<glam::Vec3>,
    last_touch: Option<TouchEvent>,
    last_bounce_event: Option<BackboardBounceEvent>,
}

impl BackboardBounceCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        touch_state: &TouchState,
        live_play_state: &LivePlayState,
    ) -> BackboardBounceState {
        if !live_play_state.is_live_play {
            self.previous_ball_velocity = ball.velocity();
            self.last_touch = None;
            self.last_bounce_event = None;
            return BackboardBounceState::default();
        }

        let bounce_events: Vec<_> = self
            .detect_bounce(frame, ball.sample(), &touch_state.touch_events)
            .into_iter()
            .collect();
        if let Some(last_bounce_event) = bounce_events.last() {
            self.last_bounce_event = Some(last_bounce_event.clone());
        }

        if let Some(last_touch) = touch_state.touch_events.last() {
            self.last_touch = Some(last_touch.clone());
        }
        self.previous_ball_velocity = ball.velocity();

        BackboardBounceState {
            bounce_events,
            last_bounce_event: self.last_bounce_event.clone(),
        }
    }
}
