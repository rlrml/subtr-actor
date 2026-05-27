use crate::{AirDribbleStats, ExportedStat, StatFieldProvider};

#[path = "air_dribble_export_counts.rs"]
mod counts;
#[path = "air_dribble_export_distance.rs"]
mod distance;
#[path = "air_dribble_export_time.rs"]
mod time;

impl StatFieldProvider for AirDribbleStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        counts::visit_air_dribble_count_fields(self, visitor);
        time::visit_air_dribble_time_fields(self, visitor);
        distance::visit_air_dribble_distance_fields(self, visitor);
    }
}
