use super::*;

pub(super) fn append_center_events(events: &mut Vec<MechanicEvent>, center: &CenterCalculator) {
    for (index, event) in center.events().iter().enumerate() {
        events.push(span_mechanic_event(
            MECHANIC_CENTER,
            index,
            event.start_frame,
            event.frame,
            event.start_time,
            event.time,
            event.player.clone(),
            event.is_team_0,
        ));
    }
}

pub(super) fn append_double_tap_events(
    events: &mut Vec<MechanicEvent>,
    double_tap: &DoubleTapCalculator,
) {
    for (index, event) in double_tap.events().iter().enumerate() {
        events.push(span_mechanic_event(
            MECHANIC_DOUBLE_TAP,
            index,
            event.backboard_frame,
            event.frame,
            event.backboard_time,
            event.time,
            event.player.clone(),
            event.is_team_0,
        ));
    }
}

pub(super) fn append_flick_events(events: &mut Vec<MechanicEvent>, flick: &FlickCalculator) {
    for (index, event) in flick.events().iter().enumerate() {
        events.push(span_mechanic_event(
            MECHANIC_FLICK,
            index,
            event.setup_start_frame,
            event.frame,
            event.setup_start_time,
            event.time,
            event.player.clone(),
            event.is_team_0,
        ));
    }
}

pub(super) fn append_musty_flick_events(
    events: &mut Vec<MechanicEvent>,
    musty_flick: &MustyFlickCalculator,
) {
    for (index, event) in musty_flick.events().iter().enumerate() {
        events.push(span_mechanic_event(
            MECHANIC_MUSTY_FLICK,
            index,
            event.dodge_frame,
            event.frame,
            event.dodge_time,
            event.time,
            event.player.clone(),
            event.is_team_0,
        ));
    }
}
