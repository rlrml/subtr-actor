use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct SpeedFlipStats {
    pub count: u32,
    pub high_confidence_count: u32,
    pub is_last_speed_flip: bool,
    pub last_speed_flip_time: Option<f32>,
    pub last_speed_flip_frame: Option<usize>,
    pub time_since_last_speed_flip: Option<f32>,
    pub frames_since_last_speed_flip: Option<usize>,
    pub last_quality: Option<f32>,
    pub best_quality: f32,
    pub cumulative_quality: f32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_event_counts: LabeledCounts,
}

impl SpeedFlipStats {
    pub fn average_quality(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.cumulative_quality / self.count as f32
        }
    }

    pub(super) fn record_event(&mut self, event: &SpeedFlipEvent) {
        self.labeled_event_counts.increment([confidence_band_label(
            event.confidence >= SPEED_FLIP_HIGH_CONFIDENCE,
        )]);
        self.sync_legacy_counts();
        self.last_speed_flip_time = Some(event.time);
        self.last_speed_flip_frame = Some(event.frame);
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
