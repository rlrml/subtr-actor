use super::*;

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
    })
}
