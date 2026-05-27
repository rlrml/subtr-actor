use super::playback_config_helpers::*;
use super::playback_config_value_goals::insert_goal_config_values;
use super::playback_config_value_groups::{
    insert_positioning_config_values, insert_pressure_config_values, insert_rotation_config_values,
};
use super::playback_config_value_rush::insert_rush_config_values;
use super::playback_config_value_territorial::insert_territorial_pressure_config_values;
use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(in crate::collector::stats::playback) fn timeline_config_value(
        &self,
    ) -> SubtrActorResult<Value> {
        let mut config = Map::new();
        insert_positioning_config_values(&mut config, module_config(&self.config, "positioning"))?;
        insert_pressure_config_values(&mut config, module_config(&self.config, "pressure"))?;
        insert_territorial_pressure_config_values(
            &mut config,
            module_config(&self.config, "territorial_pressure"),
        )?;
        insert_rotation_config_values(&mut config, module_config(&self.config, "rotation"))?;
        insert_rush_config_values(&mut config, module_config(&self.config, "rush"))?;
        insert_goal_config_values(&mut config, &self.config)?;
        Ok(Value::Object(config))
    }
}
