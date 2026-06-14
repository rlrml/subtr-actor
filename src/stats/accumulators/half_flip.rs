use super::*;

const HALF_FLIP_HIGH_CONFIDENCE: f32 = 0.78;

/// Per-player accumulated half-flip stats with confidence.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct HalfFlipStats {
    pub count: u32,
    pub high_confidence_count: u32,
    pub is_last_half_flip: bool,
    pub last_half_flip_time: Option<f32>,
    pub last_half_flip_frame: Option<usize>,
    pub time_since_last_half_flip: Option<f32>,
    pub frames_since_last_half_flip: Option<usize>,
    pub last_quality: Option<f32>,
    pub best_quality: f32,
    pub cumulative_quality: f32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_event_counts: LabeledCounts,
}

impl HalfFlipStats {
    pub fn average_quality(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.cumulative_quality / self.count as f32
        }
    }

    fn record_event(&mut self, event: &HalfFlipEvent) {
        self.labeled_event_counts.increment([confidence_band_label(
            event.confidence >= HALF_FLIP_HIGH_CONFIDENCE,
        )]);
        self.sync_legacy_counts();
        self.last_half_flip_time = Some(event.time);
        self.last_half_flip_frame = Some(event.frame);
        self.last_quality = Some(event.confidence);
        self.best_quality = self.best_quality.max(event.confidence);
        self.cumulative_quality += event.confidence;
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

/// Accumulates half-flip stats over the replay.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HalfFlipStatsAccumulator {
    player_stats: HashMap<PlayerId, HalfFlipStats>,
    current_last_half_flip_player: Option<PlayerId>,
}

impl HalfFlipStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, HalfFlipStats> {
        &self.player_stats
    }

    pub fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_half_flip = false;
            stats.time_since_last_half_flip = stats
                .last_half_flip_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_half_flip = stats
                .last_half_flip_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }

        if let Some(player_id) = self.current_last_half_flip_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_half_flip = true;
            }
        }
    }

    pub fn apply_event(&mut self, event: &HalfFlipEvent) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_half_flip = false;
        }

        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.record_event(event);
        stats.is_last_half_flip = true;
        stats.time_since_last_half_flip = Some(0.0);
        stats.frames_since_last_half_flip = Some(0);

        self.current_last_half_flip_player = Some(event.player.clone());
    }

    pub fn reset_current_last_event_marker(&mut self) {
        self.current_last_half_flip_player = None;
    }
}
