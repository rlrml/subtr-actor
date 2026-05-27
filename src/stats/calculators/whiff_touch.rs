use super::*;
use std::collections::HashSet;

impl WhiffCalculator {
    pub(super) fn finish_touched_candidates(
        &mut self,
        frame: &FrameInfo,
        touch_state: &TouchState,
    ) {
        let touched_players = touch_state
            .touch_events
            .iter()
            .filter_map(|touch| touch.player.as_ref())
            .collect::<HashSet<_>>();
        let touched_teams = touch_state
            .touch_events
            .iter()
            .map(|touch| touch.team_is_team_0)
            .collect::<HashSet<_>>();
        if touched_players.is_empty() && touched_teams.is_empty() {
            return;
        }

        let candidate_players = self.active_candidates.keys().cloned().collect::<Vec<_>>();
        for player_id in candidate_players {
            let Some(candidate) = self.active_candidates.remove(&player_id) else {
                continue;
            };
            if touched_players.contains(&candidate.player) {
                continue;
            }
            if touched_teams.contains(&!candidate.is_team_0) {
                self.emit_candidate(candidate, frame, WhiffEventKind::BeatenToBall);
            }
        }
    }
}
