use std::collections::HashMap;

use subtr_actor::*;

#[derive(Clone, Default)]
struct DerivedQualityMechanicStats {
    count: u32,
    high_confidence_count: u32,
    last_time: Option<f32>,
    last_frame: Option<usize>,
    last_resolved_time: Option<f32>,
    last_resolved_frame: Option<usize>,
    last_quality: Option<f32>,
    best_quality: f32,
    cumulative_quality: f32,
}

impl DerivedQualityMechanicStats {
    fn record(
        &mut self,
        frame: usize,
        time: f32,
        resolved_frame: usize,
        resolved_time: f32,
        confidence: f32,
        high_confidence: bool,
    ) {
        self.count += 1;
        if high_confidence {
            self.high_confidence_count += 1;
        }
        self.last_time = Some(time);
        self.last_frame = Some(frame);
        self.last_resolved_time = Some(resolved_time);
        self.last_resolved_frame = Some(resolved_frame);
        self.last_quality = Some(confidence);
        self.best_quality = self.best_quality.max(confidence);
        self.cumulative_quality += confidence;
    }
}

#[derive(Clone, Default)]
struct DerivedQualityMechanicFrameStats {
    count: u32,
    high_confidence_count: u32,
    is_last_player: bool,
    last_time: Option<f32>,
    last_frame: Option<usize>,
    time_since_last: Option<f32>,
    frames_since_last: Option<usize>,
    last_quality: Option<f32>,
    best_quality: f32,
    cumulative_quality: f32,
}

impl DerivedQualityMechanicFrameStats {
    fn from_accumulator(
        accumulator: Option<&DerivedQualityMechanicStats>,
        frame: &ReplayStatsFrame,
        is_last_player: bool,
    ) -> Self {
        let Some(accumulator) = accumulator else {
            return Self::default();
        };
        let is_resolution_frame = accumulator.last_resolved_frame == Some(frame.frame_number);
        Self {
            count: accumulator.count,
            high_confidence_count: accumulator.high_confidence_count,
            is_last_player,
            last_time: accumulator.last_time,
            last_frame: accumulator.last_frame,
            time_since_last: if is_resolution_frame {
                Some(0.0)
            } else {
                accumulator
                    .last_time
                    .map(|time| (frame.time - time).max(0.0))
            },
            frames_since_last: if is_resolution_frame {
                Some(0)
            } else {
                accumulator
                    .last_frame
                    .map(|last_frame| frame.frame_number.saturating_sub(last_frame))
            },
            last_quality: accumulator.last_quality,
            best_quality: accumulator.best_quality,
            cumulative_quality: accumulator.cumulative_quality,
        }
    }
}

fn assert_half_flip_derived_stats_match(
    scope: &str,
    actual: &HalfFlipStats,
    expected: &DerivedQualityMechanicFrameStats,
) {
    assert_eq!(actual.count, expected.count, "{scope} half_flip.count");
    assert_eq!(
        actual.high_confidence_count, expected.high_confidence_count,
        "{scope} half_flip.high_confidence_count"
    );
    assert_eq!(
        actual.is_last_half_flip, expected.is_last_player,
        "{scope} half_flip.is_last_half_flip"
    );
    assert_eq!(
        actual.last_half_flip_frame, expected.last_frame,
        "{scope} half_flip.last_half_flip_frame"
    );
    assert!(
        match (actual.last_half_flip_time, expected.last_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} half_flip.last_half_flip_time: actual {:?} expected {:?}",
        actual.last_half_flip_time,
        expected.last_time,
    );
    assert_eq!(
        actual.frames_since_last_half_flip, expected.frames_since_last,
        "{scope} half_flip.frames_since_last_half_flip",
    );
    assert!(
        match (actual.time_since_last_half_flip, expected.time_since_last) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} half_flip.time_since_last_half_flip: actual {:?} expected {:?}",
        actual.time_since_last_half_flip,
        expected.last_time,
    );
    assert!(
        match (actual.last_quality, expected.last_quality) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} half_flip.last_quality: actual {:?} expected {:?}",
        actual.last_quality,
        expected.last_quality,
    );
    assert!(
        (actual.best_quality - expected.best_quality).abs() < 0.001,
        "{scope} half_flip.best_quality: actual {:.3} expected {:.3}",
        actual.best_quality,
        expected.best_quality,
    );
    assert!(
        (actual.cumulative_quality - expected.cumulative_quality).abs() < 0.001,
        "{scope} half_flip.cumulative_quality: actual {:.3} expected {:.3}",
        actual.cumulative_quality,
        expected.cumulative_quality,
    );
}

