use super::*;

impl RushCalculator {
    pub(super) fn record_active_rush(&mut self, active_rush: &mut ActiveRush) {
        if active_rush.counted {
            return;
        }
        if active_rush.retained_possession_time() < self.config.min_possession_retained_seconds {
            return;
        }

        self.stats.record(&RushEvent {
            start_time: active_rush.start_time,
            start_frame: active_rush.start_frame,
            end_time: active_rush.last_time,
            end_frame: active_rush.last_frame,
            is_team_0: active_rush.is_team_0,
            attackers: active_rush.attackers,
            defenders: active_rush.defenders,
        });
        active_rush.counted = true;
    }

    pub(super) fn finalize_active_rush(&mut self) {
        let Some(mut active_rush) = self.active_rush.take() else {
            return;
        };
        self.record_active_rush(&mut active_rush);
        if !active_rush.counted {
            return;
        }
        self.events.push(RushEvent {
            start_time: active_rush.start_time,
            start_frame: active_rush.start_frame,
            end_time: active_rush.last_time,
            end_frame: active_rush.last_frame,
            is_team_0: active_rush.is_team_0,
            attackers: active_rush.attackers,
            defenders: active_rush.defenders,
        });
    }
}
