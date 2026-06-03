fn powerslide_frame_counts_toward_motion(frame: &ReplayStatsFrame) -> bool {
    matches!(
        frame.gameplay_phase,
        GameplayPhase::ActivePlay | GameplayPhase::KickoffWaitingForTouch
    )
}

fn assert_powerslide_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.powerslide.clone();
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut active_states: HashMap<PlayerId, DerivedPowerslideState> = HashMap::new();
    let mut players: HashMap<PlayerId, PowerslideStats> = HashMap::new();
    let mut team_zero = PowerslideStats::default();
    let mut team_one = PowerslideStats::default();

    for frame in &timeline.frames {
        let counts_toward_motion = powerslide_frame_counts_toward_motion(frame);

        while event_index < events.len() && events[event_index].frame <= frame.frame_number {
            let event = &events[event_index];
            let previous_active = active_states
                .get(&event.player)
                .is_some_and(|state| state.active);

            active_states.insert(
                event.player.clone(),
                DerivedPowerslideState {
                    active: event.active,
                    is_team_0: event.is_team_0,
                },
            );

            if counts_toward_motion && event.active && !previous_active {
                players.entry(event.player.clone()).or_default().press_count += 1;
                if event.is_team_0 {
                    team_zero.press_count += 1;
                } else {
                    team_one.press_count += 1;
                }
            }

            event_index += 1;
        }

        if counts_toward_motion {
            for player in &frame.players {
                if active_states
                    .get(&player.player_id)
                    .is_some_and(|state| state.active)
                {
                    players
                        .entry(player.player_id.clone())
                        .or_default()
                        .total_duration += frame.dt;
                    if player.is_team_0 {
                        team_zero.total_duration += frame.dt;
                    } else {
                        team_one.total_duration += frame.dt;
                    }
                }
            }
        }

        assert!(
            (frame.team_zero.powerslide.total_duration - team_zero.total_duration).abs() < 0.001,
            "{replay_path} team_zero powerslide.total_duration frame {} actual {:.3} expected {:.3}",
            frame.frame_number,
            frame.team_zero.powerslide.total_duration,
            team_zero.total_duration
        );
        assert_eq!(
            frame.team_zero.powerslide.press_count, team_zero.press_count,
            "{replay_path} team_zero powerslide.press_count frame {}",
            frame.frame_number
        );
        assert!(
            (frame.team_one.powerslide.total_duration - team_one.total_duration).abs() < 0.001,
            "{replay_path} team_one powerslide.total_duration frame {} actual {:.3} expected {:.3}",
            frame.frame_number,
            frame.team_one.powerslide.total_duration,
            team_one.total_duration
        );
        assert_eq!(
            frame.team_one.powerslide.press_count, team_one.press_count,
            "{replay_path} team_one powerslide.press_count frame {}",
            frame.frame_number
        );

        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert!(
                (player.powerslide.total_duration - expected.total_duration).abs() < 0.001,
                "{replay_path} player {} powerslide.total_duration frame {} actual {:.3} expected {:.3}",
                player.name,
                frame.frame_number,
                player.powerslide.total_duration,
                expected.total_duration
            );
            assert_eq!(
                player.powerslide.press_count, expected.press_count,
                "{replay_path} player {} powerslide.press_count frame {}",
                player.name, frame.frame_number
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed powerslide events"
    );
}

fn touch_label_for_derivation(key: &'static str, value: &str) -> StatLabel {
    match (key, value) {
        ("kind", "control") => StatLabel::new("kind", "control"),
        ("kind", "medium_hit") => StatLabel::new("kind", "medium_hit"),
        ("kind", "hard_hit") => StatLabel::new("kind", "hard_hit"),
        ("height_band", "ground") => StatLabel::new("height_band", "ground"),
        ("height_band", "low_air") => StatLabel::new("height_band", "low_air"),
        ("height_band", "high_air") => StatLabel::new("height_band", "high_air"),
        ("surface", "ground") => StatLabel::new("surface", "ground"),
        ("surface", "air") => StatLabel::new("surface", "air"),
        ("surface", "wall") => StatLabel::new("surface", "wall"),
        ("dodge_state", "no_dodge") => StatLabel::new("dodge_state", "no_dodge"),
        ("dodge_state", "dodge") => StatLabel::new("dodge_state", "dodge"),
        _ => panic!("unexpected touch label {key}={value}"),
    }
}

