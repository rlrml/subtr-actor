use super::*;

pub(crate) const RUSH_TEAM_LABELS: [StatLabel; 2] = [
    StatLabel::new("team", "team_zero"),
    StatLabel::new("team", "team_one"),
];
pub(crate) const RUSH_ATTACKER_LABELS: [StatLabel; 2] = [
    StatLabel::new("attackers", "2"),
    StatLabel::new("attackers", "3"),
];
pub(crate) const RUSH_DEFENDER_LABELS: [StatLabel; 3] = [
    StatLabel::new("defenders", "1"),
    StatLabel::new("defenders", "2"),
    StatLabel::new("defenders", "3"),
];

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RushStats {
    pub team_zero_count: u32,
    pub team_zero_two_v_one_count: u32,
    pub team_zero_two_v_two_count: u32,
    pub team_zero_two_v_three_count: u32,
    pub team_zero_three_v_one_count: u32,
    pub team_zero_three_v_two_count: u32,
    pub team_zero_three_v_three_count: u32,
    pub team_one_count: u32,
    pub team_one_two_v_one_count: u32,
    pub team_one_two_v_two_count: u32,
    pub team_one_two_v_three_count: u32,
    pub team_one_three_v_one_count: u32,
    pub team_one_three_v_two_count: u32,
    pub team_one_three_v_three_count: u32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_rush_counts: LabeledCounts,
}

impl RushStats {
    pub(crate) fn record(&mut self, event: &RushEvent) {
        self.labeled_rush_counts.increment(event.labels());
        self.sync_legacy_counts();
    }

    pub fn rush_count_with_labels(&self, labels: &[StatLabel]) -> u32 {
        self.labeled_rush_counts.count_matching(labels)
    }

    pub fn complete_labeled_rush_counts(&self) -> LabeledCounts {
        LabeledCounts::complete_from_label_sets(
            &[
                &RUSH_TEAM_LABELS,
                &RUSH_ATTACKER_LABELS,
                &RUSH_DEFENDER_LABELS,
            ],
            &self.labeled_rush_counts,
        )
    }

    pub fn with_complete_labeled_rush_counts(mut self) -> Self {
        self.labeled_rush_counts = self.complete_labeled_rush_counts();
        self
    }

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

    fn sync_legacy_counts(&mut self) {
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

    pub fn for_team(&self, is_team_zero: bool) -> RushTeamStats {
        if is_team_zero {
            RushTeamStats {
                count: self.team_zero_count,
                two_v_one_count: self.team_zero_two_v_one_count,
                two_v_two_count: self.team_zero_two_v_two_count,
                two_v_three_count: self.team_zero_two_v_three_count,
                three_v_one_count: self.team_zero_three_v_one_count,
                three_v_two_count: self.team_zero_three_v_two_count,
                three_v_three_count: self.team_zero_three_v_three_count,
            }
        } else {
            RushTeamStats {
                count: self.team_one_count,
                two_v_one_count: self.team_one_two_v_one_count,
                two_v_two_count: self.team_one_two_v_two_count,
                two_v_three_count: self.team_one_two_v_three_count,
                three_v_one_count: self.team_one_three_v_one_count,
                three_v_two_count: self.team_one_three_v_two_count,
                three_v_three_count: self.team_one_three_v_three_count,
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct RushTeamStats {
    pub count: u32,
    pub two_v_one_count: u32,
    pub two_v_two_count: u32,
    pub two_v_three_count: u32,
    pub three_v_one_count: u32,
    pub three_v_two_count: u32,
    pub three_v_three_count: u32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RushStatsAccumulator {
    stats: RushStats,
}

impl RushStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stats(&self) -> &RushStats {
        &self.stats
    }

    pub fn apply_event(&mut self, event: &RushEvent) {
        self.stats.record(event);
    }
}
