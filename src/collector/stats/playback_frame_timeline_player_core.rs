use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(crate) fn insert_timeline_player_core_stats(
        &self,
        player_value: &mut Map<String, Value>,
        frame: &StatsSnapshotFrame,
        player_key: &str,
    ) -> SubtrActorResult<()> {
        player_value.insert(
            "core".to_owned(),
            self.frame_player_stat_or_default_by_key::<CorePlayerStats>(frame, "core", player_key)?,
        );
        player_value.insert(
            "backboard".to_owned(),
            self.frame_player_stat_or_default_by_key::<BackboardPlayerStats>(
                frame,
                "backboard",
                player_key,
            )?,
        );
        player_value.insert(
            "ceiling_shot".to_owned(),
            self.frame_player_stat_or_default_by_key::<CeilingShotStats>(
                frame,
                "ceiling_shot",
                player_key,
            )?,
        );
        player_value.insert(
            "wall_aerial".to_owned(),
            self.frame_player_stat_or_default_by_key::<WallAerialStats>(
                frame,
                "wall_aerial",
                player_key,
            )?,
        );
        player_value.insert(
            "wall_aerial_shot".to_owned(),
            self.frame_player_stat_or_default_by_key::<WallAerialShotStats>(
                frame,
                "wall_aerial_shot",
                player_key,
            )?,
        );
        player_value.insert(
            "double_tap".to_owned(),
            self.frame_player_stat_or_default_by_key::<DoubleTapPlayerStats>(
                frame,
                "double_tap",
                player_key,
            )?,
        );
        player_value.insert(
            "one_timer".to_owned(),
            self.frame_player_stat_or_default_by_key::<OneTimerPlayerStats>(
                frame,
                "one_timer",
                player_key,
            )?,
        );
        player_value.insert(
            "pass".to_owned(),
            self.frame_player_stat_or_default_by_key::<PassPlayerStats>(frame, "pass", player_key)?,
        );
        Ok(())
    }
}
