use super::*;

pub(super) fn append_one_timer_events(
    events: &mut Vec<MechanicEvent>,
    one_timer: &OneTimerCalculator,
) {
    for (index, event) in one_timer.events().iter().enumerate() {
        events.push(span_mechanic_event(
            MECHANIC_ONE_TIMER,
            index,
            event.pass_start_frame,
            event.frame,
            event.pass_start_time,
            event.time,
            event.player.clone(),
            event.is_team_0,
        ));
    }
}

pub(super) fn append_pass_events(events: &mut Vec<MechanicEvent>, pass: &PassCalculator) {
    for (index, event) in pass.events().iter().enumerate() {
        events.push(span_mechanic_event(
            MECHANIC_PASS,
            index,
            event.start_frame,
            event.frame,
            event.start_time,
            event.time,
            event.passer.clone(),
            event.is_team_0,
        ));
    }
}

pub(super) fn append_wavedash_events(
    events: &mut Vec<MechanicEvent>,
    wavedash: &WavedashCalculator,
) {
    for (index, event) in wavedash.events().iter().enumerate() {
        events.push(span_mechanic_event(
            MECHANIC_WAVEDASH,
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
