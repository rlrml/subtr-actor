use super::super::*;
use super::*;

pub(super) fn tag_goals_by_height(
    goals: &[GoalContextEvent],
    kind: GoalTagKind,
    min_ball_z: f32,
) -> Vec<GoalTagEvent> {
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
            GoalTaggingContext { goal_index, goal },
            kind,
            1.0,
            vec![last_touch_evidence(touch)],
        ));
    }
    tags
}

pub(super) fn tag_goals_by_attacking_y(
    goals: &[GoalContextEvent],
    kind: GoalTagKind,
    max_attacking_y: f32,
) -> Vec<GoalTagEvent> {
    tag_goals_by_recent_attacking_y(goals, kind, max_attacking_y, f32::INFINITY)
}

pub(super) fn tag_goals_by_recent_attacking_y(
    goals: &[GoalContextEvent],
    kind: GoalTagKind,
    max_attacking_y: f32,
    max_touch_to_goal_seconds: f32,
) -> Vec<GoalTagEvent> {
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
            GoalTaggingContext { goal_index, goal },
            kind,
            1.0,
            vec![last_touch_evidence(touch)],
        ));
    }
    tags
}

pub(super) fn position_to_vec(position: GoalContextPosition) -> glam::Vec3 {
    glam::Vec3::new(position.x, position.y, position.z)
}
