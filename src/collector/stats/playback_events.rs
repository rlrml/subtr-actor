use super::*;

impl CapturedStatsData<StatsSnapshotFrame> {
    pub(in crate::collector::stats::playback) fn timeline_events(&self) -> Vec<Value> {
        let mut events = self.module_array("core", "timeline");
        events.extend(self.module_array("demo", "timeline"));
        events.sort_by(|left, right| {
            let left_time = left.get("time").and_then(Value::as_f64).unwrap_or(0.0);
            let right_time = right.get("time").and_then(Value::as_f64).unwrap_or(0.0);
            left_time.total_cmp(&right_time)
        });
        events
    }

    pub(in crate::collector::stats::playback) fn timeline_events_typed(
        &self,
    ) -> SubtrActorResult<Vec<TimelineEvent>> {
        self.timeline_events()
            .iter()
            .map(parse_timeline_event)
            .collect()
    }

    pub(in crate::collector::stats::playback) fn goal_tag_events_typed(
        &self,
    ) -> SubtrActorResult<Vec<GoalTagAssignment>> {
        let mut events = Vec::new();
        for module_name in [
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
            "bump_goal",
            "demo_goal",
            "half_volley_goal",
        ] {
            events.extend(self.module_player_events(
                module_name,
                "events",
                parse_goal_tag_event,
            )?);
        }
        events.sort_by(|left, right| {
            left.goal_index.cmp(&right.goal_index).then_with(|| {
                format!("{:?}", left.tag.kind()).cmp(&format!("{:?}", right.tag.kind()))
            })
        });
        Ok(events)
    }

