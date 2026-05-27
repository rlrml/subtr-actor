use super::builtins_snapshot_frame_core_player::CorePlayerStatsSnapshot;
use super::builtins_snapshot_frame_core_team::CoreTeamStatsSnapshot;
use super::*;

#[derive(Serialize)]
struct CoreStatsSnapshotExport {
    team_zero: CoreTeamStatsSnapshot,
    team_one: CoreTeamStatsSnapshot,
    player_stats: Vec<OwnedPlayerStatsEntry<CorePlayerStatsSnapshot>>,
}

pub(super) fn core_snapshot_frame_json(
    module_name: &str,
    graph: &AnalysisGraph,
    _replay_meta: &ReplayMeta,
) -> SubtrActorResult<Option<Value>> {
    if module_name != "core" {
        return Ok(None);
    }

    let calculator = graph_state::<MatchStatsCalculator>(graph, module_name)?;
    let mut player_stats: Vec<_> = calculator
        .player_stats()
        .iter()
        .map(|(player_id, stats)| OwnedPlayerStatsEntry {
            player_id: player_id.clone(),
            stats: CorePlayerStatsSnapshot::from(stats),
        })
        .collect();
    player_stats.sort_by(|left, right| {
        format!("{:?}", left.player_id).cmp(&format!("{:?}", right.player_id))
    });

    serialize_to_json_value(&CoreStatsSnapshotExport {
        team_zero: calculator.team_zero_stats().into(),
        team_one: calculator.team_one_stats().into(),
        player_stats,
    })
    .map(Some)
}
