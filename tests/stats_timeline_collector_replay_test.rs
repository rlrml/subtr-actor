mod common;

use std::collections::HashSet;

use common::parse_replay;
use subtr_actor::*;

fn frame_total_goals(frame: &ReplayStatsFrame) -> i32 {
    frame.team_zero.core.goals + frame.team_one.core.goals
}

fn player_snapshot_by_name<'a>(
    frame: &'a ReplayStatsFrame,
    player_name: &str,
) -> &'a PlayerStatsSnapshot {
    frame
        .players
        .iter()
        .find(|player| player.name == player_name)
        .unwrap_or_else(|| {
            panic!(
                "Missing player {player_name} in frame {} (t={:.3})",
                frame.frame_number, frame.time
            )
        })
}

fn player_names(frame: &ReplayStatsFrame) -> HashSet<&str> {
    frame
        .players
        .iter()
        .map(|player| player.name.as_str())
        .collect()
}

fn normalized_team_stats_for_live_play_comparison(
    snapshot: &TeamStatsSnapshot,
) -> TeamStatsSnapshot {
    let mut normalized = snapshot.clone();
    normalized.core = CoreTeamStats::default();
    normalize_boost_for_live_play_comparison(&mut normalized.boost);
    normalized.demo = DemoTeamStats::default();
    normalized
}

fn normalized_player_stats_for_live_play_comparison(
    snapshot: &PlayerStatsSnapshot,
) -> PlayerStatsSnapshot {
    let mut normalized = snapshot.clone();
    normalized.core = CorePlayerStats::default();
    normalize_boost_for_live_play_comparison(&mut normalized.boost);
    normalized.demo = DemoPlayerStats::default();
    normalized
}

fn normalize_boost_for_live_play_comparison(boost: &mut BoostStats) {
    boost.amount_used = 0.0;
    boost.amount_collected_inactive = 0.0;
    boost.big_pads_collected_inactive = 0;
    boost.small_pads_collected_inactive = 0;
    boost
        .labeled_amounts
        .entries
        .retain(|entry| !has_inactive_boost_activity_label(&entry.labels));
    boost
        .labeled_counts
        .entries
        .retain(|entry| !has_inactive_boost_activity_label(&entry.labels));
}

fn has_inactive_boost_activity_label(labels: &[StatLabel]) -> bool {
    labels
        .iter()
        .any(|label| label.key == "activity" && label.value == "inactive")
}

#[test]
fn test_stats_timeline_value_serializes_for_rlcs_replay() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let captured = StatsCollector::new()
        .capture_frames()
        .get_captured_data(&replay)
        .expect("Expected captured stats data");

    captured
        .into_legacy_stats_timeline_value()
        .expect("Expected stats timeline value");
}

#[test]
fn test_stats_timeline_excludes_post_goal_reset_frames_from_cumulative_stats() {
    let replay = parse_replay("assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay");
    let replay_data = ReplayDataCollector::new()
        .get_replay_data(&replay)
        .expect("Expected replay data");
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    let first_goal = replay_data
        .goal_events
        .first()
        .expect("Expected at least one goal event");
    let kickoff_countdown_start = replay_data
        .frame_data
        .metadata_frames
        .iter()
        .skip(first_goal.frame + 1)
        .find(|metadata| metadata.replicated_game_state_time_remaining > 0)
        .map(|metadata| metadata.time)
        .expect("Expected a kickoff countdown after the first goal");
    let post_goal_frames: Vec<_> = timeline
        .frames
        .iter()
        .filter(|frame| frame.time >= first_goal.time && frame.time < kickoff_countdown_start)
        .collect();

    assert!(
        post_goal_frames.len() > 1,
        "Expected multiple frames between the goal and the next kickoff countdown"
    );
    assert!(
        post_goal_frames.iter().all(|frame| !frame.is_live_play),
        "Expected all post-goal reset frames to be inactive"
    );

    let first_post_goal_frame = post_goal_frames
        .first()
        .expect("Expected a first post-goal frame");
    let last_post_goal_frame = post_goal_frames
        .last()
        .expect("Expected a last post-goal frame");

    assert_eq!(
        frame_total_goals(first_post_goal_frame),
        frame_total_goals(last_post_goal_frame),
        "Expected the score to stay fixed throughout the post-goal reset"
    );
    assert_eq!(
        last_post_goal_frame.team_zero.possession,
        first_post_goal_frame.team_zero.possession
    );
    assert_eq!(
        last_post_goal_frame.team_one.possession,
        first_post_goal_frame.team_one.possession
    );
    assert_eq!(
        normalized_team_stats_for_live_play_comparison(&last_post_goal_frame.team_zero),
        normalized_team_stats_for_live_play_comparison(&first_post_goal_frame.team_zero),
    );
    assert_eq!(
        normalized_team_stats_for_live_play_comparison(&last_post_goal_frame.team_one),
        normalized_team_stats_for_live_play_comparison(&first_post_goal_frame.team_one),
    );
    let normalized_last_players: Vec<_> = last_post_goal_frame
        .players
        .iter()
        .map(normalized_player_stats_for_live_play_comparison)
        .collect();
    let normalized_first_players: Vec<_> = first_post_goal_frame
        .players
        .iter()
        .map(normalized_player_stats_for_live_play_comparison)
        .collect();
    assert_eq!(normalized_last_players, normalized_first_players);
}

