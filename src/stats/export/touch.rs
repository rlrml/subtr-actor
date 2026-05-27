use crate::{ExportedStat, StatFieldProvider, TouchStats};

#[path = "touch_export_ball_movement.rs"]
mod ball_movement;
#[path = "touch_export_counts.rs"]
mod counts;
#[path = "touch_export_last_touch.rs"]
mod last_touch;

impl StatFieldProvider for TouchStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        counts::visit_touch_count_fields(self, visitor);
        last_touch::visit_last_touch_fields(self, visitor);
        ball_movement::visit_touch_ball_movement_fields(self, visitor);
    }
}

#[cfg(test)]
#[path = "touch_test.rs"]
mod tests;