fn apply_touch_stats_event_for_derivation(
    stats: &mut TouchStats,
    event: &TouchStatsEvent,
    frame: &ReplayStatsFrame,
) {
    stats.touch_count += 1;
    match event.kind.as_str() {
        "control" => stats.control_touch_count += 1,
        "medium_hit" => stats.medium_hit_count += 1,
        "hard_hit" => stats.hard_hit_count += 1,
        value => panic!("unexpected touch kind {value}"),
    }
    match event.height_band.as_str() {
        "ground" => {}
        "low_air" => stats.aerial_touch_count += 1,
        "high_air" => {
            stats.aerial_touch_count += 1;
            stats.high_aerial_touch_count += 1;
        }
        value => panic!("unexpected touch height band {value}"),
    }
    match event.surface.as_str() {
        "wall" => stats.wall_touch_count += 1,
        "ground" | "air" => {}
        value => panic!("unexpected touch surface {value}"),
    }
    stats.labeled_touch_counts.increment([
        touch_label_for_derivation("kind", &event.kind),
        touch_label_for_derivation("height_band", &event.height_band),
        touch_label_for_derivation("surface", &event.surface),
        touch_label_for_derivation("dodge_state", &event.dodge_state),
    ]);
    stats.last_touch_time = Some(event.time);
    stats.last_touch_frame = Some(event.frame);
    stats.time_since_last_touch = Some((frame.time - event.time).max(0.0));
    stats.frames_since_last_touch = Some(frame.frame_number.saturating_sub(event.frame));
    stats.last_ball_speed_change = Some(event.ball_speed_change);
    stats.max_ball_speed_change = stats.max_ball_speed_change.max(event.ball_speed_change);
    stats.cumulative_ball_speed_change += event.ball_speed_change;
}

fn assert_touch_final_additive_stats_close(
    replay_path: &str,
    player_name: &str,
    frame_number: usize,
    actual: &TouchStats,
    expected: &TouchStats,
) {
    assert_eq!(
        actual.touch_count, expected.touch_count,
        "{replay_path} player {player_name} touch.touch_count frame {frame_number}"
    );
    assert_eq!(
        actual.control_touch_count, expected.control_touch_count,
        "{replay_path} player {player_name} touch.control_touch_count frame {frame_number}"
    );
    assert_eq!(
        actual.medium_hit_count, expected.medium_hit_count,
        "{replay_path} player {player_name} touch.medium_hit_count frame {frame_number}"
    );
    assert_eq!(
        actual.hard_hit_count, expected.hard_hit_count,
        "{replay_path} player {player_name} touch.hard_hit_count frame {frame_number}"
    );
    assert_eq!(
        actual.aerial_touch_count, expected.aerial_touch_count,
        "{replay_path} player {player_name} touch.aerial_touch_count frame {frame_number}"
    );
    assert_eq!(
        actual.high_aerial_touch_count, expected.high_aerial_touch_count,
        "{replay_path} player {player_name} touch.high_aerial_touch_count frame {frame_number}"
    );
    assert_eq!(
        actual.wall_touch_count, expected.wall_touch_count,
        "{replay_path} player {player_name} touch.wall_touch_count frame {frame_number}"
    );
    assert!(
        (actual.max_ball_speed_change - expected.max_ball_speed_change).abs() < 0.001,
        "{replay_path} player {player_name} touch.max_ball_speed_change frame {frame_number} actual {:.3} expected {:.3}",
        actual.max_ball_speed_change,
        expected.max_ball_speed_change
    );
    assert!(
        (actual.cumulative_ball_speed_change - expected.cumulative_ball_speed_change).abs() < 0.001,
        "{replay_path} player {player_name} touch.cumulative_ball_speed_change frame {frame_number} actual {:.3} expected {:.3}",
        actual.cumulative_ball_speed_change,
        expected.cumulative_ball_speed_change
    );
    assert!(
        (actual.total_ball_travel_distance - expected.total_ball_travel_distance).abs() < 0.001,
        "{replay_path} player {player_name} touch.total_ball_travel_distance frame {frame_number} actual {:.3} expected {:.3}",
        actual.total_ball_travel_distance,
        expected.total_ball_travel_distance
    );
    assert!(
        (actual.total_ball_advance_distance - expected.total_ball_advance_distance).abs() < 0.001,
        "{replay_path} player {player_name} touch.total_ball_advance_distance frame {frame_number} actual {:.3} expected {:.3}",
        actual.total_ball_advance_distance,
        expected.total_ball_advance_distance
    );
    assert!(
        (actual.total_ball_retreat_distance - expected.total_ball_retreat_distance).abs() < 0.001,
        "{replay_path} player {player_name} touch.total_ball_retreat_distance frame {frame_number} actual {:.3} expected {:.3}",
        actual.total_ball_retreat_distance,
        expected.total_ball_retreat_distance
    );
    assert_eq!(
        actual.labeled_touch_counts, expected.labeled_touch_counts,
        "{replay_path} player {player_name} touch.labeled_touch_counts frame {frame_number}"
    );
}

