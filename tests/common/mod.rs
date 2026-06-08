use std::path::Path;

use serde::Serialize;
use serde_json::Value;
use subtr_actor::{
    CoreTeamStats, ReplayMeta, ReplayStatsFrame, ReplayStatsTimeline, ReplayStatsTimelineEvents,
    StatsEventPropertyValue, StatsEventTiming, StatsTimelineTagEvent, TeamStatsSnapshot,
};

pub fn parse_replay(path: &str) -> boxcars::Replay {
    let replay_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let data = std::fs::read(&replay_path)
        .unwrap_or_else(|_| panic!("Failed to read replay file: {}", replay_path.display()));
    boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse replay: {}", replay_path.display()))
}

#[allow(dead_code)]
pub fn mechanic_event_time_span(event: &StatsTimelineTagEvent) -> (f32, f32) {
    match event.timing {
        StatsEventTiming::Moment { time, .. } => (time, time),
        StatsEventTiming::Span {
            start_time,
            end_time,
            ..
        } => (start_time, end_time),
    }
}

#[allow(dead_code)]
pub fn mechanic_event_player_name<'a>(
    timeline: &'a ReplayStatsTimeline,
    event: &StatsTimelineTagEvent,
) -> Option<&'a str> {
    mechanic_event_player_name_in_meta(&timeline.replay_meta, event)
}

#[allow(dead_code)]
pub fn mechanic_event_player_name_in_meta<'a>(
    replay_meta: &'a ReplayMeta,
    event: &StatsTimelineTagEvent,
) -> Option<&'a str> {
    replay_meta
        .player_order()
        .find(|player| player.remote_id == event.player_id)
        .map(|player| player.name.as_str())
}

#[allow(dead_code)]
pub fn assert_mechanic_event_roughly_at_in_meta<'a>(
    replay_meta: &'a ReplayMeta,
    events: &'a [StatsTimelineTagEvent],
    kind: &str,
    player_name: &str,
    expected_start_time: f32,
    expected_end_time: f32,
    tolerance_seconds: f32,
) -> &'a StatsTimelineTagEvent {
    let event = events.iter().find(|event| {
        if event.kind != kind {
            return false;
        }
        if mechanic_event_player_name_in_meta(replay_meta, event) != Some(player_name) {
            return false;
        }
        let (start_time, end_time) = mechanic_event_time_span(event);
        (start_time - expected_start_time).abs() <= tolerance_seconds
            && (end_time - expected_end_time).abs() <= tolerance_seconds
    });

    event.unwrap_or_else(|| {
        let candidates = events
            .iter()
            .filter(|event| event.kind == kind)
            .map(|event| {
                let (start_time, end_time) = mechanic_event_time_span(event);
                let player =
                    mechanic_event_player_name_in_meta(replay_meta, event).unwrap_or("<unknown>");
                format!("{kind} by {player} at {start_time:.3}-{end_time:.3}s")
            })
            .collect::<Vec<_>>()
            .join(", ");
        panic!(
            "expected {kind} by {player_name} around \
             {expected_start_time:.3}-{expected_end_time:.3}s (+/- {tolerance_seconds:.3}s); \
             candidates: [{candidates}]"
        );
    })
}

#[allow(dead_code)]
pub fn assert_mechanic_event_roughly_at<'a>(
    timeline: &'a ReplayStatsTimeline,
    kind: &str,
    player_name: &str,
    expected_start_time: f32,
    expected_end_time: f32,
    tolerance_seconds: f32,
) -> &'a StatsTimelineTagEvent {
    assert_mechanic_event_roughly_at_in_meta(
        &timeline.replay_meta,
        &timeline.events.mechanics,
        kind,
        player_name,
        expected_start_time,
        expected_end_time,
        tolerance_seconds,
    )
}

#[allow(dead_code)]
pub fn mechanic_event_text_property<'a>(
    event: &'a StatsTimelineTagEvent,
    key: &str,
) -> Option<&'a str> {
    event.properties.iter().find_map(|property| {
        (property.key == key)
            .then_some(&property.value)
            .and_then(|value| match value {
                StatsEventPropertyValue::Text(value) => Some(value.as_str()),
                _ => None,
            })
    })
}

