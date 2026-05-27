use super::*;

pub(in crate::collector::stats::playback) fn parse_boost_pickup_comparison_event(
    value: &Value,
) -> SubtrActorResult<BoostPickupComparisonEvent> {
    let object = json_object(value, "boost pickup comparison event")?;
    Ok(BoostPickupComparisonEvent {
        comparison: decode_json_value(json_required_value(object, "comparison")?.clone())?,
        frame: json_required_usize(object, "frame")?,
        time: json_required_f32(object, "time")?,
        player_id: json_required_remote_id(object, "player_id")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        pad_type: decode_json_value(json_required_value(object, "pad_type")?.clone())?,
        field_half: decode_json_value(json_required_value(object, "field_half")?.clone())?,
        activity: decode_json_value(json_required_value(object, "activity")?.clone())?,
        reported_frame: json_optional_usize(object.get("reported_frame"))?,
        reported_time: json_optional_f32(object.get("reported_time"))?,
        inferred_frame: json_optional_usize(object.get("inferred_frame"))?,
        inferred_time: json_optional_f32(object.get("inferred_time"))?,
        boost_before: json_optional_f32(object.get("boost_before"))?,
        boost_after: json_optional_f32(object.get("boost_after"))?,
    })
}

pub(in crate::collector::stats::playback) fn parse_boost_ledger_event(
    value: &Value,
) -> SubtrActorResult<BoostLedgerEvent> {
    let object = json_object(value, "boost ledger event")?;
    Ok(BoostLedgerEvent {
        frame: json_required_usize(object, "frame")?,
        time: json_required_f32(object, "time")?,
        player_id: json_required_remote_id(object, "player_id")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        transaction: decode_json_value(json_required_value(object, "transaction")?.clone())?,
        amount: json_required_f32(object, "amount")?,
        count: json_required_usize(object, "count")? as u32,
        labels: decode_json_value(
            object
                .get("labels")
                .cloned()
                .unwrap_or_else(|| Value::Array(Vec::new())),
        )?,
        boost_before: json_optional_f32(object.get("boost_before"))?,
        boost_after: json_optional_f32(object.get("boost_after"))?,
    })
}

pub(in crate::collector::stats::playback) fn parse_boost_state_event(
    value: &Value,
) -> SubtrActorResult<BoostStateEvent> {
    let object = json_object(value, "boost state event")?;
    Ok(BoostStateEvent {
        frame: json_required_usize(object, "frame")?,
        time: json_required_f32(object, "time")?,
        player_id: json_required_remote_id(object, "player_id")?,
        is_team_0: json_required_bool(object, "is_team_0")?,
        boost_amount: json_required_f32(object, "boost_amount")?,
        boost_before: json_optional_f32(object.get("boost_before"))?,
    })
}
