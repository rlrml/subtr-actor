fn pressure_label_for_derivation(value: &str) -> StatLabel {
    match value {
        "team_zero_side" => StatLabel::new("field_half", "team_zero_side"),
        "team_one_side" => StatLabel::new("field_half", "team_one_side"),
        "neutral" => StatLabel::new("field_half", "neutral"),
        _ => panic!("unexpected pressure field_half={value}"),
    }
}

#[derive(Debug, Clone)]
struct PressureDerivationState {
    active: bool,
    field_half: String,
}

impl Default for PressureDerivationState {
    fn default() -> Self {
        Self {
            active: false,
            field_half: "neutral".to_owned(),
        }
    }
}

fn apply_pressure_event_for_derivation(state: &mut PressureDerivationState, event: &PressureEvent) {
    state.active = event.active;
    state.field_half = event.field_half.clone();
}

fn accumulate_pressure_frame_for_derivation(
    stats: &mut PressureStats,
    state: &PressureDerivationState,
    frame: &ReplayStatsFrame,
) {
    if !state.active {
        return;
    }

    stats.tracked_time += frame.dt;
    match state.field_half.as_str() {
        "team_zero_side" => stats.team_zero_side_time += frame.dt,
        "team_one_side" => stats.team_one_side_time += frame.dt,
        "neutral" => stats.neutral_time += frame.dt,
        value => panic!("unexpected pressure field half {value}"),
    }
    stats
        .labeled_time
        .add([pressure_label_for_derivation(&state.field_half)], frame.dt);
}

