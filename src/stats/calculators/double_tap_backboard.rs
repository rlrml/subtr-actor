use super::*;

impl DoubleTapCalculator {
    pub(super) fn prune_pending_backboard_bounces(&mut self, current_time: f32) {
        self.pending_backboard_bounces
            .retain(|entry| current_time - entry.time <= DOUBLE_TAP_TOUCH_WINDOW_SECONDS);
    }

    pub(super) fn record_backboard_bounces(&mut self, state: &BackboardBounceState) {
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
}
