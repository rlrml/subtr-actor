use super::*;

pub(crate) fn event_frame_and_time(frame: &FrameInfo, timing: SaEventTiming) -> (usize, f32) {
    if timing.has_timing != 0 {
        (timing.frame_number as usize, timing.time)
    } else {
        (frame.frame_number, frame.time)
    }
}

pub(crate) fn event_seconds_remaining(frame: &FrameInfo, timing: SaEventTiming) -> i32 {
    if timing.has_seconds_remaining != 0 {
        timing.seconds_remaining
    } else {
        frame.seconds_remaining.unwrap_or_default()
    }
}

pub(crate) fn explicit_touch_events(frame: &FrameInfo, events: &[SaTouchEvent]) -> Vec<TouchEvent> {
    let mut accepted = Vec::new();
    let mut seen = HashSet::new();
    for event in events {
        let (frame_number, time) = event_frame_and_time(frame, event.timing);
        let player = (event.has_player != 0).then_some(player_id(event.player_index));
        let team_is_team_0 = event.is_team_0 != 0;
        if !seen.insert((frame_number, player.clone(), team_is_team_0)) {
            continue;
        }
        accepted.push(TouchEvent {
            time,
            frame: frame_number,
            team_is_team_0,
            player,
            closest_approach_distance: (event.has_closest_approach_distance != 0)
                .then_some(event.closest_approach_distance),
            dodge_contact: false,
        });
    }
    accepted
}
