use super::*;

const CEILING_SHOT_HIGH_CONFIDENCE: f32 = 0.78;

/// Per-player accumulated ceiling-shot stats with confidence.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct CeilingShotStats {
    pub count: u32,
    pub high_confidence_count: u32,
    pub is_last_ceiling_shot: bool,
    pub last_ceiling_shot_time: Option<f32>,
    pub last_ceiling_shot_frame: Option<usize>,
    pub time_since_last_ceiling_shot: Option<f32>,
    pub frames_since_last_ceiling_shot: Option<usize>,
    pub last_confidence: Option<f32>,
    pub best_confidence: f32,
    pub cumulative_confidence: f32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_event_counts: LabeledCounts,
}

impl CeilingShotStats {
    pub fn average_confidence(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.cumulative_confidence / self.count as f32
        }
    }

    fn record_event(&mut self, event: &CeilingShotEvent) {
        self.labeled_event_counts.increment([confidence_band_label(
            event.confidence >= CEILING_SHOT_HIGH_CONFIDENCE,
        )]);
        self.sync_legacy_counts();
        self.last_ceiling_shot_time = Some(event.time);
        self.last_ceiling_shot_frame = Some(event.frame);
        self.last_confidence = Some(event.confidence);
        self.best_confidence = self.best_confidence.max(event.confidence);
        self.cumulative_confidence += event.confidence;
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

/// Accumulates ceiling-shot stats over the replay.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CeilingShotStatsAccumulator {
    player_stats: HashMap<PlayerId, CeilingShotStats>,
    current_last_ceiling_shot_player: Option<PlayerId>,
}

impl CeilingShotStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, CeilingShotStats> {
        &self.player_stats
    }

    pub fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_ceiling_shot = false;
            stats.time_since_last_ceiling_shot = stats
                .last_ceiling_shot_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_ceiling_shot = stats
                .last_ceiling_shot_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }

        if let Some(player_id) = self.current_last_ceiling_shot_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_ceiling_shot = true;
            }
        }
    }

    pub fn apply_event(&mut self, event: &CeilingShotEvent, frame: &FrameInfo) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.record_event(event);
        stats.is_last_ceiling_shot = true;
        stats.time_since_last_ceiling_shot = Some((frame.time - event.time).max(0.0));
        stats.frames_since_last_ceiling_shot = Some(frame.frame_number.saturating_sub(event.frame));

        self.current_last_ceiling_shot_player = Some(event.player.clone());

        if let Some(player_id) = self.current_last_ceiling_shot_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_ceiling_shot = true;
            }
        }
    }

    pub fn reset_current_last_event_marker(&mut self) {
        self.current_last_ceiling_shot_player = None;
    }
}
