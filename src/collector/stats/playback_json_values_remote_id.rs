use super::*;

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
