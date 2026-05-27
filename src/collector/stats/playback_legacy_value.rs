use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub fn into_legacy_stats_timeline_value(self) -> SubtrActorResult<Value> {
        self.to_legacy_stats_timeline_value()
    }

    #[deprecated(
        note = "use into_legacy_stats_timeline_value for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn into_stats_timeline_value(self) -> SubtrActorResult<Value> {
        self.into_legacy_stats_timeline_value()
    }

    pub fn to_legacy_stats_timeline_value(&self) -> SubtrActorResult<Value> {
        let mut timeline = Map::new();
        timeline.insert("config".to_owned(), self.timeline_config_value()?);
        timeline.insert(
            "replay_meta".to_owned(),
            serialize_to_json_value(&self.replay_meta)?,
        );
        timeline.insert("events".to_owned(), self.timeline_event_sets_value()?);
        timeline.insert(
            "frames".to_owned(),
            Value::Array(
                self.frames
                    .iter()
                    .map(|frame| self.timeline_frame_value(frame))
                    .collect::<SubtrActorResult<Vec<_>>>()?,
            ),
        );
        Ok(Value::Object(timeline))
    }

    #[deprecated(
        note = "use to_legacy_stats_timeline_value for full partial-sum snapshots, or StatsTimelineEventCollector for compact event-backed timelines"
    )]
    pub fn to_stats_timeline_value(&self) -> SubtrActorResult<Value> {
        self.to_legacy_stats_timeline_value()
    }
}
