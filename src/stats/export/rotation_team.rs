use super::*;

impl StatFieldProvider for RotationTeamStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::unsigned(
            "rotation",
            "first_man_changes_for_team",
            StatUnit::Count,
            self.first_man_changes_for_team,
        ));
        visitor(ExportedStat::unsigned(
            "rotation",
            "rotation_count",
            StatUnit::Count,
            self.rotation_count,
        ));
    }
}
