use std::path::Path;

use serde::Serialize;
use serde_json::Value;
use subtr_actor::{CoreTeamStats, ReplayStatsFrame, ReplayStatsTimeline, TeamStatsSnapshot};

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
pub fn assert_replay_stats_timeline_eq(left: &ReplayStatsTimeline, right: &ReplayStatsTimeline) {
    if let Some(diff) = compare_field("config", &left.config, &right.config)
        .or_else(|| compare_field("replay_meta", &left.replay_meta, &right.replay_meta))
        .or_else(|| compare_field("events", &left.events, &right.events))
        .or_else(|| compare_replay_frame_slice("frames", &left.frames, &right.frames))
    {
        panic!("replay stats timelines differ at {diff}");
    }
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
