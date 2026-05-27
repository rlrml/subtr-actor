use super::*;

pub(in crate::collector::stats::playback) fn parse_rotation_player_event(
    value: &Value,
) -> SubtrActorResult<RotationPlayerEvent> {
    let object = json_object(value, "rotation player event")?;
    Ok(RotationPlayerEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        active: json_required_bool(object, "active")?,
        became_first_man_count: json_required_usize(object, "became_first_man_count")? as u32,
        lost_first_man_count: json_required_usize(object, "lost_first_man_count")? as u32,
        current_role_state: decode_json_value(
            json_required_value(object, "current_role_state")?.clone(),
        )?,
        current_depth_state: decode_json_value(
            json_required_value(object, "current_depth_state")?.clone(),
        )?,
    })
}

pub(in crate::collector::stats::playback) fn parse_rotation_team_event(
    value: &Value,
) -> SubtrActorResult<RotationTeamEvent> {
    let object = json_object(value, "rotation team event")?;
    Ok(RotationTeamEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        first_man_changes_for_team: json_required_usize(object, "first_man_changes_for_team")?
            as u32,
        rotation_count: json_required_usize(object, "rotation_count")? as u32,
    })
}
