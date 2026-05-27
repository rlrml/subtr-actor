use super::*;

impl FiftyFiftyPlayerStats {
    pub(super) fn sync_legacy_counts(&mut self) {
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
}