fn assert_touch_events_reconstruct_final_serialized_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut touch_events = timeline.events.touch.clone();
    touch_events.sort_by(|left, right| {
        left.sample_frame
            .cmp(&right.sample_frame)
            .then_with(|| left.sample_time.total_cmp(&right.sample_time))
            .then_with(|| left.frame.cmp(&right.frame))
            .then_with(|| left.time.total_cmp(&right.time))
    });
    let mut movement_events = timeline.events.touch_ball_movement.clone();
    movement_events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });
    let mut players: HashMap<PlayerId, TouchStats> = HashMap::new();
    let final_frame = timeline
        .frames
        .last()
        .expect("touch reconstruction requires at least one frame");

    for event in &touch_events {
        apply_touch_stats_event_for_derivation(
            players.entry(event.player.clone()).or_default(),
            event,
            final_frame,
        );
    }

    for event in &movement_events {
        let stats = players.entry(event.player.clone()).or_default();
        stats.total_ball_travel_distance += event.travel_distance;
        stats.total_ball_advance_distance += event.advance_distance;
        stats.total_ball_retreat_distance += event.retreat_distance;
    }

    for player in &final_frame.players {
        let expected = players.get(&player.player_id).cloned().unwrap_or_default();
        assert_touch_final_additive_stats_close(
            replay_path,
            &player.name,
            final_frame.frame_number,
            &player.touch,
            &expected,
        );
    }
}

fn assert_core_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut player_events = timeline.events.core_player.clone();
    player_events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });
    let mut team_events = timeline.events.core_team.clone();
    team_events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut player_event_index = 0;
    let mut team_event_index = 0;
    let mut players: HashMap<PlayerId, CorePlayerStats> = HashMap::new();
    let mut team_zero = CoreTeamStats::default();
    let mut team_one = CoreTeamStats::default();

    for frame in &timeline.frames {
        while player_event_index < player_events.len()
            && player_events[player_event_index].frame <= frame.frame_number
        {
            let event = &player_events[player_event_index];
            apply_core_player_delta(
                players.entry(event.player.clone()).or_default(),
                &event.delta,
            );
            player_event_index += 1;
        }

        while team_event_index < team_events.len()
            && team_events[team_event_index].frame <= frame.frame_number
        {
            let event = &team_events[team_event_index];
            if event.is_team_0 {
                apply_core_team_delta(&mut team_zero, &event.delta);
            } else {
                apply_core_team_delta(&mut team_one, &event.delta);
            }
            team_event_index += 1;
        }

        assert_eq!(
            frame.team_zero.core, team_zero,
            "{replay_path} team_zero core frame {}",
            frame.frame_number
        );
        assert_eq!(
            frame.team_one.core, team_one,
            "{replay_path} team_one core frame {}",
            frame.frame_number
        );

        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_eq!(
                player.core, expected,
                "{replay_path} player {} core frame {}",
                player.name, frame.frame_number
            );
        }
    }

    assert_eq!(
        player_event_index,
        player_events.len(),
        "{replay_path} unprocessed core player events"
    );
    assert_eq!(
        team_event_index,
        team_events.len(),
        "{replay_path} unprocessed core team events"
    );
}