fn assert_pressure_team_stats_close(
    replay_path: &str,
    label: &str,
    frame_number: usize,
    actual: &PressureTeamStats,
    expected: &PressureTeamStats,
) {
    assert!(
        (actual.tracked_time - expected.tracked_time).abs() < 0.001,
        "{replay_path} {label}.tracked_time frame {frame_number} actual {:.3} expected {:.3}",
        actual.tracked_time,
        expected.tracked_time
    );
    assert!(
        (actual.defensive_half_time - expected.defensive_half_time).abs() < 0.001,
        "{replay_path} {label}.defensive_half_time frame {frame_number} actual {:.3} expected {:.3}",
        actual.defensive_half_time,
        expected.defensive_half_time
    );
    assert!(
        (actual.offensive_half_time - expected.offensive_half_time).abs() < 0.001,
        "{replay_path} {label}.offensive_half_time frame {frame_number} actual {:.3} expected {:.3}",
        actual.offensive_half_time,
        expected.offensive_half_time
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

fn assert_pressure_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.pressure.clone();
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut stats = PressureStats::default();
    let mut state = PressureDerivationState::default();

    for frame in &timeline.frames {
        while event_index < events.len() && events[event_index].frame <= frame.frame_number {
            apply_pressure_event_for_derivation(&mut state, &events[event_index]);
            event_index += 1;
        }

        accumulate_pressure_frame_for_derivation(&mut stats, &state, frame);
        assert_pressure_team_stats_close(
            replay_path,
            "team_zero.pressure",
            frame.frame_number,
            &frame.team_zero.pressure,
            &stats.for_team(true),
        );
        assert_pressure_team_stats_close(
            replay_path,
            "team_one.pressure",
            frame.frame_number,
            &frame.team_one.pressure,
            &stats.for_team(false),
        );
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed pressure events"
    );
}

fn movement_label_for_derivation(key: &'static str, value: &str) -> StatLabel {
    match (key, value) {
        ("speed_band", "slow") => StatLabel::new("speed_band", "slow"),
        ("speed_band", "boost") => StatLabel::new("speed_band", "boost"),
        ("speed_band", "supersonic") => StatLabel::new("speed_band", "supersonic"),
        ("height_band", "ground") => StatLabel::new("height_band", "ground"),
        ("height_band", "low_air") => StatLabel::new("height_band", "low_air"),
        ("height_band", "high_air") => StatLabel::new("height_band", "high_air"),
        _ => panic!("unexpected movement label {key}={value}"),
    }
}

fn apply_movement_event_for_derivation(stats: &mut MovementStats, event: &MovementEvent) {
    stats.tracked_time += event.dt;
    stats.total_distance += event.distance;
    stats.speed_integral += event.speed * event.dt;

    match event.speed_band.as_str() {
        "slow" => stats.time_slow_speed += event.dt,
        "boost" => stats.time_boost_speed += event.dt,
        "supersonic" => stats.time_supersonic_speed += event.dt,
        value => panic!("unexpected movement speed band {value}"),
    }

    match event.height_band.as_str() {
        "ground" => stats.time_on_ground += event.dt,
        "low_air" => stats.time_low_air += event.dt,
        "high_air" => stats.time_high_air += event.dt,
        value => panic!("unexpected movement height band {value}"),
    }

    stats.labeled_tracked_time.add(
        [
            movement_label_for_derivation("speed_band", &event.speed_band),
            movement_label_for_derivation("height_band", &event.height_band),
        ],
        event.dt,
    );
}

fn assert_movement_stats_close(
    replay_path: &str,
    label: &str,
    frame_number: usize,
    actual: &MovementStats,
    expected: &MovementStats,
) {
    assert!(
        (actual.tracked_time - expected.tracked_time).abs() < 0.001,
        "{replay_path} {label}.tracked_time frame {frame_number} actual {:.3} expected {:.3}",
        actual.tracked_time,
        expected.tracked_time
    );
    assert!(
        (actual.total_distance - expected.total_distance).abs() < 0.001,
        "{replay_path} {label}.total_distance frame {frame_number} actual {:.3} expected {:.3}",
        actual.total_distance,
        expected.total_distance
    );
    assert!(
        (actual.speed_integral - expected.speed_integral).abs() < 0.001,
        "{replay_path} {label}.speed_integral frame {frame_number} actual {:.3} expected {:.3}",
        actual.speed_integral,
        expected.speed_integral
    );
    assert!(
        (actual.time_slow_speed - expected.time_slow_speed).abs() < 0.001,
        "{replay_path} {label}.time_slow_speed frame {frame_number} actual {:.3} expected {:.3}",
        actual.time_slow_speed,
        expected.time_slow_speed
    );
    assert!(
        (actual.time_boost_speed - expected.time_boost_speed).abs() < 0.001,
        "{replay_path} {label}.time_boost_speed frame {frame_number} actual {:.3} expected {:.3}",
        actual.time_boost_speed,
        expected.time_boost_speed
    );
    assert!(
        (actual.time_supersonic_speed - expected.time_supersonic_speed).abs() < 0.001,
        "{replay_path} {label}.time_supersonic_speed frame {frame_number} actual {:.3} expected {:.3}",
        actual.time_supersonic_speed,
        expected.time_supersonic_speed
    );
    assert!(
        (actual.time_on_ground - expected.time_on_ground).abs() < 0.001,
        "{replay_path} {label}.time_on_ground frame {frame_number} actual {:.3} expected {:.3}",
        actual.time_on_ground,
        expected.time_on_ground
    );
    assert!(
        (actual.time_low_air - expected.time_low_air).abs() < 0.001,
        "{replay_path} {label}.time_low_air frame {frame_number} actual {:.3} expected {:.3}",
        actual.time_low_air,
        expected.time_low_air
    );
    assert!(
        (actual.time_high_air - expected.time_high_air).abs() < 0.001,
        "{replay_path} {label}.time_high_air frame {frame_number} actual {:.3} expected {:.3}",
        actual.time_high_air,
        expected.time_high_air
    );
    assert_labeled_float_sums_close(
        replay_path,
        label,
        frame_number,
        &actual.labeled_tracked_time,
        &expected.labeled_tracked_time,
    );
}

fn assert_movement_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.movement.clone();
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, MovementStats> = HashMap::new();
    let mut team_zero = MovementStats::default();
    let mut team_one = MovementStats::default();

    for frame in &timeline.frames {
        while event_index < events.len() && events[event_index].frame <= frame.frame_number {
            let event = &events[event_index];
            apply_movement_event_for_derivation(
                players.entry(event.player.clone()).or_default(),
                event,
            );
            if event.is_team_0 {
                apply_movement_event_for_derivation(&mut team_zero, event);
            } else {
                apply_movement_event_for_derivation(&mut team_one, event);
            }
            event_index += 1;
        }

        assert_movement_stats_close(
            replay_path,
            "team_zero.movement",
            frame.frame_number,
            &frame.team_zero.movement,
            &team_zero,
        );
        assert_movement_stats_close(
            replay_path,
            "team_one.movement",
            frame.frame_number,
            &frame.team_one.movement,
            &team_one,
        );

        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_movement_stats_close(
                replay_path,
                &format!("player {} movement", player.name),
                frame.frame_number,
                &player.movement,
                &expected,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed movement events"
    );
}

fn assert_movement_events_reconstruct_final_serialized_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut players: HashMap<PlayerId, MovementStats> = HashMap::new();
    let mut team_zero = MovementStats::default();
    let mut team_one = MovementStats::default();

    for event in &timeline.events.movement {
        apply_movement_event_for_derivation(players.entry(event.player.clone()).or_default(), event);
        if event.is_team_0 {
            apply_movement_event_for_derivation(&mut team_zero, event);
        } else {
            apply_movement_event_for_derivation(&mut team_one, event);
        }
    }

    let final_frame = timeline
        .frames
        .last()
        .expect("movement reconstruction requires at least one frame");
    assert_movement_stats_close(
        replay_path,
        "team_zero.movement",
        final_frame.frame_number,
        &final_frame.team_zero.movement,
        &team_zero,
    );
    assert_movement_stats_close(
        replay_path,
        "team_one.movement",
        final_frame.frame_number,
        &final_frame.team_one.movement,
        &team_one,
    );

    for player in &final_frame.players {
        let expected = players.get(&player.player_id).cloned().unwrap_or_default();
        assert_movement_stats_close(
            replay_path,
            &format!("player {} movement", player.name),
            final_frame.frame_number,
            &player.movement,
            &expected,
        );
    }
}

