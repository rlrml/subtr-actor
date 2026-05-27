use super::*;

pub(super) fn owned_snapshot_frame_json(
    module_name: &str,
    graph: &AnalysisGraph,
    _replay_meta: &ReplayMeta,
) -> SubtrActorResult<Option<Value>> {
    match module_name {
        "touch" => touch_snapshot_frame_json(graph, module_name),
        "movement" => movement_snapshot_frame_json(graph, module_name),
        _ => Ok(None),
    }
}

fn touch_snapshot_frame_json(
    graph: &AnalysisGraph,
    module_name: &str,
) -> SubtrActorResult<Option<Value>> {
    let calculator = graph_state::<TouchCalculator>(graph, module_name)?;
    let player_stats = calculator
        .player_stats()
        .iter()
        .map(|(player_id, stats)| OwnedPlayerStatsEntry {
            player_id: player_id.clone(),
            stats: stats.clone().with_complete_labeled_touch_counts(),
        })
        .collect();
    serialize_to_json_value(&OwnedPlayerStatsExport { player_stats }).map(Some)
}

fn movement_snapshot_frame_json(
    graph: &AnalysisGraph,
    module_name: &str,
) -> SubtrActorResult<Option<Value>> {
    let calculator = graph_state::<MovementCalculator>(graph, module_name)?;
    let player_stats = calculator
        .player_stats()
        .iter()
        .map(|(player_id, stats)| OwnedPlayerStatsEntry {
            player_id: player_id.clone(),
            stats: stats.clone().with_complete_labeled_tracked_time(),
        })
        .collect();
    serialize_to_json_value(&TeamOwnedPlayerStatsExport {
        team_zero: calculator.team_zero_stats(),
        team_one: calculator.team_one_stats(),
        player_stats,
    })
    .map(Some)
}
