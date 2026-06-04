use super::*;

#[path = "goal_tag_helpers.rs"]
mod goal_tag_helpers;
use goal_tag_helpers::*;

const DEFAULT_AERIAL_GOAL_MIN_BALL_Z: f32 = 600.0;
const DEFAULT_HIGH_AERIAL_GOAL_MIN_BALL_Z: f32 = 700.0;
const DEFAULT_LONG_DISTANCE_GOAL_MAX_ATTACKING_Y: f32 = 1024.0;
const DEFAULT_OWN_HALF_GOAL_MAX_ATTACKING_Y: f32 = 0.0;
// Avoid labeling long delayed clears as own-half goals solely from replay goal credit.
const OWN_HALF_GOAL_MAX_TOUCH_TO_GOAL_SECONDS: f32 = 8.0;
const DEFAULT_EMPTY_NET_MIN_DEFENDER_Y_MARGIN: f32 = 700.0;
const DEFAULT_EMPTY_NET_MIN_DEFENDER_DISTANCE: f32 = 1000.0;
const DEFAULT_EMPTY_NET_MAX_TOUCH_ATTACKING_Y: f32 = 3600.0;
const DEFAULT_FLICK_GOAL_MAX_EVENT_TO_GOAL_SECONDS: f32 = 3.0;
const DEFAULT_DOUBLE_TAP_GOAL_MAX_EVENT_TO_GOAL_SECONDS: f32 = 3.0;
const DEFAULT_ONE_TIMER_GOAL_MAX_EVENT_TO_GOAL_SECONDS: f32 = 3.0;
const DEFAULT_PASSING_GOAL_MAX_PASS_TO_GOAL_SECONDS: f32 = 3.0;
const DEFAULT_AIR_DRIBBLE_GOAL_MAX_END_TO_GOAL_SECONDS: f32 = 3.0;
const DEFAULT_FLIP_RESET_GOAL_MAX_EVENT_TO_GOAL_SECONDS: f32 = 8.0;
const DEFAULT_HALF_VOLLEY_GOAL_MAX_TOUCH_TO_GOAL_SECONDS: f32 = 3.0;
const DEFAULT_HALF_VOLLEY_GOAL_MIN_GOAL_ALIGNMENT: f32 = 0.55;

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
pub enum GoalTagEventStream {
    Flick,
    DoubleTap,
    OneTimer,
    Pass,
    BallCarry,
    DodgeReset,
    HalfVolley,
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
    #[ts(as = "Option<crate::interop::ts_bindings::RemoteIdTs>")]
    pub player: Option<PlayerId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<GoalContextPosition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalTagEventRef {
    pub stream: GoalTagEventStream,
    pub index: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalTagMetadata {
    pub confidence: f32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<GoalTagModifier>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_events: Vec<GoalTagEventRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<GoalTagEvidence>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[serde(tag = "kind", content = "metadata", rename_all = "snake_case")]
#[ts(export)]
pub enum GoalTag {
    AerialGoal(GoalTagMetadata),
    HighAerialGoal(GoalTagMetadata),
    LongDistanceGoal(GoalTagMetadata),
    OwnHalfGoal(GoalTagMetadata),
    EmptyNetGoal(GoalTagMetadata),
    CounterAttackGoal(GoalTagMetadata),
    FlickGoal(GoalTagMetadata),
    DoubleTapGoal(GoalTagMetadata),
    OneTimerGoal(GoalTagMetadata),
    PassingGoal(GoalTagMetadata),
    AirDribbleGoal(GoalTagMetadata),
    FlipResetGoal(GoalTagMetadata),
    HalfVolleyGoal(GoalTagMetadata),
}

impl GoalTag {
    pub fn from_parts(kind: GoalTagKind, metadata: GoalTagMetadata) -> Self {
        match kind {
            GoalTagKind::AerialGoal => Self::AerialGoal(metadata),
            GoalTagKind::HighAerialGoal => Self::HighAerialGoal(metadata),
            GoalTagKind::LongDistanceGoal => Self::LongDistanceGoal(metadata),
            GoalTagKind::OwnHalfGoal => Self::OwnHalfGoal(metadata),
            GoalTagKind::EmptyNetGoal => Self::EmptyNetGoal(metadata),
            GoalTagKind::CounterAttackGoal => Self::CounterAttackGoal(metadata),
            GoalTagKind::FlickGoal => Self::FlickGoal(metadata),
            GoalTagKind::DoubleTapGoal => Self::DoubleTapGoal(metadata),
            GoalTagKind::OneTimerGoal => Self::OneTimerGoal(metadata),
            GoalTagKind::PassingGoal => Self::PassingGoal(metadata),
            GoalTagKind::AirDribbleGoal => Self::AirDribbleGoal(metadata),
            GoalTagKind::FlipResetGoal => Self::FlipResetGoal(metadata),
            GoalTagKind::HalfVolleyGoal => Self::HalfVolleyGoal(metadata),
        }
    }

    pub fn kind(&self) -> GoalTagKind {
        match self {
            Self::AerialGoal(_) => GoalTagKind::AerialGoal,
            Self::HighAerialGoal(_) => GoalTagKind::HighAerialGoal,
            Self::LongDistanceGoal(_) => GoalTagKind::LongDistanceGoal,
            Self::OwnHalfGoal(_) => GoalTagKind::OwnHalfGoal,
            Self::EmptyNetGoal(_) => GoalTagKind::EmptyNetGoal,
            Self::CounterAttackGoal(_) => GoalTagKind::CounterAttackGoal,
            Self::FlickGoal(_) => GoalTagKind::FlickGoal,
            Self::DoubleTapGoal(_) => GoalTagKind::DoubleTapGoal,
            Self::OneTimerGoal(_) => GoalTagKind::OneTimerGoal,
            Self::PassingGoal(_) => GoalTagKind::PassingGoal,
            Self::AirDribbleGoal(_) => GoalTagKind::AirDribbleGoal,
            Self::FlipResetGoal(_) => GoalTagKind::FlipResetGoal,
            Self::HalfVolleyGoal(_) => GoalTagKind::HalfVolleyGoal,
        }
    }

    pub fn metadata(&self) -> &GoalTagMetadata {
        match self {
            Self::AerialGoal(metadata)
            | Self::HighAerialGoal(metadata)
            | Self::LongDistanceGoal(metadata)
            | Self::OwnHalfGoal(metadata)
            | Self::EmptyNetGoal(metadata)
            | Self::CounterAttackGoal(metadata)
            | Self::FlickGoal(metadata)
            | Self::DoubleTapGoal(metadata)
            | Self::OneTimerGoal(metadata)
            | Self::PassingGoal(metadata)
            | Self::AirDribbleGoal(metadata)
            | Self::FlipResetGoal(metadata)
            | Self::HalfVolleyGoal(metadata) => metadata,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GoalTagAssignment {
    pub goal_index: usize,
    pub tag: GoalTag,
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FlickGoalCalculatorConfig {
    pub max_event_to_goal_seconds: f32,
}

impl Default for FlickGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_event_to_goal_seconds: DEFAULT_FLICK_GOAL_MAX_EVENT_TO_GOAL_SECONDS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct DoubleTapGoalCalculatorConfig {
    pub max_event_to_goal_seconds: f32,
}

impl Default for DoubleTapGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_event_to_goal_seconds: DEFAULT_DOUBLE_TAP_GOAL_MAX_EVENT_TO_GOAL_SECONDS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct OneTimerGoalCalculatorConfig {
    pub max_event_to_goal_seconds: f32,
}

impl Default for OneTimerGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_event_to_goal_seconds: DEFAULT_ONE_TIMER_GOAL_MAX_EVENT_TO_GOAL_SECONDS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PassingGoalCalculatorConfig {
    pub max_pass_to_goal_seconds: f32,
}

impl Default for PassingGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_pass_to_goal_seconds: DEFAULT_PASSING_GOAL_MAX_PASS_TO_GOAL_SECONDS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct AirDribbleGoalCalculatorConfig {
    pub max_end_to_goal_seconds: f32,
}

impl Default for AirDribbleGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_end_to_goal_seconds: DEFAULT_AIR_DRIBBLE_GOAL_MAX_END_TO_GOAL_SECONDS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FlipResetGoalCalculatorConfig {
    pub max_event_to_goal_seconds: f32,
}

impl Default for FlipResetGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_event_to_goal_seconds: DEFAULT_FLIP_RESET_GOAL_MAX_EVENT_TO_GOAL_SECONDS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct HalfVolleyGoalCalculatorConfig {
    pub max_touch_to_goal_seconds: f32,
    pub min_goal_alignment: f32,
}

impl Default for HalfVolleyGoalCalculatorConfig {
    fn default() -> Self {
        Self {
            max_touch_to_goal_seconds: DEFAULT_HALF_VOLLEY_GOAL_MAX_TOUCH_TO_GOAL_SECONDS,
            min_goal_alignment: DEFAULT_HALF_VOLLEY_GOAL_MIN_GOAL_ALIGNMENT,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct GoalTaggingContext {
    goal_index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AerialGoalCalculator {
    config: AerialGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HighAerialGoalCalculator {
    config: HighAerialGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LongDistanceGoalCalculator {
    config: LongDistanceGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OwnHalfGoalCalculator {
    config: OwnHalfGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EmptyNetGoalCalculator {
    config: EmptyNetGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CounterAttackGoalCalculator {
    events: EventStream<GoalTagAssignment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlickGoalCalculator {
    config: FlickGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DoubleTapGoalCalculator {
    config: DoubleTapGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OneTimerGoalCalculator {
    config: OneTimerGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PassingGoalCalculator {
    config: PassingGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AirDribbleGoalCalculator {
    config: AirDribbleGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlipResetGoalCalculator {
    config: FlipResetGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HalfVolleyGoalCalculator {
    config: HalfVolleyGoalCalculatorConfig,
    events: EventStream<GoalTagAssignment>,
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
                    events: EventStream::new(),
                }
            }

            pub fn config(&self) -> &$config {
                &self.config
            }

            pub fn events(&self) -> &[GoalTagAssignment] {
                self.events.all()
            }

            pub fn new_events(&self) -> &[GoalTagAssignment] {
                self.events.new_events()
            }
        }
    };
}

impl_goal_tag_calculator!(AerialGoalCalculator, AerialGoalCalculatorConfig);
impl_goal_tag_calculator!(HighAerialGoalCalculator, HighAerialGoalCalculatorConfig);
impl_goal_tag_calculator!(LongDistanceGoalCalculator, LongDistanceGoalCalculatorConfig);
impl_goal_tag_calculator!(OwnHalfGoalCalculator, OwnHalfGoalCalculatorConfig);
impl_goal_tag_calculator!(EmptyNetGoalCalculator, EmptyNetGoalCalculatorConfig);
impl_goal_tag_calculator!(FlickGoalCalculator, FlickGoalCalculatorConfig);
impl_goal_tag_calculator!(DoubleTapGoalCalculator, DoubleTapGoalCalculatorConfig);
impl_goal_tag_calculator!(OneTimerGoalCalculator, OneTimerGoalCalculatorConfig);
impl_goal_tag_calculator!(PassingGoalCalculator, PassingGoalCalculatorConfig);
impl_goal_tag_calculator!(AirDribbleGoalCalculator, AirDribbleGoalCalculatorConfig);
impl_goal_tag_calculator!(FlipResetGoalCalculator, FlipResetGoalCalculatorConfig);

impl Default for CounterAttackGoalCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl CounterAttackGoalCalculator {
    pub fn new() -> Self {
        Self {
            events: EventStream::new(),
        }
    }

    pub fn events(&self) -> &[GoalTagAssignment] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[GoalTagAssignment] {
        self.events.new_events()
    }
}

impl Default for HalfVolleyGoalCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl HalfVolleyGoalCalculator {
    pub fn new() -> Self {
        Self::with_config(HalfVolleyGoalCalculatorConfig::default())
    }

    pub fn with_config(config: HalfVolleyGoalCalculatorConfig) -> Self {
        Self {
            config,
            events: EventStream::new(),
        }
    }

    pub fn config(&self) -> &HalfVolleyGoalCalculatorConfig {
        &self.config
    }

    pub fn events(&self) -> &[GoalTagAssignment] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[GoalTagAssignment] {
        self.events.new_events()
    }
}

impl AerialGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events
            .replace_all_assuming_append_only(self.tag_goals(match_stats.goal_context_events()));
        Ok(())
    }

    fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagAssignment> {
        tag_goals_by_height(goals, GoalTagKind::AerialGoal, self.config.min_ball_z)
    }
}

impl HighAerialGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events
            .replace_all_assuming_append_only(self.tag_goals(match_stats.goal_context_events()));
        Ok(())
    }

    fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagAssignment> {
        tag_goals_by_height(goals, GoalTagKind::HighAerialGoal, self.config.min_ball_z)
    }
}

impl LongDistanceGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events
            .replace_all_assuming_append_only(self.tag_goals(match_stats.goal_context_events()));
        Ok(())
    }

    fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagAssignment> {
        tag_goals_by_attacking_y(
            goals,
            GoalTagKind::LongDistanceGoal,
            self.config.max_attacking_y,
        )
    }
}

impl OwnHalfGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events
            .replace_all_assuming_append_only(self.tag_goals(match_stats.goal_context_events()));
        Ok(())
    }

    fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagAssignment> {
        tag_goals_by_recent_attacking_y(
            goals,
            GoalTagKind::OwnHalfGoal,
            self.config.max_attacking_y,
            OWN_HALF_GOAL_MAX_TOUCH_TO_GOAL_SECONDS,
        )
    }
}

impl EmptyNetGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events
            .replace_all_assuming_append_only(self.tag_goals(match_stats.goal_context_events()));
        Ok(())
    }

    fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagAssignment> {
        let mut tags = Vec::new();
        for (goal_index, goal) in goals.iter().enumerate() {
            let ctx = GoalTaggingContext { goal_index };
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

impl CounterAttackGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events
            .replace_all_assuming_append_only(self.tag_goals(match_stats.goal_context_events()));
        Ok(())
    }

    fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagAssignment> {
        goals
            .iter()
            .enumerate()
            .filter(|(_, goal)| goal.goal_buildup == GoalBuildupKind::CounterAttack)
            .map(|(goal_index, goal)| {
                goal_tag(
                    GoalTaggingContext { goal_index },
                    GoalTagKind::CounterAttackGoal,
                    1.0,
                    vec![goal_buildup_evidence(goal), goal_context_evidence(goal)],
                )
            })
            .collect()
    }
}

impl FlickGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        flick: &FlickCalculator,
    ) -> SubtrActorResult<()> {
        self.events.replace_all_assuming_append_only(
            self.tag_goals(match_stats.goal_context_events(), flick.events()),
        );
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        events: &[FlickEvent],
    ) -> Vec<GoalTagAssignment> {
        tag_goals_by_point_mechanic_event(
            goals,
            events,
            GoalTagKind::FlickGoal,
            self.config.max_event_to_goal_seconds,
        )
    }
}

impl OneTimerGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        one_timer: &OneTimerCalculator,
    ) -> SubtrActorResult<()> {
        self.events.replace_all_assuming_append_only(
            self.tag_goals(match_stats.goal_context_events(), one_timer.events()),
        );
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        events: &[OneTimerEvent],
    ) -> Vec<GoalTagAssignment> {
        tag_goals_by_point_mechanic_event(
            goals,
            events,
            GoalTagKind::OneTimerGoal,
            self.config.max_event_to_goal_seconds,
        )
    }
}

impl PassingGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        pass: &PassCalculator,
    ) -> SubtrActorResult<()> {
        self.events.replace_all_assuming_append_only(
            self.tag_goals(match_stats.goal_context_events(), pass.events()),
        );
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        events: &[PassEvent],
    ) -> Vec<GoalTagAssignment> {
        let mut tags = Vec::new();
        for (goal_index, goal) in goals.iter().enumerate() {
            let ctx = GoalTaggingContext { goal_index };
            let Some((event_index, event)) = events
                .iter()
                .enumerate()
                .filter(|(_, event)| pass_event_matches_goal(event, goal))
                .filter(|(_, event)| goal.time - event.time <= self.config.max_pass_to_goal_seconds)
                .max_by(|left, right| {
                    left.1
                        .time
                        .total_cmp(&right.1.time)
                        .then_with(|| left.1.frame.cmp(&right.1.frame))
                })
            else {
                continue;
            };

            tags.push(goal_tag_with_modifiers(
                ctx,
                GoalTagKind::PassingGoal,
                1.0,
                mechanic_goal_modifiers(goal, &event.receiver),
                mechanic_goal_evidence(goal, pass_evidence(event)),
                vec![GoalTagEventRef {
                    stream: GoalTagEventStream::Pass,
                    index: event_index,
                }],
            ));
        }
        tags
    }
}

impl DoubleTapGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        double_tap: &DoubleTapCalculator,
    ) -> SubtrActorResult<()> {
        self.events.replace_all_assuming_append_only(
            self.tag_goals(match_stats.goal_context_events(), double_tap.events()),
        );
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        events: &[DoubleTapEvent],
    ) -> Vec<GoalTagAssignment> {
        tag_goals_by_point_mechanic_event(
            goals,
            events,
            GoalTagKind::DoubleTapGoal,
            self.config.max_event_to_goal_seconds,
        )
    }
}

impl AirDribbleGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        ball_carry: &BallCarryCalculator,
    ) -> SubtrActorResult<()> {
        self.events.replace_all_assuming_append_only(
            self.tag_goals(match_stats.goal_context_events(), ball_carry.carry_events()),
        );
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        events: &[BallCarryEvent],
    ) -> Vec<GoalTagAssignment> {
        tag_goals_by_air_dribble_event(goals, events, self.config.max_end_to_goal_seconds)
    }
}

