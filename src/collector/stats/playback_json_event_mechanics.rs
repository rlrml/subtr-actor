use super::*;

pub(in crate::collector::stats::playback) fn parse_ball_carry_mechanic_event(
    value: &Value,
    index: usize,
) -> SubtrActorResult<MechanicEvent> {
    let object = json_object(value, "ball carry mechanic event")?;
    let serialized_kind = json_required_str(object, "kind")?;
    let kind = match serialized_kind {
        "carry" => "ball_carry",
        "air_dribble" => "air_dribble",
        other => other,
    };
    let mut mechanic_event = span_mechanic_event(
        kind,
        index,
        json_required_usize(object, "start_frame")?,
        json_required_usize(object, "end_frame")?,
        json_required_f32(object, "start_time")?,
        json_required_f32(object, "end_time")?,
        json_required_remote_id(object, "player_id")?,
        json_required_bool(object, "is_team_0")?,
    );
    if kind == "air_dribble" {
        mechanic_event.properties = ball_carry_mechanic_event_properties(object);
    }
    Ok(mechanic_event)
}

pub(in crate::collector::stats::playback) fn parse_dodge_reset_mechanic_event(
    value: &Value,
    index: usize,
) -> SubtrActorResult<MechanicEvent> {
    let object = json_object(value, "dodge reset mechanic event")?;
    Ok(moment_mechanic_event(
        "flip_reset",
        index,
        json_required_usize(object, "frame")?,
        json_required_f32(object, "time")?,
        json_required_remote_id(object, "player")?,
        json_required_bool(object, "is_team_0")?,
    ))
}
