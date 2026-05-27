use crate::{ExportedStat, MovementStats, StatFieldProvider};
#[cfg(test)]
use crate::{StatLabel, StatValue, LABELED_STAT_VARIANT};

#[path = "movement_export_core.rs"]
mod core;
#[path = "movement_export_percent.rs"]
mod percent;
#[path = "movement_export_time.rs"]
mod time;

impl StatFieldProvider for MovementStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        core::visit_movement_core_fields(self, visitor);
        time::visit_movement_time_fields(self, visitor);
        percent::visit_movement_percent_fields(self, visitor);
    }
}

#[cfg(test)]
#[path = "movement_test.rs"]
mod tests;
