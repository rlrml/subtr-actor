use super::*;

pub(in crate::collector::stats::playback) fn parse_movement_event(
    value: &Value,
) -> SubtrActorResult<MovementEvent> {
    let object = json_object(value, "movement event")?;
    Ok(MovementEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
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
