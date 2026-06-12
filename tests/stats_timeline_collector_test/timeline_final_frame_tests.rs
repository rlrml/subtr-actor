#[test]
fn test_stats_timeline_frame_lookup_uses_frame_number() {
    let timeline = ReplayStatsTimeline {
        config: StatsTimelineConfig {
            most_back_forward_threshold_y: PositioningCalculatorConfig::default()
                .most_back_forward_threshold_y,
            level_ball_depth_margin: PositioningCalculatorConfig::default().level_ball_depth_margin,
            closest_to_ball_switch_margin: PositioningCalculatorConfig::default()
                .closest_to_ball_switch_margin,
            closest_to_ball_switch_min_seconds: PositioningCalculatorConfig::default()
                .closest_to_ball_switch_min_seconds,
            ball_half_neutral_zone_half_width_y: BallHalfCalculatorConfig::default()
                .neutral_zone_half_width_y,
            territorial_pressure_neutral_zone_half_width_y:
                TerritorialPressureCalculatorConfig::default().neutral_zone_half_width_y,
            territorial_pressure_min_establish_seconds:
                TerritorialPressureCalculatorConfig::default().min_establish_seconds,
            territorial_pressure_min_establish_third_seconds:
                TerritorialPressureCalculatorConfig::default().min_establish_third_seconds,
            territorial_pressure_relief_grace_seconds:
                TerritorialPressureCalculatorConfig::default().relief_grace_seconds,
            territorial_pressure_confirmed_relief_grace_seconds:
                TerritorialPressureCalculatorConfig::default().confirmed_relief_grace_seconds,
            rotation_role_depth_margin: RotationCalculatorConfig::default().role_depth_margin,
            rotation_first_man_ambiguity_margin: RotationCalculatorConfig::default()
                .first_man_ambiguity_margin,
            rotation_first_man_debounce_seconds: RotationCalculatorConfig::default()
                .first_man_debounce_seconds,
            rotation_first_man_stint_end_grace_seconds: RotationCalculatorConfig::default()
                .first_man_stint_end_grace_seconds,
            rush_max_start_y: RushCalculatorConfig::default().max_start_y,
            rush_attack_support_distance_y: RushCalculatorConfig::default()
                .attack_support_distance_y,
            rush_defender_distance_y: RushCalculatorConfig::default().defender_distance_y,
            rush_min_possession_retained_seconds: RushCalculatorConfig::default()
                .min_possession_retained_seconds,
            aerial_goal_min_ball_z: AerialGoalCalculatorConfig::default().min_ball_z,
            high_aerial_goal_min_ball_z: HighAerialGoalCalculatorConfig::default().min_ball_z,
            long_distance_goal_max_attacking_y: LongDistanceGoalCalculatorConfig::default()
                .max_attacking_y,
            own_half_goal_max_attacking_y: OwnHalfGoalCalculatorConfig::default().max_attacking_y,
            empty_net_min_defender_y_margin: EmptyNetGoalCalculatorConfig::default()
                .min_defender_y_margin,
            empty_net_min_defender_distance: EmptyNetGoalCalculatorConfig::default()
                .min_defender_distance,
            empty_net_max_touch_attacking_y: EmptyNetGoalCalculatorConfig::default()
                .max_touch_attacking_y,
            flick_goal_max_event_to_goal_seconds: FlickGoalCalculatorConfig::default()
                .max_event_to_goal_seconds,
            ceiling_shot_goal_max_event_to_goal_seconds: CeilingShotGoalCalculatorConfig::default()
                .max_event_to_goal_seconds,
            double_tap_goal_max_event_to_goal_seconds: DoubleTapGoalCalculatorConfig::default()
                .max_event_to_goal_seconds,
            one_timer_goal_max_event_to_goal_seconds: OneTimerGoalCalculatorConfig::default()
                .max_event_to_goal_seconds,
            air_dribble_goal_max_end_to_goal_seconds: AirDribbleGoalCalculatorConfig::default()
                .max_end_to_goal_seconds,
            flip_reset_goal_max_event_to_goal_seconds: FlipResetGoalCalculatorConfig::default()
                .max_event_to_goal_seconds,
            flip_into_ball_goal_max_touch_to_goal_seconds:
                FlipIntoBallGoalCalculatorConfig::default().max_touch_to_goal_seconds,
            bump_goal_max_event_to_goal_seconds: BumpGoalCalculatorConfig::default()
                .max_event_to_goal_seconds,
            demo_goal_max_event_to_goal_seconds: DemoGoalCalculatorConfig::default()
                .max_event_to_goal_seconds,
            half_volley_max_bounce_to_touch_seconds: HalfVolleyCalculatorConfig::default()
                .max_bounce_to_touch_seconds,
            half_volley_min_ball_speed: HalfVolleyCalculatorConfig::default().min_ball_speed,
            half_volley_goal_max_touch_to_goal_seconds: HalfVolleyGoalCalculatorConfig::default()
                .max_touch_to_goal_seconds,
            half_volley_goal_min_goal_alignment: HalfVolleyGoalCalculatorConfig::default()
                .min_goal_alignment,
        },
        replay_meta: ReplayMeta {
            team_zero: Vec::new(),
            team_one: Vec::new(),
            game_type: Default::default(),
            season: None,
            all_headers: Vec::new(),
        },
        events: ReplayStatsTimelineEvents::default(),
        frames: vec![
            ReplayStatsFrame {
                frame_number: 10,
                time: 0.0,
                dt: 0.0,
                seconds_remaining: None,
                game_state: None,
                ball_has_been_hit: None,
                kickoff_countdown_time: None,
                gameplay_phase: GameplayPhase::ActivePlay,
                is_live_play: true,
                team_zero: default_team_stats_snapshot(),
                team_one: default_team_stats_snapshot(),
                players: Vec::new(),
            },
            ReplayStatsFrame {
                frame_number: 11,
                time: 0.1,
                dt: 0.1,
                seconds_remaining: None,
                game_state: None,
                ball_has_been_hit: None,
                kickoff_countdown_time: None,
                gameplay_phase: GameplayPhase::ActivePlay,
                is_live_play: true,
                team_zero: default_team_stats_snapshot(),
                team_one: default_team_stats_snapshot(),
                players: Vec::new(),
            },
            ReplayStatsFrame {
                frame_number: 15,
                time: 0.2,
                dt: 0.1,
                seconds_remaining: None,
                game_state: None,
                ball_has_been_hit: None,
                kickoff_countdown_time: None,
                gameplay_phase: GameplayPhase::ActivePlay,
                is_live_play: true,
                team_zero: default_team_stats_snapshot(),
                team_one: default_team_stats_snapshot(),
                players: Vec::new(),
            },
        ],
    };

    assert_eq!(timeline.frames[2].frame_number, 15);
    assert_eq!(timeline.frame_by_number(2), None);
    assert_eq!(
        timeline
            .frame_by_number(15)
            .expect("Expected frame lookup by frame number")
            .frame_number,
        15
    );
}