#[allow(dead_code)]
pub fn mechanic_event_unsigned_property(event: &StatsTimelineTagEvent, key: &str) -> Option<u32> {
    event.properties.iter().find_map(|property| {
        (property.key == key)
            .then_some(&property.value)
            .and_then(|value| match value {
                StatsEventPropertyValue::Unsigned(value) => Some(*value),
                _ => None,
            })
    })
}

#[allow(dead_code)]
pub fn assert_replay_stats_timeline_eq(left: &ReplayStatsTimeline, right: &ReplayStatsTimeline) {
    if let Some(diff) = compare_field("config", &left.config, &right.config)
        .or_else(|| compare_field("replay_meta", &left.replay_meta, &right.replay_meta))
        .or_else(|| compare_timeline_events("events", &left.events, &right.events))
        .or_else(|| compare_replay_frame_slice("frames", &left.frames, &right.frames))
    {
        panic!("replay stats timelines differ at {diff}");
    }
}

fn compare_timeline_events(
    label: &str,
    left: &ReplayStatsTimelineEvents,
    right: &ReplayStatsTimelineEvents,
) -> Option<String> {
    compare_serialized_slice(
        &format!("{label}.timeline"),
        &left.timeline,
        &right.timeline,
    )
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.core_player"),
            &left.core_player,
            &right.core_player,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.core_player_goal_context"),
            &left.core_player_goal_context,
            &right.core_player_goal_context,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.possession"),
            &left.possession,
            &right.possession,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.pressure"),
            &left.pressure,
            &right.pressure,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.movement"),
            &left.movement,
            &right.movement,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.positioning_activity"),
            &left.positioning_activity,
            &right.positioning_activity,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.positioning_possession"),
            &left.positioning_possession,
            &right.positioning_possession,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.positioning_field_zone"),
            &left.positioning_field_zone,
            &right.positioning_field_zone,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.positioning_ball_depth"),
            &left.positioning_ball_depth,
            &right.positioning_ball_depth,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.positioning_teammate_role"),
            &left.positioning_teammate_role,
            &right.positioning_teammate_role,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.positioning_ball_proximity"),
            &left.positioning_ball_proximity,
            &right.positioning_ball_proximity,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.positioning_goal_context"),
            &left.positioning_goal_context,
            &right.positioning_goal_context,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.rotation_player"),
            &left.rotation_player,
            &right.rotation_player,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.rotation_role_span"),
            &left.rotation_role_span,
            &right.rotation_role_span,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.rotation_depth_span"),
            &left.rotation_depth_span,
            &right.rotation_depth_span,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.rotation_first_man_stint"),
            &left.rotation_first_man_stint,
            &right.rotation_first_man_stint,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.rotation_team"),
            &left.rotation_team,
            &right.rotation_team,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.mechanics"),
            &left.mechanics,
            &right.mechanics,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.goal_context"),
            &left.goal_context,
            &right.goal_context,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.backboard"),
            &left.backboard,
            &right.backboard,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.ceiling_shot"),
            &left.ceiling_shot,
            &right.ceiling_shot,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.wall_aerial"),
            &left.wall_aerial,
            &right.wall_aerial,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.wall_aerial_shot"),
            &left.wall_aerial_shot,
            &right.wall_aerial_shot,
        )
    })
    .or_else(|| compare_serialized_slice(&format!("{label}.center"), &left.center, &right.center))
    .or_else(|| compare_serialized_slice(&format!("{label}.flick"), &left.flick, &right.flick))
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.musty_flick"),
            &left.musty_flick,
            &right.musty_flick,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.dodge_reset"),
            &left.dodge_reset,
            &right.dodge_reset,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.double_tap"),
            &left.double_tap,
            &right.double_tap,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.fifty_fifty"),
            &left.fifty_fifty,
            &right.fifty_fifty,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.one_timer"),
            &left.one_timer,
            &right.one_timer,
        )
    })
    .or_else(|| compare_serialized_slice(&format!("{label}.pass"), &left.pass, &right.pass))
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.ball_carry"),
            &left.ball_carry,
            &right.ball_carry,
        )
    })
    .or_else(|| compare_serialized_slice(&format!("{label}.rush"), &left.rush, &right.rush))
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.speed_flip"),
            &left.speed_flip,
            &right.speed_flip,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.half_flip"),
            &left.half_flip,
            &right.half_flip,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.half_volley"),
            &left.half_volley,
            &right.half_volley,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.wavedash"),
            &left.wavedash,
            &right.wavedash,
        )
    })
    .or_else(|| compare_serialized_slice(&format!("{label}.whiff"), &left.whiff, &right.whiff))
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.powerslide"),
            &left.powerslide,
            &right.powerslide,
        )
    })
    .or_else(|| compare_serialized_slice(&format!("{label}.touch"), &left.touch, &right.touch))
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.boost_pickups"),
            &left.boost_pickups,
            &right.boost_pickups,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.boost_ledger"),
            &left.boost_ledger,
            &right.boost_ledger,
        )
    })
    .or_else(|| {
        compare_serialized_slice(
            &format!("{label}.boost_state"),
            &left.boost_state,
            &right.boost_state,
        )
    })
    .or_else(|| compare_serialized_slice(&format!("{label}.bump"), &left.bump, &right.bump))
}

