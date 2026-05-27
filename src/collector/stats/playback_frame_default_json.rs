use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(crate) fn frame_stats_or_default<T>(
        &self,
        frame: &StatsSnapshotFrame,
        module_name: &str,
    ) -> Value
    where
        T: Default + Serialize,
    {
        frame
            .modules
            .get(module_name)
            .and_then(Value::as_object)
            .and_then(|module| module.get("stats"))
            .cloned()
            .unwrap_or_else(|| default_json_value::<T>())
    }

    pub(crate) fn frame_team_stat_or_default<T>(
        &self,
        frame: &StatsSnapshotFrame,
        module_name: &str,
        team_key: &str,
    ) -> Value
    where
        T: Default + Serialize,
    {
        frame
            .modules
            .get(module_name)
            .and_then(Value::as_object)
            .and_then(|module| module.get(team_key))
            .cloned()
            .unwrap_or_else(|| default_json_value::<T>())
    }

    pub(crate) fn frame_player_stat_or_default_by_key<T>(
        &self,
        frame: &StatsSnapshotFrame,
        module_name: &str,
        player_key: &str,
    ) -> SubtrActorResult<Value>
    where
        T: Default + Serialize,
    {
        self.frame_player_stat_or_value_by_key(
            frame,
            module_name,
            player_key,
            default_json_value::<T>(),
        )
    }

    pub(crate) fn frame_player_stat_or_value_by_key(
        &self,
        frame: &StatsSnapshotFrame,
        module_name: &str,
        player_key: &str,
        default_value: Value,
    ) -> SubtrActorResult<Value> {
        Ok(
            player_stats_value_for_key(frame.modules.get(module_name), player_key)?
                .cloned()
                .unwrap_or(default_value),
        )
    }
}
