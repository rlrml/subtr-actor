use std::collections::HashMap;

use subtr_actor::*;

#[derive(Clone, Default)]
struct DerivedBackboardPlayerStats {
    stats: BackboardPlayerStats,
}

impl DerivedBackboardPlayerStats {
    fn advance_frame(&mut self, frame: &ReplayStatsFrame, is_last_backboard_player: bool) {
        self.stats.is_last_backboard = is_last_backboard_player;
        self.stats.time_since_last_backboard = self
            .stats
            .last_backboard_time
            .map(|time| (frame.time - time).max(0.0));
        self.stats.frames_since_last_backboard = self
            .stats
            .last_backboard_frame
            .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
    }

    fn record_event(&mut self, event: &BackboardBounceEvent, frame: &ReplayStatsFrame) {
        self.stats.count += 1;
        self.stats.last_backboard_time = Some(event.time);
        self.stats.last_backboard_frame = Some(event.frame);
        self.stats.time_since_last_backboard = Some((frame.time - event.time).max(0.0));
        self.stats.frames_since_last_backboard =
            Some(frame.frame_number.saturating_sub(event.frame));
    }
}

fn assert_backboard_derived_player_stats_match(
    scope: &str,
    actual: &BackboardPlayerStats,
    expected: &BackboardPlayerStats,
) {
    assert_eq!(actual.count, expected.count, "{scope} backboard.count");
    assert_eq!(
        actual.is_last_backboard, expected.is_last_backboard,
        "{scope} backboard.is_last_backboard"
    );
    assert_eq!(
        actual.last_backboard_frame, expected.last_backboard_frame,
        "{scope} backboard.last_backboard_frame"
    );
    assert!(
        match (actual.last_backboard_time, expected.last_backboard_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} backboard.last_backboard_time: actual {:?} expected {:?}",
        actual.last_backboard_time,
        expected.last_backboard_time,
    );
    assert_eq!(
        actual.frames_since_last_backboard, expected.frames_since_last_backboard,
        "{scope} backboard.frames_since_last_backboard"
    );
    assert!(
        match (
            actual.time_since_last_backboard,
            expected.time_since_last_backboard,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} backboard.time_since_last_backboard: actual {:?} expected {:?}",
        actual.time_since_last_backboard,
        expected.time_since_last_backboard,
    );
}

pub fn assert_backboard_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.backboard.clone();
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, DerivedBackboardPlayerStats> = HashMap::new();
    let mut team_zero = BackboardTeamStats::default();
    let mut team_one = BackboardTeamStats::default();
    let mut last_backboard_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        for (player_id, stats) in players.iter_mut() {
            stats.advance_frame(frame, last_backboard_player.as_ref() == Some(player_id));
        }

        let mut processed_event = false;
        while event_index < events.len() && events[event_index].frame <= frame.frame_number {
            let event = &events[event_index];
            players
                .entry(event.player.clone())
                .or_default()
                .record_event(event, frame);
            if event.is_team_0 {
                team_zero.count += 1;
            } else {
                team_one.count += 1;
            }
            last_backboard_player = Some(event.player.clone());
            event_index += 1;
            processed_event = true;
        }

        if processed_event {
            for stats in players.values_mut() {
                stats.stats.is_last_backboard = false;
            }
        }

        if let Some(player_id) = last_backboard_player.as_ref() {
            if let Some(stats) = players.get_mut(player_id) {
                stats.stats.is_last_backboard = true;
            }
        }

        assert_eq!(
            frame.team_zero.backboard, team_zero,
            "{replay_path} team_zero backboard frame {}",
            frame.frame_number
        );
        assert_eq!(
            frame.team_one.backboard, team_one,
            "{replay_path} team_one backboard frame {}",
            frame.frame_number
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_backboard_derived_player_stats_match(
                &format!(
                    "{replay_path} player {} frame {}",
                    player.name, frame.frame_number
                ),
                &player.backboard,
                &expected.stats,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed backboard events"
    );
}

