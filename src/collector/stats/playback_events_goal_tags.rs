use super::*;

const GOAL_TAG_MODULES: &[&str] = &[
    "aerial_goal",
    "high_aerial_goal",
    "long_distance_goal",
    "own_half_goal",
    "empty_net_goal",
    "counter_attack_goal",
    "flick_goal",
    "double_tap_goal",
    "one_timer_goal",
    "passing_goal",
    "air_dribble_goal",
    "flip_reset_goal",
    "half_volley_goal",
];

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(super) fn goal_tag_events_typed(&self) -> SubtrActorResult<Vec<GoalTagEvent>> {
        let mut events = Vec::new();
        for module_name in GOAL_TAG_MODULES {
            events.extend(self.module_player_events(
                module_name,
                "events",
                parse_goal_tag_event,
            )?);
        }
        events.sort_by(|left, right| {
            left.time
                .total_cmp(&right.time)
                .then_with(|| left.frame.cmp(&right.frame))
                .then_with(|| left.goal_index.cmp(&right.goal_index))
                .then_with(|| format!("{:?}", left.kind).cmp(&format!("{:?}", right.kind)))
        });
        Ok(events)
    }

    pub(super) fn goal_tag_events_value(&self) -> Vec<Value> {
        let mut events = Vec::new();
        for module_name in GOAL_TAG_MODULES {
            events.extend(self.module_array(module_name, "events"));
        }
        events.sort_by(|left, right| {
            let left_time = left.get("time").and_then(Value::as_f64).unwrap_or(0.0);
            let right_time = right.get("time").and_then(Value::as_f64).unwrap_or(0.0);
            left_time.total_cmp(&right_time)
        });
        events
    }
}
