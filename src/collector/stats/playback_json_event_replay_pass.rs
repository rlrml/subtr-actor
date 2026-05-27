use super::*;

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
        receiver: json_required_remote_id(object, "receiver")?,
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
