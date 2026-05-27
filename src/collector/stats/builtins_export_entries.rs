use super::*;

pub(crate) fn player_stats_entries<'a, T>(
    player_stats: &'a HashMap<PlayerId, T>,
) -> Vec<PlayerStatsEntry<'a, T>> {
    let mut entries: Vec<_> = player_stats
        .iter()
        .map(|(player_id, stats)| PlayerStatsEntry {
            player_id: player_id.clone(),
            stats,
        })
        .collect();
    entries.sort_by(|left, right| {
        format!("{:?}", left.player_id).cmp(&format!("{:?}", right.player_id))
    });
    entries
}

#[derive(Serialize)]
pub(crate) struct PlayerStatsEntry<'a, T> {
    pub(crate) player_id: PlayerId,
    pub(crate) stats: &'a T,
}

#[derive(Serialize)]
pub(crate) struct OwnedPlayerStatsEntry<T> {
    pub(crate) player_id: PlayerId,
    pub(crate) stats: T,
}
