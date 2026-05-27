use crate::*;

use super::*;

#[path = "positioning_core.rs"]
mod positioning_core;
#[path = "positioning_counts.rs"]
mod positioning_counts;
#[path = "positioning_percent.rs"]
mod positioning_percent;
#[path = "positioning_time.rs"]
mod positioning_time;
#[path = "positioning_time_roles.rs"]
mod positioning_time_roles;

impl StatFieldProvider for PositioningStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        self.visit_core_positioning_stat_fields(visitor);
        self.visit_positioning_time_stat_fields(visitor);
        self.visit_positioning_role_time_stat_fields(visitor);
        self.visit_positioning_percent_stat_fields(visitor);
        self.visit_positioning_count_stat_fields(visitor);
    }
}
