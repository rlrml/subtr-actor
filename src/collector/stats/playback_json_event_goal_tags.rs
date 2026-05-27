use super::*;

pub(in crate::collector::stats::playback) fn parse_goal_tag_event(
    value: &Value,
) -> SubtrActorResult<GoalTagEvent> {
    let object = json_object(value, "goal tag event")?;
    Ok(GoalTagEvent {
        goal_index: json_required_usize(object, "goal_index")?,
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        kind: decode_json_value(json_required_value(object, "kind")?.clone())?,
        scoring_team_is_team_0: json_required_bool(object, "scoring_team_is_team_0")?,
        scorer: json_optional_remote_id(object.get("scorer"))?,
        confidence: json_required_f32(object, "confidence")?,
        modifiers: json_optional_array(object.get("modifiers"))?
            .iter()
            .map(|modifier| decode_json_value(modifier.clone()))
            .collect::<SubtrActorResult<Vec<_>>>()?,
        evidence: json_required_array(object, "evidence")?
            .iter()
            .map(parse_goal_tag_evidence)
            .collect::<SubtrActorResult<Vec<_>>>()?,
    })
}

pub(in crate::collector::stats::playback) fn parse_goal_tag_evidence(
    value: &Value,
) -> SubtrActorResult<GoalTagEvidence> {
    let object = json_object(value, "goal tag evidence")?;
    Ok(GoalTagEvidence {
        kind: decode_json_value(json_required_value(object, "kind")?.clone())?,
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_optional_remote_id(object.get("player"))?,
    })
}
