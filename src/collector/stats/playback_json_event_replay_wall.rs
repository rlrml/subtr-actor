use super::*;

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
