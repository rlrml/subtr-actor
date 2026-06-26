use super::*;

const MAX_EVENT_AFTER_GOAL_SECONDS: f32 = 0.05;
const GOAL_KICKOFF_BOUNDARY_EPSILON_SECONDS: f32 = 0.05;

pub(super) fn tag_goals_by_height(
    goals: &[GoalContextEvent],
    kind: GoalTagKind,
    min_ball_z: f32,
) -> Vec<GoalTagAssignment> {
    let mut tags = Vec::new();
    for (goal_index, goal) in goals.iter().enumerate() {
        let Some(touch) = goal.scorer_last_touch.as_ref() else {
            continue;
        };
        let Some(ball_position) = touch.ball_position else {
            continue;
        };
        if ball_position.z < min_ball_z {
            continue;
        }
        tags.push(goal_tag(
            GoalTaggingContext { goal_index },
            kind,
            1.0,
            vec![last_touch_evidence(touch)],
        ));
    }
    tags
}

pub(super) fn tag_goals_by_possession_touch_height(
    goals: &[GoalContextEvent],
    touch_events: &[TouchClassificationEvent],
    possession_events: &[PossessionEvent],
    kind: GoalTagKind,
    min_ball_z: f32,
) -> Vec<GoalTagAssignment> {
    let mut tags = Vec::new();
    for (goal_index, goal) in goals.iter().enumerate() {
        let possession_start_frame = scoring_possession_start_frame(goal, possession_events);
        let Some((event_index, touch)) =
            highest_possession_touch(goal, touch_events, possession_start_frame, min_ball_z)
        else {
            continue;
        };
        tags.push(mechanic_goal_tag(
            GoalTaggingContext { goal_index },
            kind,
            1.0,
            mechanic_goal_performer(goal, &touch.player),
            mechanic_goal_modifiers(goal, &touch.player),
            mechanic_goal_evidence(goal, leadup_touch_evidence(touch)),
            vec![GoalTagEventRef {
                stream: GoalTagEventStream::Touch,
                index: event_index,
            }],
            Vec::new(),
        ));
    }
    tags
}

/// First frame of the scoring team's possession that led to the goal, anchored
/// at the scorer's final touch when available. Walking back stops at the
/// previous neutral/opponent possession, so the window never reaches across a
/// turnover, kickoff, or prior goal. If the scoring team has no possession span
/// at the scoring touch, fall back to that touch so stale earlier possessions
/// stay out of scope.
fn scoring_possession_start_frame(
    goal: &GoalContextEvent,
    possession_events: &[PossessionEvent],
) -> usize {
    let anchor_frame = goal
        .scorer_last_touch
        .as_ref()
        .map(|touch| touch.frame)
        .unwrap_or(goal.frame);
    let scoring_state = if goal.scoring_team_is_team_0 {
        "team_zero"
    } else {
        "team_one"
    };
    let is_scoring_possession = |event: &PossessionEvent| event.possession_state == scoring_state;

    let anchored_scoring_index = possession_events
        .iter()
        .enumerate()
        .filter(|(_, event)| {
            is_scoring_possession(event)
                && event.frame <= anchor_frame
                && event.end_frame >= anchor_frame
        })
        .map(|(index, _)| index)
        .max();

    match anchored_scoring_index {
        Some(last_index) => {
            let mut start_index = last_index;
            // Walk back through directly-adjacent scoring-team spans. Possession
            // events are now sparse (neutral/loose stretches are gaps, not
            // events), so adjacency in the array no longer implies adjacency in
            // time: stop when a frame gap separates two same-team spans, since
            // that gap was a neutral loss the window must not reach across.
            while start_index > 0
                && is_scoring_possession(&possession_events[start_index - 1])
                && possession_events[start_index - 1].end_frame
                    >= possession_events[start_index].frame
            {
                start_index -= 1;
            }
            possession_events[start_index].frame
        }
        None => anchor_frame,
    }
}

/// The scoring-team touch within the goal's possession (and at or before the
/// goal) with the greatest ball height, provided that height meets `min_ball_z`.
/// Returns the touch's index in `touch_events` alongside the touch.
fn highest_possession_touch<'a>(
    goal: &GoalContextEvent,
    touch_events: &'a [TouchClassificationEvent],
    possession_start_frame: usize,
    min_ball_z: f32,
) -> Option<(usize, &'a TouchClassificationEvent)> {
    touch_events
        .iter()
        .enumerate()
        .filter(|(_, touch)| possession_touch_matches_goal(touch, goal, possession_start_frame))
        .filter_map(|(index, touch)| {
            touch
                .ball_position
                .filter(|ball_position| ball_position[2] >= min_ball_z)
                .map(|ball_position| (index, touch, ball_position[2]))
        })
        .max_by(|left, right| {
            left.2
                .total_cmp(&right.2)
                .then_with(|| left.1.frame.cmp(&right.1.frame))
        })
        .map(|(index, touch, _)| (index, touch))
}

