use super::*;

fn moment(frame: usize, time: f32) -> EventTiming {
    EventTiming::Moment { frame, time }
}

fn span(start_frame: usize, end_frame: usize, start_time: f32, end_time: f32) -> EventTiming {
    EventTiming::Span {
        start_frame,
        end_frame,
        start_time,
        end_time,
    }
}

#[allow(clippy::too_many_arguments)]
fn make_event(
    stream: &str,
    index: usize,
    timing: EventTiming,
    payload: EventPayload,
    primary_player: Option<PlayerId>,
    secondary_player: Option<PlayerId>,
    team_is_team_0: Option<bool>,
    player_position: Option<[f32; 3]>,
    ball_position: Option<[f32; 3]>,
    confidence: Option<f32>,
) -> Event {
    let frame_id = match timing {
        EventTiming::Moment { frame, .. } => frame.to_string(),
        EventTiming::Span {
            start_frame,
            end_frame,
            ..
        } => format!("{start_frame}:{end_frame}"),
    };
    Event {
        meta: EventMeta {
            id: format!("{stream}:{frame_id}:{index}"),
            stream: stream.to_owned(),
            label: stats_timeline_event_label(stream),
            timing,
            primary_player,
            secondary_player,
            player_position,
            ball_position,
            team_is_team_0,
            confidence,
            properties: Vec::new(),
        },
        payload,
    }
}

