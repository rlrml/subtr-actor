use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(super) fn timeline_events(&self) -> Vec<Value> {
        let mut events = self.module_array("core", "timeline");
        events.extend(self.module_array("demo", "timeline"));
        events.sort_by(|left, right| {
            let left_time = left.get("time").and_then(Value::as_f64).unwrap_or(0.0);
            let right_time = right.get("time").and_then(Value::as_f64).unwrap_or(0.0);
            left_time.total_cmp(&right_time)
        });
        events
    }

    pub(super) fn timeline_events_typed(&self) -> SubtrActorResult<Vec<TimelineEvent>> {
        self.timeline_events()
            .iter()
            .map(parse_timeline_event)
            .collect()
    }
}
