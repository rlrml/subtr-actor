use super::*;

/// Match-wide accumulated 50/50 win/loss/neutral tallies.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FiftyFiftyStats {
    pub count: u32,
    pub team_zero_wins: u32,
    pub team_one_wins: u32,
    pub neutral_outcomes: u32,
    pub kickoff_count: u32,
    pub kickoff_team_zero_wins: u32,
    pub kickoff_team_one_wins: u32,
    pub kickoff_neutral_outcomes: u32,
    pub team_zero_possession_after_count: u32,
    pub team_one_possession_after_count: u32,
    pub neutral_possession_after_count: u32,
    pub kickoff_team_zero_possession_after_count: u32,
    pub kickoff_team_one_possession_after_count: u32,
    pub kickoff_neutral_possession_after_count: u32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_event_counts: LabeledCounts,
}

/// Per-player accumulated 50/50 stats and post-contest possession.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FiftyFiftyPlayerStats {
    pub count: u32,
    pub wins: u32,
    pub losses: u32,
    pub neutral_outcomes: u32,
    pub kickoff_count: u32,
    pub kickoff_wins: u32,
    pub kickoff_losses: u32,
    pub kickoff_neutral_outcomes: u32,
    pub possession_after_count: u32,
    pub kickoff_possession_after_count: u32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_event_counts: LabeledCounts,
}

impl FiftyFiftyStats {
    pub(crate) fn record_event(&mut self, event: &FiftyFiftyEvent) {
        self.labeled_event_counts.increment(event.labels());
        self.sync_legacy_counts();
    }

    pub fn event_count_with_labels(&self, labels: &[StatLabel]) -> u32 {
        self.labeled_event_counts.count_matching(labels)
    }

    pub fn complete_labeled_event_counts(&self) -> LabeledCounts {
        LabeledCounts::complete_from_label_sets(
            &[
                &FIFTY_FIFTY_PHASE_LABELS,
                &FIFTY_FIFTY_TEAM_OUTCOME_LABELS,
                &FIFTY_FIFTY_POSSESSION_LABELS,
                &FIFTY_FIFTY_TEAM_ZERO_DODGE_STATE_LABELS,
                &FIFTY_FIFTY_TEAM_ONE_DODGE_STATE_LABELS,
            ],
            &self.labeled_event_counts,
        )
    }

    fn sync_legacy_counts(&mut self) {
        self.count = self.labeled_event_counts.total();
        self.team_zero_wins =
            self.event_count_with_labels(&[fifty_fifty_team_outcome_label(Some(true))]);
        self.team_one_wins =
            self.event_count_with_labels(&[fifty_fifty_team_outcome_label(Some(false))]);
        self.neutral_outcomes =
            self.event_count_with_labels(&[fifty_fifty_team_outcome_label(None)]);
        self.kickoff_count = self.event_count_with_labels(&[fifty_fifty_phase_label(true)]);
        self.kickoff_team_zero_wins = self.event_count_with_labels(&[
            fifty_fifty_phase_label(true),
            fifty_fifty_team_outcome_label(Some(true)),
        ]);
        self.kickoff_team_one_wins = self.event_count_with_labels(&[
            fifty_fifty_phase_label(true),
            fifty_fifty_team_outcome_label(Some(false)),
        ]);
        self.kickoff_neutral_outcomes = self.event_count_with_labels(&[
            fifty_fifty_phase_label(true),
            fifty_fifty_team_outcome_label(None),
        ]);
        self.team_zero_possession_after_count =
            self.event_count_with_labels(&[fifty_fifty_possession_label(Some(true))]);
        self.team_one_possession_after_count =
            self.event_count_with_labels(&[fifty_fifty_possession_label(Some(false))]);
        self.neutral_possession_after_count =
            self.event_count_with_labels(&[fifty_fifty_possession_label(None)]);
        self.kickoff_team_zero_possession_after_count = self.event_count_with_labels(&[
            fifty_fifty_phase_label(true),
            fifty_fifty_possession_label(Some(true)),
        ]);
        self.kickoff_team_one_possession_after_count = self.event_count_with_labels(&[
            fifty_fifty_phase_label(true),
            fifty_fifty_possession_label(Some(false)),
        ]);
        self.kickoff_neutral_possession_after_count = self.event_count_with_labels(&[
            fifty_fifty_phase_label(true),
            fifty_fifty_possession_label(None),
        ]);
    }

    pub fn team_zero_win_pct(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.team_zero_wins as f32 * 100.0 / self.count as f32
        }
    }

    pub fn team_one_win_pct(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.team_one_wins as f32 * 100.0 / self.count as f32
        }
    }

    pub fn kickoff_team_zero_win_pct(&self) -> f32 {
        if self.kickoff_count == 0 {
            0.0
        } else {
            self.kickoff_team_zero_wins as f32 * 100.0 / self.kickoff_count as f32
        }
    }

    pub fn kickoff_team_one_win_pct(&self) -> f32 {
        if self.kickoff_count == 0 {
            0.0
        } else {
            self.kickoff_team_one_wins as f32 * 100.0 / self.kickoff_count as f32
        }
    }
}

impl FiftyFiftyPlayerStats {
    pub(crate) fn record_event(&mut self, player_team_is_team_0: bool, event: &FiftyFiftyEvent) {
        self.labeled_event_counts
            .increment(event.player_labels(player_team_is_team_0));
        self.sync_legacy_counts();
    }