#[test]
#[ignore = "replay-backed final-frame graph parity test is slow; run explicitly when changing stats projection parity"]
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
            "ball_half",
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
            "stats_projection",
        ],
    )
    .expect("Expected analysis graph to process replay");

    let projection = graph
        .state::<StatsProjectionState>()
        .expect("missing stats projection state");
    let backboard = graph
        .state::<BackboardCalculator>()
        .expect("missing backboard calculator state");
    let double_tap = graph
        .state::<DoubleTapCalculator>()
        .expect("missing double tap calculator state");

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
        projection.fifty_fifty.stats().for_team(true)
    );
    assert_eq!(
        final_frame.team_one.fifty_fifty,
        projection.fifty_fifty.stats().for_team(false)
    );
    assert_eq!(
        final_frame.team_zero.possession,
        projection.possession.stats().for_team(true)
    );
    assert_eq!(
        final_frame.team_one.possession,
        projection.possession.stats().for_team(false)
    );
    assert_eq!(
        final_frame.team_zero.ball_half,
        projection.ball_half.stats().for_team(true)
    );
    assert_eq!(
        final_frame.team_one.ball_half,
        projection.ball_half.stats().for_team(false)
    );
    assert_eq!(
        final_frame.team_zero.rush,
        projection.rush.stats().for_team(true)
    );
    assert_eq!(
        final_frame.team_one.rush,
        projection.rush.stats().for_team(false)
    );
    assert_core_team_stats_match(
        &final_frame.team_zero.core,
        &projection.core.team_zero_stats(),
        "team zero",
    );
    assert_core_team_stats_match(
        &final_frame.team_one.core,
        &projection.core.team_one_stats(),
        "team one",
    );
    assert_eq!(
        final_frame.team_zero.ball_carry,
        projection.ball_carry.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_zero.air_dribble,
        projection.ball_carry.team_zero_air_dribble_stats().clone()
    );
    assert_eq!(
        final_frame.team_zero.backboard,
        projection.backboard.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.backboard,
        projection.backboard.team_one_stats().clone()
    );
    assert_eq!(
        final_frame.team_zero.double_tap,
        projection.double_tap.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.double_tap,
        projection.double_tap.team_one_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.ball_carry,
        projection.ball_carry.team_one_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.air_dribble,
        projection.ball_carry.team_one_air_dribble_stats().clone()
    );
    assert_eq!(
        final_frame.team_zero.boost,
        projection.boost.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.boost,
        projection.boost.team_one_stats().clone()
    );
    assert_eq!(
        final_frame.team_zero.movement,
        projection.movement.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.movement,
        projection.movement.team_one_stats().clone()
    );
    assert_eq!(
        final_frame.team_zero.powerslide,
        projection.powerslide.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.powerslide,
        projection.powerslide.team_one_stats().clone()
    );
    assert_eq!(
        final_frame.team_zero.demo,
        projection.demo.team_zero_stats().clone()
    );
    assert_eq!(
        final_frame.team_one.demo,
        projection.demo.team_one_stats().clone()
    );

    assert_eq!(
        final_frame.players.len(),
        timeline.replay_meta.player_count()
    );
    for player in &final_frame.players {
        assert_core_player_stats_match(
            &player.core,
            &projection
                .core
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default(),
            &player.name,
        );
        assert_eq!(
            player.ball_carry,
            projection
                .ball_carry
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.air_dribble,
            projection
                .ball_carry
                .player_air_dribble_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.backboard,
            projection
                .backboard
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.double_tap,
            projection
                .double_tap
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.boost,
            projection
                .boost
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            complete_movement_breakdowns_for_comparison(&player.movement),
            projection
                .movement
                .player_stats()
                .get(&player.player_id)
                .map(complete_movement_breakdowns_for_comparison)
                .unwrap_or_else(|| {
                    complete_movement_breakdowns_for_comparison(&MovementStats::default())
                })
        );
        assert_eq!(
            player.positioning,
            projection
                .positioning
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.powerslide,
            projection
                .powerslide
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
        assert_eq!(
            player.demo,
            projection
                .demo
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default()
        );
    }
    assert_eq!(
        timeline_payloads_by_stream(&timeline, "backboard", |payload| match payload {
            EventPayload::Backboard(event) => Some(event),
            _ => None,
        }),
        backboard.events()
    );
    assert_eq!(
        timeline_payloads_by_stream(&timeline, "double_tap", |payload| match payload {
            EventPayload::DoubleTap(event) => Some(event),
            _ => None,
        }),
        double_tap.events()
    );
}

