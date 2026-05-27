use std::collections::HashMap;

use subtr_actor::*;

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

pub fn assert_possession_events_reconstruct_serialized_partial_sums(
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

pub fn assert_pressure_events_reconstruct_serialized_partial_sums(
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

pub fn assert_movement_events_reconstruct_serialized_partial_sums(
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

fn apply_positioning_event_for_derivation(stats: &mut PositioningStats, event: &PositioningEvent) {
    stats.active_game_time += event.active_game_time;
    stats.tracked_time += event.tracked_time;
    stats.sum_distance_to_teammates += event.sum_distance_to_teammates;
    stats.sum_distance_to_ball += event.sum_distance_to_ball;
    stats.sum_distance_to_ball_has_possession += event.sum_distance_to_ball_has_possession;
    stats.time_has_possession += event.time_has_possession;
    stats.sum_distance_to_ball_no_possession += event.sum_distance_to_ball_no_possession;
    stats.time_no_possession += event.time_no_possession;
    stats.time_demolished += event.time_demolished;
    stats.time_no_teammates += event.time_no_teammates;
    stats.time_most_back += event.time_most_back;
    stats.time_most_forward += event.time_most_forward;
    stats.time_mid_role += event.time_mid_role;
    stats.time_other_role += event.time_other_role;
    stats.time_defensive_zone += event.time_defensive_zone;
    stats.time_neutral_zone += event.time_neutral_zone;
    stats.time_offensive_zone += event.time_offensive_zone;
    stats.time_defensive_half += event.time_defensive_half;
    stats.time_offensive_half += event.time_offensive_half;
    stats.time_closest_to_ball += event.time_closest_to_ball;
    stats.time_farthest_from_ball += event.time_farthest_from_ball;
    stats.time_behind_ball += event.time_behind_ball;
    stats.time_level_with_ball += event.time_level_with_ball;
    stats.time_in_front_of_ball += event.time_in_front_of_ball;
    stats.times_caught_ahead_of_play_on_conceded_goals +=
        event.times_caught_ahead_of_play_on_conceded_goals;
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

pub fn assert_positioning_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.positioning.clone();
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, PositioningStats> = HashMap::new();

    for frame in &timeline.frames {
        while event_index < events.len() && events[event_index].frame <= frame.frame_number {
            let event = &events[event_index];
            apply_positioning_event_for_derivation(
                players.entry(event.player.clone()).or_default(),
                event,
            );
            event_index += 1;
        }

        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_positioning_stats_close(
                replay_path,
                &format!("player {} positioning", player.name),
                frame.frame_number,
                &player.positioning,
                &expected,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed positioning events"
    );
}

#[derive(Debug, Clone, Default)]
struct RotationPlayerDerivationState {
    active: bool,
    first_man_stint_active: bool,
    current_first_man_stint_time: f32,
    non_first_man_seconds: f32,
    stats: RotationPlayerStats,
}

fn apply_rotation_player_event_for_derivation(
    state: &mut RotationPlayerDerivationState,
    event: &RotationPlayerEvent,
) {
    state.active = event.active;
    if !event.active {
        state.first_man_stint_active = false;
        state.current_first_man_stint_time = 0.0;
        state.non_first_man_seconds = 0.0;
    }
    let stats = &mut state.stats;
    stats.became_first_man_count += event.became_first_man_count;
    stats.lost_first_man_count += event.lost_first_man_count;
    stats.current_role_state = event.current_role_state;
    stats.current_depth_state = event.current_depth_state;
}

fn accumulate_rotation_player_frame_for_derivation(
    state: &mut RotationPlayerDerivationState,
    frame: &ReplayStatsFrame,
    first_man_stint_end_grace_seconds: f32,
) {
    if !state.active {
        return;
    }

    state.stats.active_game_time += frame.dt;
    state.stats.tracked_time += frame.dt;

    match state.stats.current_role_state {
        RoleState::FirstMan => {
            if !state.first_man_stint_active {
                state.first_man_stint_active = true;
                state.current_first_man_stint_time = 0.0;
                state.stats.first_man_stint_count += 1;
            }
            state.current_first_man_stint_time += frame.dt;
            state.stats.longest_first_man_stint_time = state
                .stats
                .longest_first_man_stint_time
                .max(state.current_first_man_stint_time);
            state.non_first_man_seconds = 0.0;
            state.stats.time_first_man += frame.dt;
        }
        RoleState::SecondMan => {
            update_non_first_man_stint_state(state, frame.dt, first_man_stint_end_grace_seconds);
            state.stats.time_second_man += frame.dt;
        }
        RoleState::ThirdMan => {
            update_non_first_man_stint_state(state, frame.dt, first_man_stint_end_grace_seconds);
            state.stats.time_third_man += frame.dt;
        }
        RoleState::Ambiguous => {
            update_non_first_man_stint_state(state, frame.dt, first_man_stint_end_grace_seconds);
            state.stats.time_ambiguous_role += frame.dt;
        }
        RoleState::Unknown => {
            update_non_first_man_stint_state(state, frame.dt, first_man_stint_end_grace_seconds)
        }
    }

    match state.stats.current_depth_state {
        PlayDepthState::BehindPlay => state.stats.time_behind_play += frame.dt,
        PlayDepthState::LevelWithPlay => state.stats.time_level_with_play += frame.dt,
        PlayDepthState::AheadOfPlay => state.stats.time_ahead_of_play += frame.dt,
        PlayDepthState::Unknown => {}
    }
}

fn update_non_first_man_stint_state(
    state: &mut RotationPlayerDerivationState,
    dt: f32,
    first_man_stint_end_grace_seconds: f32,
) {
    if !state.first_man_stint_active {
        return;
    }

    state.non_first_man_seconds += dt;
    if state.non_first_man_seconds > first_man_stint_end_grace_seconds {
        state.first_man_stint_active = false;
        state.current_first_man_stint_time = 0.0;
        state.non_first_man_seconds = 0.0;
    }
}

fn apply_rotation_team_event_for_derivation(
    stats: &mut RotationTeamStats,
    event: &RotationTeamEvent,
) {
    stats.first_man_changes_for_team += event.first_man_changes_for_team;
    stats.rotation_count += event.rotation_count;
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

pub fn assert_rotation_events_reconstruct_serialized_partial_sums(
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
    let mut players: HashMap<PlayerId, RotationPlayerDerivationState> = HashMap::new();
    let mut team_zero = RotationTeamStats::default();
    let mut team_one = RotationTeamStats::default();
    let first_man_stint_end_grace_seconds = timeline.config.rotation_first_man_debounce_seconds;

    for frame in &timeline.frames {
        while player_event_index < player_events.len()
            && player_events[player_event_index].frame <= frame.frame_number
        {
            let event = &player_events[player_event_index];
            apply_rotation_player_event_for_derivation(
                players.entry(event.player.clone()).or_default(),
                event,
            );
            player_event_index += 1;
        }

        while team_event_index < team_events.len()
            && team_events[team_event_index].frame <= frame.frame_number
        {
            let event = &team_events[team_event_index];
            apply_rotation_team_event_for_derivation(
                if event.is_team_0 {
                    &mut team_zero
                } else {
                    &mut team_one
                },
                event,
            );
            team_event_index += 1;
        }

        assert_rotation_team_stats_equal(
            replay_path,
            "team_zero.rotation",
            frame.frame_number,
            &frame.team_zero.rotation,
            &team_zero,
        );
        assert_rotation_team_stats_equal(
            replay_path,
            "team_one.rotation",
            frame.frame_number,
            &frame.team_one.rotation,
            &team_one,
        );

        for player in &frame.players {
            if let Some(state) = players.get_mut(&player.player_id) {
                accumulate_rotation_player_frame_for_derivation(
                    state,
                    frame,
                    first_man_stint_end_grace_seconds,
                );
            }
            let expected = players
                .get(&player.player_id)
                .map(|state| state.stats.clone())
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