impl FlipResetGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        dodge_reset: &DodgeResetCalculator,
    ) -> SubtrActorResult<()> {
        self.events.replace_all_assuming_append_only(self.tag_goals(
            match_stats.goal_context_events(),
            dodge_reset.confirmed_flip_reset_events(),
        ));
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        events: &[ConfirmedFlipResetEvent],
    ) -> Vec<GoalTagAssignment> {
        tag_goals_by_point_mechanic_event(
            goals,
            events,
            GoalTagKind::FlipResetGoal,
            self.config.max_event_to_goal_seconds,
        )
    }
}

impl HalfVolleyGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        half_volley: &HalfVolleyCalculator,
    ) -> SubtrActorResult<()> {
        self.events.replace_all_assuming_append_only(
            self.tag_goals(match_stats.goal_context_events(), half_volley.events()),
        );
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        half_volley_events: &[HalfVolleyEvent],
    ) -> Vec<GoalTagAssignment> {
        let mut tags = Vec::new();
        for (goal_index, goal) in goals.iter().enumerate() {
            let ctx = GoalTaggingContext { goal_index };
            let Some((candidate_index, candidate)) =
                self.tag_goals_by_half_volley_event(goal, half_volley_events)
            else {
                continue;
            };

            tags.push(goal_tag_with_modifiers(
                ctx,
                GoalTagKind::HalfVolleyGoal,
                1.0,
                mechanic_goal_modifiers(goal, &candidate.player),
                mechanic_goal_evidence(goal, half_volley_evidence(candidate)),
                vec![GoalTagEventRef {
                    stream: GoalTagEventStream::HalfVolley,
                    index: candidate_index,
                }],
            ));
        }
        tags
    }

    fn tag_goals_by_half_volley_event<'a>(
        &self,
        goal: &GoalContextEvent,
        half_volley_events: &'a [HalfVolleyEvent],
    ) -> Option<(usize, &'a HalfVolleyEvent)> {
        half_volley_events
            .iter()
            .enumerate()
            .filter(|(_, candidate)| self.candidate_matches_goal(candidate, goal))
            .max_by(|left, right| {
                left.1
                    .time
                    .total_cmp(&right.1.time)
                    .then_with(|| left.1.frame.cmp(&right.1.frame))
            })
    }

    fn candidate_matches_goal(&self, candidate: &HalfVolleyEvent, goal: &GoalContextEvent) -> bool {
        const MAX_EVENT_AFTER_GOAL_SECONDS: f32 = 0.05;

        if candidate.is_team_0 != goal.scoring_team_is_team_0
            || candidate.time > goal.time + MAX_EVENT_AFTER_GOAL_SECONDS
            || candidate.frame > goal.frame
            || goal.time - candidate.time > self.config.max_touch_to_goal_seconds
            || candidate.goal_alignment < self.config.min_goal_alignment
        {
            return false;
        }

        let Some(touch) = goal.scorer_last_touch.as_ref() else {
            return false;
        };
        touch.player == candidate.player && touch.frame == candidate.frame
    }
}

