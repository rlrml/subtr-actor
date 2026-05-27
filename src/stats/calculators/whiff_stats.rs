use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct WhiffStats {
    pub whiff_count: u32,
    pub beaten_to_ball_count: u32,
    pub grounded_whiff_count: u32,
    pub aerial_whiff_count: u32,
    pub dodge_whiff_count: u32,
    pub is_last_whiff: bool,
    pub last_whiff_time: Option<f32>,
    pub last_whiff_frame: Option<usize>,
    pub time_since_last_whiff: Option<f32>,
    pub frames_since_last_whiff: Option<usize>,
    pub last_closest_approach_distance: Option<f32>,
    pub best_closest_approach_distance: Option<f32>,
    pub cumulative_closest_approach_distance: f32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_whiff_counts: LabeledCounts,
}

impl WhiffStats {
    pub fn average_closest_approach_distance(&self) -> f32 {
        if self.whiff_count == 0 {
            0.0
        } else {
            self.cumulative_closest_approach_distance / self.whiff_count as f32
        }
    }

    pub(super) fn record_whiff(&mut self, event: &WhiffEvent) {
        self.labeled_whiff_counts.increment(event.labels());
        self.sync_legacy_counts();
        self.last_whiff_time = Some(event.time);
        self.last_whiff_frame = Some(event.frame);
        self.last_closest_approach_distance = Some(event.closest_approach_distance);
        self.best_closest_approach_distance = Some(
            self.best_closest_approach_distance
                .map(|distance| distance.min(event.closest_approach_distance))
                .unwrap_or(event.closest_approach_distance),
        );
        self.cumulative_closest_approach_distance += event.closest_approach_distance;
    }

    pub fn whiff_count_with_labels(&self, labels: &[StatLabel]) -> u32 {
        self.labeled_whiff_counts.count_matching(labels)
    }

    pub fn complete_labeled_whiff_counts(&self) -> LabeledCounts {
        LabeledCounts::complete_from_label_sets(
            &[&VERTICAL_STATE_LABELS, &WHIFF_DODGE_STATE_LABELS],
            &self.labeled_whiff_counts,
        )
    }

    pub fn with_complete_labeled_whiff_counts(mut self) -> Self {
        self.labeled_whiff_counts = self.complete_labeled_whiff_counts();
        self
    }
}
