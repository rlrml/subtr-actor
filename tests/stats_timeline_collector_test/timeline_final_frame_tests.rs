#[test]
fn test_stats_timeline_frame_lookup_uses_frame_number() {
    let timeline = ReplayStatsTimeline {
        config: StatsTimelineConfig {
            most_back_forward_threshold_y: PositioningCalculatorConfig::default()
                .most_back_forward_threshold_y,
            level_ball_depth_margin: PositioningCalculatorConfig::default().level_ball_depth_margin,
            pressure_neutral_zone_half_width_y: PressureCalculatorConfig::default()
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
            double_tap_goal_max_event_to_goal_seconds: DoubleTapGoalCalculatorConfig::default()
                .max_event_to_goal_seconds,
            one_timer_goal_max_event_to_goal_seconds: OneTimerGoalCalculatorConfig::default()
                .max_event_to_goal_seconds,
            air_dribble_goal_max_end_to_goal_seconds: AirDribbleGoalCalculatorConfig::default()
                .max_end_to_goal_seconds,
            flip_reset_goal_max_event_to_goal_seconds: FlipResetGoalCalculatorConfig::default()
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
            all_headers: Vec::new(),
        },
        events: ReplayStatsTimelineEvents {
            timeline: Vec::new(),
            core_player: Vec::new(),
            core_team: Vec::new(),
            possession: Vec::new(),
            pressure: Vec::new(),
            territorial_pressure: Vec::new(),
            movement: Vec::new(),
            positioning: Vec::new(),
            rotation_player: Vec::new(),
            rotation_team: Vec::new(),
            mechanics: Vec::new(),
            goal_context: Vec::new(),
            backboard: Vec::new(),
            ceiling_shot: Vec::new(),
            wall_aerial: Vec::new(),
            wall_aerial_shot: Vec::new(),
            center: Vec::new(),
            flick: Vec::new(),
            musty_flick: Vec::new(),
            dodge_reset: Vec::new(),
            double_tap: Vec::new(),
            fifty_fifty: Vec::new(),
            one_timer: Vec::new(),
            pass: Vec::new(),
            pass_last_completed: Vec::new(),
            ball_carry: Vec::new(),
            goal_tags: Vec::new(),
            rush: Vec::new(),
            speed_flip: Vec::new(),
            half_flip: Vec::new(),
            half_volley: Vec::new(),
            wavedash: Vec::new(),
            whiff: Vec::new(),
            powerslide: Vec::new(),
            touch: Vec::new(),
            touch_ball_movement: Vec::new(),
            touch_last_touch: Vec::new(),
            boost_pickups: Vec::new(),
            boost_ledger: Vec::new(),
            boost_state: Vec::new(),
            bump: Vec::new(),
        },
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

fn assert_boost_ledger_reconstructs_serialized_boost_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut ledger_events = timeline.events.boost_ledger.clone();
    ledger_events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });
    let mut state_events = timeline.events.boost_state.clone();
    state_events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut ledger_event_index = 0;
    let mut state_event_index = 0;
    let mut players: HashMap<PlayerId, DerivedBoostLedgerStats> = HashMap::new();
    let mut team_zero = DerivedBoostLedgerStats::default();
    let mut team_one = DerivedBoostLedgerStats::default();

    for frame in &timeline.frames {
        let mut state_event_players_this_frame = Vec::new();
        while state_event_index < state_events.len()
            && state_events[state_event_index].frame <= frame.frame_number
        {
            let event = &state_events[state_event_index];
            apply_boost_state_event(players.entry(event.player_id.clone()).or_default(), event);
            if event.frame == frame.frame_number {
                state_event_players_this_frame.push((event.player_id.clone(), event.is_team_0));
            }
            state_event_index += 1;
        }
        while ledger_event_index < ledger_events.len()
            && ledger_events[ledger_event_index].frame <= frame.frame_number
        {
            let event = &ledger_events[ledger_event_index];
            apply_boost_ledger_event(players.entry(event.player_id.clone()).or_default(), event);
            apply_boost_ledger_event(
                if event.is_team_0 {
                    &mut team_zero
                } else {
                    &mut team_one
                },
                event,
            );
            ledger_event_index += 1;
        }

        for (player_id, is_team_0) in state_event_players_this_frame {
            let player_stats = players.entry(player_id).or_default();
            let Some((previous_boost_amount, boost_amount)) =
                apply_boost_state_sample(player_stats, frame.dt, frame.frame_number)
            else {
                continue;
            };
            add_boost_state_sample(
                if is_team_0 {
                    &mut team_zero.stats
                } else {
                    &mut team_one.stats
                },
                previous_boost_amount,
                boost_amount,
                frame.dt,
            );
        }

        assert_boost_ledger_derived_stats_match(
            &format!("{replay_path} team_zero frame {}", frame.frame_number),
            &frame.team_zero.boost,
            &team_zero.stats,
        );
        assert_boost_ledger_derived_stats_match(
            &format!("{replay_path} team_one frame {}", frame.frame_number),
            &frame.team_one.boost,
            &team_one.stats,
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).map(|stats| &stats.stats);
            let default_stats = BoostStats::default();
            assert_boost_ledger_derived_stats_match(
                &format!(
                    "{replay_path} player {} frame {}",
                    player.name, frame.frame_number
                ),
                &player.boost,
                expected.unwrap_or(&default_stats),
            );
        }
    }
    assert_eq!(
        ledger_event_index,
        ledger_events.len(),
        "{replay_path} unprocessed boost ledger events"
    );
    assert_eq!(
        state_event_index,
        state_events.len(),
        "{replay_path} unprocessed boost state events"
    );
}