fn apply_goal_after_kickoff_delta(
    stats: &mut GoalAfterKickoffStats,
    delta: &GoalAfterKickoffStats,
) {
    if delta.goal_times().is_empty() {
        stats.kickoff_goal_count += delta.kickoff_goal_count;
        stats.short_goal_count += delta.short_goal_count;
        stats.medium_goal_count += delta.medium_goal_count;
        stats.long_goal_count += delta.long_goal_count;
    } else {
        for time in delta.goal_times() {
            stats.record_goal(*time);
        }
    }
}

fn apply_goal_buildup_delta(stats: &mut GoalBuildupStats, delta: &GoalBuildupStats) {
    stats.counter_attack_goal_count += delta.counter_attack_goal_count;
    stats.sustained_pressure_goal_count += delta.sustained_pressure_goal_count;
    stats.other_buildup_goal_count += delta.other_buildup_goal_count;
}

fn apply_goal_ball_air_time_delta(stats: &mut GoalBallAirTimeStats, delta: &GoalBallAirTimeStats) {
    if delta.goal_ball_air_times().is_empty() {
        stats.goal_ball_air_time_sample_count += delta.goal_ball_air_time_sample_count;
        stats.cumulative_goal_ball_air_time += delta.cumulative_goal_ball_air_time;
        if delta.last_goal_ball_air_time.is_some() {
            stats.last_goal_ball_air_time = delta.last_goal_ball_air_time;
        }
    } else {
        let previous_last_goal_ball_air_time = stats.last_goal_ball_air_time;
        for time in delta.goal_ball_air_times() {
            stats.record_goal(*time);
        }
        stats.last_goal_ball_air_time = delta
            .last_goal_ball_air_time
            .or(previous_last_goal_ball_air_time);
    }
}

fn apply_core_team_delta(stats: &mut CoreTeamStats, delta: &CoreTeamStats) {
    stats.score += delta.score;
    stats.goals += delta.goals;
    stats.assists += delta.assists;
    stats.saves += delta.saves;
    stats.shots += delta.shots;
    apply_goal_after_kickoff_delta(
        &mut stats.scoring_context.goal_after_kickoff,
        &delta.scoring_context.goal_after_kickoff,
    );
    apply_goal_buildup_delta(
        &mut stats.scoring_context.goal_buildup,
        &delta.scoring_context.goal_buildup,
    );
    apply_goal_ball_air_time_delta(
        &mut stats.scoring_context.goal_ball_air_time,
        &delta.scoring_context.goal_ball_air_time,
    );
}

