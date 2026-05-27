use super::*;

pub(super) fn stats_snapshot_frame_json(
    module_name: &str,
    graph: &AnalysisGraph,
    _replay_meta: &ReplayMeta,
) -> SubtrActorResult<Option<Value>> {
    match module_name {
        "fifty_fifty" => {
            let calculator = graph_state::<FiftyFiftyCalculator>(graph, module_name)?;
            serialize_to_json_value(&StatsWithPlayerStatsExport {
                stats: calculator.stats(),
                player_stats: player_stats_entries(calculator.player_stats()),
            })
            .map(Some)
        }
        "possession" => {
            stats_export::<PossessionCalculator, _>(graph, module_name, PossessionCalculator::stats)
        }
        "pressure" => {
            stats_export::<PressureCalculator, _>(graph, module_name, PressureCalculator::stats)
        }
        "territorial_pressure" => stats_export::<TerritorialPressureCalculator, _>(
            graph,
            module_name,
            TerritorialPressureCalculator::stats,
        ),
        "rush" => stats_export::<RushCalculator, _>(graph, module_name, RushCalculator::stats),
        _ => Ok(None),
    }
}

fn stats_export<C, T>(
    graph: &AnalysisGraph,
    module_name: &str,
    stats: impl Fn(&C) -> &T,
) -> SubtrActorResult<Option<Value>>
where
    C: 'static,
    T: Serialize,
{
    let calculator = graph_state::<C>(graph, module_name)?;
    serialize_to_json_value(&StatsExport {
        stats: stats(calculator),
    })
    .map(Some)
}