#[derive(Clone, Default)]
struct DerivedDoubleTapPlayerStats {
    stats: DoubleTapPlayerStats,
}

impl DerivedDoubleTapPlayerStats {
    fn advance_frame(&mut self, frame: &ReplayStatsFrame, is_last_double_tap_player: bool) {
        self.stats.is_last_double_tap = is_last_double_tap_player;
        self.stats.time_since_last_double_tap = self
            .stats
            .last_double_tap_time
            .map(|time| (frame.time - time).max(0.0));
        self.stats.frames_since_last_double_tap = self
            .stats
            .last_double_tap_frame
            .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
    }

    fn record_event(&mut self, event: &DoubleTapEvent, frame: &ReplayStatsFrame) {
        self.stats.count += 1;
        self.stats.last_double_tap_time = Some(event.time);
        self.stats.last_double_tap_frame = Some(event.frame);
        self.stats.time_since_last_double_tap = Some((frame.time - event.time).max(0.0));
        self.stats.frames_since_last_double_tap =
            Some(frame.frame_number.saturating_sub(event.frame));
    }
}

fn assert_double_tap_derived_player_stats_match(
    scope: &str,
    actual: &DoubleTapPlayerStats,
    expected: &DoubleTapPlayerStats,
) {
    assert_eq!(actual.count, expected.count, "{scope} double_tap.count");
    assert_eq!(
        actual.is_last_double_tap, expected.is_last_double_tap,
        "{scope} double_tap.is_last_double_tap"
    );
    assert_eq!(
        actual.last_double_tap_frame, expected.last_double_tap_frame,
        "{scope} double_tap.last_double_tap_frame"
    );
    assert!(
        match (actual.last_double_tap_time, expected.last_double_tap_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} double_tap.last_double_tap_time: actual {:?} expected {:?}",
        actual.last_double_tap_time,
        expected.last_double_tap_time,
    );
    assert_eq!(
        actual.frames_since_last_double_tap, expected.frames_since_last_double_tap,
        "{scope} double_tap.frames_since_last_double_tap"
    );
    assert!(
        match (
            actual.time_since_last_double_tap,
            expected.time_since_last_double_tap,
        ) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} double_tap.time_since_last_double_tap: actual {:?} expected {:?}",
        actual.time_since_last_double_tap,
        expected.time_since_last_double_tap,
    );
}

pub fn assert_double_tap_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.double_tap.clone();
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, DerivedDoubleTapPlayerStats> = HashMap::new();
    let mut team_zero = DoubleTapTeamStats::default();
    let mut team_one = DoubleTapTeamStats::default();
    let mut last_double_tap_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        for (player_id, stats) in players.iter_mut() {
            stats.advance_frame(frame, last_double_tap_player.as_ref() == Some(player_id));
        }

        let mut processed_event = false;
        while event_index < events.len() && events[event_index].frame <= frame.frame_number {
            let event = &events[event_index];
            players
                .entry(event.player.clone())
                .or_default()
                .record_event(event, frame);
            if event.is_team_0 {
                team_zero.count += 1;
            } else {
                team_one.count += 1;
            }
            last_double_tap_player = Some(event.player.clone());
            event_index += 1;
            processed_event = true;
        }

        if processed_event {
            for stats in players.values_mut() {
                stats.stats.is_last_double_tap = false;
            }
        }

        if let Some(player_id) = last_double_tap_player.as_ref() {
            if let Some(stats) = players.get_mut(player_id) {
                stats.stats.is_last_double_tap = true;
            }
        }

        assert_eq!(
            frame.team_zero.double_tap, team_zero,
            "{replay_path} team_zero double_tap frame {}",
            frame.frame_number
        );
        assert_eq!(
            frame.team_one.double_tap, team_one,
            "{replay_path} team_one double_tap frame {}",
            frame.frame_number
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_double_tap_derived_player_stats_match(
                &format!(
                    "{replay_path} player {} frame {}",
                    player.name, frame.frame_number
                ),
                &player.double_tap,
                &expected.stats,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed double-tap events"
    );
}
