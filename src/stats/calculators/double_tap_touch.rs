use super::*;

impl DoubleTapCalculator {
    pub(super) fn resolve_double_tap_touches(
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

            let matching_touch = touch_events.iter().any(|touch| {
                touch.team_is_team_0 == pending.is_team_0
                    && touch.player.as_ref() == Some(&pending.player_id)
            });
            let conflicting_touch = touch_events
                .iter()
                .any(|touch| touch.player.as_ref() != Some(&pending.player_id));

            if matching_touch
                && !conflicting_touch
                && Self::followup_touch_is_goal_directed(ball, pending.is_team_0)
            {
                completed_events.push(DoubleTapEvent {
                    time: frame.time,
                    frame: frame.frame_number,
                    player: pending.player_id.clone(),
                    is_team_0: pending.is_team_0,
                    backboard_time: pending.time,
                    backboard_frame: pending.frame,
                });
            }
            false
        });

        for event in completed_events {
            self.record_double_tap(frame, event);
        }
    }

    fn record_double_tap(&mut self, frame: &FrameInfo, event: DoubleTapEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.count += 1;
        stats.last_double_tap_time = Some(event.time);
        stats.last_double_tap_frame = Some(event.frame);
        stats.time_since_last_double_tap = Some((frame.time - event.time).max(0.0));
        stats.frames_since_last_double_tap = Some(frame.frame_number.saturating_sub(event.frame));

        let team_stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        team_stats.count += 1;
        self.current_last_double_tap_player = Some(event.player.clone());
        self.events.push(event);
    }

    fn followup_touch_is_goal_directed(ball: &BallFrameState, is_team_0: bool) -> bool {
        const GOAL_CENTER_Y: f32 = 5120.0;
        const MIN_GOAL_ALIGNMENT_COSINE: f32 = 0.6;

        let Some(ball) = ball.sample() else {
            return false;
        };

        let target_y = if is_team_0 {
            GOAL_CENTER_Y
        } else {
            -GOAL_CENTER_Y
        };
        let ball_velocity = ball.velocity();
        if ball_velocity.length_squared() <= f32::EPSILON {
            return false;
        }

        let goal_direction = glam::Vec3::new(0.0, target_y, ball.position().z) - ball.position();
        goal_direction
            .normalize_or_zero()
            .dot(ball_velocity.normalize_or_zero())
            >= MIN_GOAL_ALIGNMENT_COSINE
    }
}
