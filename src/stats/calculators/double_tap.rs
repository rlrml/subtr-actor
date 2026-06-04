use super::*;

/// Maximum time between the attributed backboard bounce and the follow-up touch.
const DOUBLE_TAP_TOUCH_WINDOW_SECONDS: f32 = 2.5;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DoubleTapEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub backboard_time: f32,
    pub backboard_frame: usize,
}

#[derive(Debug, Clone)]
struct PendingBackboardBounce {
    player_id: PlayerId,
    is_team_0: bool,
    time: f32,
    frame: usize,
}

/// Detects double taps from a backboard-bounce sequence.
///
/// Current heuristic:
///
/// 1. A [`BackboardBounceEvent`] arms a pending double tap for the player who
///    last touched the ball before the bounce. The exact backboard geometry and
///    attribution thresholds live in [`BackboardBounceCalculator`].
/// 2. The same player must touch the ball again within
///    [`DOUBLE_TAP_TOUCH_WINDOW_SECONDS`] while the replay is in live play.
/// 3. The ball's post-touch constant-velocity trajectory must project into or
///    close to the opponent goal mouth.
///
/// The detector intentionally does not aim at the center of the goal. Near-post
/// shots and cross-goal trajectories can be valid double taps even when their
/// velocity is poorly aligned with the goal center.
#[derive(Debug, Clone, Default)]
pub struct DoubleTapCalculator {
    events: EventStream<DoubleTapEvent>,
    pending_backboard_bounces: Vec<PendingBackboardBounce>,
}

impl DoubleTapCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[DoubleTapEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[DoubleTapEvent] {
        self.events.new_events()
    }

    fn prune_pending_backboard_bounces(&mut self, current_time: f32) {
        self.pending_backboard_bounces
            .retain(|entry| current_time - entry.time <= DOUBLE_TAP_TOUCH_WINDOW_SECONDS);
    }

    fn record_backboard_bounces(&mut self, state: &BackboardBounceState) {
        for event in &state.bounce_events {
            if let Some(existing) = self
                .pending_backboard_bounces
                .iter_mut()
                .find(|pending| pending.player_id == event.player)
            {
                *existing = PendingBackboardBounce {
                    player_id: event.player.clone(),
                    is_team_0: event.is_team_0,
                    time: event.time,
                    frame: event.frame,
                };
            } else {
                self.pending_backboard_bounces.push(PendingBackboardBounce {
                    player_id: event.player.clone(),
                    is_team_0: event.is_team_0,
                    time: event.time,
                    frame: event.frame,
                });
            }
        }
    }

    fn resolve_double_tap_touches(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        touch_events: &[TouchEvent],
    ) {
        if touch_events.is_empty() || self.pending_backboard_bounces.is_empty() {
            return;
        }

        let mut completed_events = Vec::new();
        self.pending_backboard_bounces.retain(|pending| {
            if frame.time <= pending.time {
                return true;
            }

            let matching_touch = Self::latest_matching_touch(touch_events, pending);

            if let Some(matching_touch) = matching_touch {
                if Self::followup_touch_projects_on_goal_mouth(ball, pending.is_team_0) {
                    completed_events.push(DoubleTapEvent {
                        time: matching_touch.time,
                        frame: matching_touch.frame,
                        player: pending.player_id.clone(),
                        player_position: matching_touch
                            .player_position
                            .map(|position| vec_to_glam(&position).to_array()),
                        is_team_0: pending.is_team_0,
                        backboard_time: pending.time,
                        backboard_frame: pending.frame,
                    });
                }
            }
            false
        });

        for event in completed_events {
            self.record_double_tap(frame, event);
        }
    }

    fn latest_matching_touch<'a>(
        touch_events: &'a [TouchEvent],
        pending: &PendingBackboardBounce,
    ) -> Option<&'a TouchEvent> {
        touch_events
            .iter()
            .filter(|touch| {
                touch.team_is_team_0 == pending.is_team_0
                    && touch.player.as_ref() == Some(&pending.player_id)
            })
            .max_by(|left, right| TouchEvent::timestamp_ordering(left, right))
    }

    fn record_double_tap(&mut self, _frame: &FrameInfo, event: DoubleTapEvent) {
        self.events.push(event);
    }

    /// Returns true when the ball's current trajectory crosses the opponent
    /// goal line within the goal mouth, with a small ball-radius based margin.
    ///
    /// This is a straight-line projection from the sampled post-touch ball
    /// velocity. It deliberately ignores gravity, wall bounces, and later
    /// touches; goal tagging handles the separate question of whether a nearby
    /// goal should receive the double-tap label.
    fn followup_touch_projects_on_goal_mouth(ball: &BallFrameState, is_team_0: bool) -> bool {
        const GOAL_LINE_Y: f32 = 5120.0;
        const GOAL_MOUTH_HEIGHT_Z: f32 = 642.775;
        const GOAL_MOUTH_TRAJECTORY_MARGIN: f32 = BALL_RADIUS_Z * 1.5;

        let Some(ball) = ball.sample() else {
            return false;
        };

        let target_y = if is_team_0 { GOAL_LINE_Y } else { -GOAL_LINE_Y };
        let ball_velocity = ball.velocity();
        if ball_velocity.length_squared() <= f32::EPSILON {
            return false;
        }

        let time_to_goal_line = (target_y - ball.position().y) / ball_velocity.y;
        if !time_to_goal_line.is_finite() || time_to_goal_line < 0.0 {
            return false;
        }

        let projected = ball.position() + ball_velocity * time_to_goal_line;
        projected.x.abs() <= BACK_WALL_GOAL_MOUTH_HALF_WIDTH_X + GOAL_MOUTH_TRAJECTORY_MARGIN
            && projected.z >= BALL_RADIUS_Z - GOAL_MOUTH_TRAJECTORY_MARGIN
            && projected.z <= GOAL_MOUTH_HEIGHT_Z + GOAL_MOUTH_TRAJECTORY_MARGIN
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        touch_state: &TouchState,
        backboard_bounce_state: &BackboardBounceState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        if !live_play_state.is_live_play {
            self.pending_backboard_bounces.clear();
        }

        self.prune_pending_backboard_bounces(frame.time);
        self.record_backboard_bounces(backboard_bounce_state);
        self.resolve_double_tap_touches(frame, ball, &touch_state.touch_events);
        Ok(())
    }
}

#[cfg(test)]
#[path = "double_tap_tests.rs"]
mod tests;