    pub(in crate::collector::stats::playback) fn mechanic_events_typed(
        &self,
    ) -> SubtrActorResult<Vec<StatsTimelineTagEvent>> {
        let mut events = Vec::new();

        for (index, value) in self.module_array("ball_carry", "events").iter().enumerate() {
            events.push(parse_ball_carry_mechanic_event(value, index)?);
        }
        for (index, value) in self
            .module_array("ceiling_shot", "events")
            .iter()
            .enumerate()
        {
            let event = parse_ceiling_shot_event(value)?;
            events.push(span_mechanic_event(
                "ceiling_shot",
                index,
                event.ceiling_contact_frame,
                event.frame,
                event.ceiling_contact_time,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        for (index, value) in self
            .module_array("wall_aerial", "events")
            .iter()
            .enumerate()
        {
            let event = parse_wall_aerial_event(value)?;
            let mut mechanic_event = span_mechanic_event(
                "wall_aerial",
                index,
                event.wall_contact_frame,
                event.frame,
                event.wall_contact_time,
                event.time,
                event.player,
                event.is_team_0,
            );
            mechanic_event.properties = vec![mechanic_event_text_property(
                "wall",
                event.wall.as_label_value(),
            )];
            events.push(mechanic_event);
        }
        for (index, value) in self
            .module_array("wall_aerial_shot", "events")
            .iter()
            .enumerate()
        {
            let event = parse_wall_aerial_shot_event(value)?;
            let mut mechanic_event = span_mechanic_event(
                "wall_aerial_shot",
                index,
                event.wall_contact_frame,
                event.frame,
                event.wall_contact_time,
                event.time,
                event.player,
                event.is_team_0,
            );
            mechanic_event.properties = vec![mechanic_event_text_property(
                "wall",
                event.wall.as_label_value(),
            )];
            events.push(mechanic_event);
        }
        for (index, value) in self.module_array("center", "events").iter().enumerate() {
            let event = parse_center_event(value)?;
            events.push(span_mechanic_event(
                "center",
                index,
                event.start_frame,
                event.frame,
                event.start_time,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        for (index, value) in self
            .module_array("dodge_reset", "on_ball_events")
            .iter()
            .enumerate()
        {
            events.push(parse_dodge_reset_mechanic_event(value, index)?);
        }
        for (index, value) in self.module_array("double_tap", "events").iter().enumerate() {
            let event = parse_double_tap_event(value)?;
            events.push(span_mechanic_event(
                "double_tap",
                index,
                event.backboard_frame,
                event.frame,
                event.backboard_time,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        for (index, value) in self.module_array("flick", "events").iter().enumerate() {
            events.push(parse_flick_mechanic_event(value, index)?);
        }
        for (index, value) in self
            .module_array("musty_flick", "events")
            .iter()
            .enumerate()
        {
            events.push(parse_musty_flick_mechanic_event(value, index)?);
        }
        for (index, value) in self.module_array("one_timer", "events").iter().enumerate() {
            let event = parse_one_timer_event(value)?;
            events.push(span_mechanic_event(
                "one_timer",
                index,
                event.pass_start_frame,
                event.frame,
                event.pass_start_time,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        for (index, value) in self.module_array("pass", "events").iter().enumerate() {
            let event = parse_pass_event(value)?;
            events.push(span_mechanic_event(
                "pass",
                index,
                event.start_frame,
                event.frame,
                event.start_time,
                event.time,
                event.passer,
                event.is_team_0,
            ));
        }
        for (index, value) in self.module_array("speed_flip", "events").iter().enumerate() {
            let event = parse_speed_flip_event(value)?;
            events.push(moment_mechanic_event(
                "speed_flip",
                index,
                event.frame,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        for (index, value) in self.module_array("half_flip", "events").iter().enumerate() {
            let event = parse_half_flip_event(value)?;
            events.push(moment_mechanic_event(
                "half_flip",
                index,
                event.frame,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        for (index, value) in self
            .module_array("half_volley", "events")
            .iter()
            .enumerate()
        {
            let event = parse_half_volley_event(value)?;
            events.push(moment_mechanic_event(
                "half_volley",
                index,
                event.frame,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        for (index, value) in self.module_array("wavedash", "events").iter().enumerate() {
            let event = parse_wavedash_event(value)?;
            events.push(span_mechanic_event(
                "wavedash",
                index,
                event.dodge_frame,
                event.frame,
                event.dodge_time,
                event.time,
                event.player,
                event.is_team_0,
            ));
        }
        events.sort_by(|left, right| {
            let left_time = mechanic_event_start_time(left);
            let right_time = mechanic_event_start_time(right);
            left_time
                .total_cmp(&right_time)
                .then_with(|| left.kind.cmp(&right.kind))
                .then_with(|| left.id.cmp(&right.id))
        });
        Ok(events)
    }

    pub(in crate::collector::stats::playback) fn timeline_event_sets_typed(
        &self,
    ) -> SubtrActorResult<ReplayStatsTimelineEvents> {
        let goal_context =
            self.module_player_events("core", "goal_context", parse_goal_context_event)?;
        let goal_tag_assignments = self.goal_tag_events_typed()?;
        let goal_context = goal_context_events_with_tags(&goal_context, &goal_tag_assignments);

        Ok(ReplayStatsTimelineEvents {
            timeline: self.timeline_events_typed()?,
            core_player: self.module_player_events(
                "core",
                "player_events",
                parse_core_player_scoreboard_event,
            )?,
            core_player_goal_context: self.module_player_events(
                "core",
                "player_goal_context_events",
                parse_core_player_goal_context_event,
            )?,
            possession: self.module_player_events(
                "possession",
                "events",
                parse_possession_event,
            )?,
            pressure: self.module_player_events("pressure", "events", parse_pressure_event)?,
            territorial_pressure: self.module_player_events(
                "territorial_pressure",
                "events",
                parse_territorial_pressure_event,
            )?,
            movement: self.module_player_events("movement", "events", parse_movement_event)?,
            positioning_activity: self.module_player_events(
                "positioning",
                "activity_events",
                parse_positioning_activity_event,
            )?,
            positioning_distance: self.module_player_events(
                "positioning",
                "distance_events",
                parse_positioning_distance_event,
            )?,
            positioning_field_zone: self.module_player_events(
                "positioning",
                "field_zone_events",
                parse_positioning_field_zone_event,
            )?,
            positioning_ball_depth: self.module_player_events(
                "positioning",
                "ball_depth_events",
                parse_positioning_ball_depth_event,
            )?,
            positioning_teammate_role: self.module_player_events(
                "positioning",
                "teammate_role_events",
                parse_positioning_teammate_role_event,
            )?,
            positioning_ball_proximity: self.module_player_events(
                "positioning",
                "ball_proximity_events",
                parse_positioning_ball_proximity_event,
            )?,
            positioning_goal_context: self.module_player_events(
                "positioning",
                "goal_context_events",
                parse_positioning_goal_context_event,
            )?,
            rotation_player: self.module_player_events(
                "rotation",
                "player_events",
                parse_rotation_player_event,
            )?,
            rotation_role_span: self.module_player_events(
                "rotation",
                "role_span_events",
                parse_rotation_role_span_event,
            )?,
            rotation_depth_span: self.module_player_events(
                "rotation",
                "depth_span_events",
                parse_rotation_depth_span_event,
            )?,
            rotation_first_man_stint: self.module_player_events(
                "rotation",
                "first_man_stint_events",
                parse_rotation_first_man_stint_event,
            )?,
            rotation_team: self.module_player_events(
                "rotation",
                "team_events",
                parse_rotation_team_event,
            )?,
            mechanics: self.mechanic_events_typed()?,
            goal_context,
            backboard: self.module_player_events("backboard", "events", parse_backboard_event)?,
            ceiling_shot: self.module_player_events(
                "ceiling_shot",
                "events",
                parse_ceiling_shot_event,
            )?,
            wall_aerial: self.module_player_events(
                "wall_aerial",
                "events",
                parse_wall_aerial_event,
            )?,
            wall_aerial_shot: self.module_player_events(
                "wall_aerial_shot",
                "events",
                parse_wall_aerial_shot_event,
            )?,
            center: self.module_player_events("center", "events", parse_center_event)?,
            flick: self.module_player_events("flick", "events", parse_flick_event)?,
            musty_flick: self.module_player_events(
                "musty_flick",
                "events",
                parse_musty_flick_event,
            )?,
            dodge_reset: self.module_player_events(
                "dodge_reset",
                "events",
                parse_dodge_reset_event,
            )?,
            double_tap: self.module_player_events(
                "double_tap",
                "events",
                parse_double_tap_event,
            )?,
            one_timer: self.module_player_events("one_timer", "events", parse_one_timer_event)?,
            fifty_fifty: self.module_player_events(
                "fifty_fifty",
                "events",
                parse_fifty_fifty_event,
            )?,
            pass: self.module_player_events("pass", "events", parse_pass_event)?,
            ball_carry: self.module_player_events(
                "ball_carry",
                "events",
                parse_ball_carry_event,
            )?,
            controlled_play: self.module_player_events(
                "controlled_play",
                "events",
                parse_controlled_play_event,
            )?,
            rush: self.module_typed_array("rush", "events")?,
            flip_impulse: self.module_player_events(
                "flip_impulse",
                "events",
                parse_flip_impulse_event,
            )?,
            speed_flip: self.module_player_events(
                "speed_flip",
                "events",
                parse_speed_flip_event,
            )?,
            half_flip: self.module_player_events("half_flip", "events", parse_half_flip_event)?,
            half_volley: self.module_player_events(
                "half_volley",
                "events",
                parse_half_volley_event,
            )?,
            wavedash: self.module_player_events("wavedash", "events", parse_wavedash_event)?,
            whiff: self.module_player_events("whiff", "events", parse_whiff_event)?,
            powerslide: self.module_player_events(
                "powerslide",
                "events",
                parse_powerslide_event,
            )?,
            touch: self.module_player_events("touch", "events", parse_touch_stats_event)?,
            touch_ball_movement: self.module_player_events(
                "touch",
                "ball_movement_events",
                parse_touch_ball_movement_event,
            )?,
            boost_pickups: self.module_player_events(
                "boost",
                "events",
                parse_boost_pickup_comparison_event,
            )?,
            boost_ledger: self.module_player_events(
                "boost",
                "ledger_events",
                parse_boost_ledger_event,
            )?,
            boost_state: self.module_player_events(
                "boost",
                "state_events",
                parse_boost_state_event,
            )?,
            bump: self.module_player_events("bump", "events", parse_bump_event)?,
        })
    }

    pub(in crate::collector::stats::playback) fn timeline_event_sets_value(
        &self,
    ) -> SubtrActorResult<Value> {
        let mut events = Map::new();
        events.insert("timeline".to_owned(), Value::Array(self.timeline_events()));
        events.insert(
            "core_player".to_owned(),
            Value::Array(self.module_array("core", "player_events")),
        );
        events.insert(
            "core_player_goal_context".to_owned(),
            Value::Array(self.module_array("core", "player_goal_context_events")),
        );
        events.insert(
            "possession".to_owned(),
            Value::Array(self.module_array("possession", "events")),
        );
        events.insert(
            "pressure".to_owned(),
            Value::Array(self.module_array("pressure", "events")),
        );
        events.insert(
            "territorial_pressure".to_owned(),
            Value::Array(self.module_array("territorial_pressure", "events")),
        );
        events.insert(
            "movement".to_owned(),
            Value::Array(self.module_array("movement", "events")),
        );
        events.insert(
            "positioning_activity".to_owned(),
            Value::Array(self.module_array("positioning", "activity_events")),
        );
        events.insert(
            "positioning_distance".to_owned(),
            Value::Array(self.module_array("positioning", "distance_events")),
        );
        events.insert(
            "positioning_field_zone".to_owned(),
            Value::Array(self.module_array("positioning", "field_zone_events")),
        );
        events.insert(
            "positioning_ball_depth".to_owned(),
            Value::Array(self.module_array("positioning", "ball_depth_events")),
        );
        events.insert(
            "positioning_teammate_role".to_owned(),
            Value::Array(self.module_array("positioning", "teammate_role_events")),
        );
        events.insert(
            "positioning_ball_proximity".to_owned(),
            Value::Array(self.module_array("positioning", "ball_proximity_events")),
        );
        events.insert(
            "positioning_goal_context".to_owned(),
            Value::Array(self.module_array("positioning", "goal_context_events")),
        );
        events.insert(
            "rotation_player".to_owned(),
            Value::Array(self.module_array("rotation", "player_events")),
        );
        events.insert(
            "rotation_team".to_owned(),
            Value::Array(self.module_array("rotation", "team_events")),
        );
        events.insert(
            "mechanics".to_owned(),
            serialize_to_json_value(&self.mechanic_events_typed()?)?,
        );
        events.insert(
            "backboard".to_owned(),
            Value::Array(self.module_array("backboard", "events")),
        );
        events.insert(
            "ceiling_shot".to_owned(),
            Value::Array(self.module_array("ceiling_shot", "events")),
        );
        events.insert(
            "wall_aerial".to_owned(),
            Value::Array(self.module_array("wall_aerial", "events")),
        );
        events.insert(
            "wall_aerial_shot".to_owned(),
            Value::Array(self.module_array("wall_aerial_shot", "events")),
        );
        events.insert(
            "center".to_owned(),
            Value::Array(self.module_array("center", "events")),
        );
        events.insert(
            "double_tap".to_owned(),
            Value::Array(self.module_array("double_tap", "events")),
        );
        events.insert(
            "one_timer".to_owned(),
            Value::Array(self.module_array("one_timer", "events")),
        );
        events.insert(
            "pass".to_owned(),
            Value::Array(self.module_array("pass", "events")),
        );
        events.insert(
            "fifty_fifty".to_owned(),
            Value::Array(self.module_array("fifty_fifty", "events")),
        );
        events.insert(
            "rush".to_owned(),
            Value::Array(self.module_array("rush", "events")),
        );
        events.insert(
            "flip_impulse".to_owned(),
            Value::Array(self.module_array("flip_impulse", "events")),
        );
        events.insert(
            "speed_flip".to_owned(),
            Value::Array(self.module_array("speed_flip", "events")),
        );
        events.insert(
            "half_flip".to_owned(),
            Value::Array(self.module_array("half_flip", "events")),
        );
        events.insert(
            "half_volley".to_owned(),
            Value::Array(self.module_array("half_volley", "events")),
        );
        events.insert(
            "wavedash".to_owned(),
            Value::Array(self.module_array("wavedash", "events")),
        );
        events.insert(
            "whiff".to_owned(),
            Value::Array(self.module_array("whiff", "events")),
        );
        events.insert(
            "touch".to_owned(),
            Value::Array(self.module_array("touch", "events")),
        );
        events.insert(
            "touch_ball_movement".to_owned(),
            Value::Array(self.module_array("touch", "ball_movement_events")),
        );
        events.insert(
            "boost_pickups".to_owned(),
            Value::Array(self.module_array("boost", "events")),
        );
        events.insert(
            "boost_ledger".to_owned(),
            Value::Array(self.module_array("boost", "ledger_events")),
        );
        events.insert(
            "boost_state".to_owned(),
            Value::Array(self.module_array("boost", "state_events")),
        );
        events.insert(
            "bump".to_owned(),
            Value::Array(self.module_array("bump", "events")),
        );
        Ok(Value::Object(events))
    }
}
