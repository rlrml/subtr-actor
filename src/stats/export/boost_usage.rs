use super::*;

impl BoostStats {
    pub(super) fn visit_usage_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visitor(ExportedStat::float(
            "boost",
            "amount_used",
            StatUnit::Boost,
            self.amount_used,
        ));
        visitor(ExportedStat::float(
            "boost",
            "amount_used_while_grounded",
            StatUnit::Boost,
            self.amount_used_while_grounded,
        ));
        visitor(ExportedStat::float(
            "boost",
            "amount_used_while_airborne",
            StatUnit::Boost,
            self.amount_used_while_airborne,
        ));
        visitor(ExportedStat::float(
            "boost",
            "amount_used_while_supersonic",
            StatUnit::Boost,
            self.amount_used_while_supersonic,
        ));
    }
}
