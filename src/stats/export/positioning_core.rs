use super::*;

impl PositioningStats {
    pub(super) fn visit_core_positioning_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::float(
            "positioning",
            "active_game_time",
            StatUnit::Seconds,
            self.active_game_time,
        ));
        visitor(ExportedStat::float(
            "positioning",
            "avg_distance_to_ball",
            StatUnit::UnrealUnits,
            self.average_distance_to_ball(),
        ));
        visitor(ExportedStat::float(
            "positioning",
            "avg_distance_to_ball_possession",
            StatUnit::UnrealUnits,
            self.average_distance_to_ball_has_possession(),
        ));
        visitor(ExportedStat::float(
            "positioning",
            "avg_distance_to_ball_no_possession",
            StatUnit::UnrealUnits,
            self.average_distance_to_ball_no_possession(),
        ));
        visitor(ExportedStat::float(
            "positioning",
            "avg_distance_to_mates",
            StatUnit::UnrealUnits,
            self.average_distance_to_teammates(),
        ));
    }
}
