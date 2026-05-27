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
