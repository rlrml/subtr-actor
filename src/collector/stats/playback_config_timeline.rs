use super::playback_config_timeline_goals::apply_goal_timeline_config;
use super::playback_config_timeline_groups::{
    apply_positioning_timeline_config, apply_pressure_timeline_config,
    apply_rotation_timeline_config, apply_territorial_pressure_timeline_config,
};
use super::playback_config_timeline_rush::apply_rush_timeline_config;
use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(in crate::collector::stats::playback) fn timeline_config(&self) -> StatsTimelineConfig {
        let mut config = default_stats_timeline_config();
        apply_positioning_timeline_config(&mut config, &self.config);
        apply_pressure_timeline_config(&mut config, &self.config);
        apply_territorial_pressure_timeline_config(&mut config, &self.config);
        apply_rotation_timeline_config(&mut config, &self.config);
        apply_rush_timeline_config(&mut config, &self.config);
        apply_goal_timeline_config(&mut config, &self.config);
        config
    }
}
