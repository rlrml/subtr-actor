use super::*;

const DEFAULT_AERIAL_GOAL_MIN_BALL_Z: f32 = 300.0;
const DEFAULT_HIGH_AERIAL_GOAL_MIN_BALL_Z: f32 = 700.0;
const DEFAULT_LONG_DISTANCE_GOAL_MAX_ATTACKING_Y: f32 = 1024.0;
const DEFAULT_OWN_HALF_GOAL_MAX_ATTACKING_Y: f32 = 0.0;
const DEFAULT_EMPTY_NET_MIN_DEFENDER_Y_MARGIN: f32 = 700.0;
const DEFAULT_EMPTY_NET_MIN_DEFENDER_DISTANCE: f32 = 1000.0;
const DEFAULT_EMPTY_NET_MAX_TOUCH_ATTACKING_Y: f32 = 3600.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GoalTagKind {
    AerialGoal,
    HighAerialGoal,
    LongDistanceGoal,
    OwnHalfGoal,
    EmptyNetGoal,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GoalTagEvidenceKind {
    GoalContext,
    ScorerLastTouch,
    DefenderPosition,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalTagEvidence {
    pub kind: GoalTagEvidenceKind,
    pub time: f32,
    pub frame: usize,
    #[ts(as = "Option<crate::ts_bindings::RemoteIdTs>")]
    pub player: Option<PlayerId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalTagEvent {
    pub goal_index: usize,
    pub time: f32,
    pub frame: usize,
    pub kind: GoalTagKind,
    pub scoring_team_is_team_0: bool,
    #[ts(as = "Option<crate::ts_bindings::RemoteIdTs>")]
    pub scorer: Option<PlayerId>,
    pub confidence: f32,
    pub evidence: Vec<GoalTagEvidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct AerialGoalCalculatorConfig {
    pub min_ball_z: f32,
}

impl Default for AerialGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            min_ball_z: DEFAULT_AERIAL_GOAL_MIN_BALL_Z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct HighAerialGoalCalculatorConfig {
    pub min_ball_z: f32,
}

impl Default for HighAerialGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            min_ball_z: DEFAULT_HIGH_AERIAL_GOAL_MIN_BALL_Z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct LongDistanceGoalCalculatorConfig {
    pub max_attacking_y: f32,
}

impl Default for LongDistanceGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_attacking_y: DEFAULT_LONG_DISTANCE_GOAL_MAX_ATTACKING_Y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct OwnHalfGoalCalculatorConfig {
    pub max_attacking_y: f32,
}

impl Default for OwnHalfGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_attacking_y: DEFAULT_OWN_HALF_GOAL_MAX_ATTACKING_Y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct EmptyNetGoalCalculatorConfig {
    pub min_defender_y_margin: f32,
    pub min_defender_distance: f32,
    pub max_touch_attacking_y: f32,
}

impl Default for EmptyNetGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            min_defender_y_margin: DEFAULT_EMPTY_NET_MIN_DEFENDER_Y_MARGIN,
            min_defender_distance: DEFAULT_EMPTY_NET_MIN_DEFENDER_DISTANCE,
            max_touch_attacking_y: DEFAULT_EMPTY_NET_MAX_TOUCH_ATTACKING_Y,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct GoalTaggingContext<'a> {
    goal_index: usize,
    goal: &'a GoalContextEvent,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AerialGoalCalculator {
    config: AerialGoalCalculatorConfig,
    events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HighAerialGoalCalculator {
    config: HighAerialGoalCalculatorConfig,
    events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LongDistanceGoalCalculator {
    config: LongDistanceGoalCalculatorConfig,
    events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OwnHalfGoalCalculator {
    config: OwnHalfGoalCalculatorConfig,
    events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EmptyNetGoalCalculator {
    config: EmptyNetGoalCalculatorConfig,
    events: Vec<GoalTagEvent>,
}

macro_rules! impl_goal_tag_calculator {
    ($calculator:ident, $config:ident) => {
        impl Default for $calculator {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $calculator {
            pub fn new() -> Self {
                Self::with_config($config::default())
            }

            pub fn with_config(config: $config) -> Self {
                Self {
                    config,
                    events: Vec::new(),
                }
            }

            pub fn config(&self) -> &$config {
                &self.config
            }

            pub fn events(&self) -> &[GoalTagEvent] {
                &self.events
            }
        }
    };
}

impl_goal_tag_calculator!(AerialGoalCalculator, AerialGoalCalculatorConfig);
impl_goal_tag_calculator!(HighAerialGoalCalculator, HighAerialGoalCalculatorConfig);
impl_goal_tag_calculator!(LongDistanceGoalCalculator, LongDistanceGoalCalculatorConfig);
impl_goal_tag_calculator!(OwnHalfGoalCalculator, OwnHalfGoalCalculatorConfig);
impl_goal_tag_calculator!(EmptyNetGoalCalculator, EmptyNetGoalCalculatorConfig);

impl AerialGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events = self.tag_goals(match_stats.goal_context_events());
        Ok(())
    }

    fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagEvent> {
        tag_goals_by_height(goals, GoalTagKind::AerialGoal, self.config.min_ball_z)
    }
}

impl HighAerialGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events = self.tag_goals(match_stats.goal_context_events());
        Ok(())
    }

    fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagEvent> {
        tag_goals_by_height(goals, GoalTagKind::HighAerialGoal, self.config.min_ball_z)
    }
}

impl LongDistanceGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events = self.tag_goals(match_stats.goal_context_events());
        Ok(())
    }

    fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagEvent> {
        tag_goals_by_attacking_y(
            goals,
            GoalTagKind::LongDistanceGoal,
            self.config.max_attacking_y,
        )
    }
}

impl OwnHalfGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events = self.tag_goals(match_stats.goal_context_events());
        Ok(())
    }

    fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagEvent> {
        tag_goals_by_attacking_y(goals, GoalTagKind::OwnHalfGoal, self.config.max_attacking_y)
    }
}