fn possession_touch_matches_goal(
    touch: &TouchClassificationEvent,
    goal: &GoalContextEvent,
    possession_start_frame: usize,
) -> bool {
    touch.is_team_0 == goal.scoring_team_is_team_0
        && touch.frame >= possession_start_frame
        && touch.frame <= goal.frame
}

pub(super) fn tag_goals_by_attacking_y(
    goals: &[GoalContextEvent],
    kind: GoalTagKind,
    max_attacking_y: f32,
) -> Vec<GoalTagAssignment> {
    tag_goals_by_recent_attacking_y(goals, kind, max_attacking_y, f32::INFINITY)
}

pub(super) fn tag_goals_by_recent_attacking_y(
    goals: &[GoalContextEvent],
    kind: GoalTagKind,
    max_attacking_y: f32,
    max_touch_to_goal_seconds: f32,
) -> Vec<GoalTagAssignment> {
    let mut tags = Vec::new();
    for (goal_index, goal) in goals.iter().enumerate() {
        let Some(touch) = goal.scorer_last_touch.as_ref() else {
            continue;
        };
        if goal.time - touch.time > max_touch_to_goal_seconds {
            continue;
        }
        let Some(ball_position) = touch.ball_position else {
            continue;
        };
        let attacking_y = normalized_y(goal.scoring_team_is_team_0, position_to_vec(ball_position));
        if attacking_y > max_attacking_y {
            continue;
        }
        tags.push(goal_tag(
            GoalTaggingContext { goal_index },
            kind,
            1.0,
            vec![last_touch_evidence(touch)],
        ));
    }
    tags
}

pub(super) fn tag_goals_by_point_mechanic_event<E: GoalMechanicPointEvent>(
    goals: &[GoalContextEvent],
    events: &[E],
    kind: GoalTagKind,
    max_event_to_goal_seconds: f32,
) -> Vec<GoalTagAssignment> {
    let mut tags = Vec::new();
    for (goal_index, goal) in goals.iter().enumerate() {
        let ctx = GoalTaggingContext { goal_index };
        let Some((event_index, event)) = events
            .iter()
            .enumerate()
            .filter(|(_, event)| point_event_matches_goal(*event, goal))
            .filter(|(_, event)| goal.time - event.event_time() <= max_event_to_goal_seconds)
            .max_by(|left, right| {
                left.1
                    .event_time()
                    .total_cmp(&right.1.event_time())
                    .then_with(|| left.1.event_frame().cmp(&right.1.event_frame()))
            })
        else {
            continue;
        };

        tags.push(mechanic_goal_tag(
            ctx,
            kind,
            event.event_confidence(),
            mechanic_goal_performer(goal, event.event_player()),
            mechanic_goal_modifiers(goal, event.event_player()),
            mechanic_goal_evidence(goal, point_mechanic_evidence(event)),
            vec![event.event_ref(event_index)],
            event.goal_tag_details(),
        ));
    }
    tags
}

pub(super) fn point_event_matches_goal<E: GoalMechanicPointEvent>(
    event: &E,
    goal: &GoalContextEvent,
) -> bool {
    event.event_team_is_team_0() == goal.scoring_team_is_team_0
        && event_time_is_in_goal_play(event.event_time(), goal)
        && event.event_frame() <= goal.frame
}

pub(super) fn pass_event_matches_goal(event: &PassEvent, goal: &GoalContextEvent) -> bool {
    event.is_team_0 == goal.scoring_team_is_team_0
        && event_time_is_in_goal_play(event.time, goal)
        && event.frame <= goal.frame
        && goal.scorer.as_ref() == Some(&event.receiver)
        && goal
            .scorer_last_touch
            .as_ref()
            .is_some_and(|touch| touch.player == event.receiver && touch.frame == event.frame)
}

