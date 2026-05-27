use std::collections::HashMap;

mod common;
mod stats_timeline_collector_backboard_double_tap;
mod stats_timeline_collector_boost_ledger;
mod stats_timeline_collector_bump_demo;
mod stats_timeline_collector_field_state;
mod stats_timeline_collector_mechanic_shots;
mod stats_timeline_collector_mechanics;
mod stats_timeline_collector_pass;
mod stats_timeline_collector_replay_actions;
mod stats_timeline_collector_shots;

use common::default_team_stats_snapshot;
use stats_timeline_collector_backboard_double_tap::{
    assert_backboard_events_reconstruct_serialized_partial_sums,
    assert_double_tap_events_reconstruct_serialized_partial_sums,
};
use stats_timeline_collector_boost_ledger::assert_boost_ledger_reconstructs_serialized_boost_partial_sums;
use stats_timeline_collector_bump_demo::{
    assert_bump_events_reconstruct_serialized_partial_sums,
    assert_demo_events_reconstruct_serialized_partial_sums,
};
use stats_timeline_collector_field_state::{
    assert_movement_events_reconstruct_serialized_partial_sums,
    assert_positioning_events_reconstruct_serialized_partial_sums,
    assert_possession_events_reconstruct_serialized_partial_sums,
    assert_pressure_events_reconstruct_serialized_partial_sums,
    assert_rotation_events_reconstruct_serialized_partial_sums,
};
use stats_timeline_collector_mechanic_shots::{
    assert_ceiling_shot_events_reconstruct_serialized_partial_sums,
    assert_dodge_reset_events_reconstruct_serialized_partial_sums,
    assert_flick_events_reconstruct_serialized_partial_sums,
    assert_musty_flick_events_reconstruct_serialized_partial_sums,
    assert_wall_aerial_events_reconstruct_serialized_partial_sums,
    assert_wall_aerial_shot_events_reconstruct_serialized_partial_sums,
};
use stats_timeline_collector_mechanics::{
    assert_quality_mechanic_events_reconstruct_serialized_partial_sums,
    assert_speed_flip_events_reconstruct_serialized_partial_sums,
};
use stats_timeline_collector_pass::assert_pass_events_reconstruct_serialized_partial_sums;
use stats_timeline_collector_replay_actions::{
    assert_ball_carry_events_reconstruct_serialized_partial_sums,
    assert_powerslide_events_reconstruct_serialized_partial_sums,
    assert_rush_events_reconstruct_serialized_partial_sums,
    assert_touch_events_reconstruct_serialized_partial_sums,
    assert_whiff_events_reconstruct_serialized_partial_sums,
};
use stats_timeline_collector_shots::{
    assert_half_volley_events_reconstruct_serialized_partial_sums,
    assert_one_timer_events_reconstruct_serialized_partial_sums,
};
use subtr_actor::*;

const REPLAY_FORMAT_EVOLUTION_DOC: &str = include_str!("../docs/replay-format-evolution.md");

fn parse_replay(path: &str) -> boxcars::Replay {
    let data = std::fs::read(path).unwrap_or_else(|_| panic!("Failed to read replay file: {path}"));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay: {path}"))
}

fn replay_format_fixture_paths() -> Vec<String> {
    let fixture_filter = std::env::var("SUBTR_ACTOR_REPLAY_FORMAT_FIXTURE").ok();
    REPLAY_FORMAT_EVOLUTION_DOC
        .lines()
        .filter_map(|line| {
            let start = line.find("| `")? + 3;
            let rest = &line[start..];
            let end = rest.find("` |")?;
            let fixture = &rest[..end];
            fixture
                .ends_with(".replay")
                .then(|| format!("assets/{fixture}"))
        })
        .filter(|path| {
            fixture_filter
                .as_ref()
                .map(|filter| path.contains(filter))
                .unwrap_or(true)
        })
        .collect()
}

