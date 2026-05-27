use super::*;

pub(crate) fn serialize_live_event_history(engine: &SaEngine) -> Vec<u8> {
    let active_demos: Vec<_> = engine
        .live_events
        .active_demos
        .iter()
        .map(|active_demo| {
            serde_json::json!({
                "attacker": &active_demo.sample.attacker,
                "victim": &active_demo.sample.victim,
            })
        })
        .collect();
    serde_json::to_vec(&serde_json::json!({
        "active_demos": active_demos,
        "demo_events": &engine.live_event_history.demo_events,
        "boost_pad_events": &engine.live_event_history.boost_pad_events,
        "touch_events": &engine.live_event_history.touch_events,
        "dodge_refreshed_events": &engine.live_event_history.dodge_refreshed_events,
        "player_stat_events": &engine.live_event_history.player_stat_events,
        "goal_events": &engine.live_event_history.goal_events,
    }))
    .unwrap_or_default()
}
