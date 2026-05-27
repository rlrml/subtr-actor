use super::*;

impl FiftyFiftyStats {
    pub(super) fn sync_legacy_counts(&mut self) {
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
}
