use serde_json::json;
use subtr_actor::{
    playlist_generation::{PlaybackBound, PlaybackBoundKind, PlaylistManifestItem},
    GoalEvent,
};

use super::candidate::{confidence_pct, event_json, MechanicCandidate};
use super::players::{player_team_label, PlayerDisplay};
use super::source_types::ReplaySourceInput;

pub(crate) fn build_playlist_item(
    source: &ReplaySourceInput,
    candidate: MechanicCandidate,
    player: Option<&PlayerDisplay>,
    start_time: f32,
    end_time: f32,
    followup_goal: Option<&GoalEvent>,
) -> PlaylistManifestItem {
    let player_label = player
        .map(|display| display.name.as_str())
        .or(candidate.player_id.as_deref())
        .unwrap_or("team event");
    let score = candidate
        .confidence
        .map(|confidence| format!(" {}%", confidence_pct(confidence)))
        .unwrap_or_default();
    let id = format!(
        "{}:{}:{}:{}",
        candidate.mechanic,
        source.source_id,
        candidate.event_frame,
        candidate.player_id.as_deref().unwrap_or("team")
    );

    PlaylistManifestItem {
        id: id.clone(),
        replay: source.source_id.clone(),
        start: PlaybackBound {
            kind: PlaybackBoundKind::Time,
            value: start_time,
        },
        end: PlaybackBound {
            kind: PlaybackBoundKind::Time,
            value: end_time,
        },
        label: format!("{}{score} - {player_label}", candidate.mechanic_label),
        meta: json!({
            "itemId": id,
            "mechanic": candidate.mechanic,
            "mechanicLabel": candidate.mechanic_label,
            "detector": candidate.detector,
            "confidence": candidate.confidence,
            "reason": candidate.reason,
            "playerId": candidate.player_id,
            "playerName": player.map(|display| display.name.clone()),
            "team": player.map(|display| display.team).or_else(|| candidate.is_team_0.map(player_team_label)),
            "target": {
                "kind": "player-span",
                "playerId": candidate.player_id,
                "startTime": start_time,
                "endTime": end_time,
                "mechanicStartTime": candidate.start_time,
                "mechanicEndTime": candidate.end_time,
                "eventTime": candidate.event_time,
                "eventFrame": candidate.event_frame,
                "goalTime": followup_goal.map(|goal| goal.time),
                "goalFrame": followup_goal.map(|goal| goal.frame),
            },
            "followupGoal": followup_goal.map(event_json),
            "event": candidate.event,
        }),
    }
}
