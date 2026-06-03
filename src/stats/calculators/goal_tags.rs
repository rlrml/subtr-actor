use super::*;

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<GoalContextPosition>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scorer_position: Option<GoalContextPosition>,
    pub confidence: f32,
    pub modifiers: Vec<GoalTagModifier>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct CounterAttackGoalCalculator {
    events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlickGoalCalculator {
    config: FlickGoalCalculatorConfig,
    events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DoubleTapGoalCalculator {
    config: DoubleTapGoalCalculatorConfig,
    events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OneTimerGoalCalculator {
    config: OneTimerGoalCalculatorConfig,
    events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PassingGoalCalculator {
    config: PassingGoalCalculatorConfig,
    events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AirDribbleGoalCalculator {
    config: AirDribbleGoalCalculatorConfig,
    events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlipResetGoalCalculator {
    config: FlipResetGoalCalculatorConfig,
    events: Vec<GoalTagEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HalfVolleyGoalCalculator {
    config: HalfVolleyGoalCalculatorConfig,
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
        Self { events: Vec::new() }
    }

    pub fn events(&self) -> &[GoalTagEvent] {
        &self.events
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
            events: Vec::new(),
        }
    }

    pub fn config(&self) -> &HalfVolleyGoalCalculatorConfig {
        &self.config
    }

    pub fn events(&self) -> &[GoalTagEvent] {
        &self.events
    }
}

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

impl CounterAttackGoalCalculator {
    pub fn update(&mut self, match_stats: &MatchStatsCalculator) -> SubtrActorResult<()> {
        self.events = self.tag_goals(match_stats.goal_context_events());
        Ok(())
    }

    fn tag_goals(&self, goals: &[GoalContextEvent]) -> Vec<GoalTagEvent> {
        goals
            .iter()
            .enumerate()
            .filter(|(_, goal)| goal.goal_buildup == GoalBuildupKind::CounterAttack)
            .map(|(goal_index, goal)| {
                goal_tag(
                    GoalTaggingContext { goal_index, goal },
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
        self.events = self.tag_goals(match_stats.goal_context_events(), flick.events());
        Ok(())
    }

    fn tag_goals(&self, goals: &[GoalContextEvent], events: &[FlickEvent]) -> Vec<GoalTagEvent> {
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
        self.events = self.tag_goals(match_stats.goal_context_events(), one_timer.events());
        Ok(())
    }

    fn tag_goals(&self, goals: &[GoalContextEvent], events: &[OneTimerEvent]) -> Vec<GoalTagEvent> {
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
        self.events = self.tag_goals(match_stats.goal_context_events(), pass.events());
        Ok(())
    }

    fn tag_goals(&self, goals: &[GoalContextEvent], events: &[PassEvent]) -> Vec<GoalTagEvent> {
        let mut tags = Vec::new();
        for (goal_index, goal) in goals.iter().enumerate() {
            let ctx = GoalTaggingContext { goal_index, goal };
            let Some(event) = events
                .iter()
                .filter(|event| pass_event_matches_goal(event, goal))
                .filter(|event| goal.time - event.time <= self.config.max_pass_to_goal_seconds)
                .max_by(|left, right| {
                    left.time
                        .total_cmp(&right.time)
                        .then_with(|| left.frame.cmp(&right.frame))
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
        self.events = self.tag_goals(match_stats.goal_context_events(), double_tap.events());
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        events: &[DoubleTapEvent],
    ) -> Vec<GoalTagEvent> {
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
        self.events = self.tag_goals(match_stats.goal_context_events(), ball_carry.carry_events());
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        events: &[BallCarryEvent],
    ) -> Vec<GoalTagEvent> {
        tag_goals_by_air_dribble_event(goals, events, self.config.max_end_to_goal_seconds)
    }
}

impl FlipResetGoalCalculator {
    pub fn update(
        &mut self,
        match_stats: &MatchStatsCalculator,
        dodge_reset: &DodgeResetCalculator,
    ) -> SubtrActorResult<()> {
        self.events = self.tag_goals(
            match_stats.goal_context_events(),
            dodge_reset.confirmed_flip_reset_events(),
        );
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        events: &[ConfirmedFlipResetEvent],
    ) -> Vec<GoalTagEvent> {
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
        self.events = self.tag_goals(match_stats.goal_context_events(), half_volley.events());
        Ok(())
    }

    fn tag_goals(
        &self,
        goals: &[GoalContextEvent],
        half_volley_events: &[HalfVolleyEvent],
    ) -> Vec<GoalTagEvent> {
        let mut tags = Vec::new();
        for (goal_index, goal) in goals.iter().enumerate() {
            let ctx = GoalTaggingContext { goal_index, goal };
            let Some(candidate) = self.tag_goals_by_half_volley_event(goal, half_volley_events)
            else {
                continue;
            };

            tags.push(goal_tag_with_modifiers(
                ctx,
                GoalTagKind::HalfVolleyGoal,
                1.0,
                mechanic_goal_modifiers(goal, &candidate.player),
                mechanic_goal_evidence(goal, half_volley_evidence(candidate)),
            ));
        }
        tags
    }

    fn tag_goals_by_half_volley_event<'a>(
        &self,
        goal: &GoalContextEvent,
        half_volley_events: &'a [HalfVolleyEvent],
    ) -> Option<&'a HalfVolleyEvent> {
        half_volley_events
            .iter()
            .filter(|candidate| self.candidate_matches_goal(candidate, goal))
            .max_by(|left, right| {
                left.time
                    .total_cmp(&right.time)
                    .then_with(|| left.frame.cmp(&right.frame))
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
    tag_goals_by_recent_attacking_y(goals, kind, max_attacking_y, f32::INFINITY)
}

fn tag_goals_by_recent_attacking_y(
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

fn tag_goals_by_point_mechanic_event<E: GoalMechanicPointEvent>(
    goals: &[GoalContextEvent],
    events: &[E],
    kind: GoalTagKind,
    max_event_to_goal_seconds: f32,
) -> Vec<GoalTagEvent> {
    let mut tags = Vec::new();
    for (goal_index, goal) in goals.iter().enumerate() {
        let ctx = GoalTaggingContext { goal_index, goal };
        let Some(event) = events
            .iter()
            .filter(|event| point_event_matches_goal(*event, goal))
            .filter(|event| goal.time - event.event_time() <= max_event_to_goal_seconds)
            .max_by(|left, right| {
                left.event_time()
                    .total_cmp(&right.event_time())
                    .then_with(|| left.event_frame().cmp(&right.event_frame()))
            })
        else {
            continue;
        };

        tags.push(goal_tag_with_modifiers(
            ctx,
            kind,
            event.event_confidence(),
            mechanic_goal_modifiers(goal, event.event_player()),
            mechanic_goal_evidence(goal, point_mechanic_evidence(event)),
        ));
    }
    tags
}

fn point_event_matches_goal<E: GoalMechanicPointEvent>(event: &E, goal: &GoalContextEvent) -> bool {
    const MAX_EVENT_AFTER_GOAL_SECONDS: f32 = 0.05;

    event.event_team_is_team_0() == goal.scoring_team_is_team_0
        && event.event_time() <= goal.time + MAX_EVENT_AFTER_GOAL_SECONDS
        && event.event_frame() <= goal.frame
}

fn pass_event_matches_goal(event: &PassEvent, goal: &GoalContextEvent) -> bool {
    const MAX_EVENT_AFTER_GOAL_SECONDS: f32 = 0.05;

    event.is_team_0 == goal.scoring_team_is_team_0
        && event.time <= goal.time + MAX_EVENT_AFTER_GOAL_SECONDS
        && event.frame <= goal.frame
        && goal.scorer.as_ref() == Some(&event.receiver)
        && goal
            .scorer_last_touch
            .as_ref()
            .is_some_and(|touch| touch.player == event.receiver && touch.frame == event.frame)
}

fn tag_goals_by_air_dribble_event(
    goals: &[GoalContextEvent],
    events: &[BallCarryEvent],
    max_end_to_goal_seconds: f32,
) -> Vec<GoalTagEvent> {
    let mut tags = Vec::new();
    for (goal_index, goal) in goals.iter().enumerate() {
        let ctx = GoalTaggingContext { goal_index, goal };
        let Some(event) = events
            .iter()
            .filter(|event| air_dribble_event_matches_goal(event, goal))
            .filter(|event| goal.time - event.end_time <= max_end_to_goal_seconds)
            .max_by(|left, right| {
                left.end_time
                    .total_cmp(&right.end_time)
                    .then_with(|| left.end_frame.cmp(&right.end_frame))
            })
        else {
            continue;
        };

        tags.push(goal_tag_with_modifiers(
            ctx,
            GoalTagKind::AirDribbleGoal,
            1.0,
            mechanic_goal_modifiers(goal, &event.player_id),
            mechanic_goal_evidence(goal, air_dribble_evidence(event)),
        ));
    }
    tags
}

fn air_dribble_event_matches_goal(event: &BallCarryEvent, goal: &GoalContextEvent) -> bool {
    const MAX_EVENT_AFTER_GOAL_SECONDS: f32 = 0.05;

    event.kind == BallCarryKind::AirDribble
        && event.is_team_0 == goal.scoring_team_is_team_0
        && event.start_time <= goal.time + MAX_EVENT_AFTER_GOAL_SECONDS
        && event.end_time <= goal.time + MAX_EVENT_AFTER_GOAL_SECONDS
        && event.end_frame <= goal.frame
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
        player_position: goal.scorer.as_ref().and_then(|scorer| {
            goal.players
                .iter()
                .find(|player| &player.player == scorer)
                .and_then(|player| player.position)
        }),
    }
}

fn last_touch_evidence(touch: &GoalTouchContext) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: GoalTagEvidenceKind::ScorerLastTouch,
        time: touch.time,
        frame: touch.frame,
        player: Some(touch.player.clone()),
        player_position: None,
    }
}

fn defender_evidence(player: &GoalPlayerContext, goal: &GoalContextEvent) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: GoalTagEvidenceKind::DefenderPosition,
        time: goal.time,
        frame: goal.frame,
        player: Some(player.player.clone()),
        player_position: player.position,
    }
}

fn goal_buildup_evidence(goal: &GoalContextEvent) -> GoalTagEvidence {
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

fn point_mechanic_evidence(event: &impl GoalMechanicPointEvent) -> GoalTagEvidence {
    GoalTagEvidence {
        kind: event.evidence_kind(),
        time: event.event_time(),
        frame: event.event_frame(),
        player: Some(event.event_player().clone()),
        player_position: None,
    }
}

fn pass_evidence(event: &PassEvent) -> GoalTagEvidence {
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

fn air_dribble_evidence(event: &BallCarryEvent) -> GoalTagEvidence {
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

fn half_volley_evidence(candidate: &HalfVolleyEvent) -> GoalTagEvidence {
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

fn mechanic_goal_modifiers(
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

fn mechanic_goal_evidence(
    goal: &GoalContextEvent,
    mechanic_evidence: GoalTagEvidence,
) -> Vec<GoalTagEvidence> {
    let mut evidence = vec![mechanic_evidence, goal_context_evidence(goal)];
    if let Some(touch) = goal.scorer_last_touch.as_ref() {
        evidence.push(last_touch_evidence(touch));
    }
    evidence
}

fn goal_tag(
    ctx: GoalTaggingContext<'_>,
    kind: GoalTagKind,
    confidence: f32,
    evidence: Vec<GoalTagEvidence>,
) -> GoalTagEvent {
    goal_tag_with_modifiers(ctx, kind, confidence, Vec::new(), evidence)
}

fn goal_tag_with_modifiers(
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
        scorer_position: ctx.goal.scorer.as_ref().and_then(|scorer| {
            ctx.goal
                .players
                .iter()
                .find(|player| &player.player == scorer)
                .and_then(|player| player.position)
        }),
        confidence,
        modifiers,
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
