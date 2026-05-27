use super::*;

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

    for (cumulative_field, average_field, sample_count_field) in CUMULATIVE_AVERAGE_FIELDS {
        insert_cumulative_from_average(
            object,
            cumulative_field,
            average_field,
            sample_count_field,
        )?;
    }

    if let Value::Object(defaults) = default_json_value::<CorePlayerStats>() {
        for (field, default_value) in defaults {
            object.entry(field).or_insert(default_value);
        }
    }

    Ok(())
}

fn insert_cumulative_from_average(
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
