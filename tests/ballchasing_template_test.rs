#![allow(unused_macros)]

use serde_json::Value;
use subtr_actor::{stats, BoostCalculator, PlayerInfo, ReplayProcessor};

macro_rules! ballchasing_fixture_test {
    ($test_name:ident, $fixture_dir:literal) => {
        #[test]
        #[ignore = "Ballchasing fixtures are opt-in and should be enabled fixture-by-fixture"]
        fn $test_name() {
            let report = subtr_actor::ballchasing::compare_fixture_directory(
                std::path::Path::new(concat!("assets/", $fixture_dir)),
                &subtr_actor::ballchasing::recommended_match_config(),
            )
            .expect("Failed to compare Ballchasing fixture");
            report.assert_matches();
        }
    };
}

ballchasing_fixture_test!(
    compare_recent_ranked_doubles_2026_03_10,
    "recent-ranked-doubles-2026-03-10"
);

ballchasing_fixture_test!(
    compare_recent_ranked_standard_2026_03_10_a,
    "recent-ranked-standard-2026-03-10-a"
);

ballchasing_fixture_test!(
    compare_recent_ranked_standard_2026_03_10_b,
    "recent-ranked-standard-2026-03-10-b"
);

fn parse_replay(path: &str) -> boxcars::Replay {
    let data = std::fs::read(path).unwrap_or_else(|_| panic!("Failed to read replay file: {path}"));
    boxcars::ParserBuilder::new(&data)
        .must_parse_network_data()
        .always_check_crc()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay file: {path}"))
}

fn json_u32(value: &Value, path: &str) -> u32 {
    value
        .pointer(path)
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or_else(|| panic!("Expected u32 at JSON pointer {path}"))
}

fn expected_player_big_pad_count(ballchasing: &Value, team_key: &str, player_name: &str) -> u32 {
    let players = ballchasing
        .pointer(&format!("/{team_key}/players"))
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("Expected players array for team {team_key}"));
    let player = players
        .iter()
        .find(|player| player.get("name").and_then(Value::as_str) == Some(player_name))
        .unwrap_or_else(|| panic!("Expected Ballchasing player {team_key}.{player_name}"));
    json_u32(player, "/stats/boost/count_collected_big")
}

fn assert_inactive_inclusive_big_pad_counts_match_ballchasing(
    boost: &BoostCalculator,
    players: &[PlayerInfo],
    ballchasing: &Value,
    team_key: &str,
) {
    let mut actual_team_count = 0;
    let mut expected_team_count = 0;

    for player in players {
        let player_stats = boost
            .player_stats()
            .get(&player.remote_id)
            .unwrap_or_else(|| panic!("Expected boost stats for {}", player.name));
        let actual = player_stats.big_pads_collected + player_stats.big_pads_collected_inactive;
        let expected = expected_player_big_pad_count(ballchasing, team_key, &player.name);
        assert_eq!(
            actual, expected,
            "inactive-inclusive big pad count mismatch for {team_key}.{}: \
             active={} inactive={}",
            player.name, player_stats.big_pads_collected, player_stats.big_pads_collected_inactive,
        );
        actual_team_count += actual;
        expected_team_count += expected;
    }

    assert_eq!(
        actual_team_count,
        json_u32(
            ballchasing,
            &format!("/{team_key}/stats/boost/count_collected_big")
        ),
        "inactive-inclusive team big pad count should match Ballchasing team stat for {team_key}"
    );
    assert_eq!(
        actual_team_count, expected_team_count,
        "inactive-inclusive team big pad count should equal player total for {team_key}"
    );
}

#[test]
#[ignore = "Documents the inactive-inclusive big-pad Ballchasing comparison for this fixture; currently still exposes a remaining blue-player gap."]
fn problematic_private_duel_big_pad_counts_match_ballchasing_with_inactive_pickups() {
    let replay = parse_replay("assets/problematic-private-duel-2026-03-20.replay");
    let ballchasing: Value = serde_json::from_slice(
        &std::fs::read("assets/problematic-private-duel-2026-03-20.ballchasing.json")
            .expect("Failed to read Ballchasing JSON fixture"),
    )
    .expect("Failed to parse Ballchasing JSON fixture");
    let replay_meta = ReplayProcessor::new(&replay)
        .expect("Expected replay processor")
        .get_replay_meta()
        .expect("Expected replay metadata");
    let graph =
        stats::analysis_graph::collect_builtin_analysis_graph_for_replay(&replay, ["boost"])
            .expect("Expected boost analysis graph to process replay");
    let boost = graph
        .state::<BoostCalculator>()
        .expect("Expected boost calculator state");

    assert_inactive_inclusive_big_pad_counts_match_ballchasing(
        boost,
        &replay_meta.team_zero,
        &ballchasing,
        "blue",
    );
    assert_inactive_inclusive_big_pad_counts_match_ballchasing(
        boost,
        &replay_meta.team_one,
        &ballchasing,
        "orange",
    );
}
