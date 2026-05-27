use super::*;

pub(in crate::collector::stats::playback) fn parse_goal_context_event(
    value: &Value,
) -> SubtrActorResult<GoalContextEvent> {
    let object = json_object(value, "goal context event")?;
    Ok(GoalContextEvent {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        scoring_team_is_team_0: json_required_bool(object, "scoring_team_is_team_0")?,
        scorer: json_optional_remote_id(object.get("scorer"))?,
        scoring_team_most_back_player: json_optional_remote_id(
            object.get("scoring_team_most_back_player"),
        )?,
        defending_team_most_back_player: json_optional_remote_id(
            object.get("defending_team_most_back_player"),
        )?,
        ball_position: json_optional_goal_context_position(object.get("ball_position"))?,
        ball_air_time_before_goal: json_optional_f32(object.get("ball_air_time_before_goal"))?,
        goal_buildup: object
            .get("goal_buildup")
            .map(|value| decode_json_value(value.clone()))
            .transpose()?
            .unwrap_or_default(),
        scorer_last_touch: match object.get("scorer_last_touch") {
            None | Some(Value::Null) => None,
            Some(value) => Some(parse_goal_touch_context(value)?),
        },
        players: json_required_array(object, "players")?
            .iter()
            .map(parse_goal_player_context)
            .collect::<SubtrActorResult<Vec<_>>>()?,
    })
}

pub(in crate::collector::stats::playback) fn parse_goal_player_context(
    value: &Value,
) -> SubtrActorResult<GoalPlayerContext> {
    let object = json_object(value, "goal player context")?;
    Ok(GoalPlayerContext {
        player: json_required_remote_id(object, "player")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        position: json_optional_goal_context_position(object.get("position"))?,
        boost_amount: json_optional_f32(object.get("boost_amount"))?,
        average_boost_in_leadup: json_optional_f32(object.get("average_boost_in_leadup"))?,
        min_boost_in_leadup: json_optional_f32(object.get("min_boost_in_leadup"))?,
        is_most_back: json_required_bool(object, "is_most_back")?,
    })
}

pub(in crate::collector::stats::playback) fn parse_goal_touch_context(
    value: &Value,
) -> SubtrActorResult<GoalTouchContext> {
    let object = json_object(value, "goal touch context")?;
    Ok(GoalTouchContext {
        time: json_required_f32(object, "time")?,
        frame: json_required_usize(object, "frame")?,
        player: json_required_remote_id(object, "player")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        ball_position: json_optional_goal_context_position(object.get("ball_position"))?,
        player_position: json_optional_goal_context_position(object.get("player_position"))?,
        players: match object.get("players").and_then(Value::as_array) {
            Some(players) => players
                .iter()
                .map(parse_goal_player_context)
                .collect::<SubtrActorResult<Vec<_>>>()?,
            None => Vec::new(),
        },
    })
}
