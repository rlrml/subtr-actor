use serde::Serialize;
use serde_json::{json, Value};
use subtr_actor::GoalEvent;

use super::config::Config;

#[derive(Debug, Clone)]
pub(crate) struct MechanicCandidate {
    pub(crate) mechanic: &'static str,
    pub(crate) mechanic_label: &'static str,
    pub(crate) detector: &'static str,
    pub(crate) player_id: Option<String>,
    pub(crate) is_team_0: Option<bool>,
    pub(crate) event_time: f32,
    pub(crate) event_frame: usize,
    pub(crate) start_time: f32,
    pub(crate) end_time: f32,
    pub(crate) confidence: Option<f32>,
    pub(crate) reason: String,
    pub(crate) event: Value,
}

pub(crate) fn confidence_pct(confidence: f32) -> u32 {
    (confidence * 100.0).round().clamp(0.0, 100.0) as u32
}

pub(crate) fn include_candidate(candidate: &MechanicCandidate, config: &Config) -> bool {
    candidate
        .confidence
        .map(|confidence| confidence >= config.min_confidence)
        .unwrap_or(true)
}

pub(crate) fn followup_goal_for_candidate<'a>(
    candidate: &MechanicCandidate,
    goal_events: &'a [GoalEvent],
    config: &Config,
) -> Option<&'a GoalEvent> {
    goal_events
        .iter()
        .filter(|goal| {
            candidate
                .is_team_0
                .map(|is_team_0| goal.scoring_team_is_team_0 == is_team_0)
                .unwrap_or(true)
        })
        .filter(|goal| goal.time >= candidate.event_time)
        .filter(|goal| goal.time - candidate.event_time <= config.goal_lookahead_seconds)
        .min_by(|left, right| left.time.total_cmp(&right.time))
}

pub(crate) fn replay_duration_seconds(replay: &boxcars::Replay) -> f32 {
    replay
        .network_frames
        .as_ref()
        .and_then(|frames| frames.frames.last())
        .map(|frame| frame.time)
        .unwrap_or(0.0)
}

pub(crate) fn enforce_min_clip_duration(
    start_time: f32,
    end_time: f32,
    replay_duration: f32,
    min_clip_seconds: f32,
) -> (f32, f32) {
    let mut start_time = start_time.clamp(0.0, replay_duration.max(0.0));
    let mut end_time = end_time.clamp(start_time, replay_duration.max(start_time));
    let duration = end_time - start_time;
    if duration >= min_clip_seconds {
        return (start_time, end_time);
    }

    let missing = min_clip_seconds - duration;
    let extend_after = missing.min((replay_duration - end_time).max(0.0));
    end_time += extend_after;
    let remaining = missing - extend_after;
    start_time = (start_time - remaining).max(0.0);
    (start_time, end_time)
}

pub(crate) fn event_json<T: Serialize>(event: &T) -> Value {
    serde_json::to_value(event).unwrap_or_else(|_| json!({ "serializationError": true }))
}
