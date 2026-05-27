use super::*;

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
