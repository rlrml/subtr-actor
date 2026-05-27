use super::*;

impl TouchCalculator {
    pub(crate) fn record_last_touch(&mut self, frame: &FrameInfo, touch_events: &[TouchEvent]) {
        let Some(last_touch) = touch_events.last() else {
            return;
        };
        self.last_touch_events.push(TouchLastTouchEvent {
            time: last_touch.time,
            frame: last_touch.frame,
            sample_time: frame.time,
            sample_frame: frame.frame_number,
            is_team_0: last_touch.team_is_team_0,
            player: last_touch.player.clone(),
        });
        self.current_last_touch_player = last_touch.player.clone();
    }

    pub(crate) fn mark_current_last_touch(&mut self) {
        if let Some(player_id) = self.current_last_touch_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_touch = true;
            }
        }
    }
}
