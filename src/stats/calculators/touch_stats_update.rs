use super::*;

impl TouchCalculator {
    pub(crate) fn apply_touch_stats(
        &mut self,
        frame: &FrameInfo,
        touch_event: &TouchEvent,
        player_id: &PlayerId,
        classification: TouchClassification,
        ball_speed_change: f32,
    ) {
        let stats = self.player_stats.entry(player_id.clone()).or_default();
        stats.touch_count += 1;
        Self::apply_touch_classification(stats, classification);
        stats.last_touch_time = Some(touch_event.time);
        stats.last_touch_frame = Some(touch_event.frame);
        stats.time_since_last_touch = Some((frame.time - touch_event.time).max(0.0));
        stats.frames_since_last_touch = Some(frame.frame_number.saturating_sub(touch_event.frame));
        stats.last_ball_speed_change = Some(ball_speed_change);
        stats.max_ball_speed_change = stats.max_ball_speed_change.max(ball_speed_change);
        stats.cumulative_ball_speed_change += ball_speed_change;
    }
}
