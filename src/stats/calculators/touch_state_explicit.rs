use super::*;

impl TouchStateCalculator {
    pub(super) fn candidate_for_player(&self, player_id: &PlayerId) -> Option<TouchEvent> {
        self.recent_touch_candidates.get(player_id).cloned()
    }

    pub(super) fn best_candidate_for_team(&self, team_is_team_0: bool) -> Option<TouchEvent> {
        self.recent_touch_candidates
            .values()
            .filter(|candidate| candidate.team_is_team_0 == team_is_team_0)
            .min_by(|left, right| touch_distance(left).total_cmp(&touch_distance(right)))
            .cloned()
    }

    pub(super) fn enrich_explicit_touch_event(&self, event: &TouchEvent) -> TouchEvent {
        let candidate = if let Some(player_id) = event.player.as_ref() {
            self.candidate_for_player(player_id)
        } else {
            self.best_candidate_for_team(event.team_is_team_0)
        };
        let Some(candidate) = candidate else {
            return event.clone();
        };

        TouchEvent {
            player: event.player.clone().or(candidate.player),
            closest_approach_distance: event
                .closest_approach_distance
                .or(candidate.closest_approach_distance),
            dodge_contact: event.dodge_contact || candidate.dodge_contact,
            ..event.clone()
        }
    }

    pub(super) fn contested_touch_candidates(&self, primary: &TouchEvent) -> Vec<TouchEvent> {
        const CONTESTED_TOUCH_DISTANCE_MARGIN: f32 = 80.0;

        let primary_distance = touch_distance(primary);
        let best_opposing_candidate = self
            .recent_touch_candidates
            .values()
            .filter(|candidate| candidate.team_is_team_0 != primary.team_is_team_0)
            .filter(|candidate| {
                touch_distance(candidate) <= primary_distance + CONTESTED_TOUCH_DISTANCE_MARGIN
            })
            .min_by(|left, right| touch_distance(left).total_cmp(&touch_distance(right)))
            .cloned();

        best_opposing_candidate.into_iter().collect()
    }
}