    pub fn event_count_with_labels(&self, labels: &[StatLabel]) -> u32 {
        self.labeled_event_counts.count_matching(labels)
    }

    pub fn complete_labeled_event_counts(&self) -> LabeledCounts {
        LabeledCounts::complete_from_label_sets(
            &[
                &FIFTY_FIFTY_PHASE_LABELS,
                &FIFTY_FIFTY_PLAYER_OUTCOME_LABELS,
                &FIFTY_FIFTY_PLAYER_POSSESSION_LABELS,
                &FIFTY_FIFTY_TOUCH_DODGE_STATE_LABELS,
            ],
            &self.labeled_event_counts,
        )
    }

    fn sync_legacy_counts(&mut self) {
        self.count = self.labeled_event_counts.total();
        self.wins = self.event_count_with_labels(&[StatLabel::new("outcome", "win")]);
        self.losses = self.event_count_with_labels(&[StatLabel::new("outcome", "loss")]);
        self.neutral_outcomes =
            self.event_count_with_labels(&[StatLabel::new("outcome", "neutral")]);
        self.kickoff_count = self.event_count_with_labels(&[fifty_fifty_phase_label(true)]);
        self.kickoff_wins = self.event_count_with_labels(&[
            fifty_fifty_phase_label(true),
            StatLabel::new("outcome", "win"),
        ]);
        self.kickoff_losses = self.event_count_with_labels(&[
            fifty_fifty_phase_label(true),
            StatLabel::new("outcome", "loss"),
        ]);
        self.kickoff_neutral_outcomes = self.event_count_with_labels(&[
            fifty_fifty_phase_label(true),
            StatLabel::new("outcome", "neutral"),
        ]);
        self.possession_after_count =
            self.event_count_with_labels(&[StatLabel::new("possession_after", "self")]);
        self.kickoff_possession_after_count = self.event_count_with_labels(&[
            fifty_fifty_phase_label(true),
            StatLabel::new("possession_after", "self"),
        ]);
    }

    pub fn win_pct(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.wins as f32 * 100.0 / self.count as f32
        }
    }

    pub fn kickoff_win_pct(&self) -> f32 {
        if self.kickoff_count == 0 {
            0.0
        } else {
            self.kickoff_wins as f32 * 100.0 / self.kickoff_count as f32
        }
    }
}

/// Per-team accumulated 50/50 stats.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FiftyFiftyTeamStats {
    pub count: u32,
    pub wins: u32,
    pub losses: u32,
    pub neutral_outcomes: u32,
    pub kickoff_count: u32,
    pub kickoff_wins: u32,
    pub kickoff_losses: u32,
    pub kickoff_neutral_outcomes: u32,
    pub possession_after_count: u32,
    pub opponent_possession_after_count: u32,
    pub neutral_possession_after_count: u32,
    pub kickoff_possession_after_count: u32,
    pub kickoff_opponent_possession_after_count: u32,
    pub kickoff_neutral_possession_after_count: u32,
}

impl FiftyFiftyStats {
    pub fn for_team(&self, is_team_zero: bool) -> FiftyFiftyTeamStats {
        let (
            wins,
            losses,
            kickoff_wins,
            kickoff_losses,
            possession_after_count,
            opponent_possession_after_count,
            kickoff_possession_after_count,
            kickoff_opponent_possession_after_count,
        ) = if is_team_zero {
            (
                self.team_zero_wins,
                self.team_one_wins,
                self.kickoff_team_zero_wins,
                self.kickoff_team_one_wins,
                self.team_zero_possession_after_count,
                self.team_one_possession_after_count,
                self.kickoff_team_zero_possession_after_count,
                self.kickoff_team_one_possession_after_count,
            )
        } else {
            (
                self.team_one_wins,
                self.team_zero_wins,
                self.kickoff_team_one_wins,
                self.kickoff_team_zero_wins,
                self.team_one_possession_after_count,
                self.team_zero_possession_after_count,
                self.kickoff_team_one_possession_after_count,
                self.kickoff_team_zero_possession_after_count,
            )
        };

        FiftyFiftyTeamStats {
            count: self.count,
            wins,
            losses,
            neutral_outcomes: self.neutral_outcomes,
            kickoff_count: self.kickoff_count,
            kickoff_wins,
            kickoff_losses,
            kickoff_neutral_outcomes: self.kickoff_neutral_outcomes,
            possession_after_count,
            opponent_possession_after_count,
            neutral_possession_after_count: self.neutral_possession_after_count,
            kickoff_possession_after_count,
            kickoff_opponent_possession_after_count,
            kickoff_neutral_possession_after_count: self.kickoff_neutral_possession_after_count,
        }
    }
}

/// Accumulates 50/50 stats over the replay from 50/50 events.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FiftyFiftyStatsAccumulator {
    stats: FiftyFiftyStats,
    player_stats: HashMap<PlayerId, FiftyFiftyPlayerStats>,
}

impl FiftyFiftyStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stats(&self) -> &FiftyFiftyStats {
        &self.stats
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, FiftyFiftyPlayerStats> {
        &self.player_stats
    }

    pub fn apply_event(&mut self, event: &FiftyFiftyEvent) {
        self.stats.record_event(event);

        if let Some(player_id) = event.team_zero_player.as_ref() {
            let stats = self.player_stats.entry(player_id.clone()).or_default();
            stats.record_event(true, event);
        }
        if let Some(player_id) = event.team_one_player.as_ref() {
            let stats = self.player_stats.entry(player_id.clone()).or_default();
            stats.record_event(false, event);
        }
    }
}
