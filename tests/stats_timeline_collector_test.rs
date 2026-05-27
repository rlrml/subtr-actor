use std::collections::HashMap;

mod common;
mod stats_timeline_collector_backboard_double_tap;
mod stats_timeline_collector_boost_ledger;
mod stats_timeline_collector_bump_demo;
mod stats_timeline_collector_mechanic_shots;
mod stats_timeline_collector_mechanics;
mod stats_timeline_collector_pass;
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
#[derive(Clone, Default)]
struct DerivedWhiffFrameStats {
    stats: WhiffStats,
}

impl DerivedWhiffFrameStats {
    fn record_event(&mut self, event: &WhiffEvent, frame: &ReplayStatsFrame) {
        match event.kind {
            WhiffEventKind::Whiff => {
                self.stats.whiff_count += 1;
                if event.aerial {
                    self.stats.aerial_whiff_count += 1;
                } else {
                    self.stats.grounded_whiff_count += 1;
                }
                if event.dodge_active {
                    self.stats.dodge_whiff_count += 1;
                }
                self.stats.last_whiff_time = Some(event.time);
                self.stats.last_whiff_frame = Some(event.frame);
                self.stats.time_since_last_whiff = Some((frame.time - event.time).max(0.0));
                self.stats.frames_since_last_whiff =
                    Some(frame.frame_number.saturating_sub(event.frame));
                self.stats.last_closest_approach_distance = Some(event.closest_approach_distance);
                self.stats.best_closest_approach_distance = Some(
                    self.stats
                        .best_closest_approach_distance
                        .map(|distance| distance.min(event.closest_approach_distance))
                        .unwrap_or(event.closest_approach_distance),
                );
                self.stats.cumulative_closest_approach_distance += event.closest_approach_distance;
                self.stats.is_last_whiff = true;
            }
            WhiffEventKind::BeatenToBall => {
                self.stats.beaten_to_ball_count += 1;
            }
        }
    }

    fn advance_live_frame(&mut self, frame: &ReplayStatsFrame, is_last_whiff_player: bool) {
        self.stats.is_last_whiff = is_last_whiff_player;
        self.stats.time_since_last_whiff = self
            .stats
            .last_whiff_time
            .map(|time| (frame.time - time).max(0.0));
        self.stats.frames_since_last_whiff = self
            .stats
            .last_whiff_frame
            .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
    }
}

fn assert_whiff_derived_stats_match(scope: &str, actual: &WhiffStats, expected: &WhiffStats) {
    assert_eq!(
        actual.whiff_count, expected.whiff_count,
        "{scope} whiff_count"
    );
    assert_eq!(
        actual.beaten_to_ball_count, expected.beaten_to_ball_count,
        "{scope} beaten_to_ball_count"
    );
    assert_eq!(
        actual.grounded_whiff_count, expected.grounded_whiff_count,
        "{scope} grounded_whiff_count"
    );
    assert_eq!(
        actual.aerial_whiff_count, expected.aerial_whiff_count,
        "{scope} aerial_whiff_count"
    );
    assert_eq!(
        actual.dodge_whiff_count, expected.dodge_whiff_count,
        "{scope} dodge_whiff_count"
    );
    assert_eq!(
        actual.is_last_whiff, expected.is_last_whiff,
        "{scope} is_last_whiff"
    );
    assert_eq!(
        actual.last_whiff_frame, expected.last_whiff_frame,
        "{scope} last_whiff_frame"
    );
    assert!(
        match (actual.last_whiff_time, expected.last_whiff_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} last_whiff_time: actual {:?} expected {:?}",
        actual.last_whiff_time,
        expected.last_whiff_time,
    );
    assert_eq!(
        actual.frames_since_last_whiff, expected.frames_since_last_whiff,
        "{scope} frames_since_last_whiff"
    );
    assert!(
        match (actual.time_since_last_whiff, expected.time_since_last_whiff) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} time_since_last_whiff: actual {:?} expected {:?}",
        actual.time_since_last_whiff,
        expected.time_since_last_whiff,
    );
    assert!(
        match (
            actual.last_closest_approach_distance,
            expected.last_closest_approach_distance,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} last_closest_approach_distance: actual {:?} expected {:?}",
        actual.last_closest_approach_distance,
        expected.last_closest_approach_distance,
    );
    assert!(
        match (
            actual.best_closest_approach_distance,
            expected.best_closest_approach_distance,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} best_closest_approach_distance: actual {:?} expected {:?}",
        actual.best_closest_approach_distance,
        expected.best_closest_approach_distance,
    );
    assert!(
        (actual.cumulative_closest_approach_distance
            - expected.cumulative_closest_approach_distance)
            .abs()
            < 0.001,
        "{scope} cumulative_closest_approach_distance: actual {:.3} expected {:.3}",
        actual.cumulative_closest_approach_distance,
        expected.cumulative_closest_approach_distance,
    );
}

