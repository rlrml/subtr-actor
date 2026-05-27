use super::*;

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
