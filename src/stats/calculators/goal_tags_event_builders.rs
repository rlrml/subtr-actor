use super::*;

pub(super) fn mechanic_goal_modifiers(
    goal: &GoalContextEvent,
    mechanic_player: &PlayerId,
) -> Vec<GoalTagModifier> {
    if goal
        .scorer
        .as_ref()
        .is_some_and(|scorer| scorer == mechanic_player)
    {
        vec![GoalTagModifier::ByScorer]
    } else {
        Vec::new()
    }
}

pub(super) fn goal_tag(
    ctx: GoalTaggingContext<'_>,
    kind: GoalTagKind,
    confidence: f32,
    evidence: Vec<GoalTagEvidence>,
) -> GoalTagEvent {
    goal_tag_with_modifiers(ctx, kind, confidence, Vec::new(), evidence)
}

pub(super) fn goal_tag_with_modifiers(
    ctx: GoalTaggingContext<'_>,
    kind: GoalTagKind,
    confidence: f32,
    modifiers: Vec<GoalTagModifier>,
    evidence: Vec<GoalTagEvidence>,
) -> GoalTagEvent {
    GoalTagEvent {
        goal_index: ctx.goal_index,
        time: ctx.goal.time,
        frame: ctx.goal.frame,
        kind,
        scoring_team_is_team_0: ctx.goal.scoring_team_is_team_0,
        scorer: ctx.goal.scorer.clone(),
        confidence,
        modifiers,
        evidence,
    }
}