fn assert_whiff_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.whiff.clone();
    events.sort_by(|left, right| {
        left.resolved_frame
            .cmp(&right.resolved_frame)
            .then_with(|| left.resolved_time.total_cmp(&right.resolved_time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, DerivedWhiffFrameStats> = HashMap::new();
    let mut frozen_players: HashMap<PlayerId, DerivedWhiffFrameStats> = HashMap::new();
    let mut last_whiff_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        if frame.is_live_play {
            for player in players.values_mut() {
                player.advance_live_frame(frame, false);
            }

            while event_index < events.len()
                && events[event_index].resolved_frame <= frame.frame_number
            {
                let event = &events[event_index];
                let player = players.entry(event.player.clone()).or_default();
                player.record_event(event, frame);
                if event.kind == WhiffEventKind::Whiff {
                    last_whiff_player = Some(event.player.clone());
                }
                event_index += 1;
            }

            if let Some(player_id) = last_whiff_player.as_ref() {
                if let Some(player) = players.get_mut(player_id) {
                    player.stats.is_last_whiff = true;
                }
            }

            for player in &frame.players {
                let expected = players.get(&player.player_id).cloned().unwrap_or_default();
                assert_whiff_derived_stats_match(
                    &format!(
                        "{replay_path} player {} frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.whiff,
                    &expected.stats,
                );
                frozen_players.insert(player.player_id.clone(), expected);
            }
        } else {
            for player in &frame.players {
                let expected = frozen_players
                    .get(&player.player_id)
                    .cloned()
                    .unwrap_or_default();
                assert_whiff_derived_stats_match(
                    &format!(
                        "{replay_path} player {} inactive frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.whiff,
                    &expected.stats,
                );
            }
            last_whiff_player = None;
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed whiff events"
    );
}

fn apply_rush_event(stats: &mut RushTeamStats, event: &RushEvent) {
    stats.count += 1;
    match (event.attackers, event.defenders) {
        (2, 1) => stats.two_v_one_count += 1,
        (2, 2) => stats.two_v_two_count += 1,
        (2, 3) => stats.two_v_three_count += 1,
        (3, 1) => stats.three_v_one_count += 1,
        (3, 2) => stats.three_v_two_count += 1,
        (3, 3) => stats.three_v_three_count += 1,
        _ => {}
    }
}

fn assert_rush_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.rush.clone();
    events.sort_by(|left, right| {
        left.start_frame
            .cmp(&right.start_frame)
            .then_with(|| left.start_time.total_cmp(&right.start_time))
            .then_with(|| left.end_frame.cmp(&right.end_frame))
    });

    let mut event_index = 0;
    let mut team_zero = RushTeamStats::default();
    let mut team_one = RushTeamStats::default();
    let min_retained_seconds = timeline.config.rush_min_possession_retained_seconds;

    for frame in &timeline.frames {
        while event_index < events.len()
            && frame.frame_number >= events[event_index].start_frame
            && frame.time - events[event_index].start_time >= min_retained_seconds
        {
            let event = &events[event_index];
            apply_rush_event(
                if event.is_team_0 {
                    &mut team_zero
                } else {
                    &mut team_one
                },
                event,
            );
            event_index += 1;
        }

        assert_eq!(
            frame.team_zero.rush, team_zero,
            "{replay_path} team_zero rush frame {}",
            frame.frame_number,
        );
        assert_eq!(
            frame.team_one.rush, team_one,
            "{replay_path} team_one rush frame {}",
            frame.frame_number,
        );
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed rush events"
    );
}

fn ball_carry_kind_label_for_derivation(kind: BallCarryKind) -> StatLabel {
    match kind {
        BallCarryKind::Carry => StatLabel::new("kind", "carry"),
        BallCarryKind::AirDribble => StatLabel::new("kind", "air_dribble"),
    }
}

fn air_dribble_origin_label_for_derivation(origin: AirDribbleOrigin) -> StatLabel {
    StatLabel::new("origin", origin.as_label_value())
}

fn apply_ball_carry_event(stats: &mut BallCarryStats, event: &BallCarryEvent) {
    stats
        .labeled_event_counts
        .increment([ball_carry_kind_label_for_derivation(event.kind)]);
    stats.carry_count = stats.labeled_event_counts.total();
    stats.total_carry_time += event.duration;
    stats.total_straight_line_distance += event.straight_line_distance;
    stats.total_path_distance += event.path_distance;
    stats.longest_carry_time = stats.longest_carry_time.max(event.duration);
    stats.furthest_carry_distance = stats
        .furthest_carry_distance
        .max(event.straight_line_distance);
    stats.fastest_carry_speed = stats.fastest_carry_speed.max(event.average_speed);
    stats.carry_speed_sum += event.average_speed;
    stats.average_horizontal_gap_sum += event.average_horizontal_gap;
    stats.average_vertical_gap_sum += event.average_vertical_gap;
}

fn apply_air_dribble_event(stats: &mut AirDribbleStats, event: &BallCarryEvent) {
    if let Some(origin) = event.air_dribble_origin {
        stats
            .labeled_event_counts
            .increment([air_dribble_origin_label_for_derivation(origin)]);
        match origin {
            AirDribbleOrigin::GroundToAir => stats.ground_to_air_count += 1,
            AirDribbleOrigin::WallToAir => stats.wall_to_air_count += 1,
        }
    }
    stats.count = stats.labeled_event_counts.total();
    stats.total_time += event.duration;
    stats.total_straight_line_distance += event.straight_line_distance;
    stats.total_path_distance += event.path_distance;
    stats.longest_time = stats.longest_time.max(event.duration);
    stats.furthest_distance = stats.furthest_distance.max(event.straight_line_distance);
    stats.fastest_speed = stats.fastest_speed.max(event.average_speed);
    stats.speed_sum += event.average_speed;
    stats.average_horizontal_gap_sum += event.average_horizontal_gap;
    stats.average_vertical_gap_sum += event.average_vertical_gap;
    stats.total_touch_count += event.touch_count;
    stats.max_touch_count = stats.max_touch_count.max(event.touch_count);
}

fn assert_ball_carry_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.ball_carry.clone();
    events.sort_by(|left, right| {
        left.end_frame
            .cmp(&right.end_frame)
            .then_with(|| left.end_time.total_cmp(&right.end_time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, BallCarryStats> = HashMap::new();
    let mut player_air_dribbles: HashMap<PlayerId, AirDribbleStats> = HashMap::new();
    let mut team_zero = BallCarryStats::default();
    let mut team_one = BallCarryStats::default();
    let mut team_zero_air_dribble = AirDribbleStats::default();
    let mut team_one_air_dribble = AirDribbleStats::default();

    for frame in &timeline.frames {
        while event_index < events.len() && events[event_index].end_frame < frame.frame_number {
            let event = &events[event_index];
            match event.kind {
                BallCarryKind::Carry => {
                    apply_ball_carry_event(
                        players.entry(event.player_id.clone()).or_default(),
                        event,
                    );
                    apply_ball_carry_event(
                        if event.is_team_0 {
                            &mut team_zero
                        } else {
                            &mut team_one
                        },
                        event,
                    );
                }
                BallCarryKind::AirDribble => {
                    apply_air_dribble_event(
                        player_air_dribbles
                            .entry(event.player_id.clone())
                            .or_default(),
                        event,
                    );
                    apply_air_dribble_event(
                        if event.is_team_0 {
                            &mut team_zero_air_dribble
                        } else {
                            &mut team_one_air_dribble
                        },
                        event,
                    );
                }
            }
            event_index += 1;
        }

        assert_eq!(
            frame.team_zero.ball_carry, team_zero,
            "{replay_path} team_zero ball_carry frame {}",
            frame.frame_number,
        );
        assert_eq!(
            frame.team_one.ball_carry, team_one,
            "{replay_path} team_one ball_carry frame {}",
            frame.frame_number,
        );
        assert_eq!(
            frame.team_zero.air_dribble, team_zero_air_dribble,
            "{replay_path} team_zero air_dribble frame {}",
            frame.frame_number,
        );
        assert_eq!(
            frame.team_one.air_dribble, team_one_air_dribble,
            "{replay_path} team_one air_dribble frame {}",
            frame.frame_number,
        );
        for player in &frame.players {
            assert_eq!(
                player.ball_carry,
                players.get(&player.player_id).cloned().unwrap_or_default(),
                "{replay_path} player {} ball_carry frame {}",
                player.name,
                frame.frame_number,
            );
            assert_eq!(
                player.air_dribble,
                player_air_dribbles
                    .get(&player.player_id)
                    .cloned()
                    .unwrap_or_default(),
                "{replay_path} player {} air_dribble frame {}",
                player.name,
                frame.frame_number,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed ball-carry events"
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DerivedPowerslideState {
    active: bool,
    is_team_0: bool,
}

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

fn assert_touch_stats_close(
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
    assert_eq!(
        actual.is_last_touch, expected.is_last_touch,
        "{replay_path} player {player_name} touch.is_last_touch frame {frame_number}"
    );
    assert_eq!(
        actual.last_touch_time, expected.last_touch_time,
        "{replay_path} player {player_name} touch.last_touch_time frame {frame_number}"
    );
    assert_eq!(
        actual.last_touch_frame, expected.last_touch_frame,
        "{replay_path} player {player_name} touch.last_touch_frame frame {frame_number}"
    );
    assert_eq!(
        actual.time_since_last_touch, expected.time_since_last_touch,
        "{replay_path} player {player_name} touch.time_since_last_touch frame {frame_number}"
    );
    assert_eq!(
        actual.frames_since_last_touch, expected.frames_since_last_touch,
        "{replay_path} player {player_name} touch.frames_since_last_touch frame {frame_number}"
    );
    assert_eq!(
        actual.last_ball_speed_change, expected.last_ball_speed_change,
        "{replay_path} player {player_name} touch.last_ball_speed_change frame {frame_number}"
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

fn assert_touch_events_reconstruct_serialized_partial_sums(
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
    let mut last_touch_events = timeline.events.touch_last_touch.clone();
    last_touch_events.sort_by(|left, right| {
        left.sample_frame
            .cmp(&right.sample_frame)
            .then_with(|| left.sample_time.total_cmp(&right.sample_time))
            .then_with(|| left.frame.cmp(&right.frame))
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut touch_event_index = 0;
    let mut movement_event_index = 0;
    let mut last_touch_event_index = 0;
    let mut current_last_touch_player: Option<PlayerId> = None;
    let mut players: HashMap<PlayerId, TouchStats> = HashMap::new();

    for frame in &timeline.frames {
        if !frame.is_live_play {
            current_last_touch_player = None;
        } else {
            for stats in players.values_mut() {
                stats.is_last_touch = false;
                if let Some(last_touch_time) = stats.last_touch_time {
                    stats.time_since_last_touch = Some((frame.time - last_touch_time).max(0.0));
                }
                if let Some(last_touch_frame) = stats.last_touch_frame {
                    stats.frames_since_last_touch =
                        Some(frame.frame_number.saturating_sub(last_touch_frame));
                }
            }

            while touch_event_index < touch_events.len()
                && touch_events[touch_event_index].sample_frame <= frame.frame_number
            {
                let event = &touch_events[touch_event_index];
                apply_touch_stats_event_for_derivation(
                    players.entry(event.player.clone()).or_default(),
                    event,
                    frame,
                );
                touch_event_index += 1;
            }

            while last_touch_event_index < last_touch_events.len()
                && last_touch_events[last_touch_event_index].sample_frame <= frame.frame_number
            {
                current_last_touch_player =
                    last_touch_events[last_touch_event_index].player.clone();
                last_touch_event_index += 1;
            }

            if let Some(player_id) = current_last_touch_player.as_ref() {
                if let Some(stats) = players.get_mut(player_id) {
                    stats.is_last_touch = true;
                }
            }
        }

        while movement_event_index < movement_events.len()
            && movement_events[movement_event_index].frame <= frame.frame_number
        {
            let event = &movement_events[movement_event_index];
            let stats = players.entry(event.player.clone()).or_default();
            stats.total_ball_travel_distance += event.travel_distance;
            stats.total_ball_advance_distance += event.advance_distance;
            stats.total_ball_retreat_distance += event.retreat_distance;
            movement_event_index += 1;
        }

        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_touch_stats_close(
                replay_path,
                &player.name,
                frame.frame_number,
                &player.touch,
                &expected,
            );
        }
    }

    assert_eq!(
        touch_event_index,
        touch_events.len(),
        "{replay_path} unprocessed touch events"
    );
    assert_eq!(
        movement_event_index,
        movement_events.len(),
        "{replay_path} unprocessed touch ball movement events"
    );
    assert_eq!(
        last_touch_event_index,
        last_touch_events.len(),
        "{replay_path} unprocessed touch last-touch events"
    );
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

fn assert_positioning_events_reconstruct_serialized_partial_sums(
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
fn test_whiff_events_reconstruct_serialized_partial_sums() {
    let replay_paths = [
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
        "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g5.replay",
        "assets/recent-ranked-standard-2026-03-10-a.replay",
    ];
    let mut found_timeline = None;

    for replay_path in replay_paths {
        let replay = parse_replay(replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));
        if !timeline.events.whiff.is_empty() {
            found_timeline = Some((replay_path, timeline));
            break;
        }
    }

    let (replay_path, timeline) =
        found_timeline.expect("expected at least one fixture to contain whiff events");
    assert_whiff_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_rush_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.rush.is_empty(),
        "expected rush fixture to contain rush events"
    );
    assert_rush_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
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
fn test_ball_carry_events_reconstruct_serialized_partial_sums() {
    let replay_paths = [
        "assets/air-dribble-goal-mouth-2026-05-24.replay",
        "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
    ];
    let mut found_timeline = None;

    for replay_path in replay_paths {
        let replay = parse_replay(replay_path);
        let timeline = StatsTimelineCollector::new()
            .get_legacy_replay_stats_timeline(&replay)
            .unwrap_or_else(|_| panic!("Expected stats timeline data for {replay_path}"));
        if !timeline.events.ball_carry.is_empty() {
            found_timeline = Some((replay_path, timeline));
            break;
        }
    }

    let (replay_path, timeline) =
        found_timeline.expect("expected at least one fixture to contain ball-carry events");
    assert_ball_carry_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_powerslide_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.powerslide.is_empty(),
        "expected powerslide fixture to contain powerslide events"
    );
    assert_powerslide_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_touch_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.touch.is_empty(),
        "expected touch fixture to contain touch events"
    );
    assert!(
        !timeline.events.touch_ball_movement.is_empty(),
        "expected touch fixture to contain ball movement credit events"
    );
    assert_touch_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
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

#[test]
fn test_possession_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.possession.is_empty(),
        "expected possession fixture to contain possession events"
    );
    assert_possession_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_pressure_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.pressure.is_empty(),
        "expected pressure fixture to contain pressure events"
    );
    assert_pressure_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_movement_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.movement.is_empty(),
        "expected movement fixture to contain movement events"
    );
    assert_movement_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_positioning_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.positioning.is_empty(),
        "expected positioning fixture to contain positioning events"
    );
    assert_positioning_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
}

#[test]
fn test_rotation_events_reconstruct_serialized_partial_sums() {
    let replay_path = "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
    let replay = parse_replay(replay_path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    assert!(
        !timeline.events.rotation_player.is_empty(),
        "expected rotation fixture to contain rotation player events"
    );
    assert_rotation_events_reconstruct_serialized_partial_sums(replay_path, &timeline);
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
