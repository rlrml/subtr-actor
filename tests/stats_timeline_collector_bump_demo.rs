use std::collections::HashMap;

use subtr_actor::*;

#[derive(Clone, Default)]
struct DerivedBumpPlayerStats {
    stats: BumpPlayerStats,
}

impl DerivedBumpPlayerStats {
    fn record_inflicted(&mut self, event: &BumpEvent) {
        self.stats.bumps_inflicted += 1;
        if event.is_team_bump {
            self.stats.team_bumps_inflicted += 1;
        }
        self.stats.last_bump_time = Some(event.time);
        self.stats.last_bump_frame = Some(event.frame);
        self.stats.last_bump_strength = Some(event.strength);
        self.stats.max_bump_strength = self.stats.max_bump_strength.max(event.strength);
        self.stats.cumulative_bump_strength += event.strength;
    }

    fn record_taken(&mut self, event: &BumpEvent) {
        self.stats.bumps_taken += 1;
        if event.is_team_bump {
            self.stats.team_bumps_taken += 1;
        }
    }
}

fn apply_bump_team_event(stats: &mut BumpTeamStats, event: &BumpEvent) {
    stats.bumps_inflicted += 1;
    if event.is_team_bump {
        stats.team_bumps_inflicted += 1;
    }
}

fn assert_bump_player_stats_match(
    scope: &str,
    actual: &BumpPlayerStats,
    expected: &BumpPlayerStats,
) {
    assert_eq!(
        actual.bumps_inflicted, expected.bumps_inflicted,
        "{scope} bump.bumps_inflicted"
    );
    assert_eq!(
        actual.bumps_taken, expected.bumps_taken,
        "{scope} bump.bumps_taken"
    );
    assert_eq!(
        actual.team_bumps_inflicted, expected.team_bumps_inflicted,
        "{scope} bump.team_bumps_inflicted"
    );
    assert_eq!(
        actual.team_bumps_taken, expected.team_bumps_taken,
        "{scope} bump.team_bumps_taken"
    );
    assert_eq!(
        actual.last_bump_frame, expected.last_bump_frame,
        "{scope} bump.last_bump_frame"
    );
    assert!(
        match (actual.last_bump_time, expected.last_bump_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} bump.last_bump_time: actual {:?} expected {:?}",
        actual.last_bump_time,
        expected.last_bump_time,
    );
    assert!(
        match (actual.last_bump_strength, expected.last_bump_strength) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} bump.last_bump_strength: actual {:?} expected {:?}",
        actual.last_bump_strength,
        expected.last_bump_strength,
    );
    assert!(
        (actual.max_bump_strength - expected.max_bump_strength).abs() < 0.001,
        "{scope} bump.max_bump_strength: actual {:.3} expected {:.3}",
        actual.max_bump_strength,
        expected.max_bump_strength,
    );
    assert!(
        (actual.cumulative_bump_strength - expected.cumulative_bump_strength).abs() < 0.001,
        "{scope} bump.cumulative_bump_strength: actual {:.3} expected {:.3}",
        actual.cumulative_bump_strength,
        expected.cumulative_bump_strength,
    );
}

pub fn assert_bump_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events = timeline.events.bump.clone();
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.time.total_cmp(&right.time))
    });

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, DerivedBumpPlayerStats> = HashMap::new();
    let mut team_zero = BumpTeamStats::default();
    let mut team_one = BumpTeamStats::default();

    for frame in &timeline.frames {
        while event_index < events.len() && events[event_index].frame <= frame.frame_number {
            let event = &events[event_index];
            players
                .entry(event.initiator.clone())
                .or_default()
                .record_inflicted(event);
            players
                .entry(event.victim.clone())
                .or_default()
                .record_taken(event);
            apply_bump_team_event(
                if event.initiator_is_team_0 {
                    &mut team_zero
                } else {
                    &mut team_one
                },
                event,
            );
            event_index += 1;
        }

        assert_eq!(
            frame.team_zero.bump, team_zero,
            "{replay_path} team_zero bump frame {}",
            frame.frame_number,
        );
        assert_eq!(
            frame.team_one.bump, team_one,
            "{replay_path} team_one bump frame {}",
            frame.frame_number,
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_bump_player_stats_match(
                &format!(
                    "{replay_path} player {} frame {}",
                    player.name, frame.frame_number
                ),
                &player.bump,
                &expected.stats,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed bump events"
    );
}

pub fn assert_demo_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    let mut events: Vec<_> = timeline
        .events
        .timeline
        .iter()
        .filter(|event| {
            matches!(
                event.kind,
                TimelineEventKind::Kill | TimelineEventKind::Death
            )
        })
        .cloned()
        .collect();
    events.sort_by(|left, right| left.time.total_cmp(&right.time));

    let mut event_index = 0;
    let mut players: HashMap<PlayerId, DemoPlayerStats> = HashMap::new();
    let mut team_zero = DemoTeamStats::default();
    let mut team_one = DemoTeamStats::default();

    for frame in &timeline.frames {
        while event_index < events.len() && events[event_index].time <= frame.time {
            let event = &events[event_index];
            if let Some(player_id) = event.player_id.as_ref() {
                match event.kind {
                    TimelineEventKind::Kill => {
                        players
                            .entry(player_id.clone())
                            .or_default()
                            .demos_inflicted += 1;
                        match event.is_team_0 {
                            Some(true) => team_zero.demos_inflicted += 1,
                            Some(false) => team_one.demos_inflicted += 1,
                            None => {}
                        }
                    }
                    TimelineEventKind::Death => {
                        players.entry(player_id.clone()).or_default().demos_taken += 1;
                    }
                    _ => {}
                }
            }
            event_index += 1;
        }

        assert_eq!(
            frame.team_zero.demo, team_zero,
            "{replay_path} team_zero demo frame {}",
            frame.frame_number,
        );
        assert_eq!(
            frame.team_one.demo, team_one,
            "{replay_path} team_one demo frame {}",
            frame.frame_number,
        );
        for player in &frame.players {
            let expected = players.get(&player.player_id).cloned().unwrap_or_default();
            assert_eq!(
                player.demo, expected,
                "{replay_path} player {} demo frame {}",
                player.name, frame.frame_number,
            );
        }
    }

    assert_eq!(
        event_index,
        events.len(),
        "{replay_path} unprocessed demo events"
    );
}
