use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(crate) fn insert_timeline_player_identity(
        &self,
        player_value: &mut Map<String, Value>,
        player: &PlayerInfo,
    ) -> SubtrActorResult<()> {
        player_value.insert(
            "player_id".to_owned(),
            serialize_to_json_value(&player.remote_id)?,
        );
        player_value.insert("name".to_owned(), serialize_to_json_value(&player.name)?);
        player_value.insert(
            "is_team_0".to_owned(),
            serialize_to_json_value(
                &self
                    .replay_meta
                    .team_zero
                    .iter()
                    .any(|team_player| team_player.remote_id == player.remote_id),
            )?,
        );
        Ok(())
    }
}
