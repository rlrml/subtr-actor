use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(in crate::collector::stats::playback) fn timeline_event_sets_value(
        &self,
    ) -> SubtrActorResult<Value> {
        let mut events = Map::new();
        events.insert("timeline".to_owned(), Value::Array(self.timeline_events()));
        self.insert_module_event_array(&mut events, "core_player", "core", "player_events");
        self.insert_module_event_array(&mut events, "core_team", "core", "team_events");
        self.insert_module_event_array(&mut events, "possession", "possession", "events");
        self.insert_module_event_array(&mut events, "pressure", "pressure", "events");
        self.insert_module_event_array(
            &mut events,
            "territorial_pressure",
            "territorial_pressure",
            "events",
        );
        self.insert_module_event_array(&mut events, "movement", "movement", "events");
        self.insert_module_event_array(&mut events, "positioning", "positioning", "events");
        self.insert_module_event_array(&mut events, "rotation_player", "rotation", "player_events");
        self.insert_module_event_array(&mut events, "rotation_team", "rotation", "team_events");
        events.insert(
            "mechanics".to_owned(),
            serialize_to_json_value(&self.mechanic_events_typed()?)?,
        );
        self.insert_module_event_array(&mut events, "backboard", "backboard", "events");
        self.insert_module_event_array(&mut events, "ceiling_shot", "ceiling_shot", "events");
        self.insert_module_event_array(&mut events, "wall_aerial", "wall_aerial", "events");
        self.insert_module_event_array(
            &mut events,
            "wall_aerial_shot",
            "wall_aerial_shot",
            "events",
        );
        self.insert_module_event_array(&mut events, "center", "center", "events");
        self.insert_module_event_array(&mut events, "double_tap", "double_tap", "events");
        self.insert_module_event_array(&mut events, "one_timer", "one_timer", "events");
        self.insert_module_event_array(&mut events, "pass", "pass", "events");
        events.insert(
            "goal_tags".to_owned(),
            Value::Array(self.goal_tag_events_value()),
        );
        self.insert_module_event_array(&mut events, "fifty_fifty", "fifty_fifty", "events");
        self.insert_module_event_array(&mut events, "rush", "rush", "events");
        self.insert_module_event_array(&mut events, "speed_flip", "speed_flip", "events");
        self.insert_module_event_array(&mut events, "half_flip", "half_flip", "events");
        self.insert_module_event_array(&mut events, "half_volley", "half_volley", "events");
        self.insert_module_event_array(&mut events, "wavedash", "wavedash", "events");
        self.insert_module_event_array(&mut events, "whiff", "whiff", "events");
        self.insert_module_event_array(&mut events, "touch", "touch", "events");
        self.insert_module_event_array(
            &mut events,
            "touch_ball_movement",
            "touch",
            "ball_movement_events",
        );
        self.insert_module_event_array(
            &mut events,
            "touch_last_touch",
            "touch",
            "last_touch_events",
        );
        self.insert_module_event_array(&mut events, "boost_pickups", "boost", "events");
        self.insert_module_event_array(&mut events, "boost_ledger", "boost", "ledger_events");
        self.insert_module_event_array(&mut events, "boost_state", "boost", "state_events");
        self.insert_module_event_array(&mut events, "bump", "bump", "events");
        Ok(Value::Object(events))
    }

    fn insert_module_event_array(
        &self,
        events: &mut Map<String, Value>,
        key: &str,
        module_name: &str,
        field: &str,
    ) {
        events.insert(
            key.to_owned(),
            Value::Array(self.module_array(module_name, field)),
        );
    }
}