fn compare_field<T: PartialEq + Serialize>(label: &str, left: &T, right: &T) -> Option<String> {
    (left != right).then(|| serialized_diff_path(label, left, right))
}

fn compare_slice<T: PartialEq + Serialize>(label: &str, left: &[T], right: &[T]) -> Option<String> {
    if left.len() != right.len() {
        return Some(format!(
            "{label}.len: left={}, right={}",
            left.len(),
            right.len()
        ));
    }

    left.iter()
        .zip(right.iter())
        .enumerate()
        .find(|(_, (left_item, right_item))| left_item != right_item)
        .map(|(index, (left_item, right_item))| {
            serialized_diff_path(&format!("{label}[{index}]"), left_item, right_item)
        })
}

fn compare_serialized_slice<T: Serialize>(label: &str, left: &[T], right: &[T]) -> Option<String> {
    let left_value = serde_json::to_value(left).expect("left side should serialize for debugging");
    let right_value =
        serde_json::to_value(right).expect("right side should serialize for debugging");
    (left_value != right_value)
        .then(|| first_json_diff(label.to_owned(), &left_value, &right_value))
}

fn compare_replay_frame_slice(
    label: &str,
    left: &[ReplayStatsFrame],
    right: &[ReplayStatsFrame],
) -> Option<String> {
    if left.len() != right.len() {
        return Some(format!(
            "{label}.len: left={}, right={}",
            left.len(),
            right.len()
        ));
    }

    left.iter()
        .zip(right.iter())
        .enumerate()
        .find_map(|(index, (left_frame, right_frame))| {
            compare_replay_frame(&format!("{label}[{index}]"), left_frame, right_frame)
        })
}

fn compare_replay_frame(
    label: &str,
    left: &ReplayStatsFrame,
    right: &ReplayStatsFrame,
) -> Option<String> {
    compare_field(
        &format!("{label}.frame_number"),
        &left.frame_number,
        &right.frame_number,
    )
    .or_else(|| compare_field(&format!("{label}.time"), &left.time, &right.time))
    .or_else(|| compare_field(&format!("{label}.dt"), &left.dt, &right.dt))
    .or_else(|| {
        compare_field(
            &format!("{label}.seconds_remaining"),
            &left.seconds_remaining,
            &right.seconds_remaining,
        )
    })
    .or_else(|| {
        compare_field(
            &format!("{label}.game_state"),
            &left.game_state,
            &right.game_state,
        )
    })
    .or_else(|| {
        compare_field(
            &format!("{label}.ball_has_been_hit"),
            &left.ball_has_been_hit,
            &right.ball_has_been_hit,
        )
    })
    .or_else(|| {
        compare_field(
            &format!("{label}.kickoff_countdown_time"),
            &left.kickoff_countdown_time,
            &right.kickoff_countdown_time,
        )
    })
    .or_else(|| {
        compare_field(
            &format!("{label}.gameplay_phase"),
            &left.gameplay_phase,
            &right.gameplay_phase,
        )
    })
    .or_else(|| {
        compare_field(
            &format!("{label}.is_live_play"),
            &left.is_live_play,
            &right.is_live_play,
        )
    })
    .or_else(|| {
        compare_team_snapshot(
            &format!("{label}.team_zero"),
            &left.team_zero,
            &right.team_zero,
        )
    })
    .or_else(|| {
        compare_team_snapshot(
            &format!("{label}.team_one"),
            &left.team_one,
            &right.team_one,
        )
    })
    .or_else(|| compare_slice(&format!("{label}.players"), &left.players, &right.players))
}

