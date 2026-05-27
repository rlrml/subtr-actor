use super::*;

pub(in crate::collector::stats::playback) fn parse_musty_flick_mechanic_event(
    value: &Value,
    index: usize,
) -> SubtrActorResult<MechanicEvent> {
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
