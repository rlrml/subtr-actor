use super::*;

impl PositioningStats {
    pub(super) fn visit_positioning_count_stat_fields(
        &self,
        visitor: &mut dyn FnMut(ExportedStat),
    ) {
        visitor(ExportedStat::unsigned(
            "positioning",
            "times_caught_ahead_of_play_on_conceded_goals",
            StatUnit::Count,
            self.times_caught_ahead_of_play_on_conceded_goals,
        ));
    }
}