fn event_start_time(event: &Event) -> f32 {
    event.meta.timing.start().1
}

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
            "ceiling_shot_goal",
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

    pub(in crate::collector::stats::playback) fn timeline_event_sets_typed(
        &self,
    ) -> SubtrActorResult<ReplayStatsTimelineEvents> {
        let mut events = Vec::new();

        for (index, event) in self.timeline_events_typed()?.into_iter().enumerate() {
            events.push(make_event(
                "timeline",
                index,
                moment(event.frame.unwrap_or_default(), event.time),
                EventPayload::Timeline(event.clone()),
                event.player_id.clone(),
                None,
                event.is_team_0,
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("core", "player_events", parse_core_player_scoreboard_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "core_player",
                index,
                moment(event.frame, event.time),
                EventPayload::CorePlayer(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events(
                "core",
                "player_goal_context_events",
                parse_core_player_goal_context_event,
            )?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "core_player_goal_context",
                index,
                moment(event.frame, event.time),
                EventPayload::CorePlayerGoalContext(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("possession", "events", parse_possession_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "possession",
                index,
                span(event.frame, event.end_frame, event.time, event.end_time),
                EventPayload::Possession(event.clone()),
                event.player_id.clone(),
                None,
                None,
                None,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("pressure", "events", parse_pressure_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "pressure",
                index,
                span(event.frame, event.end_frame, event.time, event.end_time),
                EventPayload::Pressure(event.clone()),
                None,
                None,
                None,
                None,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events(
                "territorial_pressure",
                "events",
                parse_territorial_pressure_event,
            )?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "territorial_pressure",
                index,
                span(
                    event.start_frame,
                    event.end_frame,
                    event.start_time,
                    event.end_time,
                ),
                EventPayload::TerritorialPressure(event.clone()),
                None,
                None,
                Some(event.team_is_team_0),
                None,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("movement", "events", parse_movement_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "movement",
                index,
                span(event.frame, event.end_frame, event.time, event.end_time),
                EventPayload::Movement(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events(
                "positioning",
                "activity_events",
                parse_positioning_activity_event,
            )?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "positioning_activity",
                index,
                span(event.frame, event.end_frame, event.time, event.end_time),
                EventPayload::PositioningActivity(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events(
                "positioning",
                "possession_events",
                parse_positioning_possession_event,
            )?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "positioning_possession",
                index,
                moment(event.frame, event.time),
                EventPayload::PositioningPossession(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events(
                "positioning",
                "field_zone_events",
                parse_positioning_field_zone_event,
            )?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "positioning_field_zone",
                index,
                span(event.frame, event.end_frame, event.time, event.end_time),
                EventPayload::PositioningFieldZone(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events(
                "positioning",
                "ball_depth_events",
                parse_positioning_ball_depth_event,
            )?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "positioning_ball_depth",
                index,
                moment(event.frame, event.time),
                EventPayload::PositioningBallDepth(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events(
                "positioning",
                "teammate_role_events",
                parse_positioning_teammate_role_event,
            )?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "positioning_teammate_role",
                index,
                moment(event.frame, event.time),
                EventPayload::PositioningTeammateRole(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events(
                "positioning",
                "ball_proximity_events",
                parse_positioning_ball_proximity_event,
            )?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "positioning_ball_proximity",
                index,
                moment(event.frame, event.time),
                EventPayload::PositioningBallProximity(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events(
                "positioning",
                "goal_context_events",
                parse_positioning_goal_context_event,
            )?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "positioning_goal_context",
                index,
                moment(event.frame, event.time),
                EventPayload::PositioningGoalContext(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("rotation", "player_events", parse_rotation_player_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "rotation_player",
                index,
                span(event.frame, event.end_frame, event.time, event.end_time),
                EventPayload::RotationPlayer(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events(
                "rotation",
                "role_span_events",
                parse_rotation_role_span_event,
            )?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "rotation_role_span",
                index,
                span(event.frame, event.end_frame, event.time, event.end_time),
                EventPayload::RotationRoleSpan(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events(
                "rotation",
                "depth_span_events",
                parse_rotation_depth_span_event,
            )?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "rotation_depth_span",
                index,
                span(event.frame, event.end_frame, event.time, event.end_time),
                EventPayload::RotationDepthSpan(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events(
                "rotation",
                "first_man_stint_events",
                parse_rotation_first_man_stint_event,
            )?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "rotation_first_man_stint",
                index,
                span(event.frame, event.end_frame, event.time, event.end_time),
                EventPayload::RotationFirstManStint(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("rotation", "team_events", parse_rotation_team_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "rotation_team",
                index,
                moment(event.frame, event.time),
                EventPayload::RotationTeam(event.clone()),
                Some(event.next_first_man.clone()),
                Some(event.previous_first_man.clone()),
                Some(event.is_team_0),
                None,
                None,
                None,
            ));
        }

        let goal_context =
            self.module_player_events("core", "goal_context", parse_goal_context_event)?;
        let goal_tag_assignments = self.goal_tag_events_typed()?;
        let goal_context = goal_context_events_with_tags(&goal_context, &goal_tag_assignments);

        for (index, event) in goal_context.into_iter().enumerate() {
            events.push(make_event(
                "goal_context",
                index,
                moment(event.frame, event.time),
                EventPayload::GoalContext(event.clone()),
                event.scorer.clone(),
                None,
                Some(event.scoring_team_is_team_0),
                None,
                event
                    .ball_position
                    .map(|position| [position.x, position.y, position.z]),
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("backboard", "events", parse_backboard_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "backboard",
                index,
                moment(event.frame, event.time),
                EventPayload::Backboard(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("ball_carry", "events", parse_ball_carry_event)?
            .into_iter()
            .enumerate()
        {
            let stream = match event.kind {
                BallCarryKind::Carry => "ball_carry",
                BallCarryKind::AirDribble => "air_dribble",
            };
            events.push(make_event(
                stream,
                index,
                span(
                    event.start_frame,
                    event.end_frame,
                    event.start_time,
                    event.end_time,
                ),
                EventPayload::BallCarry(event.clone()),
                Some(event.player_id.clone()),
                None,
                Some(event.is_team_0),
                Some(event.end_position),
                Some(event.end_position),
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("ceiling_shot", "events", parse_ceiling_shot_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "ceiling_shot",
                index,
                span(
                    event.ceiling_contact_frame,
                    event.frame,
                    event.ceiling_contact_time,
                    event.time,
                ),
                EventPayload::CeilingShot(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                Some(event.touch_position),
                Some(event.confidence),
            ));
        }

        for (index, event) in self
            .module_player_events("wall_aerial", "events", parse_wall_aerial_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "wall_aerial",
                index,
                span(
                    event.wall_contact_frame,
                    event.frame,
                    event.wall_contact_time,
                    event.time,
                ),
                EventPayload::WallAerial(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                Some(event.player_position),
                Some(event.ball_position),
                Some(event.confidence),
            ));
        }

        for (index, event) in self
            .module_player_events("wall_aerial_shot", "events", parse_wall_aerial_shot_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "wall_aerial_shot",
                index,
                span(
                    event.takeoff_frame,
                    event.frame,
                    event.takeoff_time,
                    event.time,
                ),
                EventPayload::WallAerialShot(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                Some(event.player_position),
                Some(event.ball_position),
                Some(event.confidence),
            ));
        }

        for (index, event) in self
            .module_player_events("center", "events", parse_center_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "center",
                index,
                span(event.start_frame, event.frame, event.start_time, event.time),
                EventPayload::Center(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                Some(event.end_ball_position),
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("dodge_reset", "on_ball_events", parse_dodge_reset_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "dodge_reset",
                index,
                moment(event.frame, event.time),
                EventPayload::DodgeReset(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("double_tap", "events", parse_double_tap_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "double_tap",
                index,
                span(
                    event.backboard_frame,
                    event.frame,
                    event.backboard_time,
                    event.time,
                ),
                EventPayload::DoubleTap(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("one_timer", "events", parse_one_timer_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "one_timer",
                index,
                span(
                    event.pass_start_frame,
                    event.frame,
                    event.pass_start_time,
                    event.time,
                ),
                EventPayload::OneTimer(event.clone()),
                Some(event.player.clone()),
                Some(event.passer.clone()),
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("pass", "events", parse_pass_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "pass",
                index,
                span(event.start_frame, event.frame, event.start_time, event.time),
                EventPayload::Pass(event.clone()),
                Some(event.passer.clone()),
                Some(event.receiver.clone()),
                Some(event.is_team_0),
                event.passer_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("controlled_play", "events", parse_controlled_play_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "controlled_play",
                index,
                span(
                    event.start_frame,
                    event.end_frame,
                    event.start_time,
                    event.end_time,
                ),
                EventPayload::ControlledPlay(event.clone()),
                Some(event.player_id.clone()),
                None,
                Some(event.is_team_0),
                None,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("fifty_fifty", "events", parse_fifty_fifty_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "fifty_fifty",
                index,
                span(
                    event.start_frame,
                    event.resolve_frame,
                    event.start_time,
                    event.resolve_time,
                ),
                EventPayload::FiftyFifty(event.clone()),
                event
                    .team_zero_player
                    .clone()
                    .or(event.team_one_player.clone()),
                None,
                event.winning_team_is_team_0,
                None,
                Some(event.midpoint),
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("kickoff", "events", parse_kickoff_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "kickoff",
                index,
                span(
                    event.start_frame,
                    event.end_frame,
                    event.start_time,
                    event.end_time,
                ),
                EventPayload::Kickoff(Box::new(event.clone())),
                event.first_touch_player.clone(),
                None,
                event.first_touch_team_is_team_0,
                None,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_typed_array::<RushEvent>("rush", "events")?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "rush",
                index,
                span(
                    event.start_frame,
                    event.end_frame,
                    event.start_time,
                    event.end_time,
                ),
                EventPayload::Rush(event.clone()),
                None,
                None,
                Some(event.is_team_0),
                None,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("dodge", "events", parse_dodge_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "dodge",
                index,
                span(
                    event.frame,
                    event.resolved_frame,
                    event.time,
                    event.resolved_time,
                ),
                EventPayload::Dodge(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event
                    .dodge_impulse
                    .as_ref()
                    .map(|dodge_impulse| dodge_impulse.end_position),
                None,
                event
                    .dodge_impulse
                    .as_ref()
                    .map(|dodge_impulse| dodge_impulse.confidence),
            ));
        }

        for (index, event) in self
            .module_player_events("speed_flip", "events", parse_speed_flip_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "speed_flip",
                index,
                span(
                    event.frame,
                    event.resolved_frame,
                    event.time,
                    event.resolved_time,
                ),
                EventPayload::SpeedFlip(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                Some(event.end_position),
                None,
                Some(event.confidence),
            ));
        }

        for (index, event) in self
            .module_player_events("half_flip", "events", parse_half_flip_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "half_flip",
                index,
                moment(event.frame, event.time),
                EventPayload::HalfFlip(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                Some(event.end_position),
                None,
                Some(event.confidence),
            ));
        }

        for (index, event) in self
            .module_player_events("half_volley", "events", parse_half_volley_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "half_volley",
                index,
                moment(event.frame, event.time),
                EventPayload::HalfVolley(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("wavedash", "events", parse_wavedash_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "wavedash",
                index,
                span(event.dodge_frame, event.frame, event.dodge_time, event.time),
                EventPayload::Wavedash(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                Some(event.landing_position),
                None,
                Some(event.confidence),
            ));
        }

        for (index, event) in self
            .module_player_events("whiff", "events", parse_whiff_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "whiff",
                index,
                span(
                    event.frame,
                    event.resolved_frame,
                    event.time,
                    event.resolved_time,
                ),
                EventPayload::Whiff(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("powerslide", "events", parse_powerslide_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "powerslide",
                index,
                moment(event.frame, event.time),
                EventPayload::Powerslide(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("touch", "events", parse_touch_stats_event)?
            .into_iter()
            .enumerate()
        {
            let timing =
                event
                    .ball_movement
                    .as_ref()
                    .map_or(moment(event.frame, event.time), |movement| {
                        span(
                            movement.start_frame,
                            movement.end_frame,
                            movement.start_time,
                            movement.end_time,
                        )
                    });
            events.push(make_event(
                "touch",
                index,
                timing,
                EventPayload::Touch(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("boost", "events", parse_boost_pickup_comparison_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "boost_pickups",
                index,
                moment(event.frame, event.time),
                EventPayload::BoostPickup(event.clone()),
                Some(event.player_id.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("boost", "ledger_events", parse_boost_ledger_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "boost_ledger",
                index,
                span(event.frame, event.end_frame, event.time, event.end_time),
                EventPayload::BoostLedger(event.clone()),
                Some(event.player_id.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("boost", "bucket_events", parse_boost_bucket_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "boost_bucket",
                index,
                span(event.frame, event.end_frame, event.time, event.end_time),
                EventPayload::BoostBucket(event.clone()),
                Some(event.player_id.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("boost", "state_events", parse_boost_state_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "boost_state",
                index,
                span(event.frame, event.end_frame, event.time, event.end_time),
                EventPayload::BoostState(event.clone()),
                Some(event.player_id.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                None,
            ));
        }

        for (index, event) in self
            .module_player_events("bump", "events", parse_bump_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "bump",
                index,
                moment(event.frame, event.time),
                EventPayload::Bump(event.clone()),
                Some(event.initiator.clone()),
                Some(event.victim.clone()),
                Some(event.initiator_is_team_0),
                Some(event.initiator_position),
                None,
                Some(event.confidence),
            ));
        }

        for (index, event) in self
            .module_player_events("flick", "events", parse_flick_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "flick",
                index,
                span(
                    event.setup_start_frame,
                    event.frame,
                    event.setup_start_time,
                    event.time,
                ),
                EventPayload::Flick(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                Some(event.confidence),
            ));
        }

        for (index, event) in self
            .module_player_events("musty_flick", "events", parse_musty_flick_event)?
            .into_iter()
            .enumerate()
        {
            events.push(make_event(
                "musty_flick",
                index,
                span(event.dodge_frame, event.frame, event.dodge_time, event.time),
                EventPayload::MustyFlick(event.clone()),
                Some(event.player.clone()),
                None,
                Some(event.is_team_0),
                event.player_position,
                None,
                Some(event.confidence),
            ));
        }

        events.sort_by(|left, right| {
            event_start_time(left)
                .total_cmp(&event_start_time(right))
                .then_with(|| left.meta.stream.cmp(&right.meta.stream))
                .then_with(|| left.meta.id.cmp(&right.meta.id))
        });

        Ok(ReplayStatsTimelineEvents { events })
    }

    pub(in crate::collector::stats::playback) fn timeline_event_sets_value(
        &self,
    ) -> SubtrActorResult<Value> {
        serialize_to_json_value(&self.timeline_event_sets_typed()?)
    }
}
