use std::collections::HashMap;

use subtr_actor::*;

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

pub fn assert_whiff_events_reconstruct_serialized_partial_sums(
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

pub fn assert_rush_events_reconstruct_serialized_partial_sums(
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

pub fn assert_ball_carry_events_reconstruct_serialized_partial_sums(
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

pub fn assert_powerslide_events_reconstruct_serialized_partial_sums(
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

pub fn assert_touch_events_reconstruct_serialized_partial_sums(
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
