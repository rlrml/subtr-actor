use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(crate) fn frame_stats_or_default_typed<T>(
        &self,
        frame: &StatsSnapshotFrame,
        module_name: &str,
    ) -> SubtrActorResult<T>
    where
        T: Default + DeserializeOwned + Serialize,
    {
        decode_json_value(self.frame_stats_or_default::<T>(frame, module_name))
    }

    pub(crate) fn frame_team_stat_or_default_typed<T>(
        &self,
        frame: &StatsSnapshotFrame,
        module_name: &str,
        team_key: &str,
    ) -> SubtrActorResult<T>
    where
        T: Default + DeserializeOwned + Serialize,
    {
        decode_json_value(self.frame_team_stat_or_default::<T>(frame, module_name, team_key))
    }

    pub(crate) fn frame_player_stat_or_default_typed_by_key<T>(
        &self,
        frame: &StatsSnapshotFrame,
        module_name: &str,
        player_key: &str,
    ) -> SubtrActorResult<T>
    where
        T: Default + DeserializeOwned + Serialize,
    {
        self.frame_player_stat_or_default_with_by_key(frame, module_name, player_key, T::default)
    }

    pub(crate) fn frame_core_player_stat_or_default_by_key(
        &self,
        frame: &StatsSnapshotFrame,
        player_key: &str,
    ) -> SubtrActorResult<CorePlayerStats> {
        decode_core_player_stats_value(self.frame_player_stat_or_value_by_key(
            frame,
            "core",
            player_key,
            default_json_value::<CorePlayerStats>(),
        )?)
    }

    pub(crate) fn frame_player_stat_or_default_with_by_key<T, F>(
        &self,
        frame: &StatsSnapshotFrame,
        module_name: &str,
        player_key: &str,
        default: F,
    ) -> SubtrActorResult<T>
    where
        T: DeserializeOwned + Serialize,
        F: FnOnce() -> T,
    {
        decode_json_value(self.frame_player_stat_or_value_by_key(
            frame,
            module_name,
            player_key,
            serialize_to_json_value(&default())?,
        )?)
    }
}
