use subtr_actor::{PlayerId, ReplayMeta, StatsTimelineEventCollector};

const COLONELPANIC_NO_SPEED_FLIP_REPLAY: &str =
    "assets/colonelpanic8-double-tap-third-goal-2026-05-24.replay";

fn parse_replay(path: &str) -> boxcars::Replay {
    let data = std::fs::read(path).unwrap_or_else(|_| panic!("Failed to read replay file: {path}"));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay: {path}"))
}

fn player_ids_by_name<'a>(replay_meta: &'a ReplayMeta, name: &str) -> Vec<&'a PlayerId> {
    replay_meta
        .team_zero
        .iter()
        .chain(replay_meta.team_one.iter())
        .filter(|player| player.name == name)
        .map(|player| &player.remote_id)
        .collect()
}

#[test]
fn colonelpanic_replay_has_no_speed_flip_at_normalized_28_1_seconds() {
    let replay = parse_replay(COLONELPANIC_NO_SPEED_FLIP_REPLAY);
    let timeline = StatsTimelineEventCollector::new()
        .get_replay_stats_timeline_scaffold(&replay)
        .expect("stats timeline should build");
    let colonelpanic_ids = player_ids_by_name(&timeline.replay_meta, "colonelpanic8");
    assert!(
        !colonelpanic_ids.is_empty(),
        "fixture should contain colonelpanic8"
    );
    let colonelpanic_speed_flips = timeline
        .events
        .speed_flip
        .iter()
        .filter(|event| colonelpanic_ids.contains(&&event.player))
        .collect::<Vec<_>>();
    assert!(
        !colonelpanic_speed_flips.is_empty(),
        "fixture should still exercise speed-flip detection"
    );

    let event_near_reported_time = colonelpanic_speed_flips
        .iter()
        .any(|event| event.frame.abs_diff(837) <= 3 || (event.time - 31.695_719).abs() <= 0.15);
    assert!(
        !event_near_reported_time,
        "unexpected colonelpanic8 speed flip near viewer time 28.1s/raw frame 837: {colonelpanic_speed_flips:#?}"
    );
}
