use subtr_actor::{PlayerId, ReplayMeta, StatsTimelineEventCollector};

const COLONELPANIC_NO_SPEED_FLIP_REPLAY: &str =
    "assets/colonelpanic8-double-tap-third-goal-2026-05-24.replay";
const ROCKET_SENSE_REVIEWED_DUEL_REPLAY: &str = "assets/post-eac-ranked-duel-2026-04-28-a.replay";

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

    let event_near_reported_time = colonelpanic_speed_flips
        .iter()
        .any(|event| event.frame.abs_diff(837) <= 3 || (event.time - 31.695_719).abs() <= 0.15);
    assert!(
        !event_near_reported_time,
        "unexpected colonelpanic8 speed flip near viewer time 28.1s/raw frame 837: {colonelpanic_speed_flips:#?}"
    );
}

#[test]
fn reviewed_post_eac_duel_keeps_confirmed_speed_flip_and_rejects_nearby_false_positive() {
    let replay = parse_replay(ROCKET_SENSE_REVIEWED_DUEL_REPLAY);
    let timeline = StatsTimelineEventCollector::new()
        .get_replay_stats_timeline_scaffold(&replay)
        .expect("stats timeline should build");
    let oside_ids = player_ids_by_name(&timeline.replay_meta, "OSIDE_SMURF");
    let adamboi_ids = player_ids_by_name(&timeline.replay_meta, "Adamboi04");
    assert!(!oside_ids.is_empty(), "fixture should contain OSIDE_SMURF");
    assert!(!adamboi_ids.is_empty(), "fixture should contain Adamboi04");

    let oside_confirmed = timeline.events.speed_flip.iter().any(|event| {
        oside_ids.contains(&&event.player)
            && (event.frame.abs_diff(1848) <= 3 || (event.time - 90.561_89).abs() <= 0.15)
    });
    assert!(
        oside_confirmed,
        "expected the Rocket Sense confirmed OSIDE_SMURF speed flip near frame 1848; got {:#?}",
        timeline.events.speed_flip
    );

    let adamboi_rejected = [110_usize, 576, 1020, 1557, 1644, 1850];
    let adamboi_false_positive = timeline.events.speed_flip.iter().find(|event| {
        adamboi_ids.contains(&&event.player)
            && adamboi_rejected
                .iter()
                .any(|rejected_frame| event.frame.abs_diff(*rejected_frame) <= 3)
    });
    assert!(
        adamboi_false_positive.is_none(),
        "unexpected speed flip for Rocket Sense rejected Adamboi04 candidate: {adamboi_false_positive:#?}"
    );
}
