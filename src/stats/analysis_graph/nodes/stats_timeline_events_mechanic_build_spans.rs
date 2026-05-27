use super::*;

pub(super) fn append_ball_carry_events(
    events: &mut Vec<MechanicEvent>,
    ball_carry: &BallCarryCalculator,
) {
    for (index, event) in ball_carry.carry_events().iter().enumerate() {
        let kind = match event.kind {
            BallCarryKind::Carry => MECHANIC_BALL_CARRY,
            BallCarryKind::AirDribble => MECHANIC_AIR_DRIBBLE,
        };
        let mut mechanic_event = span_mechanic_event(
            kind,
            index,
            event.start_frame,
            event.end_frame,
            event.start_time,
            event.end_time,
            event.player_id.clone(),
            event.is_team_0,
        );
        mechanic_event.properties = ball_carry_mechanic_event_properties(event);
        events.push(mechanic_event);
    }
}

pub(super) fn append_span_mechanic_events(
    events: &mut Vec<MechanicEvent>,
    sources: &MechanicEventSources<'_>,
) {
    append_ceiling_shot_events(events, sources.ceiling_shot);
    append_wall_aerial_events(events, sources.wall_aerial);
    append_wall_aerial_shot_events(events, sources.wall_aerial_shot);
    append_center_events(events, sources.center);
    append_double_tap_events(events, sources.double_tap);
    append_flick_events(events, sources.flick);
    append_musty_flick_events(events, sources.musty_flick);
    append_one_timer_events(events, sources.one_timer);
    append_pass_events(events, sources.pass);
}

fn ball_carry_mechanic_event_properties(event: &BallCarryEvent) -> Vec<MechanicEventProperty> {
    let mut properties = Vec::new();
    if let Some(origin) = event.air_dribble_origin {
        properties.push(mechanic_event_text_property(
            "origin",
            origin.as_label_value(),
        ));
    }
    if event.kind == BallCarryKind::AirDribble {
        properties.push(mechanic_event_unsigned_property(
            "touch_count",
            event.touch_count,
        ));
    }
    properties
}
