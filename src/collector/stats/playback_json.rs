use boxcars::{Ps4Id, PsyNetId, RemoteId, SwitchId};

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

pub(in crate::collector::stats::playback) fn parse_timeline_event(
    value: &Value,
) -> SubtrActorResult<TimelineEvent> {
    let object = json_object(value, "timeline event")?;
    Ok(TimelineEvent {
        time: json_required_f32(object, "time")?,
        frame: json_optional_usize(object.get("frame"))?,
        kind: decode_json_value(json_required_value(object, "kind")?.clone())?,
        player_id: json_optional_remote_id(object.get("player_id"))?,
        player_position: json_optional_vec3(object.get("player_position"))?,
        is_team_0: json_optional_bool(object.get("is_team_0")),
    })
}

pub(in crate::collector::stats::playback) fn json_object<'a>(
    value: &'a Value,
    context: &str,
) -> SubtrActorResult<&'a serde_json::Map<String, Value>> {
    value.as_object().ok_or_else(|| {
        SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
            "Expected {context} to be a JSON object"
        )))
    })
}

pub(in crate::collector::stats::playback) fn json_required_value<'a>(
    object: &'a serde_json::Map<String, Value>,
    field: &str,
) -> SubtrActorResult<&'a Value> {
    object.get(field).ok_or_else(|| {
        SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
            "Missing JSON field '{field}'"
        )))
    })
}

pub(in crate::collector::stats::playback) fn json_required_array<'a>(
    object: &'a serde_json::Map<String, Value>,
    field: &str,
) -> SubtrActorResult<&'a Vec<Value>> {
    json_required_value(object, field)?
        .as_array()
        .ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
                "Expected JSON field '{field}' to be an array"
            )))
        })
}

pub(in crate::collector::stats::playback) fn json_optional_array(
    value: Option<&Value>,
) -> SubtrActorResult<&[Value]> {
    match value {
        Some(Value::Array(values)) => Ok(values),
        Some(_) => SubtrActorError::new_result(SubtrActorErrorVariant::StatsSerializationError(
            "Expected optional JSON value to be an array".to_owned(),
        )),
        None => Ok(&[]),
    }
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

pub(in crate::collector::stats::playback) fn json_required_f32(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> SubtrActorResult<f32> {
    json_f32(json_required_value(object, field)?).ok_or_else(|| {
        SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
            "Expected JSON field '{field}' to be a float"
        )))
    })
}

pub(in crate::collector::stats::playback) fn json_required_usize(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> SubtrActorResult<usize> {
    json_required_value(object, field)?
        .as_u64()
        .map(|number| number as usize)
        .ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
                "Expected JSON field '{field}' to be an unsigned integer"
            )))
        })
}

pub(in crate::collector::stats::playback) fn json_required_u32(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> SubtrActorResult<u32> {
    json_required_value(object, field)?
        .as_u64()
        .and_then(|number| u32::try_from(number).ok())
        .ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
                "Expected JSON field '{field}' to be an unsigned 32-bit integer"
            )))
        })
}

pub(in crate::collector::stats::playback) fn json_required_i32(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> SubtrActorResult<i32> {
    json_required_value(object, field)?
        .as_i64()
        .map(|number| number as i32)
        .ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
                "Expected JSON field '{field}' to be a signed integer"
            )))
        })
}

pub(in crate::collector::stats::playback) fn json_required_bool(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> SubtrActorResult<bool> {
    json_required_value(object, field)?
        .as_bool()
        .ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
                "Expected JSON field '{field}' to be a bool"
            )))
        })
}

pub(in crate::collector::stats::playback) fn json_required_str<'a>(
    object: &'a serde_json::Map<String, Value>,
    field: &str,
) -> SubtrActorResult<&'a str> {
    json_required_value(object, field)?.as_str().ok_or_else(|| {
        SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
            "Expected JSON field '{field}' to be a string"
        )))
    })
}

pub(in crate::collector::stats::playback) fn json_optional_bool(
    value: Option<&Value>,
) -> Option<bool> {
    value.and_then(Value::as_bool)
}