fn apply_core_player_delta(stats: &mut CorePlayerStats, delta: &CorePlayerStats) {
    stats.score += delta.score;
    stats.goals += delta.goals;
    stats.assists += delta.assists;
    stats.saves += delta.saves;
    stats.shots += delta.shots;
    stats.scoring_context.goals_conceded_while_last_defender +=
        delta.scoring_context.goals_conceded_while_last_defender;
    stats.scoring_context.goals_for_while_most_back +=
        delta.scoring_context.goals_for_while_most_back;
    stats.scoring_context.goals_against_while_most_back +=
        delta.scoring_context.goals_against_while_most_back;
    stats.scoring_context.goal_against_boost_sample_count +=
        delta.scoring_context.goal_against_boost_sample_count;
    stats.scoring_context.cumulative_boost_on_goals_against +=
        delta.scoring_context.cumulative_boost_on_goals_against;
    if delta.scoring_context.last_boost_on_goal_against.is_some() {
        stats.scoring_context.last_boost_on_goal_against =
            delta.scoring_context.last_boost_on_goal_against;
    }
    stats.scoring_context.goal_against_boost_leadup_sample_count +=
        delta.scoring_context.goal_against_boost_leadup_sample_count;
    stats
        .scoring_context
        .cumulative_average_boost_in_goal_against_leadup += delta
        .scoring_context
        .cumulative_average_boost_in_goal_against_leadup;
    stats
        .scoring_context
        .cumulative_min_boost_in_goal_against_leadup += delta
        .scoring_context
        .cumulative_min_boost_in_goal_against_leadup;
    if delta
        .scoring_context
        .last_average_boost_in_goal_against_leadup
        .is_some()
    {
        stats
            .scoring_context
            .last_average_boost_in_goal_against_leadup = delta
            .scoring_context
            .last_average_boost_in_goal_against_leadup;
    }
    if delta
        .scoring_context
        .last_min_boost_in_goal_against_leadup
        .is_some()
    {
        stats.scoring_context.last_min_boost_in_goal_against_leadup =
            delta.scoring_context.last_min_boost_in_goal_against_leadup;
    }
    stats.scoring_context.goal_against_position_sample_count +=
        delta.scoring_context.goal_against_position_sample_count;
    stats.scoring_context.cumulative_goal_against_position_x +=
        delta.scoring_context.cumulative_goal_against_position_x;
    stats.scoring_context.cumulative_goal_against_position_y +=
        delta.scoring_context.cumulative_goal_against_position_y;
    stats.scoring_context.cumulative_goal_against_position_z +=
        delta.scoring_context.cumulative_goal_against_position_z;
    if delta.scoring_context.last_goal_against_position.is_some() {
        stats.scoring_context.last_goal_against_position =
            delta.scoring_context.last_goal_against_position;
    }
    stats
        .scoring_context
        .scoring_goal_last_touch_position_sample_count += delta
        .scoring_context
        .scoring_goal_last_touch_position_sample_count;
    stats
        .scoring_context
        .cumulative_scoring_goal_last_touch_position_x += delta
        .scoring_context
        .cumulative_scoring_goal_last_touch_position_x;
    stats
        .scoring_context
        .cumulative_scoring_goal_last_touch_position_y += delta
        .scoring_context
        .cumulative_scoring_goal_last_touch_position_y;
    stats
        .scoring_context
        .cumulative_scoring_goal_last_touch_position_z += delta
        .scoring_context
        .cumulative_scoring_goal_last_touch_position_z;
    if delta
        .scoring_context
        .last_scoring_goal_last_touch_position
        .is_some()
    {
        stats.scoring_context.last_scoring_goal_last_touch_position =
            delta.scoring_context.last_scoring_goal_last_touch_position;
    }
    apply_goal_after_kickoff_delta(
        &mut stats.scoring_context.goal_after_kickoff,
        &delta.scoring_context.goal_after_kickoff,
    );
    apply_goal_buildup_delta(
        &mut stats.scoring_context.goal_buildup,
        &delta.scoring_context.goal_buildup,
    );
    apply_goal_ball_air_time_delta(
        &mut stats.scoring_context.goal_ball_air_time,
        &delta.scoring_context.goal_ball_air_time,
    );
}

fn possession_label_for_derivation(key: &'static str, value: &str) -> StatLabel {
    match (key, value) {
        ("possession_state", "team_zero") => StatLabel::new("possession_state", "team_zero"),
        ("possession_state", "team_one") => StatLabel::new("possession_state", "team_one"),
        ("possession_state", "neutral") => StatLabel::new("possession_state", "neutral"),
        ("field_third", "team_zero_third") => StatLabel::new("field_third", "team_zero_third"),
        ("field_third", "neutral_third") => StatLabel::new("field_third", "neutral_third"),
        ("field_third", "team_one_third") => StatLabel::new("field_third", "team_one_third"),
        _ => panic!("unexpected possession label {key}={value}"),
    }
}

#[derive(Debug, Clone, Default)]
struct PossessionDerivationState {
    active: bool,
    possession_state: String,
    field_third: Option<String>,
}

fn apply_possession_event_for_derivation(
    state: &mut PossessionDerivationState,
    event: &PossessionEvent,
) {
    state.active = event.active;
    state.possession_state = event.possession_state.clone();
    state.field_third = event.field_third.clone();
}

