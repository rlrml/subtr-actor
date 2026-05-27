use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(crate) fn insert_timeline_player_extra_stats(
        &self,
        player_value: &mut Map<String, Value>,
        frame: &StatsSnapshotFrame,
        player_key: &str,
    ) -> SubtrActorResult<()> {
        player_value.insert(
            "dodge_reset".to_owned(),
            self.frame_player_stat_or_default_by_key::<DodgeResetStats>(
                frame,
                "dodge_reset",
                player_key,
            )?,
        );
        player_value.insert(
            "ball_carry".to_owned(),
            self.frame_player_stat_or_default_by_key::<BallCarryStats>(
                frame,
                "ball_carry",
                player_key,
            )?,
        );
        player_value.insert(
            "air_dribble".to_owned(),
            self.frame_player_stat_or_default_by_key::<AirDribbleStats>(
                frame,
                "air_dribble",
                player_key,
            )?,
        );
        player_value.insert(
            "boost".to_owned(),
            self.frame_player_stat_or_default_by_key::<BoostStats>(frame, "boost", player_key)?,
        );
        player_value.insert(
            "bump".to_owned(),
            self.frame_player_stat_or_default_by_key::<BumpPlayerStats>(frame, "bump", player_key)?,
        );
        player_value.insert(
            "movement".to_owned(),
            self.frame_player_stat_or_value_by_key(
                frame,
                "movement",
                player_key,
                if frame.modules.contains_key("movement") {
                    serialize_to_json_value(
                        &MovementStats::default().with_complete_labeled_tracked_time(),
                    )?
                } else {
                    default_json_value::<MovementStats>()
                },
            )?,
        );
        player_value.insert(
            "positioning".to_owned(),
            self.frame_player_stat_or_default_by_key::<PositioningStats>(
                frame,
                "positioning",
                player_key,
            )?,
        );
        player_value.insert(
            "rotation".to_owned(),
            self.frame_player_stat_or_default_by_key::<RotationPlayerStats>(
                frame, "rotation", player_key,
            )?,
        );
        player_value.insert(
            "powerslide".to_owned(),
            self.frame_player_stat_or_default_by_key::<PowerslideStats>(
                frame,
                "powerslide",
                player_key,
            )?,
        );
        player_value.insert(
            "demo".to_owned(),
            self.frame_player_stat_or_default_by_key::<DemoPlayerStats>(frame, "demo", player_key)?,
        );
        Ok(())
    }
}
