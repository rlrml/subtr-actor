use super::playback_json::*;
use super::*;

pub(in crate::collector::stats::playback) fn parse_dodge_reset_event(
    value: &Value,
) -> SubtrActorResult<DodgeResetEvent> {
    let object = json_object(value, "dodge reset event")?;
    Ok(DodgeResetEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        player_position: json_optional_vec3(object.get("player_position"))?,
        counter_value: json_required_i32(object, "counter_value")?,
        on_ball: json_required_bool(object, "on_ball")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_powerslide_event(
    value: &Value,
) -> SubtrActorResult<PowerslideEvent> {
    let object = json_object(value, "powerslide event")?;
    Ok(PowerslideEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        player_position: json_optional_vec3(object.get("player_position"))?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        active: json_required_bool(object, "active")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_core_player_stats_event(
    value: &Value,
) -> SubtrActorResult<CorePlayerStatsEvent> {
    let object = json_object(value, "core player stats event")?;
    Ok(CorePlayerStatsEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        player_position: json_optional_vec3(object.get("player_position"))?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        delta: decode_json_value(json_required_value(object, "delta")?.clone())?,
    })
}

pub(in crate::collector::stats::playback) fn parse_core_team_stats_event(
    value: &Value,
) -> SubtrActorResult<CoreTeamStatsEvent> {
    let object = json_object(value, "core team stats event")?;
    Ok(CoreTeamStatsEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        delta: decode_json_value(json_required_value(object, "delta")?.clone())?,
    })
}

pub(in crate::collector::stats::playback) fn parse_possession_event(
    value: &Value,
) -> SubtrActorResult<PossessionEvent> {
    let object = json_object(value, "possession event")?;
    Ok(PossessionEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        active: json_required_bool(object, "active")?,
        duration: json_required_f32(object, "duration")?,
        possession_state: json_required_str(object, "possession_state")?.to_owned(),
        field_third: match object.get("field_third") {
            None | Some(Value::Null) => None,
            Some(_) => Some(json_required_str(object, "field_third")?.to_owned()),
        },
    })
}

pub(in crate::collector::stats::playback) fn parse_pressure_event(
    value: &Value,
) -> SubtrActorResult<PressureEvent> {
    let object = json_object(value, "pressure event")?;
    Ok(PressureEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        active: json_required_bool(object, "active")?,
        duration: json_required_f32(object, "duration")?,
        field_half: json_required_str(object, "field_half")?.to_owned(),
    })
}

pub(in crate::collector::stats::playback) fn parse_territorial_pressure_event(
    value: &Value,
) -> SubtrActorResult<TerritorialPressureEvent> {
    decode_json_value(value.clone())
}

pub(in crate::collector::stats::playback) fn parse_movement_event(
    value: &Value,
) -> SubtrActorResult<MovementEvent> {
    let object = json_object(value, "movement event")?;
    Ok(MovementEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        player_position: json_optional_vec3(object.get("player_position"))?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        dt: json_required_f32(object, "dt")?,
        speed: json_required_f32(object, "speed")?,
        distance: json_required_f32(object, "distance")?,
        speed_band: json_required_str(object, "speed_band")?.to_owned(),
        height_band: json_required_str(object, "height_band")?.to_owned(),
    })
}

pub(in crate::collector::stats::playback) fn parse_positioning_event(
    value: &Value,
) -> SubtrActorResult<PositioningEvent> {
    let object = json_object(value, "positioning event")?;
    Ok(PositioningEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        player_position: json_optional_vec3(object.get("player_position"))?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        active_game_time: json_required_f32(object, "active_game_time")?,
        tracked_time: json_required_f32(object, "tracked_time")?,
        sum_distance_to_teammates: json_required_f32(object, "sum_distance_to_teammates")?,
        sum_distance_to_ball: json_required_f32(object, "sum_distance_to_ball")?,
        sum_distance_to_ball_has_possession: json_required_f32(
            object,
            "sum_distance_to_ball_has_possession",
        )?,
        time_has_possession: json_required_f32(object, "time_has_possession")?,
        sum_distance_to_ball_no_possession: json_required_f32(
            object,
            "sum_distance_to_ball_no_possession",
        )?,
        time_no_possession: json_required_f32(object, "time_no_possession")?,
        time_demolished: json_required_f32(object, "time_demolished")?,
        time_no_teammates: json_required_f32(object, "time_no_teammates")?,
        time_most_back: json_required_f32(object, "time_most_back")?,
        time_most_forward: json_required_f32(object, "time_most_forward")?,
        time_mid_role: json_required_f32(object, "time_mid_role")?,
        time_other_role: json_required_f32(object, "time_other_role")?,
        time_defensive_zone: json_required_f32(object, "time_defensive_third")?,
        time_neutral_zone: json_required_f32(object, "time_neutral_third")?,
        time_offensive_zone: json_required_f32(object, "time_offensive_third")?,
        time_defensive_half: json_required_f32(object, "time_defensive_half")?,
        time_offensive_half: json_required_f32(object, "time_offensive_half")?,
        time_closest_to_ball: json_required_f32(object, "time_closest_to_ball")?,
        time_farthest_from_ball: json_required_f32(object, "time_farthest_from_ball")?,
        time_behind_ball: json_required_f32(object, "time_behind_ball")?,
        time_level_with_ball: json_required_f32(object, "time_level_with_ball")?,
        time_in_front_of_ball: json_required_f32(object, "time_in_front_of_ball")?,
        times_caught_ahead_of_play_on_conceded_goals: json_required_usize(
            object,
            "times_caught_ahead_of_play_on_conceded_goals",
        )? as u32,
    })
}

