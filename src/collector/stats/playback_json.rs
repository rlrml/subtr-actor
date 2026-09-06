use super::*;

pub(in crate::collector::stats::playback) fn player_stats_value_for_key<'a>(
    module: Option<&'a Value>,
    player_key: &str,
) -> SubtrActorResult<Option<&'a Value>> {
    let Some(entries) = module
        .and_then(Value::as_object)
        .and_then(|module| module.get("player_stats"))
        .and_then(Value::as_array)
    else {
        return Ok(None);
    };

    for entry in entries {
        let Some(entry_object) = entry.as_object() else {
            continue;
        };
        let Some(player_id) = entry_object.get("player_id") else {
            continue;
        };
        let Some(player_stats) = entry_object.get("stats") else {
            continue;
        };
        if player_id_key(player_id)? == player_key {
            return Ok(Some(player_stats));
        }
    }

    Ok(None)
}

pub(in crate::collector::stats::playback) fn player_info_key(
    player: &PlayerInfo,
) -> SubtrActorResult<String> {
    player_id_key(&serialize_to_json_value(&player.remote_id)?)
}

pub(in crate::collector::stats::playback) fn player_id_key(
    player_id: &Value,
) -> SubtrActorResult<String> {
    serde_json::to_string(player_id).map_err(|error| {
        SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
            error.to_string(),
        ))
    })
}

pub(in crate::collector::stats::playback) fn default_json_value<T>() -> Value
where
    T: Default + Serialize,
{
    serde_json::to_value(T::default()).expect("default stats should serialize to json")
}

pub(in crate::collector::stats::playback) fn decode_json_value<T>(
    value: Value,
) -> SubtrActorResult<T>
where
    T: DeserializeOwned,
{
    serde_json::from_value(value).map_err(|error| {
        SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
            error.to_string(),
        ))
    })
}

pub(in crate::collector::stats::playback) fn decode_core_player_stats_value(
    mut value: Value,
) -> SubtrActorResult<CorePlayerStats> {
    normalize_core_player_stats_snapshot(&mut value)?;
    decode_json_value(value)
}

pub(in crate::collector::stats::playback) fn normalize_core_player_stats_snapshot(
    value: &mut Value,
) -> SubtrActorResult<()> {
    let Some(object) = value.as_object_mut() else {
        return Ok(());
    };

    insert_cumulative_from_average(
        object,
        "cumulative_boost_on_goals_against",
        "average_boost_on_goals_against",
        "goal_against_boost_sample_count",
    )?;
    insert_cumulative_from_average(
        object,
        "cumulative_average_boost_in_goal_against_leadup",
        "average_boost_in_goal_against_leadup",
        "goal_against_boost_leadup_sample_count",
    )?;
    insert_cumulative_from_average(
        object,
        "cumulative_min_boost_in_goal_against_leadup",
        "average_min_boost_in_goal_against_leadup",
        "goal_against_boost_leadup_sample_count",
    )?;
    insert_cumulative_from_average(
        object,
        "cumulative_goal_against_position_x",
        "average_goal_against_position_x",
        "goal_against_position_sample_count",
    )?;
    insert_cumulative_from_average(
        object,
        "cumulative_goal_against_position_y",
        "average_goal_against_position_y",
        "goal_against_position_sample_count",
    )?;
    insert_cumulative_from_average(
        object,
        "cumulative_goal_against_position_z",
        "average_goal_against_position_z",
        "goal_against_position_sample_count",
    )?;
    insert_cumulative_from_average(
        object,
        "cumulative_scoring_goal_last_touch_position_x",
        "average_scoring_goal_last_touch_position_x",
        "scoring_goal_last_touch_position_sample_count",
    )?;
    insert_cumulative_from_average(
        object,
        "cumulative_scoring_goal_last_touch_position_y",
        "average_scoring_goal_last_touch_position_y",
        "scoring_goal_last_touch_position_sample_count",
    )?;
    insert_cumulative_from_average(
        object,
        "cumulative_scoring_goal_last_touch_position_z",
        "average_scoring_goal_last_touch_position_z",
        "scoring_goal_last_touch_position_sample_count",
    )?;
    insert_cumulative_from_average(
        object,
        "cumulative_goal_ball_air_time",
        "average_goal_ball_air_time",
        "goal_ball_air_time_sample_count",
    )?;

    if let Value::Object(defaults) = default_json_value::<CorePlayerStats>() {
        for (field, default_value) in defaults {
            object.entry(field).or_insert(default_value);
        }
    }

    Ok(())
}

pub(in crate::collector::stats::playback) fn insert_cumulative_from_average(
    object: &mut Map<String, Value>,
    cumulative_field: &str,
    average_field: &str,
    sample_count_field: &str,
) -> SubtrActorResult<()> {
    if object.contains_key(cumulative_field) {
        return Ok(());
    }

    let average = object
        .get(average_field)
        .and_then(Value::as_f64)
        .unwrap_or(0.0) as f32;
    let sample_count = object
        .get(sample_count_field)
        .and_then(Value::as_u64)
        .unwrap_or(0) as f32;
    object.insert(
        cumulative_field.to_owned(),
        serialize_to_json_value(&(average * sample_count))?,
    );

    Ok(())
}

pub(in crate::collector::stats::playback) fn json_f32(value: &Value) -> Option<f32> {
    value.as_f64().map(|number| number as f32)
}

pub(in crate::collector::stats::playback) fn json_config_f32(
    config: Option<&Map<String, Value>>,
    key: &str,
    legacy_key: &str,
) -> Option<f32> {
    config.and_then(|config| {
        config
            .get(key)
            .or_else(|| config.get(legacy_key))
            .and_then(json_f32)
    })
}
