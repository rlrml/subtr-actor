use std::collections::HashMap;

use subtr_actor::*;

#[derive(Clone, Default)]
struct DerivedPassPlayerStats {
    stats: PassPlayerStats,
}

impl DerivedPassPlayerStats {
    fn advance_frame(&mut self, frame: &ReplayStatsFrame, is_last_completed_pass_player: bool) {
        self.stats.is_last_completed_pass = is_last_completed_pass_player;
        self.stats.time_since_last_completed_pass = self
            .stats
            .last_completed_pass_time
            .map(|time| (frame.time - time).max(0.0));
        self.stats.frames_since_last_completed_pass = self
            .stats
            .last_completed_pass_frame
            .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
    }

    fn record_completed_pass(&mut self, event: &PassEvent, frame: &ReplayStatsFrame) {
        self.stats.completed_pass_count += 1;
        self.stats.total_pass_distance += event.ball_travel_distance;
        self.stats.total_pass_advance += event.ball_advance_distance;
        self.stats.longest_pass_distance = self
            .stats
            .longest_pass_distance
            .max(event.ball_travel_distance);
        self.stats.last_completed_pass_time = Some(event.time);
        self.stats.last_completed_pass_frame = Some(event.frame);
        self.stats.time_since_last_completed_pass = Some((frame.time - event.time).max(0.0));
        self.stats.frames_since_last_completed_pass =
            Some(frame.frame_number.saturating_sub(event.frame));
    }

    fn record_received_pass(&mut self) {
        self.stats.received_pass_count += 1;
    }
}

fn assert_pass_derived_player_stats_match(
    scope: &str,
    actual: &PassPlayerStats,
    expected: &PassPlayerStats,
) {
    assert_eq!(
        actual.completed_pass_count, expected.completed_pass_count,
        "{scope} pass.completed_pass_count"
    );
    assert_eq!(
        actual.received_pass_count, expected.received_pass_count,
        "{scope} pass.received_pass_count"
    );
    assert!(
        (actual.total_pass_distance - expected.total_pass_distance).abs() < 0.001,
        "{scope} pass.total_pass_distance: actual {:.3} expected {:.3}",
        actual.total_pass_distance,
        expected.total_pass_distance,
    );
    assert!(
        (actual.total_pass_advance - expected.total_pass_advance).abs() < 0.001,
        "{scope} pass.total_pass_advance: actual {:.3} expected {:.3}",
        actual.total_pass_advance,
        expected.total_pass_advance,
    );
    assert!(
        (actual.longest_pass_distance - expected.longest_pass_distance).abs() < 0.001,
        "{scope} pass.longest_pass_distance: actual {:.3} expected {:.3}",
        actual.longest_pass_distance,
        expected.longest_pass_distance,
    );
    assert_eq!(
        actual.is_last_completed_pass, expected.is_last_completed_pass,
        "{scope} pass.is_last_completed_pass"
    );
    assert_eq!(
        actual.last_completed_pass_frame, expected.last_completed_pass_frame,
        "{scope} pass.last_completed_pass_frame"
    );
    assert!(
        match (
            actual.last_completed_pass_time,
            expected.last_completed_pass_time,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} pass.last_completed_pass_time: actual {:?} expected {:?}",
        actual.last_completed_pass_time,
        expected.last_completed_pass_time,
    );
    assert_eq!(
        actual.frames_since_last_completed_pass, expected.frames_since_last_completed_pass,
        "{scope} pass.frames_since_last_completed_pass"
    );
    assert!(
        match (
            actual.time_since_last_completed_pass,
            expected.time_since_last_completed_pass,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} pass.time_since_last_completed_pass: actual {:?} expected {:?}",
        actual.time_since_last_completed_pass,
        expected.time_since_last_completed_pass,
    );
}