pub(in crate::collector::stats::playback) fn parse_rotation_player_event(
    value: &Value,
) -> SubtrActorResult<RotationPlayerEvent> {
    let object = json_object(value, "rotation player event")?;
    Ok(RotationPlayerEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        player_position: json_optional_vec3(object.get("player_position"))?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        active: json_required_bool(object, "active")?,
        active_game_time: json_required_f32(object, "active_game_time")?,
        tracked_time: json_required_f32(object, "tracked_time")?,
        time_first_man: json_required_f32(object, "time_first_man")?,
        time_second_man: json_required_f32(object, "time_second_man")?,
        time_third_man: json_required_f32(object, "time_third_man")?,
        time_ambiguous_role: json_required_f32(object, "time_ambiguous_role")?,
        time_behind_play: json_required_f32(object, "time_behind_play")?,
        time_level_with_play: json_required_f32(object, "time_level_with_play")?,
        time_ahead_of_play: json_required_f32(object, "time_ahead_of_play")?,
        longest_first_man_stint_time: json_required_f32(object, "longest_first_man_stint_time")?,
        first_man_stint_count: json_required_usize(object, "first_man_stint_count")? as u32,
        became_first_man_count: json_required_usize(object, "became_first_man_count")? as u32,
        lost_first_man_count: json_required_usize(object, "lost_first_man_count")? as u32,
        current_role_state: decode_json_value(
            json_required_value(object, "current_role_state")?.clone(),
        )?,
        current_depth_state: decode_json_value(
            json_required_value(object, "current_depth_state")?.clone(),
        )?,
    })
}

pub(in crate::collector::stats::playback) fn parse_rotation_team_event(
    value: &Value,
) -> SubtrActorResult<RotationTeamEvent> {
    let object = json_object(value, "rotation team event")?;
    Ok(RotationTeamEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        first_man_changes_for_team: json_required_usize(object, "first_man_changes_for_team")?
            as u32,
        rotation_count: json_required_usize(object, "rotation_count")? as u32,
    })
}

