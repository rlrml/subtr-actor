mod common;

use common::parse_replay;
use subtr_actor::*;

fn complete_movement_breakdowns_for_comparison(movement: &MovementStats) -> MovementStats {
    movement.clone().with_complete_labeled_tracked_time()
}

#[test]
fn test_stats_timeline_collector_final_frame_matches_analysis_graph() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");
    let final_frame = timeline.frames.last().expect("Expected at least one frame");

    let graph = stats::analysis_graph::collect_builtin_analysis_graph_for_replay(
        &replay,
        [
            "fifty_fifty",
            "possession",
            "pressure",
            "rush",
            "core",
            "backboard",
            "double_tap",
            "ball_carry",
            "boost",
            "movement",
            "positioning",
            "powerslide",
            "demo",
        ],
    )
    .expect("Expected analysis graph to process replay");

    let possession = graph
        .state::<PossessionCalculator>()
        .expect("missing possession calculator state");
    let fifty_fifty = graph
        .state::<FiftyFiftyCalculator>()
        .expect("missing fifty_fifty calculator state");
    let pressure = graph
        .state::<PressureCalculator>()
        .expect("missing pressure calculator state");
    let rush = graph
        .state::<RushCalculator>()
        .expect("missing rush calculator state");
    let match_stats = graph
        .state::<MatchStatsCalculator>()
        .expect("missing match stats calculator state");
    let backboard = graph
        .state::<BackboardCalculator>()
        .expect("missing backboard calculator state");
    let double_tap = graph
        .state::<DoubleTapCalculator>()
        .expect("missing double tap calculator state");
    let ball_carry = graph
        .state::<BallCarryCalculator>()
        .expect("missing ball carry calculator state");
    let boost = graph
        .state::<BoostCalculator>()
        .expect("missing boost calculator state");
    let movement = graph
        .state::<MovementCalculator>()
        .expect("missing movement calculator state");
    let positioning = graph
        .state::<PositioningCalculator>()
        .expect("missing positioning calculator state");
    let powerslide = graph
        .state::<PowerslideCalculator>()
        .expect("missing powerslide calculator state");
    let demo = graph
        .state::<DemoCalculator>()
        .expect("missing demo calculator state");

    let assert_core_team_stats_match =
        |actual: &CoreTeamStats, expected: &CoreTeamStats, team_label: &str| {
            assert_eq!(actual.score, expected.score, "{team_label} score");
            assert_eq!(actual.goals, expected.goals, "{team_label} goals");
            assert_eq!(actual.assists, expected.assists, "{team_label} assists");
            assert_eq!(actual.saves, expected.saves, "{team_label} saves");
            assert_eq!(actual.shots, expected.shots, "{team_label} shots");
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.kickoff_goal_count,
                expected
                    .scoring_context
                    .goal_after_kickoff
                    .kickoff_goal_count,
                "{team_label} kickoff-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.short_goal_count,
                expected.scoring_context.goal_after_kickoff.short_goal_count,
                "{team_label} short-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.medium_goal_count,
                expected
                    .scoring_context
                    .goal_after_kickoff
                    .medium_goal_count,
                "{team_label} medium-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.long_goal_count,
                expected.scoring_context.goal_after_kickoff.long_goal_count,
                "{team_label} long-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_buildup, expected.scoring_context.goal_buildup,
                "{team_label} goal buildup classifications",
            );
        };

    let assert_core_player_stats_match =
        |actual: &CorePlayerStats, expected: &CorePlayerStats, player_label: &str| {
            assert_eq!(actual.score, expected.score, "{player_label} score");
            assert_eq!(actual.goals, expected.goals, "{player_label} goals");
            assert_eq!(actual.assists, expected.assists, "{player_label} assists");
            assert_eq!(actual.saves, expected.saves, "{player_label} saves");
            assert_eq!(actual.shots, expected.shots, "{player_label} shots");
            assert_eq!(
                actual.scoring_context.goals_conceded_while_last_defender,
                expected.scoring_context.goals_conceded_while_last_defender,
                "{player_label} last-defender concessions",
            );
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.kickoff_goal_count,
                expected
                    .scoring_context
                    .goal_after_kickoff
                    .kickoff_goal_count,
                "{player_label} kickoff-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.short_goal_count,
                expected.scoring_context.goal_after_kickoff.short_goal_count,
                "{player_label} short-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.medium_goal_count,
                expected
                    .scoring_context
                    .goal_after_kickoff
                    .medium_goal_count,
                "{player_label} medium-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_after_kickoff.long_goal_count,
                expected.scoring_context.goal_after_kickoff.long_goal_count,
                "{player_label} long-goal bucket counts",
            );
            assert_eq!(
                actual.scoring_context.goal_buildup, expected.scoring_context.goal_buildup,
                "{player_label} goal buildup classifications",
            );
        };

    assert_eq!(
        final_frame.team_zero.fifty_fifty,
        fifty_fifty.stats().for_team(true)
    );
    assert_eq!(
        final_frame.team_one.fifty_fifty,
        fifty_fifty.stats().for_team(false)
    );
    assert_eq!(
        final_frame.team_zero.possession,
        possession.stats().for_team(true)
    );
    assert_eq!(
        final_frame.team_one.possession,
        possession.stats().for_team(false)
    );
    assert_eq!(
        final_frame.team_zero.pressure,
        pressure.stats().for_team(true)
    );
    assert_eq!(
        final_frame.team_one.pressure,
        pressure.stats().for_team(false)
    );
    assert_eq!(final_frame.team_zero.rush, rush.stats().for_team(true));
    assert_eq!(final_frame.team_one.rush, rush.stats().for_team(false));
    assert_core_team_stats_match(
        &final_frame.team_zero.core,
        &match_stats.team_zero_stats(),
        "team zero",
    );
    assert_core_team_stats_match(
        &final_frame.team_one.core,
        &match_stats.team_one_stats(),
        "team one",
    );
    assert_eq!(
        final_frame.team_zero.ball_carry,
        ball_carry.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_zero.air_dribble,
        ball_carry.team_zero_air_dribble_stats().clone()
    );
    assert_eq!(
        final_frame.team_zero.backboard,
        backboard.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.backboard,
        backboard.team_one_stats().clone()
    );
    assert_eq!(
        final_frame.team_zero.double_tap,
        double_tap.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.double_tap,
        double_tap.team_one_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.ball_carry,
        ball_carry.team_one_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.air_dribble,
        ball_carry.team_one_air_dribble_stats().clone()
    );
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
        assert_core_player_stats_match(
            &player.core,
            &match_stats
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default(),
            &player.name,
        );
        assert_eq!(
            player.ball_carry,
            ball_carry
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.air_dribble,
            ball_carry
                .player_air_dribble_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.backboard,
            backboard
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.double_tap,
            double_tap
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
            complete_movement_breakdowns_for_comparison(&player.movement),
            movement
                .player_stats()
                .get(&player.player_id)
                .map(complete_movement_breakdowns_for_comparison)
                .unwrap_or_else(|| {
                    complete_movement_breakdowns_for_comparison(&MovementStats::default())
                })
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
    assert_eq!(timeline.events.backboard, backboard.events());
    assert_eq!(timeline.events.double_tap, double_tap.events());
}