fn assert_positioning_stats_close(
    replay_path: &str,
    label: &str,
    frame_number: usize,
    actual: &PositioningStats,
    expected: &PositioningStats,
) {
    macro_rules! assert_close_field {
        ($field:ident) => {
            assert!(
                (actual.$field - expected.$field).abs() < 0.001,
                "{replay_path} {label}.{} frame {frame_number} actual {:.3} expected {:.3}",
                stringify!($field),
                actual.$field,
                expected.$field
            );
        };
    }

    assert_close_field!(active_game_time);
    assert_close_field!(tracked_time);
    assert_close_field!(sum_distance_to_teammates);
    assert_close_field!(sum_distance_to_ball);
    assert_close_field!(sum_distance_to_ball_has_possession);
    assert_close_field!(time_has_possession);
    assert_close_field!(sum_distance_to_ball_no_possession);
    assert_close_field!(time_no_possession);
    assert_close_field!(time_demolished);
    assert_close_field!(time_no_teammates);
    assert_close_field!(time_most_back);
    assert_close_field!(time_most_forward);
    assert_close_field!(time_mid_role);
    assert_close_field!(time_other_role);
    assert_close_field!(time_defensive_zone);
    assert_close_field!(time_neutral_zone);
    assert_close_field!(time_offensive_zone);
    assert_close_field!(time_defensive_half);
    assert_close_field!(time_offensive_half);
    assert_close_field!(time_closest_to_ball);
    assert_close_field!(time_farthest_from_ball);
    assert_close_field!(time_behind_ball);
    assert_close_field!(time_level_with_ball);
    assert_close_field!(time_in_front_of_ball);
    assert_eq!(
        actual.times_caught_ahead_of_play_on_conceded_goals,
        expected.times_caught_ahead_of_play_on_conceded_goals,
        "{replay_path} {label}.times_caught_ahead_of_play_on_conceded_goals frame {frame_number}"
    );
}

fn assert_positioning_events_reconstruct_final_serialized_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut accumulator = PositioningStatsAccumulator::new();
    for event in &timeline.events.positioning_activity {
        accumulator.apply_activity_event(event);
    }
    for event in &timeline.events.positioning_possession {
        accumulator.apply_possession_event(event);
    }
    for event in &timeline.events.positioning_field_zone {
        accumulator.apply_field_zone_event(event);
    }
    for event in &timeline.events.positioning_ball_depth {
        accumulator.apply_ball_depth_event(event);
    }
    for event in &timeline.events.positioning_teammate_role {
        accumulator.apply_teammate_role_event(event);
    }
    for event in &timeline.events.positioning_ball_proximity {
        accumulator.apply_ball_proximity_event(event);
    }
    for event in &timeline.events.positioning_goal_context {
        accumulator.apply_goal_context_event(event);
    }

    let final_frame = timeline
        .frames
        .last()
        .expect("positioning reconstruction requires at least one frame");
    for player in &final_frame.players {
        let mut expected = accumulator
            .player_stats()
            .get(&player.player_id)
            .cloned()
            .unwrap_or_default();
        // Distance magnitudes are a continuous signal shipped on the frame snapshot, not
        // reconstructed from events, so carry them through from the authoritative snapshot.
        // (The signal itself is validated against the export in the scaffold parity test.)
        // Possession time IS event-reconstructed (positioning_possession), so it is asserted.
        expected.sum_distance_to_teammates = player.positioning.sum_distance_to_teammates;
        expected.sum_distance_to_ball = player.positioning.sum_distance_to_ball;
        expected.sum_distance_to_ball_has_possession =
            player.positioning.sum_distance_to_ball_has_possession;
        expected.sum_distance_to_ball_no_possession =
            player.positioning.sum_distance_to_ball_no_possession;
        assert_positioning_stats_close(
            replay_path,
            &format!("player {} positioning", player.name),
            final_frame.frame_number,
            &player.positioning,
            &expected,
        );
    }
}

