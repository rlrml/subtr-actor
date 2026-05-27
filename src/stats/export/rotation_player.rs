use super::rotation_player_counts::visit_rotation_player_count_fields;
use super::rotation_player_percent::visit_rotation_player_percent_fields;
use super::rotation_player_time::visit_rotation_player_time_fields;
use super::*;

impl StatFieldProvider for RotationPlayerStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visit_rotation_player_time_fields(self, visitor);
        visit_rotation_player_percent_fields(self, visitor);
        visit_rotation_player_count_fields(self, visitor);
    }
}
