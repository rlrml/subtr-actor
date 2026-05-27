use super::*;

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
