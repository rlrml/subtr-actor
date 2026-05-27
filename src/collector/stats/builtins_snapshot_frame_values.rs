use super::*;

pub(super) fn player_stats_snapshot_value<T: Serialize>(
    player_stats: &HashMap<PlayerId, T>,
) -> SubtrActorResult<Value> {
    serialize_to_json_value(&PlayerStatsExport {
        player_stats: player_stats_entries(player_stats),
    })
}

pub(super) fn team_player_stats_snapshot_value<Team: Serialize, Player: Serialize>(
    team_zero: &Team,
    team_one: &Team,
    player_stats: &HashMap<PlayerId, Player>,
) -> SubtrActorResult<Value> {
    serialize_to_json_value(&TeamPlayerStatsExport {
        team_zero,
        team_one,
        player_stats: player_stats_entries(player_stats),
    })
}