pub(in crate::collector::stats::playback) fn parse_touch_stats_event(
    value: &Value,
) -> SubtrActorResult<TouchStatsEvent> {
    let object = json_object(value, "touch stats event")?;
    let time = json_required_f32(object, "time")?;
    let frame = json_required_usize(object, "frame")?;
    Ok(TouchStatsEvent {
        time,
        frame,
        sample_time: json_optional_f32(object.get("sample_time"))?.unwrap_or(time),
        sample_frame: json_optional_usize(object.get("sample_frame"))?.unwrap_or(frame),
        player: json_required_remote_id(object, "player")?,
        player_position: json_optional_vec3(object.get("player_position"))?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        kind: json_required_str(object, "kind")?.to_owned(),
        height_band: json_required_str(object, "height_band")?.to_owned(),
        surface: json_required_str(object, "surface")?.to_owned(),
        dodge_state: json_required_str(object, "dodge_state")?.to_owned(),
        ball_speed_change: json_required_f32(object, "ball_speed_change")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_touch_ball_movement_event(
    value: &Value,
) -> SubtrActorResult<TouchBallMovementEvent> {
    let object = json_object(value, "touch ball movement event")?;
    Ok(TouchBallMovementEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        player_position: json_optional_vec3(object.get("player_position"))?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        travel_distance: json_required_f32(object, "travel_distance")?,
        advance_distance: json_required_f32(object, "advance_distance")?,
        retreat_distance: json_required_f32(object, "retreat_distance")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_touch_last_touch_event(
    value: &Value,
) -> SubtrActorResult<TouchLastTouchEvent> {
    let object = json_object(value, "touch last-touch event")?;
    let time = json_required_f32(object, "time")?;
    let frame = json_required_usize(object, "frame")?;
    Ok(TouchLastTouchEvent {
        time,
        frame,
        sample_time: json_optional_f32(object.get("sample_time"))?.unwrap_or(time),
        sample_frame: json_optional_usize(object.get("sample_frame"))?.unwrap_or(frame),
        is_team_0: json_required_bool(object, "is_team_0")?,
        player: json_optional_remote_id(object.get("player"))?,
        player_position: json_optional_vec3(object.get("player_position"))?,
    })
}

pub(in crate::collector::stats::playback) fn parse_flick_mechanic_event(
    value: &Value,
    index: usize,
) -> SubtrActorResult<StatsTimelineTagEvent> {
    let object = json_object(value, "flick mechanic event")?;
    Ok(span_mechanic_event(
        "flick",
        index,
        json_required_usize(object, "setup_start_frame")?,
        json_required_usize(object, "frame")?,
        json_required_f32(object, "setup_start_time")?,
        json_required_f32(object, "time")?,
        json_required_remote_id(object, "player")?,
        json_required_bool(object, "is_team_0")?,
    ))
}

pub(in crate::collector::stats::playback) fn parse_flick_event(
    value: &Value,
) -> SubtrActorResult<FlickEvent> {
    let object = json_object(value, "flick event")?;
    let time = json_required_f32(object, "time")?;
    let frame = json_required_usize(object, "frame")?;
    Ok(FlickEvent {
        time,
        frame,
        sample_time: json_optional_f32(object.get("sample_time"))?.unwrap_or(time),
        sample_frame: json_optional_usize(object.get("sample_frame"))?.unwrap_or(frame),
        player: json_required_remote_id(object, "player")?,
        player_position: json_optional_vec3(object.get("player_position"))?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        dodge_time: json_required_f32(object, "dodge_time")?,
        dodge_frame: json_required_usize(object, "dodge_frame")?,
        time_since_dodge: json_required_f32(object, "time_since_dodge")?,
        setup_start_time: json_required_f32(object, "setup_start_time")?,
        setup_start_frame: json_required_usize(object, "setup_start_frame")?,
        setup_duration: json_required_f32(object, "setup_duration")?,
        setup_touch_count: json_required_usize(object, "setup_touch_count")? as u32,
        average_horizontal_gap: json_required_f32(object, "average_horizontal_gap")?,
        average_vertical_gap: json_required_f32(object, "average_vertical_gap")?,
        ball_speed_change: json_required_f32(object, "ball_speed_change")?,
        ball_impulse: json_required_vec3(object, "ball_impulse")?,
        impulse_away_alignment: json_required_f32(object, "impulse_away_alignment")?,
        vertical_impulse: json_required_f32(object, "vertical_impulse")?,
        confidence: json_required_f32(object, "confidence")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_musty_flick_mechanic_event(
    value: &Value,
    index: usize,
) -> SubtrActorResult<StatsTimelineTagEvent> {
    let object = json_object(value, "musty flick mechanic event")?;
    Ok(span_mechanic_event(
        "musty_flick",
        index,
        json_required_usize(object, "dodge_frame")?,
        json_required_usize(object, "frame")?,
        json_required_f32(object, "dodge_time")?,
        json_required_f32(object, "time")?,
        json_required_remote_id(object, "player")?,
        json_required_bool(object, "is_team_0")?,
    ))
}

pub(in crate::collector::stats::playback) fn parse_musty_flick_event(
    value: &Value,
) -> SubtrActorResult<MustyFlickEvent> {
    let object = json_object(value, "musty flick event")?;
    let time = json_required_f32(object, "time")?;
    let frame = json_required_usize(object, "frame")?;
    Ok(MustyFlickEvent {
        time,
        frame,
        sample_time: json_optional_f32(object.get("sample_time"))?.unwrap_or(time),
        sample_frame: json_optional_usize(object.get("sample_frame"))?.unwrap_or(frame),
        player: json_required_remote_id(object, "player")?,
        player_position: json_optional_vec3(object.get("player_position"))?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        aerial: json_required_bool(object, "aerial")?,
        dodge_time: json_required_f32(object, "dodge_time")?,
        dodge_frame: json_required_usize(object, "dodge_frame")?,
        time_since_dodge: json_required_f32(object, "time_since_dodge")?,
        confidence: json_required_f32(object, "confidence")?,
        local_ball_position: json_required_vec3(object, "local_ball_position")?,
        rear_alignment: json_required_f32(object, "rear_alignment")?,
        top_alignment: json_required_f32(object, "top_alignment")?,
        forward_approach_speed: json_required_f32(object, "forward_approach_speed")?,
        pitch_rate: json_required_f32(object, "pitch_rate")?,
        ball_speed_change: json_required_f32(object, "ball_speed_change")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_goal_context_event(
    value: &Value,
) -> SubtrActorResult<GoalContextEvent> {
    let object = json_object(value, "goal context event")?;
    Ok(GoalContextEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        scoring_team_is_team_0: json_required_bool(object, "scoring_team_is_team_0")?,
        scorer: json_optional_remote_id(object.get("scorer"))?,
        scoring_team_most_back_player: json_optional_remote_id(
            object.get("scoring_team_most_back_player"),
        )?,
        defending_team_most_back_player: json_optional_remote_id(
            object.get("defending_team_most_back_player"),
        )?,
        ball_position: json_optional_goal_context_position(object.get("ball_position"))?,
        ball_speed_at_goal: json_optional_f32(object.get("ball_speed_at_goal"))?,
        ball_air_time_before_goal: json_optional_f32(object.get("ball_air_time_before_goal"))?,
        goal_buildup: object
            .get("goal_buildup")
            .map(|value| decode_json_value(value.clone()))
            .transpose()?
            .unwrap_or_default(),
        scorer_last_touch: match object.get("scorer_last_touch") {
            None | Some(Value::Null) => None,
            Some(value) => Some(parse_goal_touch_context(value)?),
        },
        players: json_required_array(object, "players")?
            .iter()
            .map(parse_goal_player_context)
            .collect::<SubtrActorResult<Vec<_>>>()?,
    })
}

pub(in crate::collector::stats::playback) fn parse_goal_player_context(
    value: &Value,
) -> SubtrActorResult<GoalPlayerContext> {
    let object = json_object(value, "goal player context")?;
    Ok(GoalPlayerContext {
        player: json_required_remote_id(object, "player")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        position: json_optional_goal_context_position(object.get("position"))?,
        boost_amount: json_optional_f32(object.get("boost_amount"))?,
        average_boost_in_leadup: json_optional_f32(object.get("average_boost_in_leadup"))?,
        min_boost_in_leadup: json_optional_f32(object.get("min_boost_in_leadup"))?,
        is_most_back: json_required_bool(object, "is_most_back")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_goal_touch_context(
    value: &Value,
) -> SubtrActorResult<GoalTouchContext> {
    let object = json_object(value, "goal touch context")?;
    Ok(GoalTouchContext {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        ball_position: json_optional_goal_context_position(object.get("ball_position"))?,
        ball_speed_after_touch: json_optional_f32(object.get("ball_speed_after_touch"))?,
        player_position: json_optional_goal_context_position(object.get("player_position"))?,
        players: match object.get("players").and_then(Value::as_array) {
            Some(players) => players
                .iter()
                .map(parse_goal_player_context)
                .collect::<SubtrActorResult<Vec<_>>>()?,
            None => Vec::new(),
        },
    })
}

pub(in crate::collector::stats::playback) fn parse_backboard_event(
    value: &Value,
) -> SubtrActorResult<BackboardBounceEvent> {
    let object = json_object(value, "backboard event")?;
    Ok(BackboardBounceEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        player_position: json_optional_vec3(object.get("player_position"))?,
        is_team_0: json_required_bool(object, "is_team_0")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_ceiling_shot_event(
    value: &Value,
) -> SubtrActorResult<CeilingShotEvent> {
    let object = json_object(value, "ceiling shot event")?;
    Ok(CeilingShotEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        player_position: json_optional_vec3(object.get("player_position"))?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        ceiling_contact_time: json_required_f32(object, "ceiling_contact_time")?,
        ceiling_contact_frame: json_required_usize(object, "ceiling_contact_frame")?,
        time_since_ceiling_contact: json_required_f32(object, "time_since_ceiling_contact")?,
        ceiling_contact_position: json_required_vec3(object, "ceiling_contact_position")?,
        touch_position: json_required_vec3(object, "touch_position")?,
        local_ball_position: json_required_vec3(object, "local_ball_position")?,
        separation_from_ceiling: json_required_f32(object, "separation_from_ceiling")?,
        roof_alignment: json_required_f32(object, "roof_alignment")?,
        forward_alignment: json_required_f32(object, "forward_alignment")?,
        forward_approach_speed: json_required_f32(object, "forward_approach_speed")?,
        ball_speed_change: json_required_f32(object, "ball_speed_change")?,
        confidence: json_required_f32(object, "confidence")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_wall_aerial_event(
    value: &Value,
) -> SubtrActorResult<WallAerialEvent> {
    let object = json_object(value, "wall aerial event")?;
    let time = json_required_f32(object, "time")?;
    let frame = json_required_usize(object, "frame")?;
    Ok(WallAerialEvent {
        time,
        frame,
        sample_time: json_optional_f32(object.get("sample_time"))?.unwrap_or(time),
        sample_frame: json_optional_usize(object.get("sample_frame"))?.unwrap_or(frame),
        player: json_required_remote_id(object, "player")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        wall: decode_json_value(json_required_value(object, "wall")?.clone())?,
        wall_contact_time: json_required_f32(object, "wall_contact_time")?,
        wall_contact_frame: json_required_usize(object, "wall_contact_frame")?,
        takeoff_time: json_required_f32(object, "takeoff_time")?,
        takeoff_frame: json_required_usize(object, "takeoff_frame")?,
        time_since_takeoff: json_required_f32(object, "time_since_takeoff")?,
        wall_contact_position: json_required_vec3(object, "wall_contact_position")?,
        takeoff_position: json_required_vec3(object, "takeoff_position")?,
        player_position: json_required_vec3(object, "player_position")?,
        ball_position: json_required_vec3(object, "ball_position")?,
        setup_start_time: json_required_f32(object, "setup_start_time")?,
        setup_start_frame: json_required_usize(object, "setup_start_frame")?,
        setup_duration: json_required_f32(object, "setup_duration")?,
        ball_speed: json_required_f32(object, "ball_speed")?,
        ball_speed_change: json_required_f32(object, "ball_speed_change")?,
        goal_alignment: json_required_f32(object, "goal_alignment")?,
        confidence: json_required_f32(object, "confidence")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_wall_aerial_shot_event(
    value: &Value,
) -> SubtrActorResult<WallAerialShotEvent> {
    let object = json_object(value, "wall aerial shot event")?;
    Ok(WallAerialShotEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        wall: decode_json_value(json_required_value(object, "wall")?.clone())?,
        wall_contact_time: json_required_f32(object, "wall_contact_time")?,
        wall_contact_frame: json_required_usize(object, "wall_contact_frame")?,
        takeoff_time: json_required_f32(object, "takeoff_time")?,
        takeoff_frame: json_required_usize(object, "takeoff_frame")?,
        time_since_takeoff: json_required_f32(object, "time_since_takeoff")?,
        wall_contact_position: json_required_vec3(object, "wall_contact_position")?,
        takeoff_position: json_required_vec3(object, "takeoff_position")?,
        player_position: json_required_vec3(object, "player_position")?,
        ball_position: json_required_vec3(object, "ball_position")?,
        ball_speed: json_optional_f32(object.get("ball_speed"))?,
        goal_alignment: json_optional_f32(object.get("goal_alignment"))?,
        confidence: json_required_f32(object, "confidence")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_center_event(
    value: &Value,
) -> SubtrActorResult<CenterEvent> {
    let object = json_object(value, "center event")?;
    Ok(CenterEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        player_position: json_optional_vec3(object.get("player_position"))?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        start_time: json_required_f32(object, "start_time")?,
        start_frame: json_required_usize(object, "start_frame")?,
        duration: json_required_f32(object, "duration")?,
        start_ball_position: json_required_vec3(object, "start_ball_position")?,
        end_ball_position: json_required_vec3(object, "end_ball_position")?,
        ball_travel_distance: json_required_f32(object, "ball_travel_distance")?,
        ball_advance_distance: json_required_f32(object, "ball_advance_distance")?,
        lateral_centering_distance: json_required_f32(object, "lateral_centering_distance")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_double_tap_event(
    value: &Value,
) -> SubtrActorResult<DoubleTapEvent> {
    let object = json_object(value, "double tap event")?;
    Ok(DoubleTapEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        player_position: json_optional_vec3(object.get("player_position"))?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        backboard_time: json_required_f32(object, "backboard_time")?,
        backboard_frame: json_required_usize(object, "backboard_frame")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_pass_event(
    value: &Value,
) -> SubtrActorResult<PassEvent> {
    let object = json_object(value, "pass event")?;
    let time = json_required_f32(object, "time")?;
    let frame = json_required_usize(object, "frame")?;
    Ok(PassEvent {
        time,
        frame,
        sample_time: json_optional_f32(object.get("sample_time"))?.unwrap_or(time),
        sample_frame: json_optional_usize(object.get("sample_frame"))?.unwrap_or(frame),
        passer: json_required_remote_id(object, "passer")?,
        passer_position: json_optional_vec3(object.get("passer_position"))?,
        receiver: json_required_remote_id(object, "receiver")?,
        receiver_position: json_optional_vec3(object.get("receiver_position"))?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        start_time: json_required_f32(object, "start_time")?,
        start_frame: json_required_usize(object, "start_frame")?,
        duration: json_required_f32(object, "duration")?,
        ball_travel_distance: json_required_f32(object, "ball_travel_distance")?,
        ball_advance_distance: json_required_f32(object, "ball_advance_distance")?,
        pass_kind: parse_pass_kind(object.get("pass_kind"))?,
    })
}

pub(in crate::collector::stats::playback) fn parse_pass_last_completed_event(
    value: &Value,
) -> SubtrActorResult<PassLastCompletedEvent> {
    let object = json_object(value, "pass last completed event")?;
    Ok(PassLastCompletedEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_optional_remote_id(object.get("player"))?,
        player_position: json_optional_vec3(object.get("player_position"))?,
    })
}

pub(in crate::collector::stats::playback) fn parse_pass_kind(
    value: Option<&Value>,
) -> SubtrActorResult<PassKind> {
    let Some(value) = value else {
        return Ok(PassKind::Direct);
    };
    let kind = value.as_str().ok_or_else(|| {
        SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
            "Expected JSON field 'pass_kind' to be a string".to_owned(),
        ))
    })?;
    match kind {
        "direct" => Ok(PassKind::Direct),
        "backboard" => Ok(PassKind::Backboard),
        "fifty_fifty" => Ok(PassKind::FiftyFifty),
        "fifty_fifty_backboard" => Ok(PassKind::FiftyFiftyBackboard),
        other => Err(SubtrActorError::new(
            SubtrActorErrorVariant::StatsSerializationError(format!("Unknown pass kind '{other}'")),
        )),
    }
}

pub(in crate::collector::stats::playback) fn parse_ball_carry_event(
    value: &Value,
) -> SubtrActorResult<BallCarryEvent> {
    let object = json_object(value, "ball carry event")?;
    Ok(BallCarryEvent {
        player_id: json_required_remote_id(object, "player_id")?,
        start_position: json_required_vec3(object, "start_position")?,
        end_position: json_required_vec3(object, "end_position")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        kind: parse_ball_carry_kind(json_required_str(object, "kind")?)?,
        start_frame: json_required_usize(object, "start_frame")?,
        end_frame: json_required_usize(object, "end_frame")?,
        start_time: json_required_f32(object, "start_time")?,
        end_time: json_required_f32(object, "end_time")?,
        duration: json_required_f32(object, "duration")?,
        straight_line_distance: json_required_f32(object, "straight_line_distance")?,
        path_distance: json_required_f32(object, "path_distance")?,
        average_horizontal_gap: json_required_f32(object, "average_horizontal_gap")?,
        average_vertical_gap: json_required_f32(object, "average_vertical_gap")?,
        average_speed: json_required_f32(object, "average_speed")?,
        touch_count: json_required_usize(object, "touch_count")? as u32,
        air_touch_count: json_required_usize(object, "air_touch_count")? as u32,
        air_dribble_origin: parse_air_dribble_origin(object.get("air_dribble_origin"))?,
    })
}

pub(in crate::collector::stats::playback) fn parse_ball_carry_kind(
    kind: &str,
) -> SubtrActorResult<BallCarryKind> {
    match kind {
        "carry" => Ok(BallCarryKind::Carry),
        "air_dribble" => Ok(BallCarryKind::AirDribble),
        other => Err(SubtrActorError::new(
            SubtrActorErrorVariant::StatsSerializationError(format!(
                "Unknown ball carry kind '{other}'"
            )),
        )),
    }
}

pub(in crate::collector::stats::playback) fn parse_air_dribble_origin(
    value: Option<&Value>,
) -> SubtrActorResult<Option<AirDribbleOrigin>> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let origin = value.as_str().ok_or_else(|| {
        SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
            "Expected optional JSON field 'air_dribble_origin' to be a string".to_owned(),
        ))
    })?;
    match origin {
        "ground_to_air" => Ok(Some(AirDribbleOrigin::GroundToAir)),
        "wall_to_air" => Ok(Some(AirDribbleOrigin::WallToAir)),
        other => Err(SubtrActorError::new(
            SubtrActorErrorVariant::StatsSerializationError(format!(
                "Unknown air dribble origin '{other}'"
            )),
        )),
    }
}

pub(in crate::collector::stats::playback) fn parse_one_timer_event(
    value: &Value,
) -> SubtrActorResult<OneTimerEvent> {
    let object = json_object(value, "one timer event")?;
    Ok(OneTimerEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        player_position: json_optional_vec3(object.get("player_position"))?,
        passer: json_required_remote_id(object, "passer")?,
        passer_position: json_optional_vec3(object.get("passer_position"))?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        pass_start_time: json_required_f32(object, "pass_start_time")?,
        pass_start_frame: json_required_usize(object, "pass_start_frame")?,
        pass_duration: json_required_f32(object, "pass_duration")?,
        pass_travel_distance: json_required_f32(object, "pass_travel_distance")?,
        pass_advance_distance: json_required_f32(object, "pass_advance_distance")?,
        ball_speed: json_required_f32(object, "ball_speed")?,
        goal_alignment: json_required_f32(object, "goal_alignment")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_half_volley_event(
    value: &Value,
) -> SubtrActorResult<HalfVolleyEvent> {
    let object = json_object(value, "half volley event")?;
    let time = json_required_f32(object, "time")?;
    let frame = json_required_usize(object, "frame")?;
    Ok(HalfVolleyEvent {
        time,
        frame,
        sample_time: json_optional_f32(object.get("sample_time"))?.unwrap_or(time),
        sample_frame: json_optional_usize(object.get("sample_frame"))?.unwrap_or(frame),
        player: json_required_remote_id(object, "player")?,
        player_position: json_optional_vec3(object.get("player_position"))?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        bounce_time: json_required_f32(object, "bounce_time")?,
        bounce_frame: json_required_usize(object, "bounce_frame")?,
        bounce_to_touch_seconds: json_required_f32(object, "bounce_to_touch_seconds")?,
        ball_speed: json_required_f32(object, "ball_speed")?,
        goal_alignment: json_required_f32(object, "goal_alignment")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_goal_tag_event(
    value: &Value,
) -> SubtrActorResult<GoalTagEvent> {
    let object = json_object(value, "goal tag event")?;
    Ok(GoalTagEvent {
        goal_index: json_required_usize(object, "goal_index")?,
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        kind: decode_json_value(json_required_value(object, "kind")?.clone())?,
        scoring_team_is_team_0: json_required_bool(object, "scoring_team_is_team_0")?,
        scorer: json_optional_remote_id(object.get("scorer"))?,
        scorer_position: json_optional_goal_context_position(object.get("scorer_position"))?,
        confidence: json_required_f32(object, "confidence")?,
        modifiers: json_optional_array(object.get("modifiers"))?
            .iter()
            .map(|modifier| decode_json_value(modifier.clone()))
            .collect::<SubtrActorResult<Vec<_>>>()?,
        evidence: json_required_array(object, "evidence")?
            .iter()
            .map(parse_goal_tag_evidence)
            .collect::<SubtrActorResult<Vec<_>>>()?,
    })
}

pub(in crate::collector::stats::playback) fn parse_goal_tag_evidence(
    value: &Value,
) -> SubtrActorResult<GoalTagEvidence> {
    let object = json_object(value, "goal tag evidence")?;
    Ok(GoalTagEvidence {
        kind: decode_json_value(json_required_value(object, "kind")?.clone())?,
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_optional_remote_id(object.get("player"))?,
        player_position: json_optional_goal_context_position(object.get("player_position"))?,
    })
}

pub(in crate::collector::stats::playback) fn parse_fifty_fifty_event(
    value: &Value,
) -> SubtrActorResult<FiftyFiftyEvent> {
    let object = json_object(value, "fifty fifty event")?;
    Ok(FiftyFiftyEvent {
        start_time: json_required_f32(object, "start_time")?,
        start_frame: json_required_usize(object, "start_frame")?,
        resolve_time: json_required_f32(object, "resolve_time")?,
        resolve_frame: json_required_usize(object, "resolve_frame")?,
        is_kickoff: json_required_bool(object, "is_kickoff")?,
        team_zero_player: json_optional_remote_id(object.get("team_zero_player"))?,
        team_one_player: json_optional_remote_id(object.get("team_one_player"))?,
        team_zero_touch_time: json_optional_f32(object.get("team_zero_touch_time"))?,
        team_zero_touch_frame: json_optional_usize(object.get("team_zero_touch_frame"))?,
        team_zero_dodge_contact: json_optional_bool(object.get("team_zero_dodge_contact"))
            .unwrap_or(false),
        team_one_touch_time: json_optional_f32(object.get("team_one_touch_time"))?,
        team_one_touch_frame: json_optional_usize(object.get("team_one_touch_frame"))?,
        team_one_dodge_contact: json_optional_bool(object.get("team_one_dodge_contact"))
            .unwrap_or(false),
        team_zero_position: json_required_vec3(object, "team_zero_position")?,
        team_one_position: json_required_vec3(object, "team_one_position")?,
        midpoint: json_required_vec3(object, "midpoint")?,
        plane_normal: json_required_vec3(object, "plane_normal")?,
        winning_team_is_team_0: json_optional_bool(object.get("winning_team_is_team_0")),
        possession_team_is_team_0: json_optional_bool(object.get("possession_team_is_team_0")),
    })
}

pub(in crate::collector::stats::playback) fn parse_speed_flip_event(
    value: &Value,
) -> SubtrActorResult<SpeedFlipEvent> {
    let object = json_object(value, "speed flip event")?;
    let time = json_required_f32(object, "time")?;
    let frame = json_required_usize(object, "frame")?;
    Ok(SpeedFlipEvent {
        time,
        frame,
        resolved_time: json_optional_f32(object.get("resolved_time"))?.unwrap_or(time),
        resolved_frame: json_optional_usize(object.get("resolved_frame"))?.unwrap_or(frame),
        player: json_required_remote_id(object, "player")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        time_since_kickoff_start: json_required_f32(object, "time_since_kickoff_start")?,
        start_position: json_required_vec3(object, "start_position")?,
        end_position: json_required_vec3(object, "end_position")?,
        start_speed: json_required_f32(object, "start_speed")?,
        max_speed: json_required_f32(object, "max_speed")?,
        best_alignment: json_required_f32(object, "best_alignment")?,
        diagonal_score: json_required_f32(object, "diagonal_score")?,
        cancel_score: json_required_f32(object, "cancel_score")?,
        speed_score: json_required_f32(object, "speed_score")?,
        confidence: json_required_f32(object, "confidence")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_half_flip_event(
    value: &Value,
) -> SubtrActorResult<HalfFlipEvent> {
    let object = json_object(value, "half flip event")?;
    Ok(HalfFlipEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        start_position: json_required_vec3(object, "start_position")?,
        end_position: json_required_vec3(object, "end_position")?,
        start_speed: json_required_f32(object, "start_speed")?,
        end_speed: json_required_f32(object, "end_speed")?,
        start_backward_alignment: json_required_f32(object, "start_backward_alignment")?,
        best_reorientation_alignment: json_required_f32(object, "best_reorientation_alignment")?,
        best_forward_reversal: json_required_f32(object, "best_forward_reversal")?,
        max_forward_vertical: json_required_f32(object, "max_forward_vertical")?,
        confidence: json_required_f32(object, "confidence")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_wavedash_event(
    value: &Value,
) -> SubtrActorResult<WavedashEvent> {
    let object = json_object(value, "wavedash event")?;
    Ok(WavedashEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        dodge_time: json_required_f32(object, "dodge_time")?,
        dodge_frame: json_required_usize(object, "dodge_frame")?,
        time_since_dodge: json_required_f32(object, "time_since_dodge")?,
        dodge_position: json_required_vec3(object, "dodge_position")?,
        landing_position: json_required_vec3(object, "landing_position")?,
        start_speed: json_required_f32(object, "start_speed")?,
        landing_speed: json_required_f32(object, "landing_speed")?,
        horizontal_speed_gain: json_required_f32(object, "horizontal_speed_gain")?,
        landing_uprightness: json_required_f32(object, "landing_uprightness")?,
        confidence: json_required_f32(object, "confidence")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_whiff_event(
    value: &Value,
) -> SubtrActorResult<WhiffEvent> {
    let object = json_object(value, "whiff event")?;
    let time = json_required_f32(object, "time")?;
    let frame = json_required_usize(object, "frame")?;
    Ok(WhiffEvent {
        kind: match object.get("kind").and_then(Value::as_str) {
            None | Some("whiff") => WhiffEventKind::Whiff,
            Some("beaten_to_ball") => WhiffEventKind::BeatenToBall,
            Some(kind) => {
                return SubtrActorError::new_result(
                    SubtrActorErrorVariant::StatsSerializationError(format!(
                        "Unknown whiff event kind '{kind}'"
                    )),
                );
            }
        },
        time,
        frame,
        resolved_time: json_optional_f32(object.get("resolved_time"))?.unwrap_or(time),
        resolved_frame: json_optional_usize(object.get("resolved_frame"))?.unwrap_or(frame),
        player: json_required_remote_id(object, "player")?,
        player_position: json_optional_vec3(object.get("player_position"))?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        closest_approach_distance: json_required_f32(object, "closest_approach_distance")?,
        forward_alignment: json_required_f32(object, "forward_alignment")?,
        approach_speed: json_required_f32(object, "approach_speed")?,
        dodge_active: json_required_bool(object, "dodge_active")?,
        aerial: json_required_bool(object, "aerial")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_bump_event(
    value: &Value,
) -> SubtrActorResult<BumpEvent> {
    let object = json_object(value, "bump event")?;
    Ok(BumpEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        initiator: json_required_remote_id(object, "initiator")?,
        victim: json_required_remote_id(object, "victim")?,
        initiator_is_team_0: json_required_bool(object, "initiator_is_team_0")?,
        victim_is_team_0: json_required_bool(object, "victim_is_team_0")?,
        is_team_bump: json_required_bool(object, "is_team_bump")?,
        strength: json_required_f32(object, "strength")?,
        confidence: json_required_f32(object, "confidence")?,
        contact_distance: json_required_f32(object, "contact_distance")?,
        closing_speed: json_required_f32(object, "closing_speed")?,
        victim_impulse: json_required_f32(object, "victim_impulse")?,
        initiator_position: json_required_vec3(object, "initiator_position")?,
        victim_position: json_required_vec3(object, "victim_position")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_boost_pickup_comparison_event(
    value: &Value,
) -> SubtrActorResult<BoostPickupComparisonEvent> {
    let object = json_object(value, "boost pickup comparison event")?;
    Ok(BoostPickupComparisonEvent {
        comparison: decode_json_value(json_required_value(object, "comparison")?.clone())?,
        frame: json_required_usize(object, "frame")?,
        time: json_required_f32(object, "time")?,
        player_id: json_required_remote_id(object, "player_id")?,
        player_position: json_optional_vec3(object.get("player_position"))?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        pad_type: decode_json_value(json_required_value(object, "pad_type")?.clone())?,
        field_half: decode_json_value(json_required_value(object, "field_half")?.clone())?,
        activity: decode_json_value(json_required_value(object, "activity")?.clone())?,
        reported_frame: json_optional_usize(object.get("reported_frame"))?,
        reported_time: json_optional_f32(object.get("reported_time"))?,
        inferred_frame: json_optional_usize(object.get("inferred_frame"))?,
        inferred_time: json_optional_f32(object.get("inferred_time"))?,
        boost_before: json_optional_f32(object.get("boost_before"))?,
        boost_after: json_optional_f32(object.get("boost_after"))?,
    })
}

pub(in crate::collector::stats::playback) fn parse_boost_ledger_event(
    value: &Value,
) -> SubtrActorResult<BoostLedgerEvent> {
    let object = json_object(value, "boost ledger event")?;
    Ok(BoostLedgerEvent {
        frame: json_required_usize(object, "frame")?,
        time: json_required_f32(object, "time")?,
        player_id: json_required_remote_id(object, "player_id")?,
        player_position: json_optional_vec3(object.get("player_position"))?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        transaction: decode_json_value(json_required_value(object, "transaction")?.clone())?,
        amount: json_required_f32(object, "amount")?,
        count: json_required_usize(object, "count")? as u32,
        labels: decode_json_value(
            object
                .get("labels")
                .cloned()
                .unwrap_or_else(|| Value::Array(Vec::new())),
        )?,
        boost_before: json_optional_f32(object.get("boost_before"))?,
        boost_after: json_optional_f32(object.get("boost_after"))?,
    })
}

pub(in crate::collector::stats::playback) fn parse_boost_state_event(
    value: &Value,
) -> SubtrActorResult<BoostStateEvent> {
    let object = json_object(value, "boost state event")?;
    Ok(BoostStateEvent {
        frame: json_required_usize(object, "frame")?,
        time: json_required_f32(object, "time")?,
        player_id: json_required_remote_id(object, "player_id")?,
        player_position: json_optional_vec3(object.get("player_position"))?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        boost_amount: json_required_f32(object, "boost_amount")?,
        boost_before: json_optional_f32(object.get("boost_before"))?,
    })
}
