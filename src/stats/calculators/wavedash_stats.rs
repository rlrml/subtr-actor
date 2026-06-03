use super::*;

const WAVEDASH_HIGH_CONFIDENCE: f32 = 0.75;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct WavedashStats {
    pub count: u32,
    pub high_confidence_count: u32,
    pub is_last_wavedash: bool,
    pub last_wavedash_time: Option<f32>,
    pub last_wavedash_frame: Option<usize>,
    pub time_since_last_wavedash: Option<f32>,
    pub frames_since_last_wavedash: Option<usize>,
    pub last_quality: Option<f32>,
    pub best_quality: f32,
    pub cumulative_quality: f32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_event_counts: LabeledCounts,
}

impl WavedashStats {
    pub fn average_quality(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.cumulative_quality / self.count as f32
        }
    }

    fn record_event(&mut self, event: &WavedashEvent) {
        self.labeled_event_counts.increment([confidence_band_label(
            event.confidence >= WAVEDASH_HIGH_CONFIDENCE,
        )]);
        self.sync_legacy_counts();
        self.last_wavedash_time = Some(event.time);
        self.last_wavedash_frame = Some(event.frame);
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

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WavedashStatsAccumulator {
    player_stats: HashMap<PlayerId, WavedashStats>,
    current_last_wavedash_player: Option<PlayerId>,
}

impl WavedashStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, WavedashStats> {
        &self.player_stats
    }

    pub fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_wavedash = false;
            stats.time_since_last_wavedash = stats
                .last_wavedash_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_wavedash = stats
                .last_wavedash_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }

        if let Some(player_id) = self.current_last_wavedash_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_wavedash = true;
            }
        }
    }

    pub fn apply_event(&mut self, event: &WavedashEvent) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_wavedash = false;
        }

        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.record_event(event);
        stats.is_last_wavedash = true;
        stats.time_since_last_wavedash = Some(0.0);
        stats.frames_since_last_wavedash = Some(0);

        self.current_last_wavedash_player = Some(event.player.clone());
    }

    pub fn reset_current_last_event_marker(&mut self) {
        self.current_last_wavedash_player = None;
    }
}
