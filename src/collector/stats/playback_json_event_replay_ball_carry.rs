use super::*;

pub(in crate::collector::stats::playback) fn parse_ball_carry_event(
    value: &Value,
) -> SubtrActorResult<BallCarryEvent> {
    let object = json_object(value, "ball carry event")?;
    Ok(BallCarryEvent {
        player_id: json_required_remote_id(object, "player_id")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        kind: parse_ball_carry_kind(json_required_str(object, "kind")?)?,
        start_frame: json_required_usize(object, "start_frame")?,
        end_frame: json_required_usize(object, "end_frame")?,
        start_time: json_required_f32(object, "start_time")?,
        end_time: json_required_f32(object, "end_time")?,
        duration: json_required_f32(object, "duration")?,
        straight_line_distance: json_required_f32(object, "straight_line_distance")?,
        path_distance: json_required_f32(object, "path_distance")?,
        average_horizontal_gap: json_required_f32(object, "average_horizontal_gap")?,
        average_vertical_gap: json_required_f32(object, "average_vertical_gap")?,
        average_speed: json_required_f32(object, "average_speed")?,
        touch_count: json_required_usize(object, "touch_count")? as u32,
        air_touch_count: json_required_usize(object, "air_touch_count")? as u32,
        air_dribble_origin: parse_air_dribble_origin(object.get("air_dribble_origin"))?,
    })
}

pub(in crate::collector::stats::playback) fn parse_ball_carry_kind(
    kind: &str,
) -> SubtrActorResult<BallCarryKind> {
    match kind {
        "carry" => Ok(BallCarryKind::Carry),
        "air_dribble" => Ok(BallCarryKind::AirDribble),
        other => Err(SubtrActorError::new(
            SubtrActorErrorVariant::StatsSerializationError(format!(
                "Unknown ball carry kind '{other}'"
            )),
        )),
    }
}

pub(in crate::collector::stats::playback) fn parse_air_dribble_origin(
    value: Option<&Value>,
) -> SubtrActorResult<Option<AirDribbleOrigin>> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let origin = value.as_str().ok_or_else(|| {
        SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
            "Expected optional JSON field 'air_dribble_origin' to be a string".to_owned(),
        ))
    })?;
    match origin {
        "ground_to_air" => Ok(Some(AirDribbleOrigin::GroundToAir)),
        "wall_to_air" => Ok(Some(AirDribbleOrigin::WallToAir)),
        other => Err(SubtrActorError::new(
            SubtrActorErrorVariant::StatsSerializationError(format!(
                "Unknown air dribble origin '{other}'"
            )),
        )),
    }
}
