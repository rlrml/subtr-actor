use super::*;

pub(crate) fn explicit_demolish_events(
    frame: &FrameInfo,
    events: &[SaDemolishEvent],
) -> Vec<DemolishInfo> {
    events
        .iter()
        .map(|event| {
            let (frame_number, time) = event_frame_and_time(frame, event.timing);
            DemolishInfo {
                time,
                seconds_remaining: event_seconds_remaining(frame, event.timing),
                frame: frame_number,
                attacker: player_id(event.attacker_index),
                victim: player_id(event.victim_index),
                attacker_velocity: vec3(event.attacker_velocity),
                victim_velocity: vec3(event.victim_velocity),
                victim_location: vec3(event.victim_location),
            }
        })
        .collect()
}
