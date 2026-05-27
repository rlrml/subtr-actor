use super::*;

impl FlickCalculator {
    pub(super) fn setup_summary(setup: &ActiveFlickSetup) -> FlickSetupSummary {
        FlickSetupSummary {
            is_team_0: setup.is_team_0,
            start_time: setup.start_time,
            start_frame: setup.start_frame,
            last_time: setup.last_time,
            last_frame: setup.last_frame,
            duration: setup.duration,
            average_horizontal_gap: setup.horizontal_gap_integral
                / setup.duration.max(f32::EPSILON),
            average_vertical_gap: setup.vertical_gap_integral / setup.duration.max(f32::EPSILON),
            touch_count: setup.touch_count,
        }
    }

    pub(super) fn setup_qualifies(setup: &FlickSetupSummary) -> bool {
        setup.duration >= FLICK_MIN_SETUP_SECONDS
    }

    pub(super) fn store_recent_setup(&mut self, player_id: PlayerId, setup: FlickSetupSummary) {
        if Self::setup_qualifies(&setup) {
            self.recent_setups.insert(player_id, setup);
        }
    }

    pub(super) fn finish_setup(&mut self, player_id: &PlayerId) {
        let Some(setup) = self.active_setups.remove(player_id) else {
            return;
        };
        self.store_recent_setup(player_id.clone(), Self::setup_summary(&setup));
    }

    pub(super) fn recent_setup_for_player(
        &self,
        player_id: &PlayerId,
        current_time: f32,
    ) -> Option<FlickSetupSummary> {
        if let Some(active) = self.active_setups.get(player_id) {
            return Some(Self::setup_summary(active));
        }

        self.recent_setups
            .get(player_id)
            .filter(|setup| current_time - setup.last_time <= FLICK_MAX_SETUP_STALE_SECONDS)
            .cloned()
    }
}