fn assert_rotation_player_stats_close(
    replay_path: &str,
    label: &str,
    frame_number: usize,
    actual: &RotationPlayerStats,
    expected: &RotationPlayerStats,
) {
    macro_rules! assert_close_field {
        ($field:ident) => {
            assert!(
                (actual.$field - expected.$field).abs() < 0.001,
                "{replay_path} {label}.{} frame {frame_number} actual {:.3} expected {:.3}",
                stringify!($field),
                actual.$field,
                expected.$field
            );
        };
    }

    assert_close_field!(active_game_time);
    assert_close_field!(tracked_time);
    assert_close_field!(time_first_man);
    assert_close_field!(time_second_man);
    assert_close_field!(time_third_man);
    assert_close_field!(time_ambiguous_role);
    assert_close_field!(time_behind_play);
    assert_close_field!(time_level_with_play);
    assert_close_field!(time_ahead_of_play);
    assert_close_field!(longest_first_man_stint_time);
    assert_eq!(
        actual.first_man_stint_count, expected.first_man_stint_count,
        "{replay_path} {label}.first_man_stint_count frame {frame_number}"
    );
    assert_eq!(
        actual.became_first_man_count, expected.became_first_man_count,
        "{replay_path} {label}.became_first_man_count frame {frame_number}"
    );
    assert_eq!(
        actual.lost_first_man_count, expected.lost_first_man_count,
        "{replay_path} {label}.lost_first_man_count frame {frame_number}"
    );
    assert_eq!(
        actual.current_role_state, expected.current_role_state,
        "{replay_path} {label}.current_role_state frame {frame_number}"
    );
    assert_eq!(
        actual.current_depth_state, expected.current_depth_state,
        "{replay_path} {label}.current_depth_state frame {frame_number}"
    );
}

fn assert_rotation_team_stats_equal(
    replay_path: &str,
    label: &str,
    frame_number: usize,
    actual: &RotationTeamStats,
    expected: &RotationTeamStats,
) {
    assert_eq!(
        actual.first_man_changes_for_team, expected.first_man_changes_for_team,
        "{replay_path} {label}.first_man_changes_for_team frame {frame_number}"
    );
    assert_eq!(
        actual.rotation_count, expected.rotation_count,
        "{replay_path} {label}.rotation_count frame {frame_number}"
    );
}

fn assert_rotation_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut player_events = timeline.events.rotation_player.clone();
    player_events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });
    let mut team_events = timeline.events.rotation_team.clone();
    team_events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut player_event_index = 0;
    let mut team_event_index = 0;
    let mut accumulator = RotationStatsAccumulator::with_first_man_stint_end_grace_seconds(
        timeline.config.rotation_first_man_debounce_seconds,
    );

    for frame in &timeline.frames {
        while player_event_index < player_events.len()
            && player_events[player_event_index].end_frame <= frame.frame_number
        {
            let event = &player_events[player_event_index];
            accumulator.apply_player_event(event);
            player_event_index += 1;
        }

        while team_event_index < team_events.len()
            && team_events[team_event_index].frame <= frame.frame_number
        {
            let event = &team_events[team_event_index];
            accumulator.apply_team_event(event);
            team_event_index += 1;
        }

        assert_rotation_team_stats_equal(
            replay_path,
            "team_zero.rotation",
            frame.frame_number,
            &frame.team_zero.rotation,
            accumulator.team_zero_stats(),
        );
        assert_rotation_team_stats_equal(
            replay_path,
            "team_one.rotation",
            frame.frame_number,
            &frame.team_one.rotation,
            accumulator.team_one_stats(),
        );

        for player in &frame.players {
            let expected = accumulator
                .player_stats()
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default();
            assert_rotation_player_stats_close(
                replay_path,
                &format!("player {} rotation", player.name),
                frame.frame_number,
                &player.rotation,
                &expected,
            );
        }
    }

    assert_eq!(
        player_event_index,
        player_events.len(),
        "{replay_path} unprocessed rotation player events"
    );
    assert_eq!(
        team_event_index,
        team_events.len(),
        "{replay_path} unprocessed rotation team events"
    );
}

