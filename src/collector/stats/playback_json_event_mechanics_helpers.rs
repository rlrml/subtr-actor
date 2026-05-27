use super::*;

pub(in crate::collector::stats::playback) fn moment_mechanic_event(
    kind: &str,
    index: usize,
    frame: usize,
    time: f32,
    player_id: PlayerId,
    is_team_0: bool,
) -> MechanicEvent {
    MechanicEvent {
        id: format!("{kind}:{frame}:{index}"),
        kind: kind.to_owned(),
        player_id,
        is_team_0,
        timing: MechanicTiming::Moment { frame, time },
        properties: Vec::new(),
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::collector::stats::playback) fn span_mechanic_event(
    kind: &str,
    index: usize,
    start_frame: usize,
    end_frame: usize,
    start_time: f32,
    end_time: f32,
    player_id: PlayerId,
    is_team_0: bool,
) -> MechanicEvent {
    MechanicEvent {
        id: format!("{kind}:{start_frame}:{end_frame}:{index}"),
        kind: kind.to_owned(),
        player_id,
        is_team_0,
        timing: MechanicTiming::Span {
            start_frame,
            end_frame,
            start_time,
            end_time,
        },
        properties: Vec::new(),
    }
}

pub(in crate::collector::stats::playback) fn mechanic_event_start_time(
    event: &MechanicEvent,
) -> f32 {
    match event.timing {
        MechanicTiming::Moment { time, .. } => time,
        MechanicTiming::Span { start_time, .. } => start_time,
    }
}

pub(in crate::collector::stats::playback) fn mechanic_event_text_property(
    key: &str,
    value: &str,
) -> MechanicEventProperty {
    MechanicEventProperty {
        key: key.to_owned(),
        value: MechanicEventPropertyValue::Text(value.to_owned()),
    }
}

pub(in crate::collector::stats::playback) fn mechanic_event_unsigned_property(
    key: &str,
    value: u32,
) -> MechanicEventProperty {
    MechanicEventProperty {
        key: key.to_owned(),
        value: MechanicEventPropertyValue::Unsigned(value),
    }
}

pub(in crate::collector::stats::playback) fn ball_carry_mechanic_event_properties(
    object: &serde_json::Map<String, Value>,
) -> Vec<MechanicEventProperty> {
    let mut properties = Vec::new();
    if let Some(origin) = object.get("air_dribble_origin").and_then(Value::as_str) {
        properties.push(mechanic_event_text_property("origin", origin));
    }
    if let Some(touch_count) = object.get("touch_count").and_then(Value::as_u64) {
        properties.push(mechanic_event_unsigned_property(
            "touch_count",
            touch_count as u32,
        ));
    }
    properties
}
