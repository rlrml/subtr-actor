use super::*;

const EMPTY_GOAL_TAG_MODULES: &[&str] = &[
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

pub(super) fn empty_goal_tag_snapshot_frame_json(
    module_name: &str,
    _graph: &AnalysisGraph,
    _replay_meta: &ReplayMeta,
) -> SubtrActorResult<Option<Value>> {
    if EMPTY_GOAL_TAG_MODULES.contains(&module_name) {
        serialize_to_json_value(&serde_json::json!({})).map(Some)
    } else {
        Ok(None)
    }
}
