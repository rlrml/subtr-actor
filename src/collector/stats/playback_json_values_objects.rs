use super::*;

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
