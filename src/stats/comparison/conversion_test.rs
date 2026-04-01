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
    let mut match_collector = ReducerCollector::new(MatchStatsReducer::new());
    let mut boost_collector = ReducerCollector::new(BoostReducer::new());
    let mut movement_collector = ReducerCollector::new(MovementReducer::new());
    let mut positioning_collector = ReducerCollector::new(PositioningReducer::new());
    let mut demo_collector = ReducerCollector::new(DemoReducer::new());
    let mut powerslide_collector = ReducerCollector::new(PowerslideReducer::new());

    let mut processor = ReplayProcessor::new(replay)?;
    let mut collectors: [&mut dyn Collector; 6] = [
        &mut match_collector,
        &mut boost_collector,
        &mut movement_collector,
        &mut positioning_collector,
        &mut demo_collector,
        &mut powerslide_collector,
    ];
    processor.process_all(&mut collectors)?;

    Ok(ComputedComparableStats {
        replay_meta: processor.get_replay_meta()?,
        match_stats: match_collector.into_inner(),
        boost: boost_collector.into_inner(),
        movement: movement_collector.into_inner(),
        positioning: positioning_collector.into_inner(),
        demo: demo_collector.into_inner(),
        powerslide: powerslide_collector.into_inner(),
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
