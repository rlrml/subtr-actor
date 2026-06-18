use super::*;

const ONE_TIMER_MIN_BALL_SPEED: f32 = 1000.0;
const ONE_TIMER_MIN_GOAL_ALIGNMENT_COSINE: f32 = 0.65;
const GOAL_CENTER_Y: f32 = 5120.0;
const GOAL_MOUTH_HEIGHT_Z: f32 = 642.775;
const GOAL_MOUTH_TRAJECTORY_MARGIN: f32 = BALL_RADIUS_Z * 1.5;
/// The post-touch trajectory must cross the opponent goal mouth within this many
/// seconds for the touch to read as a one-timer (i.e. it must actually be on
/// net, not merely aimed in the goal's general direction).
const ONE_TIMER_MAX_TIME_TO_GOAL_SECONDS: f32 = 4.0;

/// A first-touch shot taken off an incoming pass without trapping the ball.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct OneTimerEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub passer: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub passer_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub pass_start_time: f32,
    pub pass_start_frame: usize,
    pub pass_duration: f32,
    pub pass_travel_distance: f32,
    pub pass_advance_distance: f32,
    pub ball_speed: f32,
    pub goal_alignment: f32,
}

/// Detects one-timers from ball state and upstream pass detection.
#[derive(Debug, Clone, Default)]
pub struct OneTimerCalculator {
    events: EventStream<OneTimerEvent>,
    processed_pass_events: usize,
}

impl OneTimerCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[OneTimerEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[OneTimerEvent] {
        self.events.new_events()
    }

    fn one_timer_event_for_pass(pass: &PassEvent, ball: &BallFrameState) -> Option<OneTimerEvent> {
        // A one-timer is a direct first-touch redirect of a pass. If the ball
        // bounced off the backboard between the passer's touch and the
        // receiver's finish, that is a double tap / backboard play, not a
        // one-timer, so exclude the backboard pass kinds here.
        if matches!(
            pass.pass_kind,
            PassKind::Backboard | PassKind::FiftyFiftyBackboard
        ) {
            return None;
        }

        let ball = ball.sample()?;
        let ball_position = ball.position();
        let ball_velocity = ball.velocity();
        let ball_speed = ball_velocity.length();
        if ball_speed < ONE_TIMER_MIN_BALL_SPEED {
            return None;
        }

        let target_y = if pass.is_team_0 {
            GOAL_CENTER_Y
        } else {
            -GOAL_CENTER_Y
        };
        let goal_direction = glam::Vec3::new(0.0, target_y, ball_position.z) - ball_position;
        let goal_alignment = goal_direction
            .normalize_or_zero()
            .dot(ball_velocity.normalize_or_zero());
        if goal_alignment < ONE_TIMER_MIN_GOAL_ALIGNMENT_COSINE {
            return None;
        }

        if !Self::trajectory_on_net(ball_position, ball_velocity, target_y) {
            return None;
        }

        Some(OneTimerEvent {
            time: pass.time,
            frame: pass.frame,
            player: pass.receiver.clone(),
            player_position: pass.receiver_position,
            passer: pass.passer.clone(),
            passer_position: pass.passer_position,
            is_team_0: pass.is_team_0,
            pass_start_time: pass.start_time,
            pass_start_frame: pass.start_frame,
            pass_duration: pass.duration,
            pass_travel_distance: pass.ball_travel_distance,
            pass_advance_distance: pass.ball_advance_distance,
            ball_speed,
            goal_alignment,
        })
    }

    /// Whether the post-touch ball trajectory, extended in a straight line to
    /// the opponent goal plane, actually crosses the goal mouth (the shot is "on
    /// net"). This is stricter than the goal-direction alignment check, which
    /// only requires the ball to be heading toward the goal's general area.
    fn trajectory_on_net(position: glam::Vec3, velocity: glam::Vec3, target_goal_y: f32) -> bool {
        if velocity.y.abs() <= f32::EPSILON {
            return false;
        }
        let time_to_goal_line = (target_goal_y - position.y) / velocity.y;
        if !time_to_goal_line.is_finite()
            || !(0.0..=ONE_TIMER_MAX_TIME_TO_GOAL_SECONDS).contains(&time_to_goal_line)
        {
            return false;
        }
        let projected = position + velocity * time_to_goal_line;
        projected.x.abs() <= BACK_WALL_GOAL_MOUTH_HALF_WIDTH_X + GOAL_MOUTH_TRAJECTORY_MARGIN
            && projected.z >= BALL_RADIUS_Z - GOAL_MOUTH_TRAJECTORY_MARGIN
            && projected.z <= GOAL_MOUTH_HEIGHT_Z + GOAL_MOUTH_TRAJECTORY_MARGIN
    }

    fn record_one_timer(&mut self, _frame: &FrameInfo, event: OneTimerEvent) {
        self.events.push(event);
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        pass_calculator: &PassCalculator,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        if !live_play_state.is_live_play {
            self.processed_pass_events = pass_calculator.events().len();
            return Ok(());
        }

        for pass in &pass_calculator.events()[self.processed_pass_events..] {
            if pass.frame != frame.frame_number {
                continue;
            }
            if let Some(event) = Self::one_timer_event_for_pass(pass, ball) {
                self.record_one_timer(frame, event);
            }
        }
        self.processed_pass_events = pass_calculator.events().len();

        Ok(())
    }
}

#[cfg(test)]
#[path = "one_timer_tests.rs"]
mod tests;
