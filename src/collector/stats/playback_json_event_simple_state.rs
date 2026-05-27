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
        field_half: json_required_str(object, "field_half")?.to_owned(),
    })
}

pub(in crate::collector::stats::playback) fn parse_territorial_pressure_event(
    value: &Value,
) -> SubtrActorResult<TerritorialPressureEvent> {
    decode_json_value(value.clone())
}