pub(in crate::collector::stats::playback) fn json_optional_f32(
    value: Option<&Value>,
) -> SubtrActorResult<Option<f32>> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(value) => json_f32(value).map(Some).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
                "Expected optional JSON value to be a float".to_owned(),
            ))
        }),
    }
}

pub(in crate::collector::stats::playback) fn json_optional_usize(
    value: Option<&Value>,
) -> SubtrActorResult<Option<usize>> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_u64()
            .map(|number| Some(number as usize))
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
                    "Expected optional JSON value to be an unsigned integer".to_owned(),
                ))
            }),
    }
}

pub(in crate::collector::stats::playback) fn json_optional_u32(
    value: Option<&Value>,
) -> SubtrActorResult<Option<u32>> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_u64()
            .and_then(|number| u32::try_from(number).ok())
            .map(Some)
            .ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
                    "Expected optional JSON value to be an unsigned 32-bit integer".to_owned(),
                ))
            }),
    }
}

pub(in crate::collector::stats::playback) fn json_goal_context_position(
    value: &Value,
) -> SubtrActorResult<GoalContextPosition> {
    let object = json_object(value, "goal context position")?;
    Ok(GoalContextPosition {
        x: json_required_f32(object, "x")?,
        y: json_required_f32(object, "y")?,
        z: json_required_f32(object, "z")?,
    })
}

pub(in crate::collector::stats::playback) fn json_optional_goal_context_position(
    value: Option<&Value>,
) -> SubtrActorResult<Option<GoalContextPosition>> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(value) => json_goal_context_position(value).map(Some),
    }
}

pub(in crate::collector::stats::playback) fn json_required_vec3(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> SubtrActorResult<[f32; 3]> {
    let array = json_required_value(object, field)?
        .as_array()
        .ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
                "Expected JSON field '{field}' to be a 3-element array"
            )))
        })?;
    if array.len() != 3 {
        return SubtrActorError::new_result(SubtrActorErrorVariant::StatsSerializationError(
            format!("Expected JSON field '{field}' to contain exactly 3 elements"),
        ));
    }
    Ok([
        json_f32(&array[0]).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
                "Expected JSON field '{field}[0]' to be a float"
            )))
        })?,
        json_f32(&array[1]).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
                "Expected JSON field '{field}[1]' to be a float"
            )))
        })?,
        json_f32(&array[2]).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
                "Expected JSON field '{field}[2]' to be a float"
            )))
        })?,
    ])
}

pub(in crate::collector::stats::playback) fn json_required_vec2(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> SubtrActorResult<[f32; 2]> {
    let array = json_required_value(object, field)?
        .as_array()
        .ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
                "Expected JSON field '{field}' to be a 2-element array"
            )))
        })?;
    if array.len() != 2 {
        return SubtrActorError::new_result(SubtrActorErrorVariant::StatsSerializationError(
            format!("Expected JSON field '{field}' to contain exactly 2 elements"),
        ));
    }
    Ok([
        json_f32(&array[0]).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
                "Expected JSON field '{field}[0]' to be a float"
            )))
        })?,
        json_f32(&array[1]).ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(format!(
                "Expected JSON field '{field}[1]' to be a float"
            )))
        })?,
    ])
}

pub(in crate::collector::stats::playback) fn json_optional_vec3(
    value: Option<&Value>,
) -> SubtrActorResult<Option<[f32; 3]>> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(value) => {
            let array = value.as_array().ok_or_else(|| {
                SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
                    "Expected optional JSON value to be a 3-element array".to_owned(),
                ))
            })?;
            if array.len() != 3 {
                return SubtrActorError::new_result(
                    SubtrActorErrorVariant::StatsSerializationError(
                        "Expected optional JSON value to contain exactly 3 elements".to_owned(),
                    ),
                );
            }
            Ok(Some([
                json_f32(&array[0]).ok_or_else(|| {
                    SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
                        "Expected optional JSON value[0] to be a float".to_owned(),
                    ))
                })?,
                json_f32(&array[1]).ok_or_else(|| {
                    SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
                        "Expected optional JSON value[1] to be a float".to_owned(),
                    ))
                })?,
                json_f32(&array[2]).ok_or_else(|| {
                    SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
                        "Expected optional JSON value[2] to be a float".to_owned(),
                    ))
                })?,
            ]))
        }
    }
}

