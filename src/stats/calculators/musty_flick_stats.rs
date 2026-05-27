use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct MustyFlickStats {
    pub count: u32,
    pub aerial_count: u32,
    pub high_confidence_count: u32,
    pub is_last_musty: bool,
    pub last_musty_time: Option<f32>,
    pub last_musty_frame: Option<usize>,
    pub time_since_last_musty: Option<f32>,
    pub frames_since_last_musty: Option<usize>,
    pub last_confidence: Option<f32>,
    pub best_confidence: f32,
    pub cumulative_confidence: f32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_event_counts: LabeledCounts,
}

impl MustyFlickStats {
    pub fn average_confidence(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.cumulative_confidence / self.count as f32
        }
    }

    pub(super) fn record_event(&mut self, event: &MustyFlickEvent, aerial: bool) {
        self.labeled_event_counts.increment([
            vertical_state_label(aerial),
            confidence_band_label(event.confidence >= MUSTY_HIGH_CONFIDENCE),
        ]);
        self.sync_legacy_counts();
        self.last_musty_time = Some(event.time);
        self.last_musty_frame = Some(event.frame);
        self.last_confidence = Some(event.confidence);
        self.best_confidence = self.best_confidence.max(event.confidence);
        self.cumulative_confidence += event.confidence;
    }

    pub fn event_count_with_labels(&self, labels: &[StatLabel]) -> u32 {
        self.labeled_event_counts.count_matching(labels)
    }

    pub fn complete_labeled_event_counts(&self) -> LabeledCounts {
        LabeledCounts::complete_from_label_sets(
            &[&VERTICAL_STATE_LABELS, &CONFIDENCE_BAND_LABELS],
            &self.labeled_event_counts,
        )
    }

    fn sync_legacy_counts(&mut self) {
        self.count = self.labeled_event_counts.total();
        self.aerial_count = self.event_count_with_labels(&[vertical_state_label(true)]);
        self.high_confidence_count = self.event_count_with_labels(&[confidence_band_label(true)]);
    }
}
