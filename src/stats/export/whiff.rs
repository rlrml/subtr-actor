use crate::{ExportedStat, StatFieldProvider, WhiffStats};
#[cfg(test)]
use crate::{StatLabel, StatValue, LABELED_STAT_VARIANT};

#[path = "whiff_export_counts.rs"]
mod counts;
#[path = "whiff_export_distance.rs"]
mod distance;
#[path = "whiff_export_last.rs"]
mod last;

impl StatFieldProvider for WhiffStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        counts::visit_whiff_count_fields(self, visitor);
        last::visit_last_whiff_fields(self, visitor);
        distance::visit_whiff_distance_fields(self, visitor);
    }
}

#[cfg(test)]
#[path = "whiff_test.rs"]
mod tests;
