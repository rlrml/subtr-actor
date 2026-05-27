use super::*;

impl MatchStatsCalculator {
    pub(super) fn last_defender(
        &self,
        players: &PlayerFrameState,
        defending_team_is_team_0: bool,
    ) -> Option<PlayerId> {
        players
            .players
            .iter()
            .filter(|player| player.is_team_0 == defending_team_is_team_0)
            .filter_map(|player| {
                player
                    .position()
                    .map(|position| (player.player_id.clone(), position.y))
            })
            .reduce(|current, candidate| {
                back_player_by_side(current, candidate, defending_team_is_team_0)
            })
            .map(|(player_id, _)| player_id)
    }

    pub(super) fn most_back_player(
        players: &PlayerFrameState,
        team_is_team_0: bool,
    ) -> Option<PlayerId> {
        players
            .players
            .iter()
            .filter(|player| player.is_team_0 == team_is_team_0)
            .filter_map(|player| {
                player.position().map(|position| {
                    (
                        player.player_id.clone(),
                        normalized_y(team_is_team_0, position),
                    )
                })
            })
            .min_by(|left, right| left.1.total_cmp(&right.1))
            .map(|(player_id, _)| player_id)
    }

    pub(super) fn player_position(
        players: &PlayerFrameState,
        player_id: &PlayerId,
    ) -> Option<glam::Vec3> {
        players
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
            .and_then(PlayerSample::position)
    }
}

fn back_player_by_side(
    current: (PlayerId, f32),
    candidate: (PlayerId, f32),
    defending_team_is_team_0: bool,
) -> (PlayerId, f32) {
    if defending_team_is_team_0 {
        if candidate.1 < current.1 {
            candidate
        } else {
            current
        }
    } else if candidate.1 > current.1 {
        candidate
    } else {
        current
    }
}