fn assert_pass_team_stats_match(scope: &str, actual: &PassTeamStats, expected: &PassTeamStats) {
    assert_eq!(
        actual.completed_pass_count, expected.completed_pass_count,
        "{scope} pass.completed_pass_count"
    );
    assert!(
        (actual.total_pass_distance - expected.total_pass_distance).abs() < 0.001,
        "{scope} pass.total_pass_distance: actual {:.3} expected {:.3}",
        actual.total_pass_distance,
        expected.total_pass_distance,
    );
    assert!(
        (actual.total_pass_advance - expected.total_pass_advance).abs() < 0.001,
        "{scope} pass.total_pass_advance: actual {:.3} expected {:.3}",
        actual.total_pass_advance,
        expected.total_pass_advance,
    );
    assert!(
        (actual.longest_pass_distance - expected.longest_pass_distance).abs() < 0.001,
        "{scope} pass.longest_pass_distance: actual {:.3} expected {:.3}",
        actual.longest_pass_distance,
        expected.longest_pass_distance,
    );
}

pub fn assert_pass_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.pass.clone();
    events.sort_by(|left, right| {
        left.sample_frame
            .cmp(&right.sample_frame)
            .then_with(|| left.sample_time.total_cmp(&right.sample_time))
    });
    let mut last_completed_events = timeline.events.pass_last_completed.clone();
    last_completed_events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });
    let has_last_completed_events = !last_completed_events.is_empty();

    let mut event_index = 0;
    let mut last_completed_event_index = 0;
    let mut players: HashMap<PlayerId, DerivedPassPlayerStats> = HashMap::new();
    let mut team_zero = PassTeamStats::default();
    let mut team_one = PassTeamStats::default();
    let mut last_completed_pass_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        for (player_id, stats) in players.iter_mut() {
            stats.advance_frame(
                frame,
                frame.is_live_play && last_completed_pass_player.as_ref() == Some(player_id),
            );
        }

        if !frame.is_live_play {
            last_completed_pass_player = None;
        } else {
            let mut processed_event = false;
            while event_index < events.len()
                && events[event_index].sample_frame <= frame.frame_number
            {
                let event = &events[event_index];
                players
                    .entry(event.passer.clone())
                    .or_default()
                    .record_completed_pass(event, frame);
                players
                    .entry(event.receiver.clone())
                    .or_default()
                    .record_received_pass();

                let team_stats = if event.is_team_0 {
                    &mut team_zero
                } else {
                    &mut team_one
                };
                team_stats.completed_pass_count += 1;
                team_stats.total_pass_distance += event.ball_travel_distance;
                team_stats.total_pass_advance += event.ball_advance_distance;
                team_stats.longest_pass_distance = team_stats
                    .longest_pass_distance
                    .max(event.ball_travel_distance);

                last_completed_pass_player = Some(event.passer.clone());
                event_index += 1;
                processed_event = true;
            }

            if !has_last_completed_events && processed_event {
                for stats in players.values_mut() {
                    stats.stats.is_last_completed_pass = false;
                }
            }

            if !has_last_completed_events {
                if let Some(player_id) = last_completed_pass_player.as_ref() {
                    if let Some(stats) = players.get_mut(player_id) {
                        stats.stats.is_last_completed_pass = true;
                    }
                }
            }
        }

        let mut processed_last_completed_event = false;
        while last_completed_event_index < last_completed_events.len()
            && last_completed_events[last_completed_event_index].frame <= frame.frame_number
        {
            last_completed_pass_player = last_completed_events[last_completed_event_index]
                .player
                .clone();
            last_completed_event_index += 1;
            processed_last_completed_event = true;
        }
        if processed_last_completed_event {
            for stats in players.values_mut() {
                stats.stats.is_last_completed_pass = false;
            }
            if let Some(player_id) = last_completed_pass_player.as_ref() {
                if let Some(stats) = players.get_mut(player_id) {
                    stats.stats.is_last_completed_pass = true;
                }
            }
        }

        assert_pass_team_stats_match(
            &format!("{replay_path} team_zero frame {}", frame.frame_number),
            &frame.team_zero.pass,
            &team_zero,
        );
        assert_pass_team_stats_match(
            &format!("{replay_path} team_one frame {}", frame.frame_number),
            &frame.team_one.pass,
            &team_one,
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_pass_derived_player_stats_match(
                &format!(
                    "{replay_path} player {} frame {}",
                    player.name, frame.frame_number
                ),
                &player.pass,
                &expected.stats,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed pass events"
    );
    assert_eq!(
        last_completed_event_index,
        last_completed_events.len(),
        "{replay_path} unprocessed pass-last-completed events"
    );
}
