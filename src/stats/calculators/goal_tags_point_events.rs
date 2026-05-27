use super::*;

pub(super) trait GoalMechanicPointEvent {
    fn event_time(&self) -> f32;
    fn event_frame(&self) -> usize;
    fn event_player(&self) -> &PlayerId;
    fn event_team_is_team_0(&self) -> bool;
    fn event_confidence(&self) -> f32;
    fn evidence_kind(&self) -> GoalTagEvidenceKind;
}

impl GoalMechanicPointEvent for FlickEvent {
    fn event_time(&self) -> f32 {
        self.time
    }

    fn event_frame(&self) -> usize {
        self.frame
    }

    fn event_player(&self) -> &PlayerId {
        &self.player
    }

    fn event_team_is_team_0(&self) -> bool {
        self.is_team_0
    }

    fn event_confidence(&self) -> f32 {
        self.confidence
    }

    fn evidence_kind(&self) -> GoalTagEvidenceKind {
        GoalTagEvidenceKind::Flick
    }
}

impl GoalMechanicPointEvent for OneTimerEvent {
    fn event_time(&self) -> f32 {
        self.time
    }

    fn event_frame(&self) -> usize {
        self.frame
    }

    fn event_player(&self) -> &PlayerId {
        &self.player
    }

    fn event_team_is_team_0(&self) -> bool {
        self.is_team_0
    }

    fn event_confidence(&self) -> f32 {
        1.0
    }

    fn evidence_kind(&self) -> GoalTagEvidenceKind {
        GoalTagEvidenceKind::OneTimer
    }
}

impl GoalMechanicPointEvent for DoubleTapEvent {
    fn event_time(&self) -> f32 {
        self.time
    }

    fn event_frame(&self) -> usize {
        self.frame
    }

    fn event_player(&self) -> &PlayerId {
        &self.player
    }

    fn event_team_is_team_0(&self) -> bool {
        self.is_team_0
    }

    fn event_confidence(&self) -> f32 {
        1.0
    }

    fn evidence_kind(&self) -> GoalTagEvidenceKind {
        GoalTagEvidenceKind::DoubleTap
    }
}

impl GoalMechanicPointEvent for ConfirmedFlipResetEvent {
    fn event_time(&self) -> f32 {
        self.time
    }

    fn event_frame(&self) -> usize {
        self.frame
    }

    fn event_player(&self) -> &PlayerId {
        &self.player
    }

    fn event_team_is_team_0(&self) -> bool {
        self.is_team_0
    }

    fn event_confidence(&self) -> f32 {
        1.0
    }

    fn evidence_kind(&self) -> GoalTagEvidenceKind {
        GoalTagEvidenceKind::FlipReset
    }
}
