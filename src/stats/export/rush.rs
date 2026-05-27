use crate::{ExportedStat, RushStats, StatFieldProvider};
#[cfg(test)]
use crate::{StatLabel, StatValue, LABELED_STAT_VARIANT};

#[path = "rush_export_labeled.rs"]
mod labeled;
#[path = "rush_export_legacy.rs"]
mod legacy;

impl StatFieldProvider for RushStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        legacy::visit_legacy_rush_fields(self, visitor);
        labeled::visit_labeled_rush_fields(self, visitor);
    }
}

#[cfg(test)]
#[path = "rush_test.rs"]
mod tests;
