use super::*;

pub(in crate::collector::stats::playback) fn parse_backboard_event(
    value: &Value,
) -> SubtrActorResult<BackboardBounceEvent> {
    let object = json_object(value, "backboard event")?;
    Ok(BackboardBounceEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_ceiling_shot_event(
    value: &Value,
) -> SubtrActorResult<CeilingShotEvent> {
    let object = json_object(value, "ceiling shot event")?;
    Ok(CeilingShotEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        ceiling_contact_time: json_required_f32(object, "ceiling_contact_time")?,
        ceiling_contact_frame: json_required_usize(object, "ceiling_contact_frame")?,
        time_since_ceiling_contact: json_required_f32(object, "time_since_ceiling_contact")?,
        ceiling_contact_position: json_required_vec3(object, "ceiling_contact_position")?,
        touch_position: json_required_vec3(object, "touch_position")?,
        local_ball_position: json_required_vec3(object, "local_ball_position")?,
        separation_from_ceiling: json_required_f32(object, "separation_from_ceiling")?,
        roof_alignment: json_required_f32(object, "roof_alignment")?,
        forward_alignment: json_required_f32(object, "forward_alignment")?,
        forward_approach_speed: json_required_f32(object, "forward_approach_speed")?,
        ball_speed_change: json_required_f32(object, "ball_speed_change")?,
        confidence: json_required_f32(object, "confidence")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_center_event(
    value: &Value,
) -> SubtrActorResult<CenterEvent> {
    let object = json_object(value, "center event")?;
    Ok(CenterEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        start_time: json_required_f32(object, "start_time")?,
        start_frame: json_required_usize(object, "start_frame")?,
        duration: json_required_f32(object, "duration")?,
        start_ball_position: json_required_vec3(object, "start_ball_position")?,
        end_ball_position: json_required_vec3(object, "end_ball_position")?,
        ball_travel_distance: json_required_f32(object, "ball_travel_distance")?,
        ball_advance_distance: json_required_f32(object, "ball_advance_distance")?,
        lateral_centering_distance: json_required_f32(object, "lateral_centering_distance")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_double_tap_event(
    value: &Value,
) -> SubtrActorResult<DoubleTapEvent> {
    let object = json_object(value, "double tap event")?;
    Ok(DoubleTapEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        backboard_time: json_required_f32(object, "backboard_time")?,
        backboard_frame: json_required_usize(object, "backboard_frame")?,
    })
}
