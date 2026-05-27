use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(crate) fn insert_timeline_player_mechanic_stats(
        &self,
        player_value: &mut Map<String, Value>,
        frame: &StatsSnapshotFrame,
        player_key: &str,
    ) -> SubtrActorResult<()> {
        player_value.insert(
            "fifty_fifty".to_owned(),
            self.frame_player_stat_or_default_by_key::<FiftyFiftyPlayerStats>(
                frame,
                "fifty_fifty",
                player_key,
            )?,
        );
        player_value.insert(
            "speed_flip".to_owned(),
            self.frame_player_stat_or_default_by_key::<SpeedFlipStats>(
                frame,
                "speed_flip",
                player_key,
            )?,
        );
        player_value.insert(
            "half_flip".to_owned(),
            self.frame_player_stat_or_default_by_key::<HalfFlipStats>(
                frame,
                "half_flip",
                player_key,
            )?,
        );
        player_value.insert(
            "half_volley".to_owned(),
            self.frame_player_stat_or_default_by_key::<HalfVolleyPlayerStats>(
                frame,
                "half_volley",
                player_key,
            )?,
        );
        player_value.insert(
            "wavedash".to_owned(),
            self.frame_player_stat_or_default_by_key::<WavedashStats>(
                frame, "wavedash", player_key,
            )?,
        );
        player_value.insert(
            "touch".to_owned(),
            self.frame_player_stat_or_value_by_key(
                frame,
                "touch",
                player_key,
                if frame.modules.contains_key("touch") {
                    serialize_to_json_value(
                        &TouchStats::default().with_complete_labeled_touch_counts(),
                    )?
                } else {
                    default_json_value::<TouchStats>()
                },
            )?,
        );
        player_value.insert(
            "whiff".to_owned(),
            self.frame_player_stat_or_default_by_key::<WhiffStats>(frame, "whiff", player_key)?,
        );
        player_value.insert(
            "flick".to_owned(),
            self.frame_player_stat_or_default_by_key::<FlickStats>(frame, "flick", player_key)?,
        );
        player_value.insert(
            "musty_flick".to_owned(),
            self.frame_player_stat_or_default_by_key::<MustyFlickStats>(
                frame,
                "musty_flick",
                player_key,
            )?,
        );
        Ok(())
    }
}
