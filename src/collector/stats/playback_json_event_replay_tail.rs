use super::*;

pub(in crate::collector::stats::playback) fn parse_one_timer_event(
    value: &Value,
) -> SubtrActorResult<OneTimerEvent> {
    let object = json_object(value, "one timer event")?;
    Ok(OneTimerEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        passer: json_required_remote_id(object, "passer")?,
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
        is_team_0: json_required_bool(object, "is_team_0")?,
        bounce_time: json_required_f32(object, "bounce_time")?,
        bounce_frame: json_required_usize(object, "bounce_frame")?,
        bounce_to_touch_seconds: json_required_f32(object, "bounce_to_touch_seconds")?,
        ball_speed: json_required_f32(object, "ball_speed")?,
        goal_alignment: json_required_f32(object, "goal_alignment")?,
    })
}
