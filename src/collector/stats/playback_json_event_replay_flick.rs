use super::*;

pub(in crate::collector::stats::playback) fn parse_flick_mechanic_event(
    value: &Value,
    index: usize,
) -> SubtrActorResult<MechanicEvent> {
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