fn assert_wavedash_derived_stats_match(
    scope: &str,
    actual: &WavedashStats,
    expected: &DerivedQualityMechanicFrameStats,
) {
    assert_eq!(actual.count, expected.count, "{scope} wavedash.count");
    assert_eq!(
        actual.high_confidence_count, expected.high_confidence_count,
        "{scope} wavedash.high_confidence_count"
    );
    assert_eq!(
        actual.is_last_wavedash, expected.is_last_player,
        "{scope} wavedash.is_last_wavedash"
    );
    assert_eq!(
        actual.last_wavedash_frame, expected.last_frame,
        "{scope} wavedash.last_wavedash_frame"
    );
    assert!(
        match (actual.last_wavedash_time, expected.last_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} wavedash.last_wavedash_time: actual {:?} expected {:?}",
        actual.last_wavedash_time,
        expected.last_time,
    );
    assert_eq!(
        actual.frames_since_last_wavedash, expected.frames_since_last,
        "{scope} wavedash.frames_since_last_wavedash",
    );
    assert!(
        match (actual.time_since_last_wavedash, expected.time_since_last) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} wavedash.time_since_last_wavedash: actual {:?} expected {:?}",
        actual.time_since_last_wavedash,
        expected.last_time,
    );
    assert!(
        match (actual.last_quality, expected.last_quality) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} wavedash.last_quality: actual {:?} expected {:?}",
        actual.last_quality,
        expected.last_quality,
    );
    assert!(
        (actual.best_quality - expected.best_quality).abs() < 0.001,
        "{scope} wavedash.best_quality: actual {:.3} expected {:.3}",
        actual.best_quality,
        expected.best_quality,
    );
    assert!(
        (actual.cumulative_quality - expected.cumulative_quality).abs() < 0.001,
        "{scope} wavedash.cumulative_quality: actual {:.3} expected {:.3}",
        actual.cumulative_quality,
        expected.cumulative_quality,
    );
}

fn assert_speed_flip_derived_stats_match(
    scope: &str,
    actual: &SpeedFlipStats,
    expected: &DerivedQualityMechanicFrameStats,
) {
    assert_eq!(actual.count, expected.count, "{scope} speed_flip.count");
    assert_eq!(
        actual.high_confidence_count, expected.high_confidence_count,
        "{scope} speed_flip.high_confidence_count"
    );
    assert_eq!(
        actual.is_last_speed_flip, expected.is_last_player,
        "{scope} speed_flip.is_last_speed_flip"
    );
    assert_eq!(
        actual.last_speed_flip_frame, expected.last_frame,
        "{scope} speed_flip.last_speed_flip_frame"
    );
    assert!(
        match (actual.last_speed_flip_time, expected.last_time) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} speed_flip.last_speed_flip_time: actual {:?} expected {:?}",
        actual.last_speed_flip_time,
        expected.last_time,
    );
    assert_eq!(
        actual.frames_since_last_speed_flip, expected.frames_since_last,
        "{scope} speed_flip.frames_since_last_speed_flip",
    );
    assert!(
        match (actual.time_since_last_speed_flip, expected.time_since_last) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} speed_flip.time_since_last_speed_flip: actual {:?} expected {:?}",
        actual.time_since_last_speed_flip,
        expected.time_since_last,
    );
    assert!(
        match (actual.last_quality, expected.last_quality) {
            (Some(actual), Some(expected)) => (actual - expected).abs() < 0.001,
            (None, None) => true,
            _ => false,
        },
        "{scope} speed_flip.last_quality: actual {:?} expected {:?}",
        actual.last_quality,
        expected.last_quality,
    );
    assert!(
        (actual.best_quality - expected.best_quality).abs() < 0.001,
        "{scope} speed_flip.best_quality: actual {:.3} expected {:.3}",
        actual.best_quality,
        expected.best_quality,
    );
    assert!(
        (actual.cumulative_quality - expected.cumulative_quality).abs() < 0.001,
        "{scope} speed_flip.cumulative_quality: actual {:.3} expected {:.3}",
        actual.cumulative_quality,
        expected.cumulative_quality,
    );
}

