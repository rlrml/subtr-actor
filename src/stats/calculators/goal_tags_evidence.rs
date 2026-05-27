use super::*;

pub(super) fn goal_context_evidence(goal: &GoalContextEvent) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: GoalTagEvidenceKind::GoalContext,
        time: goal.time,
        frame: goal.frame,
        player: goal.scorer.clone(),
    }
}

pub(super) fn last_touch_evidence(touch: &GoalTouchContext) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: GoalTagEvidenceKind::ScorerLastTouch,
        time: touch.time,
        frame: touch.frame,
        player: Some(touch.player.clone()),
    }
}

pub(super) fn defender_evidence(
    player: &GoalPlayerContext,
    goal: &GoalContextEvent,
) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: GoalTagEvidenceKind::DefenderPosition,
        time: goal.time,
        frame: goal.frame,
        player: Some(player.player.clone()),
    }
}

pub(super) fn goal_buildup_evidence(goal: &GoalContextEvent) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: GoalTagEvidenceKind::GoalBuildup,
        time: goal.time,
        frame: goal.frame,
        player: goal.scorer.clone(),
    }
}

pub(super) fn point_mechanic_evidence(event: &impl GoalMechanicPointEvent) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: event.evidence_kind(),
        time: event.event_time(),
        frame: event.event_frame(),
        player: Some(event.event_player().clone()),
    }
}

pub(super) fn pass_evidence(event: &PassEvent) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: GoalTagEvidenceKind::Pass,
        time: event.time,
        frame: event.frame,
        player: Some(event.passer.clone()),
    }
}

pub(super) fn air_dribble_evidence(event: &BallCarryEvent) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: GoalTagEvidenceKind::AirDribble,
        time: event.end_time,
        frame: event.end_frame,
        player: Some(event.player_id.clone()),
    }
}

pub(super) fn half_volley_evidence(candidate: &HalfVolleyEvent) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: GoalTagEvidenceKind::HalfVolley,
        time: candidate.time,
        frame: candidate.frame,
        player: Some(candidate.player.clone()),
    }
}

pub(super) fn mechanic_goal_evidence(
    goal: &GoalContextEvent,
    mechanic_evidence: GoalTagEvidence,
) -> Vec<GoalTagEvidence> {
    let mut evidence = vec![mechanic_evidence, goal_context_evidence(goal)];
    if let Some(touch) = goal.scorer_last_touch.as_ref() {
        evidence.push(last_touch_evidence(touch));
    }
    evidence
}
