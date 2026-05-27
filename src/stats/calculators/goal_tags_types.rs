use super::super::*;
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GoalTagKind {
    AerialGoal,
    HighAerialGoal,
    LongDistanceGoal,
    OwnHalfGoal,
    EmptyNetGoal,
    CounterAttackGoal,
    FlickGoal,
    DoubleTapGoal,
    OneTimerGoal,
    PassingGoal,
    AirDribbleGoal,
    FlipResetGoal,
    HalfVolleyGoal,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GoalTagEvidenceKind {
    GoalContext,
    ScorerLastTouch,
    DefenderPosition,
    GoalBuildup,
    Flick,
    DoubleTap,
    OneTimer,
    Pass,
    AirDribble,
    FlipReset,
    HalfVolley,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GoalTagModifier {
    ByScorer,
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
    pub modifiers: Vec<GoalTagModifier>,
    pub evidence: Vec<GoalTagEvidence>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct GoalTaggingContext<'a> {
    pub(super) goal_index: usize,
    pub(super) goal: &'a GoalContextEvent,
}
