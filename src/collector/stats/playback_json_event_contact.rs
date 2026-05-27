use super::*;

pub(in crate::collector::stats::playback) fn parse_fifty_fifty_event(
    value: &Value,
) -> SubtrActorResult<FiftyFiftyEvent> {
    let object = json_object(value, "fifty fifty event")?;
    Ok(FiftyFiftyEvent {
        start_time: json_required_f32(object, "start_time")?,
        start_frame: json_required_usize(object, "start_frame")?,
        resolve_time: json_required_f32(object, "resolve_time")?,
        resolve_frame: json_required_usize(object, "resolve_frame")?,
        is_kickoff: json_required_bool(object, "is_kickoff")?,
        team_zero_player: json_optional_remote_id(object.get("team_zero_player"))?,
        team_one_player: json_optional_remote_id(object.get("team_one_player"))?,
        team_zero_touch_time: json_optional_f32(object.get("team_zero_touch_time"))?,
        team_zero_touch_frame: json_optional_usize(object.get("team_zero_touch_frame"))?,
        team_zero_dodge_contact: json_optional_bool(object.get("team_zero_dodge_contact"))
            .unwrap_or(false),
        team_one_touch_time: json_optional_f32(object.get("team_one_touch_time"))?,
        team_one_touch_frame: json_optional_usize(object.get("team_one_touch_frame"))?,
        team_one_dodge_contact: json_optional_bool(object.get("team_one_dodge_contact"))
            .unwrap_or(false),
        team_zero_position: json_required_vec3(object, "team_zero_position")?,
        team_one_position: json_required_vec3(object, "team_one_position")?,
        midpoint: json_required_vec3(object, "midpoint")?,
        plane_normal: json_required_vec3(object, "plane_normal")?,
        winning_team_is_team_0: json_optional_bool(object.get("winning_team_is_team_0")),
        possession_team_is_team_0: json_optional_bool(object.get("possession_team_is_team_0")),
    })
}

pub(in crate::collector::stats::playback) fn parse_whiff_event(
    value: &Value,
) -> SubtrActorResult<WhiffEvent> {
    let object = json_object(value, "whiff event")?;
    let time = json_required_f32(object, "time")?;
    let frame = json_required_usize(object, "frame")?;
    Ok(WhiffEvent {
        kind: match object.get("kind").and_then(Value::as_str) {
            None | Some("whiff") => WhiffEventKind::Whiff,
            Some("beaten_to_ball") => WhiffEventKind::BeatenToBall,
            Some(kind) => {
                return SubtrActorError::new_result(
                    SubtrActorErrorVariant::StatsSerializationError(format!(
                        "Unknown whiff event kind '{kind}'"
                    )),
                );
            }
        },
        time,
        frame,
        resolved_time: json_optional_f32(object.get("resolved_time"))?.unwrap_or(time),
        resolved_frame: json_optional_usize(object.get("resolved_frame"))?.unwrap_or(frame),
        player: json_required_remote_id(object, "player")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        closest_approach_distance: json_required_f32(object, "closest_approach_distance")?,
        forward_alignment: json_required_f32(object, "forward_alignment")?,
        approach_speed: json_required_f32(object, "approach_speed")?,
        dodge_active: json_required_bool(object, "dodge_active")?,
        aerial: json_required_bool(object, "aerial")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_bump_event(
    value: &Value,
) -> SubtrActorResult<BumpEvent> {
    let object = json_object(value, "bump event")?;
    Ok(BumpEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        initiator: json_required_remote_id(object, "initiator")?,
        victim: json_required_remote_id(object, "victim")?,
        initiator_is_team_0: json_required_bool(object, "initiator_is_team_0")?,
        victim_is_team_0: json_required_bool(object, "victim_is_team_0")?,
        is_team_bump: json_required_bool(object, "is_team_bump")?,
        strength: json_required_f32(object, "strength")?,
        confidence: json_required_f32(object, "confidence")?,
        contact_distance: json_required_f32(object, "contact_distance")?,
        closing_speed: json_required_f32(object, "closing_speed")?,
        victim_impulse: json_required_f32(object, "victim_impulse")?,
        initiator_position: json_required_vec3(object, "initiator_position")?,
        victim_position: json_required_vec3(object, "victim_position")?,
    })
}
