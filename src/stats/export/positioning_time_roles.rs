use super::*;

impl PositioningStats {
    pub(super) fn visit_positioning_role_time_stat_fields(
        &self,
        visitor: &mut dyn FnMut(ExportedStat),
    ) {
        visitor(ExportedStat::float(
            "positioning",
            "time_no_teammates",
            StatUnit::Seconds,
            self.time_no_teammates,
        ));
        visitor(ExportedStat::float(
            "positioning",
            "time_most_back",
            StatUnit::Seconds,
            self.time_most_back,
        ));
        visitor(ExportedStat::float(
            "positioning",
            "time_most_forward",
            StatUnit::Seconds,
            self.time_most_forward,
        ));
        visitor(ExportedStat::float(
            "positioning",
            "time_mid_role",
            StatUnit::Seconds,
            self.time_mid_role,
        ));
        visitor(ExportedStat::float(
            "positioning",
            "time_other_role",
            StatUnit::Seconds,
            self.time_other_role,
        ));
        visitor(ExportedStat::float(
            "positioning",
            "time_closest_to_ball",
            StatUnit::Seconds,
            self.time_closest_to_ball,
        ));
        visitor(ExportedStat::float(
            "positioning",
            "time_farthest_from_ball",
            StatUnit::Seconds,
            self.time_farthest_from_ball,
        ));
    }
}
