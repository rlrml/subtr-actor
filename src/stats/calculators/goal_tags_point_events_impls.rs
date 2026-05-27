use super::*;

macro_rules! impl_goal_mechanic_point_event {
    ($event:ty, $kind:expr, $confidence:expr) => {
        impl GoalMechanicPointEvent for $event {
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
                $confidence(self)
            }

            fn evidence_kind(&self) -> GoalTagEvidenceKind {
                $kind
            }
        }
    };
}

impl_goal_mechanic_point_event!(
    FlickEvent,
    GoalTagEvidenceKind::Flick,
    |event: &FlickEvent| event.confidence
);
impl_goal_mechanic_point_event!(OneTimerEvent, GoalTagEvidenceKind::OneTimer, |_| 1.0);
impl_goal_mechanic_point_event!(DoubleTapEvent, GoalTagEvidenceKind::DoubleTap, |_| 1.0);
impl_goal_mechanic_point_event!(
    ConfirmedFlipResetEvent,
    GoalTagEvidenceKind::FlipReset,
    |_| 1.0
);
