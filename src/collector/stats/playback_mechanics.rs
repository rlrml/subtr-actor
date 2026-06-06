use super::playback_json::*;
use super::*;

pub(in crate::collector::stats::playback) fn moment_mechanic_event(
    kind: &str,
    index: usize,
    frame: usize,
    time: f32,
    player_id: PlayerId,
    is_team_0: bool,
) -> StatsTimelineTagEvent {
    StatsTimelineTagEvent {
        id: format!("{kind}:{frame}:{index}"),
        kind: kind.to_owned(),
        player_id,
        player_position: None,
        is_team_0,
        timing: StatsEventTiming::Moment { frame, time },
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
) -> StatsTimelineTagEvent {
    StatsTimelineTagEvent {
        id: format!("{kind}:{start_frame}:{end_frame}:{index}"),
        kind: kind.to_owned(),
        player_id,
        player_position: None,
        is_team_0,
        timing: StatsEventTiming::Span {
            start_frame,
            end_frame,
            start_time,
            end_time,
        },
        properties: Vec::new(),
    }
}

pub(in crate::collector::stats::playback) fn mechanic_event_start_time(
    event: &StatsTimelineTagEvent,
) -> f32 {
    match event.timing {
        StatsEventTiming::Moment { time, .. } => time,
        StatsEventTiming::Span { start_time, .. } => start_time,
    }
}

pub(in crate::collector::stats::playback) fn mechanic_event_text_property(
    key: &str,
    value: &str,
) -> StatsEventProperty {
    StatsEventProperty {
        key: key.to_owned(),
        value: StatsEventPropertyValue::Text(value.to_owned()),
    }
}

pub(in crate::collector::stats::playback) fn mechanic_event_unsigned_property(
    key: &str,
    value: u32,
) -> StatsEventProperty {
    StatsEventProperty {
        key: key.to_owned(),
        value: StatsEventPropertyValue::Unsigned(value),
    }
}

pub(in crate::collector::stats::playback) fn mechanic_event_float_property(
    key: &str,
    value: f32,
) -> StatsEventProperty {
    StatsEventProperty {
        key: key.to_owned(),
        value: StatsEventPropertyValue::Float(value),
    }
}

pub(in crate::collector::stats::playback) fn flick_mechanic_event_properties(
    object: &serde_json::Map<String, Value>,
) -> Vec<StatsEventProperty> {
    vec![
        mechanic_event_text_property(
            "flick_kind",
            object
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or("other"),
        ),
        mechanic_event_text_property(
            "setup_rotation_direction",
            object
                .get("setup_rotation_direction")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
        ),
        mechanic_event_float_property(
            "setup_rotation_degrees",
            object
                .get("setup_rotation_degrees")
                .and_then(Value::as_f64)
                .unwrap_or(0.0) as f32,
        ),
    ]
}

pub(in crate::collector::stats::playback) fn ball_carry_mechanic_event_properties(
    object: &serde_json::Map<String, Value>,
) -> Vec<StatsEventProperty> {
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

pub(in crate::collector::stats::playback) fn parse_ball_carry_mechanic_event(
    value: &Value,
    index: usize,
) -> SubtrActorResult<StatsTimelineTagEvent> {
    let object = json_object(value, "ball carry mechanic event")?;
    let serialized_kind = json_required_str(object, "kind")?;
    let kind = match serialized_kind {
        "carry" => "ball_carry",
        "air_dribble" => "air_dribble",
        other => other,
    };
    let mut mechanic_event = span_mechanic_event(
        kind,
        index,
        json_required_usize(object, "start_frame")?,
        json_required_usize(object, "end_frame")?,
        json_required_f32(object, "start_time")?,
        json_required_f32(object, "end_time")?,
        json_required_remote_id(object, "player_id")?,
        json_required_bool(object, "is_team_0")?,
    );
    if kind == "air_dribble" {
        mechanic_event.properties = ball_carry_mechanic_event_properties(object);
    }
    Ok(mechanic_event)
}

pub(in crate::collector::stats::playback) fn parse_dodge_reset_mechanic_event(
    value: &Value,
    index: usize,
) -> SubtrActorResult<StatsTimelineTagEvent> {
    let object = json_object(value, "dodge reset mechanic event")?;
    Ok(moment_mechanic_event(
        "flip_reset",
        index,
        json_required_usize(object, "frame")?,
        json_required_f32(object, "time")?,
        json_required_remote_id(object, "player")?,
        json_required_bool(object, "is_team_0")?,
    ))
}