pub fn assert_quality_mechanic_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    const HALF_FLIP_HIGH_CONFIDENCE: f32 = 0.78;
    const WAVEDASH_HIGH_CONFIDENCE: f32 = 0.75;

    let mut half_flip_event_index = 0;
    let mut wavedash_event_index = 0;
    let mut half_flip_players: HashMap<PlayerId, DerivedQualityMechanicStats> = HashMap::new();
    let mut wavedash_players: HashMap<PlayerId, DerivedQualityMechanicStats> = HashMap::new();
    let mut half_flip_frame_stats: HashMap<PlayerId, DerivedQualityMechanicFrameStats> =
        HashMap::new();
    let mut wavedash_frame_stats: HashMap<PlayerId, DerivedQualityMechanicFrameStats> =
        HashMap::new();
    let mut last_half_flip_player: Option<PlayerId> = None;
    let mut last_wavedash_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        if frame.is_live_play {
            while half_flip_event_index < timeline.events.half_flip.len()
                && timeline.events.half_flip[half_flip_event_index].frame <= frame.frame_number
            {
                let event = &timeline.events.half_flip[half_flip_event_index];
                half_flip_players
                    .entry(event.player.clone())
                    .or_default()
                    .record(
                        event.frame,
                        event.time,
                        event.frame,
                        event.time,
                        event.confidence,
                        event.confidence >= HALF_FLIP_HIGH_CONFIDENCE,
                    );
                last_half_flip_player = Some(event.player.clone());
                half_flip_event_index += 1;
            }

            while wavedash_event_index < timeline.events.wavedash.len()
                && timeline.events.wavedash[wavedash_event_index].frame <= frame.frame_number
            {
                let event = &timeline.events.wavedash[wavedash_event_index];
                wavedash_players
                    .entry(event.player.clone())
                    .or_default()
                    .record(
                        event.frame,
                        event.time,
                        event.frame,
                        event.time,
                        event.confidence,
                        event.confidence >= WAVEDASH_HIGH_CONFIDENCE,
                    );
                last_wavedash_player = Some(event.player.clone());
                wavedash_event_index += 1;
            }

            for player in &frame.players {
                let half_flip_expected = DerivedQualityMechanicFrameStats::from_accumulator(
                    half_flip_players.get(&player.player_id),
                    frame,
                    last_half_flip_player.as_ref() == Some(&player.player_id),
                );
                half_flip_frame_stats.insert(player.player_id.clone(), half_flip_expected.clone());
                assert_half_flip_derived_stats_match(
                    &format!(
                        "{replay_path} player {} frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.half_flip,
                    &half_flip_expected,
                );

                let wavedash_expected = DerivedQualityMechanicFrameStats::from_accumulator(
                    wavedash_players.get(&player.player_id),
                    frame,
                    last_wavedash_player.as_ref() == Some(&player.player_id),
                );
                wavedash_frame_stats.insert(player.player_id.clone(), wavedash_expected.clone());
                assert_wavedash_derived_stats_match(
                    &format!(
                        "{replay_path} player {} frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.wavedash,
                    &wavedash_expected,
                );
            }
        } else {
            for player in &frame.players {
                let half_flip_expected = half_flip_frame_stats
                    .get(&player.player_id)
                    .cloned()
                    .unwrap_or_default();
                assert_half_flip_derived_stats_match(
                    &format!(
                        "{replay_path} player {} inactive frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.half_flip,
                    &half_flip_expected,
                );

                let wavedash_expected = wavedash_frame_stats
                    .get(&player.player_id)
                    .cloned()
                    .unwrap_or_default();
                assert_wavedash_derived_stats_match(
                    &format!(
                        "{replay_path} player {} inactive frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.wavedash,
                    &wavedash_expected,
                );
            }
            last_half_flip_player = None;
            last_wavedash_player = None;
        }
    }

    assert_eq!(
        half_flip_event_index,
        timeline.events.half_flip.len(),
        "{replay_path} unprocessed half-flip events"
    );
    assert_eq!(
        wavedash_event_index,
        timeline.events.wavedash.len(),
        "{replay_path} unprocessed wavedash events"
    );
}

pub fn assert_speed_flip_events_reconstruct_serialized_partial_sums(
    replay_path: &str,
    timeline: &ReplayStatsTimeline,
) {
    const SPEED_FLIP_HIGH_CONFIDENCE: f32 = 0.75;

    let mut speed_flip_event_index = 0;
    let mut speed_flip_players: HashMap<PlayerId, DerivedQualityMechanicStats> = HashMap::new();
    let mut speed_flip_frame_stats: HashMap<PlayerId, DerivedQualityMechanicFrameStats> =
        HashMap::new();
    let mut last_speed_flip_player: Option<PlayerId> = None;

    for frame in &timeline.frames {
        let speed_flip_stats_advance = frame.is_live_play || frame.ball_has_been_hit == Some(false);
        if speed_flip_stats_advance {
            while speed_flip_event_index < timeline.events.speed_flip.len()
                && timeline.events.speed_flip[speed_flip_event_index].resolved_frame
                    <= frame.frame_number
            {
                let event = &timeline.events.speed_flip[speed_flip_event_index];
                speed_flip_players
                    .entry(event.player.clone())
                    .or_default()
                    .record(
                        event.frame,
                        event.time,
                        event.resolved_frame,
                        event.resolved_time,
                        event.confidence,
                        event.confidence >= SPEED_FLIP_HIGH_CONFIDENCE,
                    );
                last_speed_flip_player = Some(event.player.clone());
                speed_flip_event_index += 1;
            }

            for player in &frame.players {
                let expected = DerivedQualityMechanicFrameStats::from_accumulator(
                    speed_flip_players.get(&player.player_id),
                    frame,
                    last_speed_flip_player.as_ref() == Some(&player.player_id),
                );
                speed_flip_frame_stats.insert(player.player_id.clone(), expected.clone());
                assert_speed_flip_derived_stats_match(
                    &format!(
                        "{replay_path} player {} frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.speed_flip,
                    &expected,
                );
            }
        } else {
            for player in &frame.players {
                let expected = speed_flip_frame_stats
                    .get(&player.player_id)
                    .cloned()
                    .unwrap_or_default();
                assert_speed_flip_derived_stats_match(
                    &format!(
                        "{replay_path} player {} frozen frame {}",
                        player.name, frame.frame_number
                    ),
                    &player.speed_flip,
                    &expected,
                );
            }
        }
    }

    assert_eq!(
        speed_flip_event_index,
        timeline.events.speed_flip.len(),
        "{replay_path} unprocessed speed-flip events"
    );
}
