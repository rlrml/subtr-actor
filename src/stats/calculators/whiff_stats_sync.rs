use super::*;

impl WhiffStats {
    pub(super) fn sync_legacy_counts(&mut self) {
        self.whiff_count = self.labeled_whiff_counts.total();
        self.grounded_whiff_count = self.whiff_count_with_labels(&[vertical_state_label(false)]);
        self.aerial_whiff_count = self.whiff_count_with_labels(&[vertical_state_label(true)]);
        self.dodge_whiff_count = self.whiff_count_with_labels(&[whiff_dodge_state_label(true)]);
    }
}