pub(super) fn tag_goals_by_air_dribble_event(
    goals: &[GoalContextEvent],
    events: &[BallCarryEvent],
    max_end_to_goal_seconds: f32,
) -> Vec<GoalTagAssignment> {
    let mut tags = Vec::new();
    for (goal_index, goal) in goals.iter().enumerate() {
        let ctx = GoalTaggingContext { goal_index };
        let Some((event_index, event)) = events
            .iter()
            .enumerate()
            .filter(|(_, event)| air_dribble_event_matches_goal(event, goal))
            .filter(|(_, event)| goal.time - event.end_time <= max_end_to_goal_seconds)
            .max_by(|left, right| {
                left.1
                    .end_time
                    .total_cmp(&right.1.end_time)
                    .then_with(|| left.1.end_frame.cmp(&right.1.end_frame))
            })
        else {
            continue;
        };

        tags.push(mechanic_goal_tag(
            ctx,
            GoalTagKind::AirDribbleGoal,
            1.0,
            mechanic_goal_performer(goal, &event.player_id),
            mechanic_goal_modifiers(goal, &event.player_id),
            mechanic_goal_evidence(goal, air_dribble_evidence(event)),
            vec![GoalTagEventRef {
                stream: GoalTagEventStream::BallCarry,
                index: event_index,
            }],
            Vec::new(),
        ));
    }
    tags
}

pub(super) fn air_dribble_event_matches_goal(
    event: &BallCarryEvent,
    goal: &GoalContextEvent,
) -> bool {
    event.kind == BallCarryKind::AirDribble
        && event.is_team_0 == goal.scoring_team_is_team_0
        && event_time_is_in_goal_play(event.start_time, goal)
        && event_time_is_in_goal_play(event.end_time, goal)
        && event.end_frame <= goal.frame
}

pub(super) fn bump_event_matches_goal(event: &BumpEvent, goal: &GoalContextEvent) -> bool {
    !event.is_team_bump
        && event.initiator_is_team_0 == goal.scoring_team_is_team_0
        && event_time_is_in_goal_play(event.time, goal)
        && event.frame <= goal.frame
}

pub(super) fn demo_event_matches_goal(event: &DemolitionEvent, goal: &GoalContextEvent) -> bool {
    // Replay-authored demolition events can land just after the goal context
    // frame even when the demo belongs to the scoring play.
    event.attacker_is_team_0 == Some(goal.scoring_team_is_team_0)
        && event_time_is_in_goal_play(event.time, goal)
}

fn event_time_is_in_goal_play(event_time: f32, goal: &GoalContextEvent) -> bool {
    if event_time > goal.time + MAX_EVENT_AFTER_GOAL_SECONDS {
        return false;
    }

    if event_time <= goal.time {
        if let Some(time_after_kickoff) = goal.time_after_kickoff {
            let event_to_goal_seconds = goal.time - event_time;
            if event_to_goal_seconds > time_after_kickoff + GOAL_KICKOFF_BOUNDARY_EPSILON_SECONDS {
                return false;
            }
        }
    }

    true
}

pub(super) fn position_to_vec(position: GoalContextPosition) -> glam::Vec3 {
    glam::Vec3::new(position.x, position.y, position.z)
}

pub(super) fn goal_context_evidence(goal: &GoalContextEvent) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: GoalTagEvidenceKind::GoalContext,
        time: goal.time,
        frame: goal.frame,
        player: goal.scorer.clone(),
        player_position: goal.scorer.as_ref().and_then(|scorer| {
            goal.players
                .iter()
                .find(|player| &player.player == scorer)
                .and_then(|player| player.position)
        }),
    }
}

pub(super) fn last_touch_evidence(touch: &GoalTouchContext) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: GoalTagEvidenceKind::ScorerLastTouch,
        time: touch.time,
        frame: touch.frame,
        player: Some(touch.player.clone()),
        player_position: None,
    }
}

pub(super) fn leadup_touch_evidence(touch: &TouchClassificationEvent) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: GoalTagEvidenceKind::LeadupTouch,
        time: touch.time,
        frame: touch.frame,
        player: Some(touch.player.clone()),
        player_position: touch
            .ball_position
            .map(|ball_position| GoalContextPosition {
                x: ball_position[0],
                y: ball_position[1],
                z: ball_position[2],
            }),
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
        player_position: player.position,
    }
}

pub(super) fn goal_buildup_evidence(goal: &GoalContextEvent) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: GoalTagEvidenceKind::GoalBuildup,
        time: goal.time,
        frame: goal.frame,
        player: goal.scorer.clone(),
        player_position: goal.scorer.as_ref().and_then(|scorer| {
            goal.players
                .iter()
                .find(|player| &player.player == scorer)
                .and_then(|player| player.position)
        }),
    }
}

pub(super) fn point_mechanic_evidence(event: &impl GoalMechanicPointEvent) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: event.evidence_kind(),
        time: event.event_time(),
        frame: event.event_frame(),
        player: Some(event.event_player().clone()),
        player_position: None,
    }
}