trait GoalMechanicPointEvent {
    fn event_time(&self) -> f32;
    fn event_frame(&self) -> usize;
    fn event_player(&self) -> &PlayerId;
    fn event_team_is_team_0(&self) -> bool;
    fn event_confidence(&self) -> f32;
    fn evidence_kind(&self) -> GoalTagEvidenceKind;
    fn event_stream(&self) -> GoalTagEventStream;

    fn event_ref(&self, index: usize) -> GoalTagEventRef {
        GoalTagEventRef {
            stream: self.event_stream(),
            index,
        }
    }
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

    fn event_stream(&self) -> GoalTagEventStream {
        GoalTagEventStream::Flick
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

    fn event_stream(&self) -> GoalTagEventStream {
        GoalTagEventStream::OneTimer
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

    fn event_stream(&self) -> GoalTagEventStream {
        GoalTagEventStream::DoubleTap
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

    fn event_stream(&self) -> GoalTagEventStream {
        GoalTagEventStream::DodgeReset
    }
}

pub fn combined_goal_tag_assignments(
    calculators: &[&[GoalTagAssignment]],
) -> Vec<GoalTagAssignment> {
    let mut assignments: Vec<_> = calculators
        .iter()
        .flat_map(|events| events.iter().cloned())
        .collect();
    assignments.sort_by(|left, right| {
        left.goal_index
            .cmp(&right.goal_index)
            .then_with(|| format!("{:?}", left.tag.kind()).cmp(&format!("{:?}", right.tag.kind())))
    });
    assignments
}

pub fn goal_context_events_with_tags(
    goals: &[GoalContextEvent],
    assignments: &[GoalTagAssignment],
) -> Vec<GoalContextEvent> {
    let mut goals_with_tags = goals.to_vec();
    for assignment in assignments {
        let Some(goal) = goals_with_tags.get_mut(assignment.goal_index) else {
            continue;
        };
        goal.tags.push(assignment.tag.clone());
    }
    for goal in &mut goals_with_tags {
        goal.tags.sort_by(|left, right| {
            format!("{:?}", left.kind())
                .cmp(&format!("{:?}", right.kind()))
                .then_with(|| {
                    right
                        .metadata()
                        .confidence
                        .total_cmp(&left.metadata().confidence)
                })
        });
    }
    goals_with_tags
}

#[cfg(test)]
#[path = "goal_tags_tests.rs"]
mod tests;
