use super::*;

impl BumpCalculator {
    pub(super) fn active_fifty_fifty_matches_pair(
        event: &ActiveFiftyFifty,
        left: &PlayerId,
        right: &PlayerId,
    ) -> bool {
        Self::optional_player_pair_matches(
            event.team_zero_player.as_ref(),
            event.team_one_player.as_ref(),
            left,
            right,
        )
    }

    pub(super) fn resolved_fifty_fifty_matches_pair(
        event: &FiftyFiftyEvent,
        left: &PlayerId,
        right: &PlayerId,
    ) -> bool {
        Self::optional_player_pair_matches(
            event.team_zero_player.as_ref(),
            event.team_one_player.as_ref(),
            left,
            right,
        )
    }

    pub(super) fn optional_player_pair_matches(
        team_zero_player: Option<&PlayerId>,
        team_one_player: Option<&PlayerId>,
        left: &PlayerId,
        right: &PlayerId,
    ) -> bool {
        matches!(
            (team_zero_player, team_one_player),
            (Some(team_zero_player), Some(team_one_player))
                if (team_zero_player == left && team_one_player == right)
                    || (team_zero_player == right && team_one_player == left)
        )
    }
}
