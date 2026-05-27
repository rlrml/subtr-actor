use super::*;

impl RushStats {
    fn team_count(&self, is_team_zero: bool) -> u32 {
        self.rush_count_with_labels(&[rush_team_label(is_team_zero)])
    }

    fn matchup_count(&self, is_team_zero: bool, attackers: usize, defenders: usize) -> u32 {
        self.rush_count_with_labels(&[
            rush_team_label(is_team_zero),
            rush_attackers_label(attackers),
            rush_defenders_label(defenders),
        ])
    }

    pub(super) fn sync_legacy_counts(&mut self) {
        self.team_zero_count = self.team_count(true);
        self.team_zero_two_v_one_count = self.matchup_count(true, 2, 1);
        self.team_zero_two_v_two_count = self.matchup_count(true, 2, 2);
        self.team_zero_two_v_three_count = self.matchup_count(true, 2, 3);
        self.team_zero_three_v_one_count = self.matchup_count(true, 3, 1);
        self.team_zero_three_v_two_count = self.matchup_count(true, 3, 2);
        self.team_zero_three_v_three_count = self.matchup_count(true, 3, 3);
        self.team_one_count = self.team_count(false);
        self.team_one_two_v_one_count = self.matchup_count(false, 2, 1);
        self.team_one_two_v_two_count = self.matchup_count(false, 2, 2);
        self.team_one_two_v_three_count = self.matchup_count(false, 2, 3);
        self.team_one_three_v_one_count = self.matchup_count(false, 3, 1);
        self.team_one_three_v_two_count = self.matchup_count(false, 3, 2);
        self.team_one_three_v_three_count = self.matchup_count(false, 3, 3);
    }
}