fn accumulate_possession_frame_for_derivation(
    stats: &mut PossessionStats,
    state: &PossessionDerivationState,
    frame: &ReplayStatsFrame,
) {
    if !state.active {
        return;
    }

    stats.tracked_time += frame.dt;
    match state.possession_state.as_str() {
        "team_zero" => stats.team_zero_time += frame.dt,
        "team_one" => stats.team_one_time += frame.dt,
        "neutral" => stats.neutral_time += frame.dt,
        value => panic!("unexpected possession state {value}"),
    }

    let state_label = possession_label_for_derivation("possession_state", &state.possession_state);
    if let Some(field_third) = state.field_third.as_deref() {
        stats.labeled_time.add(
            [
                state_label,
                possession_label_for_derivation("field_third", field_third),
            ],
            frame.dt,
        );
    } else {
        stats.labeled_time.add([state_label], frame.dt);
    }
}

fn assert_labeled_float_sums_close(
    replay_path: &str,
    label: &str,
    frame_number: usize,
    actual: &LabeledFloatSums,
    expected: &LabeledFloatSums,
) {
    assert_eq!(
        actual.entries.len(),
        expected.entries.len(),
        "{replay_path} {label}.labeled_time entry count frame {frame_number}"
    );
    for (actual_entry, expected_entry) in actual.entries.iter().zip(&expected.entries) {
        assert_eq!(
            actual_entry.labels, expected_entry.labels,
            "{replay_path} {label}.labeled_time labels frame {frame_number}"
        );
        assert!(
            (actual_entry.value - expected_entry.value).abs() < 0.001,
            "{replay_path} {label}.labeled_time {:?} frame {frame_number} actual {:.3} expected {:.3}",
            actual_entry.labels,
            actual_entry.value,
            expected_entry.value
        );
    }
}

fn assert_possession_team_stats_close(
    replay_path: &str,
    label: &str,
    frame_number: usize,
    actual: &PossessionTeamStats,
    expected: &PossessionTeamStats,
) {
    assert!(
        (actual.tracked_time - expected.tracked_time).abs() < 0.001,
        "{replay_path} {label}.tracked_time frame {frame_number} actual {:.3} expected {:.3}",
        actual.tracked_time,
        expected.tracked_time
    );
    assert!(
        (actual.possession_time - expected.possession_time).abs() < 0.001,
        "{replay_path} {label}.possession_time frame {frame_number} actual {:.3} expected {:.3}",
        actual.possession_time,
        expected.possession_time
    );
    assert!(
        (actual.opponent_possession_time - expected.opponent_possession_time).abs() < 0.001,
        "{replay_path} {label}.opponent_possession_time frame {frame_number} actual {:.3} expected {:.3}",
        actual.opponent_possession_time,
        expected.opponent_possession_time
    );
    assert!(
        (actual.neutral_time - expected.neutral_time).abs() < 0.001,
        "{replay_path} {label}.neutral_time frame {frame_number} actual {:.3} expected {:.3}",
        actual.neutral_time,
        expected.neutral_time
    );
    assert_labeled_float_sums_close(
        replay_path,
        label,
        frame_number,
        &actual.labeled_time,
        &expected.labeled_time,
    );
}

fn assert_possession_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.possession.clone();
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut stats = PossessionStats::default();
    let mut state = PossessionDerivationState {
        active: false,
        possession_state: "neutral".to_owned(),
        field_third: None,
    };

    for frame in &timeline.frames {
        while event_index < events.len() && events[event_index].frame <= frame.frame_number {
            apply_possession_event_for_derivation(&mut state, &events[event_index]);
            event_index += 1;
        }

        accumulate_possession_frame_for_derivation(&mut stats, &state, frame);
        assert_possession_team_stats_close(
            replay_path,
            "team_zero.possession",
            frame.frame_number,
            &frame.team_zero.possession,
            &stats.for_team(true),
        );
        assert_possession_team_stats_close(
            replay_path,
            "team_one.possession",
            frame.frame_number,
            &frame.team_one.possession,
            &stats.for_team(false),
        );
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed possession events"
    );
}