fn fifty_fifty_phase_label_for_derivation(is_kickoff: bool) -> StatLabel {
    if is_kickoff {
        StatLabel::new("phase", "kickoff")
    } else {
        StatLabel::new("phase", "open_play")
    }
}

fn fifty_fifty_player_outcome_label_for_derivation(
    player_team_is_team_0: bool,
    winning_team_is_team_0: Option<bool>,
) -> StatLabel {
    match winning_team_is_team_0 {
        Some(winning_team) if winning_team == player_team_is_team_0 => {
            StatLabel::new("outcome", "win")
        }
        Some(_) => StatLabel::new("outcome", "loss"),
        None => StatLabel::new("outcome", "neutral"),
    }
}

fn fifty_fifty_player_possession_label_for_derivation(
    player_team_is_team_0: bool,
    possession_team_is_team_0: Option<bool>,
) -> StatLabel {
    match possession_team_is_team_0 {
        Some(possession_team) if possession_team == player_team_is_team_0 => {
            StatLabel::new("possession_after", "self")
        }
        Some(_) => StatLabel::new("possession_after", "opponent"),
        None => StatLabel::new("possession_after", "neutral"),
    }
}

fn fifty_fifty_player_dodge_state_label_for_derivation(
    player_team_is_team_0: bool,
    event: &FiftyFiftyEvent,
) -> StatLabel {
    let dodge_contact = if player_team_is_team_0 {
        event.team_zero_dodge_contact
    } else {
        event.team_one_dodge_contact
    };
    if dodge_contact {
        StatLabel::new("dodge_state", "dodge")
    } else {
        StatLabel::new("dodge_state", "no_dodge")
    }
}

fn apply_fifty_fifty_team_event(
    stats: &mut FiftyFiftyTeamStats,
    team_is_team_0: bool,
    event: &FiftyFiftyEvent,
) {
    stats.count += 1;
    match event.winning_team_is_team_0 {
        Some(winning_team) if winning_team == team_is_team_0 => stats.wins += 1,
        Some(_) => stats.losses += 1,
        None => stats.neutral_outcomes += 1,
    }
    match event.possession_team_is_team_0 {
        Some(possession_team) if possession_team == team_is_team_0 => {
            stats.possession_after_count += 1;
        }
        Some(_) => stats.opponent_possession_after_count += 1,
        None => stats.neutral_possession_after_count += 1,
    }
    if event.is_kickoff {
        stats.kickoff_count += 1;
        match event.winning_team_is_team_0 {
            Some(winning_team) if winning_team == team_is_team_0 => stats.kickoff_wins += 1,
            Some(_) => stats.kickoff_losses += 1,
            None => stats.kickoff_neutral_outcomes += 1,
        }
        match event.possession_team_is_team_0 {
            Some(possession_team) if possession_team == team_is_team_0 => {
                stats.kickoff_possession_after_count += 1;
            }
            Some(_) => stats.kickoff_opponent_possession_after_count += 1,
            None => stats.kickoff_neutral_possession_after_count += 1,
        }
    }
}

fn apply_fifty_fifty_player_event(
    stats: &mut FiftyFiftyPlayerStats,
    player_team_is_team_0: bool,
    event: &FiftyFiftyEvent,
) {
    stats.labeled_event_counts.increment([
        fifty_fifty_phase_label_for_derivation(event.is_kickoff),
        fifty_fifty_player_outcome_label_for_derivation(
            player_team_is_team_0,
            event.winning_team_is_team_0,
        ),
        fifty_fifty_player_possession_label_for_derivation(
            player_team_is_team_0,
            event.possession_team_is_team_0,
        ),
        fifty_fifty_player_dodge_state_label_for_derivation(player_team_is_team_0, event),
    ]);
    stats.count += 1;
    match event.winning_team_is_team_0 {
        Some(winning_team) if winning_team == player_team_is_team_0 => stats.wins += 1,
        Some(_) => stats.losses += 1,
        None => stats.neutral_outcomes += 1,
    }
    if event.possession_team_is_team_0 == Some(player_team_is_team_0) {
        stats.possession_after_count += 1;
    }
    if event.is_kickoff {
        stats.kickoff_count += 1;
        match event.winning_team_is_team_0 {
            Some(winning_team) if winning_team == player_team_is_team_0 => stats.kickoff_wins += 1,
            Some(_) => stats.kickoff_losses += 1,
            None => stats.kickoff_neutral_outcomes += 1,
        }
        if event.possession_team_is_team_0 == Some(player_team_is_team_0) {
            stats.kickoff_possession_after_count += 1;
        }
    }
}