#[test]
fn test_stats_timeline_old_replay_with_substitutions_discovers_late_players() {
    let replay = parse_replay("assets/old-ballchasing-midfield-car.replay");
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");
    let final_frame = timeline.frames.last().expect("Expected timeline frames");
    let names = player_names(final_frame);

    for expected_name in [
        "CritRomney",
        "DatLilBabyG",
        "b_corner",
        "Raptor_Attacks_",
        "jboy42069",
        "remrocker29",
        "a093q262",
        "Q-money219",
    ] {
        assert!(
            names.contains(expected_name),
            "Expected final stats timeline frame to include late player {expected_name}, got {names:?}"
        );
    }
}

#[test]
fn test_stats_timeline_awards_touch_for_on_ball_reset_in_dodges_replay() {
    let replay =
        parse_replay("assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay");
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    let pre_reset_frame = timeline
        .frame_by_number(2114)
        .expect("Expected pre-reset frame in timeline");
    let reset_frame = timeline
        .frame_by_number(2117)
        .expect("Expected dodge-reset frame in timeline");
    let post_reset_window = (2115..=2118)
        .map(|frame_number| {
            timeline
                .frame_by_number(frame_number)
                .unwrap_or_else(|| panic!("Expected frame {frame_number} in timeline"))
        })
        .collect::<Vec<_>>();

    let pre_reset_player = player_snapshot_by_name(pre_reset_frame, "rayman ty");
    let reset_player = player_snapshot_by_name(reset_frame, "rayman ty");
    let max_touch_count_in_window = post_reset_window
        .iter()
        .map(|frame| {
            player_snapshot_by_name(frame, "rayman ty")
                .touch
                .touch_count
        })
        .max()
        .expect("Expected non-empty post-reset window");

    assert_eq!(
        reset_player.dodge_reset.on_ball_count,
        pre_reset_player.dodge_reset.on_ball_count + 1,
        "Expected rayman ty to get an on-ball reset by frame 2117"
    );
    assert!(
        max_touch_count_in_window > pre_reset_player.touch.touch_count,
        "Expected rayman ty's touch count to increase within frames 2115..=2118 after the on-ball reset, but it stayed at {}",
        pre_reset_player.touch.touch_count
    );
}

#[test]
fn test_stats_timeline_first_kickoff_credits_both_players() {
    let replay =
        parse_replay("assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay");
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .expect("Expected stats timeline data");

    let baseline_frame = timeline
        .frame_by_number(170)
        .expect("Expected baseline kickoff frame in timeline");
    let kickoff_resolution_frame = timeline
        .frame_by_number(205)
        .expect("Expected kickoff-resolution frame in timeline");

    let baseline_orange = player_snapshot_by_name(baseline_frame, "tykop");
    let baseline_blue = player_snapshot_by_name(baseline_frame, "mrtyzz.");
    let kickoff_resolution_orange = player_snapshot_by_name(kickoff_resolution_frame, "tykop");
    let kickoff_resolution_blue = player_snapshot_by_name(kickoff_resolution_frame, "mrtyzz.");

    assert!(
        kickoff_resolution_orange.touch.touch_count > baseline_orange.touch.touch_count,
        "Expected tykop to receive a credited touch during the first kickoff sequence by frame 205"
    );
    assert!(
        kickoff_resolution_blue.touch.touch_count > baseline_blue.touch.touch_count,
        "Expected mrtyzz. to receive a credited touch during the first kickoff sequence by frame 205"
    );
}
