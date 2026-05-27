use super::*;

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
