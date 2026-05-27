use super::*;

impl FlickCalculator {
    pub(super) fn track_dodge_starts(&mut self, frame: &FrameInfo, players: &PlayerFrameState) {
        for player in &players.players {
            let was_dodge_active = self
                .previous_dodge_active
                .insert(player.player_id.clone(), player.dodge_active)
                .unwrap_or(false);
            if !player.dodge_active || was_dodge_active {
                continue;
            }

            let Some(setup) = self.recent_setup_for_player(&player.player_id, frame.time) else {
                continue;
            };
            if !Self::setup_qualifies(&setup) {
                continue;
            }
            if frame.time - setup.last_time > FLICK_MAX_CONTROL_TO_DODGE_SECONDS {
                continue;
            }

            self.recent_dodge_starts.insert(
                player.player_id.clone(),
                RecentDodgeStart {
                    time: frame.time,
                    frame: frame.frame_number,
                    setup,
                },
            );
        }
    }

    pub(super) fn prune_recent_state(&mut self, current_time: f32) {
        self.recent_setups
            .retain(|_, setup| current_time - setup.last_time <= FLICK_MAX_SETUP_STALE_SECONDS);
        self.recent_dodge_starts
            .retain(|_, dodge| current_time - dodge.time <= FLICK_MAX_DODGE_TO_TOUCH_SECONDS);
    }
}
