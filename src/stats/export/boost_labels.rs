use super::*;

impl BoostStats {
    pub(super) fn visit_labeled_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        for entry in &self.labeled_amounts.entries {
            visitor(ExportedStat::float_labeled(
                "boost",
                "amount",
                StatUnit::Boost,
                entry.labels.clone(),
                entry.value,
            ));
        }
        for entry in &self.labeled_counts.entries {
            visitor(ExportedStat::unsigned_labeled(
                "boost",
                "count",
                StatUnit::Count,
                entry.labels.clone(),
                entry.count,
            ));
        }
    }
}
