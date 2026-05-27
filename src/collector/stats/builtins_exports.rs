use super::*;

#[path = "builtins_export_entries.rs"]
mod builtins_export_entries;
pub(crate) use builtins_export_entries::*;

#[derive(Serialize)]
pub(crate) struct PlayerStatsExport<'a, T> {
    pub(crate) player_stats: Vec<PlayerStatsEntry<'a, T>>,
}

#[derive(Serialize)]
pub(crate) struct OwnedPlayerStatsExport<T> {
    pub(crate) player_stats: Vec<OwnedPlayerStatsEntry<T>>,
}

#[derive(Serialize)]
pub(crate) struct PlayerStatsWithEventsExport<'a, T, E> {
    pub(crate) player_stats: Vec<PlayerStatsEntry<'a, T>>,
    pub(crate) events: &'a [E],
}

#[derive(Serialize)]
pub(crate) struct TeamPlayerStatsExport<'a, Team, Player> {
    pub(crate) team_zero: &'a Team,
    pub(crate) team_one: &'a Team,
    pub(crate) player_stats: Vec<PlayerStatsEntry<'a, Player>>,
}

#[derive(Serialize)]
pub(crate) struct TeamOwnedPlayerStatsExport<'a, Team, Player> {
    pub(crate) team_zero: &'a Team,
    pub(crate) team_one: &'a Team,
    pub(crate) player_stats: Vec<OwnedPlayerStatsEntry<Player>>,
}

#[derive(Serialize)]
pub(crate) struct TeamPlayerStatsWithEventsExport<'a, Team, Player, Event> {
    pub(crate) team_zero: &'a Team,
    pub(crate) team_one: &'a Team,
    pub(crate) player_stats: Vec<PlayerStatsEntry<'a, Player>>,
    pub(crate) events: &'a [Event],
}

#[derive(Serialize)]
pub(crate) struct TeamPlayerStatsWithCollectedEventsExport<'a, Team, Player, Event> {
    pub(crate) team_zero: &'a Team,
    pub(crate) team_one: &'a Team,
    pub(crate) player_stats: Vec<PlayerStatsEntry<'a, Player>>,
    pub(crate) events: Vec<&'a Event>,
}

#[derive(Serialize)]
pub(crate) struct StatsExport<'a, T> {
    pub(crate) stats: &'a T,
}

#[derive(Serialize)]
pub(crate) struct StatsWithEventsExport<'a, T, E> {
    pub(crate) stats: &'a T,
    pub(crate) events: &'a [E],
}

#[derive(Serialize)]
pub(crate) struct EventsExport<'a, E> {
    pub(crate) events: &'a [E],
}

#[derive(Serialize)]
pub(crate) struct StatsWithPlayerEventsExport<'a, T, Player, E> {
    pub(crate) stats: &'a T,
    pub(crate) player_stats: Vec<PlayerStatsEntry<'a, Player>>,
    pub(crate) events: &'a [E],
}

#[derive(Serialize)]
pub(crate) struct StatsWithPlayerStatsExport<'a, T, Player> {
    pub(crate) stats: &'a T,
    pub(crate) player_stats: Vec<PlayerStatsEntry<'a, Player>>,
}
