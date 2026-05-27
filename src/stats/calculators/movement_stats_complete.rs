use super::*;

impl MovementStats {
    pub fn complete_labeled_tracked_time(&self) -> LabeledFloatSums {
        let mut entries: Vec<_> = ALL_PLAYER_VERTICAL_BANDS
            .into_iter()
            .flat_map(|height_band| {
                ALL_MOVEMENT_SPEED_BANDS.into_iter().map(move |speed_band| {
                    let mut labels = vec![speed_band.as_label(), height_band.as_label()];
                    labels.sort();
                    LabeledFloatSumEntry {
                        value: self.labeled_tracked_time.sum_exact(&labels),
                        labels,
                    }
                })
            })
            .collect();

        entries.sort_by(|left, right| left.labels.cmp(&right.labels));

        LabeledFloatSums { entries }
    }

    pub fn with_complete_labeled_tracked_time(mut self) -> Self {
        self.labeled_tracked_time = self.complete_labeled_tracked_time();
        self
    }
}