fn compare_team_snapshot(
    label: &str,
    left: &TeamStatsSnapshot,
    right: &TeamStatsSnapshot,
) -> Option<String> {
    compare_field(
        &format!("{label}.fifty_fifty"),
        &left.fifty_fifty,
        &right.fifty_fifty,
    )
    .or_else(|| {
        compare_field(
            &format!("{label}.possession"),
            &left.possession,
            &right.possession,
        )
    })
    .or_else(|| {
        compare_field(
            &format!("{label}.pressure"),
            &left.pressure,
            &right.pressure,
        )
    })
    .or_else(|| compare_field(&format!("{label}.rush"), &left.rush, &right.rush))
    .or_else(|| compare_core_team_stats(&format!("{label}.core"), &left.core, &right.core))
    .or_else(|| {
        compare_field(
            &format!("{label}.backboard"),
            &left.backboard,
            &right.backboard,
        )
    })
    .or_else(|| {
        compare_field(
            &format!("{label}.double_tap"),
            &left.double_tap,
            &right.double_tap,
        )
    })
    .or_else(|| {
        compare_field(
            &format!("{label}.ball_carry"),
            &left.ball_carry,
            &right.ball_carry,
        )
    })
    .or_else(|| compare_field(&format!("{label}.boost"), &left.boost, &right.boost))
    .or_else(|| {
        compare_field(
            &format!("{label}.movement"),
            &left.movement,
            &right.movement,
        )
    })
    .or_else(|| {
        compare_field(
            &format!("{label}.powerslide"),
            &left.powerslide,
            &right.powerslide,
        )
    })
    .or_else(|| compare_field(&format!("{label}.demo"), &left.demo, &right.demo))
}

fn compare_core_team_stats(
    label: &str,
    left: &CoreTeamStats,
    right: &CoreTeamStats,
) -> Option<String> {
    compare_field(&format!("{label}.score"), &left.score, &right.score)
        .or_else(|| compare_field(&format!("{label}.goals"), &left.goals, &right.goals))
        .or_else(|| compare_field(&format!("{label}.assists"), &left.assists, &right.assists))
        .or_else(|| compare_field(&format!("{label}.saves"), &left.saves, &right.saves))
        .or_else(|| compare_field(&format!("{label}.shots"), &left.shots, &right.shots))
        .or_else(|| {
            compare_field(
                &format!("{label}.scoring_context.goal_after_kickoff"),
                &left.scoring_context.goal_after_kickoff,
                &right.scoring_context.goal_after_kickoff,
            )
        })
        .or_else(|| {
            compare_field(
                &format!("{label}.scoring_context.goal_buildup"),
                &left.scoring_context.goal_buildup,
                &right.scoring_context.goal_buildup,
            )
        })
}

fn serialized_diff_path<T: Serialize>(label: &str, left: &T, right: &T) -> String {
    let left = serde_json::to_value(left).expect("left side should serialize for debugging");
    let right = serde_json::to_value(right).expect("right side should serialize for debugging");
    first_json_diff(label.to_owned(), &left, &right)
}

fn first_json_diff(path: String, left: &Value, right: &Value) -> String {
    match (left, right) {
        (Value::Object(left_map), Value::Object(right_map)) => {
            let mut keys = left_map.keys().chain(right_map.keys()).collect::<Vec<_>>();
            keys.sort_unstable();
            keys.dedup();
            for key in keys {
                let next_path = format!("{path}.{key}");
                match (left_map.get(key), right_map.get(key)) {
                    (Some(left_value), Some(right_value)) if left_value != right_value => {
                        return first_json_diff(next_path, left_value, right_value);
                    }
                    (Some(_), None) => {
                        return format!("{next_path}: missing on right");
                    }
                    (None, Some(_)) => {
                        return format!("{next_path}: missing on left");
                    }
                    _ => {}
                }
            }
            path
        }
        (Value::Array(left_items), Value::Array(right_items)) => {
            if left_items.len() != right_items.len() {
                return format!(
                    "{path}.len: left={}, right={}",
                    left_items.len(),
                    right_items.len()
                );
            }

            for (index, (left_item, right_item)) in
                left_items.iter().zip(right_items.iter()).enumerate()
            {
                if left_item != right_item {
                    return first_json_diff(format!("{path}[{index}]"), left_item, right_item);
                }
            }

            path
        }
        _ => format!("{path}: left={left}, right={right}"),
    }
}