pub(super) fn pass_evidence(event: &PassEvent) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: GoalTagEvidenceKind::Pass,
        time: event.time,
        frame: event.frame,
        player: Some(event.passer.clone()),
        player_position: event.passer_position.map(|position| GoalContextPosition {
            x: position[0],
            y: position[1],
            z: position[2],
        }),
    }
}

pub(super) fn air_dribble_evidence(event: &BallCarryEvent) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: GoalTagEvidenceKind::AirDribble,
        time: event.end_time,
        frame: event.end_frame,
        player: Some(event.player_id.clone()),
        player_position: Some(GoalContextPosition {
            x: event.end_position[0],
            y: event.end_position[1],
            z: event.end_position[2],
        }),
    }
}

pub(super) fn flip_into_ball_evidence(event: &TouchClassificationEvent) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: GoalTagEvidenceKind::FlipIntoBall,
        time: event.time,
        frame: event.frame,
        player: Some(event.player.clone()),
        player_position: event.player_position.map(|position| GoalContextPosition {
            x: position[0],
            y: position[1],
            z: position[2],
        }),
    }
}

pub(super) fn bump_evidence(event: &BumpEvent) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: GoalTagEvidenceKind::Bump,
        time: event.time,
        frame: event.frame,
        player: Some(event.initiator.clone()),
        player_position: Some(GoalContextPosition {
            x: event.initiator_position[0],
            y: event.initiator_position[1],
            z: event.initiator_position[2],
        }),
    }
}

pub(super) fn demo_evidence(event: &DemolitionEvent) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: GoalTagEvidenceKind::Demo,
        time: event.time,
        frame: event.frame,
        player: Some(event.attacker.clone()),
        player_position: event.attacker_position.map(|position| GoalContextPosition {
            x: position[0],
            y: position[1],
            z: position[2],
        }),
    }
}

pub(super) fn half_volley_evidence(candidate: &HalfVolleyEvent) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: GoalTagEvidenceKind::HalfVolley,
        time: candidate.time,
        frame: candidate.frame,
        player: Some(candidate.player.clone()),
        player_position: candidate
            .player_position
            .map(|position| GoalContextPosition {
                x: position[0],
                y: position[1],
                z: position[2],
            }),
    }
}

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

pub(super) fn mechanic_goal_performer(
    goal: &GoalContextEvent,
    mechanic_player: &PlayerId,
) -> GoalTagPerformer {
    if goal
        .scorer
        .as_ref()
        .is_some_and(|scorer| scorer == mechanic_player)
    {
        GoalTagPerformer::Scorer
    } else {
        GoalTagPerformer::Teammate
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

pub(super) fn goal_tag(
    ctx: GoalTaggingContext,
    kind: GoalTagKind,
    confidence: f32,
    evidence: Vec<GoalTagEvidence>,
) -> GoalTagAssignment {
    goal_tag_with_modifiers(ctx, kind, confidence, Vec::new(), evidence, Vec::new())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn mechanic_goal_tag(
    ctx: GoalTaggingContext,
    kind: GoalTagKind,
    confidence: f32,
    performer: GoalTagPerformer,
    modifiers: Vec<GoalTagModifier>,
    evidence: Vec<GoalTagEvidence>,
    related_events: Vec<GoalTagEventRef>,
    details: Vec<GoalTagDetail>,
) -> GoalTagAssignment {
    goal_tag_with_metadata(
        ctx,
        kind,
        GoalTagMetadata {
            confidence,
            performer: Some(performer),
            modifiers,
            related_events,
            details,
            evidence,
        },
    )
}

pub(super) fn goal_tag_with_modifiers(
    ctx: GoalTaggingContext,
    kind: GoalTagKind,
    confidence: f32,
    modifiers: Vec<GoalTagModifier>,
    evidence: Vec<GoalTagEvidence>,
    related_events: Vec<GoalTagEventRef>,
) -> GoalTagAssignment {
    goal_tag_with_metadata(
        ctx,
        kind,
        GoalTagMetadata {
            confidence,
            performer: None,
            modifiers,
            related_events,
            details: Vec::new(),
            evidence,
        },
    )
}

pub(super) fn goal_tag_with_metadata(
    ctx: GoalTaggingContext,
    kind: GoalTagKind,
    metadata: GoalTagMetadata,
) -> GoalTagAssignment {
    GoalTagAssignment {
        goal_index: ctx.goal_index,
        tag: GoalTag::from_parts(kind, metadata),
    }
}
