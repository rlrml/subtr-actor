use super::fifty_fifty_team_count_fields::visit_team_count_fields;
use super::fifty_fifty_team_helpers::{visit_team_labeled_count_fields, visit_team_percent_fields};
use super::*;

impl StatFieldProvider for FiftyFiftyStats {
    fn visit_stat_fields(&self, visitor: &mut dyn FnMut(ExportedStat)) {
        visit_team_count_fields(self, visitor);
        visit_team_percent_fields(self, visitor);
        visit_team_labeled_count_fields(self, visitor);
    }
}
