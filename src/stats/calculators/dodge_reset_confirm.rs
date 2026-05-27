use super::*;

impl DodgeResetCalculator {
    pub(super) fn apply_confirmed_flip_reset_touch(
        &mut self,
        players: &PlayerFrameState,
        touch_event: &TouchEvent,
    ) {
        let Some(player_id) = touch_event.player.as_ref() else {
            return;
        };
        if !self.pending_reset_dodge_started.contains(player_id)
            || !Self::player_dodge_active(players, player_id)
        {
            return;
        }

        let Some(reset_event) = self.pending_on_ball_resets.get(player_id).cloned() else {
            return;
        };
        let time_since_reset = touch_event.time - reset_event.time;
        if !reset_delay_is_valid(time_since_reset) {
            self.clear_stale_pending_reset(player_id, time_since_reset);
            return;
        }

        self.confirmed_flip_reset_events.push(confirmed_event(
            touch_event,
            &reset_event,
            time_since_reset,
        ));
        self.pending_on_ball_resets.remove(player_id);
        self.pending_reset_dodge_started.remove(player_id);
    }

    fn clear_stale_pending_reset(&mut self, player_id: &PlayerId, time_since_reset: f32) {
        if time_since_reset > FLIP_RESET_MAX_DODGE_TOUCH_DELAY_SECONDS {
            self.pending_on_ball_resets.remove(player_id);
            self.pending_reset_dodge_started.remove(player_id);
        }
    }
}

fn reset_delay_is_valid(time_since_reset: f32) -> bool {
    (FLIP_RESET_MIN_DODGE_TOUCH_DELAY_SECONDS..=FLIP_RESET_MAX_DODGE_TOUCH_DELAY_SECONDS)
        .contains(&time_since_reset)
}

fn confirmed_event(
    touch_event: &TouchEvent,
    reset_event: &DodgeRefreshedEvent,
    time_since_reset: f32,
) -> ConfirmedFlipResetEvent {
    ConfirmedFlipResetEvent {
        time: touch_event.time,
        frame: touch_event.frame,
        reset_time: reset_event.time,
        reset_frame: reset_event.frame,
        player: touch_event.player.clone().expect("checked by caller"),
        is_team_0: touch_event.team_is_team_0,
        counter_value: reset_event.counter_value,
        time_since_reset,
    }
}
