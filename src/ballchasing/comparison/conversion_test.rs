use super::{
    build_actual_comparable_stats, compute_comparable_stats, raw_boost_amount_as_comparable_units,
    ComputedComparableStats,
};
use crate::*;

fn parse_replay(path: &str) -> boxcars::Replay {
    let data = std::fs::read(path).unwrap_or_else(|_| panic!("Failed to read replay file: {path}"));
    boxcars::ParserBuilder::new(&data)
        .must_parse_network_data()
        .always_check_crc()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay file: {path}"))
}

fn compute_comparable_stats_reference(
    replay: &boxcars::Replay,
) -> SubtrActorResult<ComputedComparableStats> {
    let replay_meta = ReplayProcessor::new(replay)?.get_replay_meta()?;
    let graph = stats::analysis_nodes::collect_builtin_analysis_graph_for_replay(
        replay,
        [
            "core",
            "boost",
            "movement",
            "positioning",
            "demo",
            "powerslide",
        ],
    )?;

    Ok(ComputedComparableStats {
        replay_meta,
        match_stats: graph
            .state::<MatchStatsCalculator>()
            .cloned()
            .unwrap_or_default(),
        boost: graph
            .state::<BoostCalculator>()
            .cloned()
            .unwrap_or_default(),
        movement: graph
            .state::<MovementCalculator>()
            .cloned()
            .unwrap_or_default(),
        positioning: graph
            .state::<PositioningCalculator>()
            .cloned()
            .unwrap_or_default(),
        demo: graph.state::<DemoCalculator>().cloned().unwrap_or_default(),
        powerslide: graph
            .state::<PowerslideCalculator>()
            .cloned()
            .unwrap_or_default(),
    })
}

#[test]
fn test_raw_boost_amount_conversion_matches_percent_scale() {
    assert_eq!(raw_boost_amount_as_comparable_units(255.0), 100.0);
    assert!((raw_boost_amount_as_comparable_units(30.6) - 12.0).abs() < 0.1);
    assert!((raw_boost_amount_as_comparable_units(510.0) - 200.0).abs() < 0.1);
}

#[test]
fn comparable_stats_collector_matches_reference_bundle() {
    let replay = parse_replay("assets/replays/new_boost_format.replay");
    let combined_start = std::time::Instant::now();
    let combined =
        compute_comparable_stats(&replay).expect("combined comparable stats should succeed");
    let combined_duration = combined_start.elapsed();
    let reference_start = std::time::Instant::now();
    let reference = compute_comparable_stats_reference(&replay)
        .expect("reference comparable stats should succeed");
    let reference_duration = reference_start.elapsed();

    eprintln!("combined={combined_duration:?} reference={reference_duration:?}");

    let actual = build_actual_comparable_stats(&combined);
    let expected = build_actual_comparable_stats(&reference);

    assert_eq!(actual, expected);
}
