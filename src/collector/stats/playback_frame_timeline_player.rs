use super::*;

#[path = "playback_frame_timeline_player_core.rs"]
mod playback_frame_timeline_player_core;
#[path = "playback_frame_timeline_player_extra.rs"]
mod playback_frame_timeline_player_extra;
#[path = "playback_frame_timeline_player_identity.rs"]
mod playback_frame_timeline_player_identity;
#[path = "playback_frame_timeline_player_mechanics.rs"]
mod playback_frame_timeline_player_mechanics;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(crate) fn timeline_player_value(
        &self,
        frame: &StatsSnapshotFrame,
        player: &PlayerInfo,
    ) -> SubtrActorResult<Value> {
        let player_key = player_info_key(player)?;
        let mut player_value = Map::new();
        self.insert_timeline_player_identity(&mut player_value, player)?;
        self.insert_timeline_player_core_stats(&mut player_value, frame, &player_key)?;
        self.insert_timeline_player_mechanic_stats(&mut player_value, frame, &player_key)?;
        self.insert_timeline_player_extra_stats(&mut player_value, frame, &player_key)?;
        Ok(Value::Object(player_value))
    }
}
