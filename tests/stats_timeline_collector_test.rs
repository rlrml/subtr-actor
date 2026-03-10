use subtr_actor::*;

fn parse_replay(path: &str) -> boxcars::Replay {
    let data = std::fs::read(path).unwrap_or_else(|_| panic!("Failed to read replay file: {path}"));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay: {path}"))
}

#[test]
fn test_stats_timeline_collector_final_frame_matches_reducers() {
    let replay = parse_replay("assets/replays/test/rlcs.replay");
    let timeline = StatsTimelineCollector::new()
        .get_replay_data(&replay)
        .expect("Expected stats timeline data");
    let final_frame = timeline.frames.last().expect("Expected at least one frame");

    let mut possession_collector = ReducerCollector::new(PossessionReducer::new());
    let mut match_collector = ReducerCollector::new(MatchStatsReducer::new());
    let mut boost_collector = ReducerCollector::new(BoostReducer::new());
    let mut movement_collector = ReducerCollector::new(MovementReducer::new());
    let mut positioning_collector = ReducerCollector::new(PositioningReducer::new());
    let mut powerslide_collector = ReducerCollector::new(PowerslideReducer::new());
    let mut demo_collector = ReducerCollector::new(DemoReducer::new());

    let mut processor = ReplayProcessor::new(&replay).expect("Expected replay processor");
    let mut collectors: [&mut dyn Collector; 7] = [
        &mut possession_collector,
        &mut match_collector,
        &mut boost_collector,
        &mut movement_collector,
        &mut positioning_collector,
        &mut powerslide_collector,
        &mut demo_collector,
    ];
    processor
        .process_all(&mut collectors)
        .expect("Expected reducers to process replay");

    let possession = possession_collector.into_inner();
    let match_stats = match_collector.into_inner();
    let boost = boost_collector.into_inner();
    let movement = movement_collector.into_inner();
    let positioning = positioning_collector.into_inner();
    let powerslide = powerslide_collector.into_inner();
    let demo = demo_collector.into_inner();

    assert_eq!(final_frame.possession, possession.stats().clone());
    assert_eq!(final_frame.team_zero.core, match_stats.team_zero_stats());
    assert_eq!(final_frame.team_one.core, match_stats.team_one_stats());
    assert_eq!(final_frame.team_zero.boost, boost.team_zero_stats().clone());
    assert_eq!(final_frame.team_one.boost, boost.team_one_stats().clone());
    assert_eq!(
        final_frame.team_zero.movement,
        movement.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.movement,
        movement.team_one_stats().clone()
    );
    assert_eq!(
        final_frame.team_zero.powerslide,
        powerslide.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.powerslide,
        powerslide.team_one_stats().clone()
    );
    assert_eq!(final_frame.team_zero.demo, demo.team_zero_stats().clone());
    assert_eq!(final_frame.team_one.demo, demo.team_one_stats().clone());

    assert_eq!(
        final_frame.players.len(),
        timeline.replay_meta.player_count()
    );
    for player in &final_frame.players {
        assert_eq!(
            player.core,
            match_stats
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.boost,
            boost
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.movement,
            movement
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.positioning,
            positioning
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.powerslide,
            powerslide
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.demo,
            demo.player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
    }
}

#[test]
fn test_stats_timeline_collector_frames_are_sorted_and_cumulative() {
    let replay = parse_replay("assets/replays/test/rlcs.replay");
    let timeline = StatsTimelineCollector::new()
        .get_replay_data(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.frames.is_empty(),
        "Expected stats timeline frames"
    );
    assert!(
        timeline
            .frames
            .windows(2)
            .all(|frames| frames[1].time >= frames[0].time),
        "Expected frame times to be nondecreasing"
    );
    assert!(
        timeline
            .frames
            .windows(2)
            .all(|frames| frames[1].team_zero.core.goals >= frames[0].team_zero.core.goals),
        "Expected team-zero goals to accumulate monotonically"
    );
    assert!(
        timeline.frames.windows(2).all(|frames| {
            frames[1].team_zero.boost.amount_collected >= frames[0].team_zero.boost.amount_collected
        }),
        "Expected collected boost to accumulate monotonically"
    );
    assert!(
        timeline
            .timeline_events
            .windows(2)
            .all(|events| events[1].time >= events[0].time),
        "Expected emitted timeline events to be time sorted"
    );
}
