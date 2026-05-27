use super::*;

pub(crate) const MIN_BOOST_PAD_RESPAWN_SECONDS: f32 = 4.0;
pub(crate) const GOAL_EVENT_DEDUPE_WINDOW_SECONDS: f32 = 3.0;
pub(crate) const MAX_DEMOLISH_KNOWN_FRAMES_PASSED: usize = 150;

pub(crate) fn boost_pad_pickup_sequence_is_recent(
    sequence_times: &HashMap<(String, u8), f32>,
    pad_id: &str,
    sequence: u8,
    event_time: f32,
) -> bool {
    sequence_times
        .get(&(pad_id.to_owned(), sequence))
        .is_some_and(|last_time| {
            let elapsed = event_time - *last_time;
            (0.0..MIN_BOOST_PAD_RESPAWN_SECONDS).contains(&elapsed)
        })
}

pub(crate) fn demolish_is_known(
    known_demolishes: &[(DemoEventSample, usize)],
    sample: &DemoEventSample,
    frame_number: usize,
) -> bool {
    known_demolishes.iter().any(|(existing, existing_frame)| {
        existing.attacker == sample.attacker
            && existing.victim == sample.victim
            && frame_number.abs_diff(*existing_frame) < MAX_DEMOLISH_KNOWN_FRAMES_PASSED
    })
}

pub(crate) fn goal_event_is_duplicate(previous: &GoalEvent, candidate: &GoalEvent) -> bool {
    match (
        candidate.team_zero_score,
        candidate.team_one_score,
        previous.team_zero_score,
        previous.team_one_score,
    ) {
        (Some(team_zero), Some(team_one), Some(prev_team_zero), Some(prev_team_one)) => {
            team_zero == prev_team_zero && team_one == prev_team_one
        }
        _ => {
            previous.scoring_team_is_team_0 == candidate.scoring_team_is_team_0
                && (candidate.time - previous.time).abs() <= GOAL_EVENT_DEDUPE_WINDOW_SECONDS
        }
    }
}
