use super::*;

#[path = "playback_frame_timeline_team_aggregate.rs"]
mod playback_frame_timeline_team_aggregate;
#[path = "playback_frame_timeline_team_modules.rs"]
mod playback_frame_timeline_team_modules;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(crate) fn timeline_team_value(
        &self,
        frame: &StatsSnapshotFrame,
        team_key: &str,
    ) -> SubtrActorResult<Value> {
        let is_team_zero = team_key == "team_zero";
        let mut team = Map::new();
        self.insert_timeline_team_aggregate_stats(&mut team, frame, team_key, is_team_zero)?;
        self.insert_timeline_team_module_stats(&mut team, frame, team_key)?;
        Ok(Value::Object(team))
    }
}