impl EmptyNetGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events = self.tag_goals(match_stats.goal_context_events());
        Ok(())
    }

    fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagEvent> {
        let mut tags = Vec::new();
        for (goal_index, goal) in goals.iter().enumerate() {
            let ctx = GoalTaggingContext { goal_index, goal };
            let Some(touch) = goal.scorer_last_touch.as_ref() else {
                continue;
            };
            let Some(ball_position) = touch.ball_position else {
                continue;
            };
            let ball = position_to_vec(ball_position);
            let touch_attacking_y = normalized_y(goal.scoring_team_is_team_0, ball);
            if touch_attacking_y > self.config.max_touch_attacking_y {
                continue;
            }

            let player_contexts = if touch.players.is_empty() {
                &goal.players
            } else {
                &touch.players
            };

            let defenders: Vec<_> = player_contexts
                .iter()
                .filter(|player| player.is_team_0 != goal.scoring_team_is_team_0)
                .filter_map(|player| {
                    player
                        .position
                        .map(|position| (player, position_to_vec(position)))
                })
                .collect();
            if defenders.is_empty() {
                continue;
            }

            let mut closest_defender_distance = f32::INFINITY;
            let mut smallest_y_margin = f32::INFINITY;
            let mut evidence = vec![last_touch_evidence(touch), goal_context_evidence(goal)];

            for (defender, position) in defenders {
                closest_defender_distance = closest_defender_distance.min(position.distance(ball));
                let defender_attacking_y = normalized_y(goal.scoring_team_is_team_0, position);
                let y_margin = touch_attacking_y - defender_attacking_y;
                smallest_y_margin = smallest_y_margin.min(y_margin);
                evidence.push(defender_evidence(defender, goal));
            }

            if smallest_y_margin < self.config.min_defender_y_margin {
                continue;
            }
            if closest_defender_distance < self.config.min_defender_distance {
                continue;
            }

            tags.push(goal_tag(ctx, GoalTagKind::EmptyNetGoal, 1.0, evidence));
        }
        tags
    }
}

fn tag_goals_by_height(
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

fn tag_goals_by_attacking_y(
    goals: &[GoalContextEvent],
    kind: GoalTagKind,
    max_attacking_y: f32,
) -> Vec<GoalTagEvent> {
    let mut tags = Vec::new();
    for (goal_index, goal) in goals.iter().enumerate() {
        let Some(touch) = goal.scorer_last_touch.as_ref() else {
            continue;
        };
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

fn position_to_vec(position: GoalContextPosition) -> glam::Vec3 {
    glam::Vec3::new(position.x, position.y, position.z)
}

fn goal_context_evidence(goal: &GoalContextEvent) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: GoalTagEvidenceKind::GoalContext,
        time: goal.time,
        frame: goal.frame,
        player: goal.scorer.clone(),
    }
}

fn last_touch_evidence(touch: &GoalTouchContext) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: GoalTagEvidenceKind::ScorerLastTouch,
        time: touch.time,
        frame: touch.frame,
        player: Some(touch.player.clone()),
    }
}

fn defender_evidence(player: &GoalPlayerContext, goal: &GoalContextEvent) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: GoalTagEvidenceKind::DefenderPosition,
        time: goal.time,
        frame: goal.frame,
        player: Some(player.player.clone()),
    }
}

fn goal_tag(
    ctx: GoalTaggingContext<'_>,
    kind: GoalTagKind,
    confidence: f32,
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
        evidence,
    }
}

pub fn combined_goal_tag_events(calculators: &[&[GoalTagEvent]]) -> Vec<GoalTagEvent> {
    let mut events: Vec<_> = calculators
        .iter()
        .flat_map(|events| events.iter().cloned())
        .collect();
    events.sort_by(|left, right| {
        left.time
            .total_cmp(&right.time)
            .then_with(|| left.frame.cmp(&right.frame))
            .then_with(|| left.goal_index.cmp(&right.goal_index))
            .then_with(|| format!("{:?}", left.kind).cmp(&format!("{:?}", right.kind)))
    });
    events
}

#[cfg(test)]
#[path = "goal_tags_tests.rs"]
mod tests;