pub(in crate::collector::stats::playback) fn json_required_remote_id(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> SubtrActorResult<PlayerId> {
    json_remote_id(json_required_value(object, field)?)
}

pub(in crate::collector::stats::playback) fn json_optional_remote_id(
    value: Option<&Value>,
) -> SubtrActorResult<Option<PlayerId>> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(value) => Ok(Some(json_remote_id(value)?)),
    }
}

pub(in crate::collector::stats::playback) fn json_remote_id(
    value: &Value,
) -> SubtrActorResult<PlayerId> {
    let object = json_object(value, "remote id")?;
    if object.len() != 1 {
        return SubtrActorError::new_result(SubtrActorErrorVariant::StatsSerializationError(
            "Expected remote id to contain exactly one variant".to_owned(),
        ));
    }

    let (variant, payload) = object.iter().next().expect("validated single variant");
    match variant.as_str() {
        "PlayStation" => {
            let payload = json_object(payload, "playstation remote id")?;
            Ok(RemoteId::PlayStation(Ps4Id {
                online_id: json_u64(json_required_value(payload, "online_id")?)?,
                name: json_required_value(payload, "name")?
                    .as_str()
                    .ok_or_else(|| {
                        SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
                            "Expected PlayStation name to be a string".to_owned(),
                        ))
                    })?
                    .to_owned(),
                unknown1: json_u8_vec(json_required_value(payload, "unknown1")?)?,
            }))
        }
        "PsyNet" => {
            let payload = json_object(payload, "psynet remote id")?;
            Ok(RemoteId::PsyNet(PsyNetId {
                online_id: json_u64(json_required_value(payload, "online_id")?)?,
                unknown1: json_u8_vec(json_required_value(payload, "unknown1")?)?,
            }))
        }
        "SplitScreen" => Ok(RemoteId::SplitScreen(json_u64(payload)? as u32)),
        "Steam" => Ok(RemoteId::Steam(json_u64(payload)?)),
        "Switch" => {
            let payload = json_object(payload, "switch remote id")?;
            Ok(RemoteId::Switch(SwitchId {
                online_id: json_u64(json_required_value(payload, "online_id")?)?,
                unknown1: json_u8_vec(json_required_value(payload, "unknown1")?)?,
            }))
        }
        "Xbox" => Ok(RemoteId::Xbox(json_u64(payload)?)),
        "QQ" => Ok(RemoteId::QQ(json_u64(payload)?)),
        "Epic" => Ok(RemoteId::Epic(
            payload
                .as_str()
                .ok_or_else(|| {
                    SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
                        "Expected Epic remote id payload to be a string".to_owned(),
                    ))
                })?
                .to_owned(),
        )),
        variant => SubtrActorError::new_result(SubtrActorErrorVariant::StatsSerializationError(
            format!("Unknown remote id variant '{variant}'"),
        )),
    }
}

pub(in crate::collector::stats::playback) fn json_u64(value: &Value) -> SubtrActorResult<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
        .ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
                "Expected JSON value to be a u64".to_owned(),
            ))
        })
}

pub(in crate::collector::stats::playback) fn json_u8_vec(
    value: &Value,
) -> SubtrActorResult<Vec<u8>> {
    value
        .as_array()
        .ok_or_else(|| {
            SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
                "Expected JSON value to be an array of bytes".to_owned(),
            ))
        })?
        .iter()
        .map(|entry| {
            entry
                .as_u64()
                .and_then(|number| u8::try_from(number).ok())
                .ok_or_else(|| {
                    SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
                        "Expected JSON array entry to be a byte".to_owned(),
                    ))
                })
        })
        .collect()
}
