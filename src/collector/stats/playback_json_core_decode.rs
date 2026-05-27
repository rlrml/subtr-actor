use super::*;

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

pub(in crate::collector::stats::playback) fn parse_timeline_event(
    value: &Value,
) -> SubtrActorResult<TimelineEvent> {
    let object = json_object(value, "timeline event")?;
    Ok(TimelineEvent {
        time: json_required_f32(object, "time")?,
        frame: json_optional_usize(object.get("frame"))?,
        kind: decode_json_value(json_required_value(object, "kind")?.clone())?,
        player_id: json_optional_remote_id(object.get("player_id"))?,
        is_team_0: json_optional_bool(object.get("is_team_0")),
    })
}