fn asset_replay_fixture_paths() -> Vec<String> {
    let fixture_filter = std::env::var("SUBTR_ACTOR_REPLAY_FIXTURE").ok();
    let mut replay_paths = std::fs::read_dir("assets")
        .expect("expected checked-in replay asset directory")
        .filter_map(|entry| {
            let entry = entry.expect("expected replay asset directory entry");
            let path = entry.path();
            (path
                .extension()
                .is_some_and(|extension| extension == "replay"))
            .then(|| {
                path.to_str()
                    .expect("expected replay fixture path to be valid UTF-8")
                    .to_owned()
            })
        })
        .filter(|path| {
            fixture_filter
                .as_ref()
                .map(|filter| path.contains(filter))
                .unwrap_or(true)
        })
        .collect::<Vec<_>>();
    replay_paths.sort();
    replay_paths
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

fn assert_fifty_fifty_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.fifty_fifty.clone();
    events.sort_by(|left, right| {
        left.resolve_frame
            .cmp(&right.resolve_frame)
            .then_with(|| left.resolve_time.total_cmp(&right.resolve_time))
    });

    let mut event_index = 0;
    let mut team_zero = FiftyFiftyTeamStats::default();
    let mut team_one = FiftyFiftyTeamStats::default();
    let mut players: HashMap<PlayerId, FiftyFiftyPlayerStats> = HashMap::new();

    for frame in &timeline.frames {
        while event_index < events.len() && events[event_index].resolve_frame <= frame.frame_number
        {
            let event = &events[event_index];
            apply_fifty_fifty_team_event(&mut team_zero, true, event);
            apply_fifty_fifty_team_event(&mut team_one, false, event);
            if let Some(player_id) = event.team_zero_player.as_ref() {
                apply_fifty_fifty_player_event(
                    players.entry(player_id.clone()).or_default(),
                    true,
                    event,
                );
            }
            if let Some(player_id) = event.team_one_player.as_ref() {
                apply_fifty_fifty_player_event(
                    players.entry(player_id.clone()).or_default(),
                    false,
                    event,
                );
            }
            event_index += 1;
        }

        assert_eq!(
            frame.team_zero.fifty_fifty, team_zero,
            "{replay_path} team_zero fifty_fifty frame {}",
            frame.frame_number,
        );
        assert_eq!(
            frame.team_one.fifty_fifty, team_one,
            "{replay_path} team_one fifty_fifty frame {}",
            frame.frame_number,
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_eq!(
                player.fifty_fifty, expected,
                "{replay_path} player {} fifty_fifty frame {}",
                player.name, frame.frame_number,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed fifty-fifty events"
    );
}

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
fn test_fifty_fifty_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.fifty_fifty.is_empty(),
        "expected fifty-fifty fixture to contain fifty-fifty events"
    );
    assert_fifty_fifty_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_core_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.core_player.is_empty(),
        "expected core fixture to contain player stat events"
    );
    assert!(
        !timeline.events.core_team.is_empty(),
        "expected core fixture to contain team stat events"
    );
    assert_core_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

fn assert_converted_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    assert_boost_ledger_reconstructs_serialized_boost_partial_sums(replay_path, timeline);
    assert_core_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_possession_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_pressure_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_movement_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_positioning_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_rotation_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_quality_mechanic_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_speed_flip_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_whiff_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_backboard_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_double_tap_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_demo_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_fifty_fifty_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_bump_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_rush_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_pass_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_one_timer_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_ball_carry_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_wall_aerial_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_wall_aerial_shot_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_flick_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_ceiling_shot_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_musty_flick_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_dodge_reset_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_powerslide_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_touch_events_reconstruct_serialized_partial_sums(replay_path, timeline);
    assert_half_volley_events_reconstruct_serialized_partial_sums(replay_path, timeline);
}

fn assert_replay_paths_reconstruct_serialized_partial_sums(replay_paths: Vec<String>) {
    for replay_path in replay_paths {
        eprintln!("checking {replay_path}");
        let replay = parse_replay(&replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));
        assert_converted_events_reconstruct_serialized_partial_sums(&replay_path, &timeline);
    }
}

#[test]
#[ignore = "wide replay-format parity is slow; run explicitly when changing compact timeline derivation"]
fn replay_format_fixture_events_reconstruct_serialized_partial_sums() {
    let replay_paths = replay_format_fixture_paths();
    assert!(
        !replay_paths.is_empty(),
        "expected replay-format docs to list checked-in fixtures"
    );
    assert!(
        std::env::var("SUBTR_ACTOR_REPLAY_FORMAT_FIXTURE").is_ok() || replay_paths.len() >= 10,
        "expected replay-format docs to list checked-in fixtures"
    );

    assert_replay_paths_reconstruct_serialized_partial_sums(replay_paths);
}

#[test]
#[ignore = "all replay asset event parity is slow; run explicitly before removing transferred partial sums"]
fn all_asset_fixture_events_reconstruct_serialized_partial_sums() {
    let replay_paths = asset_replay_fixture_paths();
    assert!(
        !replay_paths.is_empty(),
        "expected checked-in replay asset fixtures"
    );
    assert!(
        std::env::var("SUBTR_ACTOR_REPLAY_FIXTURE").is_ok() || replay_paths.len() >= 20,
        "expected broad replay fixture coverage"
    );

    assert_replay_paths_reconstruct_serialized_partial_sums(replay_paths);
}
