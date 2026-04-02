use super::*;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BackboardBounceEvent {
    pub time: f32,
    pub frame: usize,
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
    live_play_tracker: LivePlayTracker,
}

impl BackboardBounceCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    fn detect_bounce(&self, sample: &CoreSample) -> Option<BackboardBounceEvent> {
        const BACKBOARD_MIN_BALL_Z: f32 = 500.0;
        const BACKBOARD_MIN_NORMALIZED_Y: f32 = 4700.0;
        const BACKBOARD_MAX_ABS_X: f32 = 1600.0;
        const BACKBOARD_MIN_APPROACH_SPEED_Y: f32 = 350.0;
        const BACKBOARD_MIN_REBOUND_SPEED_Y: f32 = 250.0;
        const BACKBOARD_TOUCH_ATTRIBUTION_MAX_SECONDS: f32 = 2.5;

        if !sample.touch_events.is_empty() {
            return None;
        }

        let last_touch = self.last_touch.as_ref()?;
        let player = last_touch.player.clone()?;
        let current_ball = sample.ball.as_ref()?;
        let previous_ball_velocity = self.previous_ball_velocity?;

        if (sample.time - last_touch.time).max(0.0) > BACKBOARD_TOUCH_ATTRIBUTION_MAX_SECONDS {
            return None;
        }

        let ball_position = current_ball.position();
        if ball_position.x.abs() > BACKBOARD_MAX_ABS_X || ball_position.z < BACKBOARD_MIN_BALL_Z {
            return None;
        }

        let normalized_position_y = normalized_y(last_touch.team_is_team_0, ball_position);
        if normalized_position_y < BACKBOARD_MIN_NORMALIZED_Y {
            return None;
        }

        let previous_normalized_velocity_y = if last_touch.team_is_team_0 {
            previous_ball_velocity.y
        } else {
            -previous_ball_velocity.y
        };
        let current_normalized_velocity_y = if last_touch.team_is_team_0 {
            current_ball.velocity().y
        } else {
            -current_ball.velocity().y
        };

        if previous_normalized_velocity_y < BACKBOARD_MIN_APPROACH_SPEED_Y {
            return None;
        }
        if current_normalized_velocity_y > -BACKBOARD_MIN_REBOUND_SPEED_Y {
            return None;
        }

        Some(BackboardBounceEvent {
            time: sample.time,
            frame: sample.frame_number,
            player,
            is_team_0: last_touch.team_is_team_0,
        })
    }

    pub fn update(&mut self, sample: &CoreSample) -> BackboardBounceState {
        let live_play = self.live_play_tracker.is_live_play(sample);
        if !live_play {
            self.previous_ball_velocity = sample.ball.as_ref().map(BallSample::velocity);
            self.last_touch = None;
            self.last_bounce_event = None;
            return BackboardBounceState::default();
        }

        let bounce_events: Vec<_> = self.detect_bounce(sample).into_iter().collect();
        if let Some(last_bounce_event) = bounce_events.last() {
            self.last_bounce_event = Some(last_bounce_event.clone());
        }

        if let Some(last_touch) = sample.touch_events.last() {
            self.last_touch = Some(last_touch.clone());
        }
        self.previous_ball_velocity = sample.ball.as_ref().map(BallSample::velocity);

        BackboardBounceState {
            bounce_events,
            last_bounce_event: self.last_bounce_event.clone(),
        }
    }
}
