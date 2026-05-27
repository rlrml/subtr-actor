use super::*;

pub(super) fn append_moment_mechanic_events(
    events: &mut Vec<MechanicEvent>,
    sources: &MechanicEventSources<'_>,
) {
    for (index, event) in sources
        .dodge_reset
        .confirmed_flip_reset_events()
        .iter()
        .enumerate()
    {
        events.push(moment_mechanic_event(
            MECHANIC_FLIP_RESET,
            index,
            event.frame,
            event.time,
            event.player.clone(),
            event.is_team_0,
        ));
    }
    append_speed_flip_events(events, sources.speed_flip);
    append_half_flip_events(events, sources.half_flip);
    append_half_volley_events(events, sources.half_volley);
}

fn append_speed_flip_events(events: &mut Vec<MechanicEvent>, speed_flip: &SpeedFlipCalculator) {
    for (index, event) in speed_flip.events().iter().enumerate() {
        events.push(moment_mechanic_event(
            MECHANIC_SPEED_FLIP,
            index,
            event.frame,
            event.time,
            event.player.clone(),
            event.is_team_0,
        ));
    }
}

fn append_half_flip_events(events: &mut Vec<MechanicEvent>, half_flip: &HalfFlipCalculator) {
    for (index, event) in half_flip.events().iter().enumerate() {
        events.push(moment_mechanic_event(
            MECHANIC_HALF_FLIP,
            index,
            event.frame,
            event.time,
            event.player.clone(),
            event.is_team_0,
        ));
    }
}

fn append_half_volley_events(events: &mut Vec<MechanicEvent>, half_volley: &HalfVolleyCalculator) {
    for (index, event) in half_volley.events().iter().enumerate() {
        events.push(moment_mechanic_event(
            MECHANIC_HALF_VOLLEY,
            index,
            event.frame,
            event.time,
            event.player.clone(),
            event.is_team_0,
        ));
    }
}
