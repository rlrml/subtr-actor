use super::*;

const FLICK_HIGH_CONFIDENCE: f32 = 0.80;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FlickStats {
    pub count: u32,
    pub high_confidence_count: u32,
    pub is_last_flick: bool,
    pub last_flick_time: Option<f32>,
    pub last_flick_frame: Option<usize>,
    pub time_since_last_flick: Option<f32>,
    pub frames_since_last_flick: Option<usize>,
    pub last_confidence: Option<f32>,
    pub best_confidence: f32,
    pub cumulative_confidence: f32,
    pub cumulative_setup_duration: f32,
    pub cumulative_ball_speed_change: f32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_event_counts: LabeledCounts,
}

impl FlickStats {
    pub fn average_confidence(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.cumulative_confidence / self.count as f32
        }
    }

    pub fn average_setup_duration(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.cumulative_setup_duration / self.count as f32
        }
    }

    pub fn average_ball_speed_change(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.cumulative_ball_speed_change / self.count as f32
        }
    }

    fn record_event(&mut self, event: &FlickEvent) {
        self.labeled_event_counts.increment([confidence_band_label(
            event.confidence >= FLICK_HIGH_CONFIDENCE,
        )]);
        self.sync_legacy_counts();
        self.last_flick_time = Some(event.time);
        self.last_flick_frame = Some(event.frame);
        self.last_confidence = Some(event.confidence);
        self.best_confidence = self.best_confidence.max(event.confidence);
        self.cumulative_confidence += event.confidence;
        self.cumulative_setup_duration += event.setup_duration;
        self.cumulative_ball_speed_change += event.ball_speed_change;
    }

    pub fn event_count_with_labels(&self, labels: &[StatLabel]) -> u32 {
        self.labeled_event_counts.count_matching(labels)
    }

    pub fn complete_labeled_event_counts(&self) -> LabeledCounts {
        LabeledCounts::complete_from_label_sets(
            &[&CONFIDENCE_BAND_LABELS],
            &self.labeled_event_counts,
        )
    }

    fn sync_legacy_counts(&mut self) {
        self.count = self.labeled_event_counts.total();
        self.high_confidence_count = self.event_count_with_labels(&[confidence_band_label(true)]);
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FlickStatsAccumulator {
    player_stats: HashMap<PlayerId, FlickStats>,
    current_last_flick_player: Option<PlayerId>,
}

impl FlickStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, FlickStats> {
        &self.player_stats
    }

    pub fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_flick = false;
            stats.time_since_last_flick = stats
                .last_flick_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_flick = stats
                .last_flick_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }

        if let Some(player_id) = self.current_last_flick_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_flick = true;
            }
        }
    }

    pub fn apply_event(&mut self, event: &FlickEvent, frame: &FrameInfo) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.record_event(event);
        stats.is_last_flick = true;
        stats.time_since_last_flick = Some((frame.time - event.time).max(0.0));
        stats.frames_since_last_flick = Some(frame.frame_number.saturating_sub(event.frame));

        self.current_last_flick_player = Some(event.player.clone());
    }

    pub fn reset_current_last_event_marker(&mut self) {
        self.current_last_flick_player = None;
    }
}
