use super::*;

impl TouchStats {
    pub fn complete_labeled_touch_counts(&self) -> LabeledCounts {
        let mut entries: Vec<_> = ALL_PLAYER_VERTICAL_BANDS
            .into_iter()
            .flat_map(|height_band| {
                ALL_TOUCH_SURFACES.into_iter().flat_map(move |surface| {
                    ALL_TOUCH_DODGE_STATES
                        .into_iter()
                        .flat_map(move |dodge_state| {
                            ALL_TOUCH_KINDS.into_iter().map(move |kind| {
                                labeled_count_entry(self, kind, height_band, surface, dodge_state)
                            })
                        })
                })
            })
            .collect();
        entries.sort_by(|left, right| left.labels.cmp(&right.labels));
        LabeledCounts { entries }
    }

    pub fn with_complete_labeled_touch_counts(mut self) -> Self {
        self.labeled_touch_counts = self.complete_labeled_touch_counts();
        self
    }
}

fn labeled_count_entry(
    stats: &TouchStats,
    kind: TouchKind,
    height_band: PlayerVerticalBand,
    surface: TouchSurface,
    dodge_state: TouchDodgeState,
) -> LabeledCountEntry {
    let mut labels = vec![
        kind.as_label(),
        height_band.as_label(),
        surface.as_label(),
        dodge_state.as_label(),
    ];
    labels.sort();
    LabeledCountEntry {
        count: stats.labeled_touch_counts.count_exact(&labels),
        labels,
    }
}
