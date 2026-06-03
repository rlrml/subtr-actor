use super::*;

#[path = "match_stats_helpers.rs"]
mod match_stats_helpers;

const GOAL_AFTER_KICKOFF_BUCKET_KICKOFF_MAX_SECONDS: f32 = 10.0;
const GOAL_AFTER_KICKOFF_BUCKET_SHORT_MAX_SECONDS: f32 = 20.0;
const GOAL_AFTER_KICKOFF_BUCKET_MEDIUM_MAX_SECONDS: f32 = 40.0;
const GOAL_BUILDUP_LOOKBACK_SECONDS: f32 = 12.0;
const COUNTER_ATTACK_MAX_ATTACK_SECONDS: f32 = 4.0;
const COUNTER_ATTACK_MIN_DEFENSIVE_HALF_SECONDS: f32 = 4.0;
const COUNTER_ATTACK_MIN_DEFENSIVE_THIRD_SECONDS: f32 = 1.0;
const SUSTAINED_PRESSURE_MIN_ATTACK_SECONDS: f32 = 6.0;
const SUSTAINED_PRESSURE_MIN_OFFENSIVE_HALF_SECONDS: f32 = 7.0;
const SUSTAINED_PRESSURE_MIN_OFFENSIVE_THIRD_SECONDS: f32 = 3.5;
const GOAL_CONTEXT_BOOST_LEADUP_SECONDS: f32 = 5.0;
const BALL_GROUND_CONTACT_MAX_Z: f32 = BALL_RADIUS_Z + 5.0;
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalAfterKickoffStats {
    pub kickoff_goal_count: u32,
    pub short_goal_count: u32,
    pub medium_goal_count: u32,
    pub long_goal_count: u32,
    #[serde(default, skip_serializing)]
    goal_times: Vec<f32>,
}

impl GoalAfterKickoffStats {
    pub fn goal_times(&self) -> &[f32] {
        &self.goal_times
    }

    pub fn record_goal(&mut self, time_after_kickoff: f32) {
        let clamped_time = time_after_kickoff.max(0.0);
        self.goal_times.push(clamped_time);
        self.goal_times.sort_by(|left, right| left.total_cmp(right));
        if clamped_time < GOAL_AFTER_KICKOFF_BUCKET_KICKOFF_MAX_SECONDS {
            self.kickoff_goal_count += 1;
        } else if clamped_time < GOAL_AFTER_KICKOFF_BUCKET_SHORT_MAX_SECONDS {
            self.short_goal_count += 1;
        } else if clamped_time < GOAL_AFTER_KICKOFF_BUCKET_MEDIUM_MAX_SECONDS {
            self.medium_goal_count += 1;
        } else {
            self.long_goal_count += 1;
        }
    }

    pub fn average_goal_time_after_kickoff(&self) -> f32 {
        if self.goal_times.is_empty() {
            0.0
        } else {
            self.goal_times.iter().sum::<f32>() / self.goal_times.len() as f32
        }
    }

    pub fn median_goal_time_after_kickoff(&self) -> f32 {
        if self.goal_times.is_empty() {
            return 0.0;
        }

        let mut sorted_times = self.goal_times.clone();
        sorted_times.sort_by(|a, b| a.total_cmp(b));
        let midpoint = sorted_times.len() / 2;
        if sorted_times.len().is_multiple_of(2) {
            (sorted_times[midpoint - 1] + sorted_times[midpoint]) * 0.5
        } else {
            sorted_times[midpoint]
        }
    }

    fn merge(&mut self, other: &Self) {
        self.kickoff_goal_count += other.kickoff_goal_count;
        self.short_goal_count += other.short_goal_count;
        self.medium_goal_count += other.medium_goal_count;
        self.long_goal_count += other.long_goal_count;
        self.goal_times.extend(other.goal_times.iter().copied());
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalBallAirTimeStats {
    pub goal_ball_air_time_sample_count: u32,
    pub cumulative_goal_ball_air_time: f32,
    pub last_goal_ball_air_time: Option<f32>,
    #[serde(default, skip_serializing)]
    goal_ball_air_times: Vec<f32>,
}

impl GoalBallAirTimeStats {
    pub fn goal_ball_air_times(&self) -> &[f32] {
        &self.goal_ball_air_times
    }

    pub fn record_goal(&mut self, ball_air_time: f32) {
        let clamped_time = ball_air_time.max(0.0);
        self.goal_ball_air_time_sample_count += 1;
        self.cumulative_goal_ball_air_time += clamped_time;
        self.last_goal_ball_air_time = Some(clamped_time);
        self.goal_ball_air_times.push(clamped_time);
        self.goal_ball_air_times
            .sort_by(|left, right| left.total_cmp(right));
    }

    pub fn average_goal_ball_air_time(&self) -> f32 {
        if self.goal_ball_air_time_sample_count == 0 {
            0.0
        } else {
            self.cumulative_goal_ball_air_time / self.goal_ball_air_time_sample_count as f32
        }
    }

    pub fn median_goal_ball_air_time(&self) -> f32 {
        if self.goal_ball_air_times.is_empty() {
            return 0.0;
        }

        let mut sorted_times = self.goal_ball_air_times.clone();
        sorted_times.sort_by(|a, b| a.total_cmp(b));
        let midpoint = sorted_times.len() / 2;
        if sorted_times.len().is_multiple_of(2) {
            (sorted_times[midpoint - 1] + sorted_times[midpoint]) * 0.5
        } else {
            sorted_times[midpoint]
        }
    }

    fn merge(&mut self, other: &Self) {
        self.goal_ball_air_time_sample_count += other.goal_ball_air_time_sample_count;
        self.cumulative_goal_ball_air_time += other.cumulative_goal_ball_air_time;
        self.last_goal_ball_air_time = other
            .last_goal_ball_air_time
            .or(self.last_goal_ball_air_time);
        self.goal_ball_air_times
            .extend(other.goal_ball_air_times.iter().copied());
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GoalBuildupKind {
    CounterAttack,
    SustainedPressure,
    #[default]
    Other,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalBuildupStats {
    pub counter_attack_goal_count: u32,
    pub sustained_pressure_goal_count: u32,
    pub other_buildup_goal_count: u32,
}

impl GoalBuildupStats {
    fn record(&mut self, kind: GoalBuildupKind) {
        match kind {
            GoalBuildupKind::CounterAttack => self.counter_attack_goal_count += 1,
            GoalBuildupKind::SustainedPressure => self.sustained_pressure_goal_count += 1,
            GoalBuildupKind::Other => self.other_buildup_goal_count += 1,
        }
    }

    fn merge(&mut self, other: &Self) {
        self.counter_attack_goal_count += other.counter_attack_goal_count;
        self.sustained_pressure_goal_count += other.sustained_pressure_goal_count;
        self.other_buildup_goal_count += other.other_buildup_goal_count;
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PlayerScoringContextStats {
    pub goals_conceded_while_last_defender: u32,
    pub goals_for_while_most_back: u32,
    pub goals_against_while_most_back: u32,
    pub goal_against_boost_sample_count: u32,
    pub cumulative_boost_on_goals_against: f32,
    pub last_boost_on_goal_against: Option<f32>,
    pub goal_against_boost_leadup_sample_count: u32,
    pub cumulative_average_boost_in_goal_against_leadup: f32,
    pub cumulative_min_boost_in_goal_against_leadup: f32,
    pub last_average_boost_in_goal_against_leadup: Option<f32>,
    pub last_min_boost_in_goal_against_leadup: Option<f32>,
    pub goal_against_position_sample_count: u32,
    pub cumulative_goal_against_position_x: f32,
    pub cumulative_goal_against_position_y: f32,
    pub cumulative_goal_against_position_z: f32,
    pub last_goal_against_position: Option<GoalContextPosition>,
    pub scoring_goal_last_touch_position_sample_count: u32,
    pub cumulative_scoring_goal_last_touch_position_x: f32,
    pub cumulative_scoring_goal_last_touch_position_y: f32,
    pub cumulative_scoring_goal_last_touch_position_z: f32,
    pub last_scoring_goal_last_touch_position: Option<GoalContextPosition>,
    #[serde(flatten)]
    pub goal_after_kickoff: GoalAfterKickoffStats,
    #[serde(flatten)]
    pub goal_buildup: GoalBuildupStats,
    #[serde(default, flatten)]
    pub goal_ball_air_time: GoalBallAirTimeStats,
}

impl PlayerScoringContextStats {
    fn record_goal_against_snapshot(
        &mut self,
        boost_amount: Option<f32>,
        position: Option<GoalContextPosition>,
        boost_leadup: Option<BoostLeadupStats>,
    ) {
        if let Some(boost_amount) = boost_amount {
            self.goal_against_boost_sample_count += 1;
            self.cumulative_boost_on_goals_against += boost_amount;
            self.last_boost_on_goal_against = Some(boost_amount);
        }

        if let Some(boost_leadup) = boost_leadup {
            self.goal_against_boost_leadup_sample_count += 1;
            self.cumulative_average_boost_in_goal_against_leadup += boost_leadup.average_boost;
            self.cumulative_min_boost_in_goal_against_leadup += boost_leadup.min_boost;
            self.last_average_boost_in_goal_against_leadup = Some(boost_leadup.average_boost);
            self.last_min_boost_in_goal_against_leadup = Some(boost_leadup.min_boost);
        }

        if let Some(position) = position {
            self.goal_against_position_sample_count += 1;
            self.cumulative_goal_against_position_x += position.x;
            self.cumulative_goal_against_position_y += position.y;
            self.cumulative_goal_against_position_z += position.z;
            self.last_goal_against_position = Some(position);
        }
    }

    fn record_scoring_goal_last_touch_position(&mut self, position: GoalContextPosition) {
        self.scoring_goal_last_touch_position_sample_count += 1;
        self.cumulative_scoring_goal_last_touch_position_x += position.x;
        self.cumulative_scoring_goal_last_touch_position_y += position.y;
        self.cumulative_scoring_goal_last_touch_position_z += position.z;
        self.last_scoring_goal_last_touch_position = Some(position);
    }

    fn record_goal_ball_air_time(&mut self, ball_air_time: f32) {
        self.goal_ball_air_time.record_goal(ball_air_time);
    }

    fn average_boost_on_goals_against(&self) -> f32 {
        if self.goal_against_boost_sample_count == 0 {
            0.0
        } else {
            self.cumulative_boost_on_goals_against / self.goal_against_boost_sample_count as f32
        }
    }

    fn average_boost_in_goal_against_leadup(&self) -> f32 {
        if self.goal_against_boost_leadup_sample_count == 0 {
            0.0
        } else {
            self.cumulative_average_boost_in_goal_against_leadup
                / self.goal_against_boost_leadup_sample_count as f32
        }
    }

    fn average_min_boost_in_goal_against_leadup(&self) -> f32 {
        if self.goal_against_boost_leadup_sample_count == 0 {
            0.0
        } else {
            self.cumulative_min_boost_in_goal_against_leadup
                / self.goal_against_boost_leadup_sample_count as f32
        }
    }

    fn average_goal_against_position_x(&self) -> f32 {
        if self.goal_against_position_sample_count == 0 {
            0.0
        } else {
            self.cumulative_goal_against_position_x / self.goal_against_position_sample_count as f32
        }
    }

    fn average_goal_against_position_y(&self) -> f32 {
        if self.goal_against_position_sample_count == 0 {
            0.0
        } else {
            self.cumulative_goal_against_position_y / self.goal_against_position_sample_count as f32
        }
    }

    fn average_goal_against_position_z(&self) -> f32 {
        if self.goal_against_position_sample_count == 0 {
            0.0
        } else {
            self.cumulative_goal_against_position_z / self.goal_against_position_sample_count as f32
        }
    }

    fn average_scoring_goal_last_touch_position_x(&self) -> f32 {
        if self.scoring_goal_last_touch_position_sample_count == 0 {
            0.0
        } else {
            self.cumulative_scoring_goal_last_touch_position_x
                / self.scoring_goal_last_touch_position_sample_count as f32
        }
    }

    fn average_scoring_goal_last_touch_position_y(&self) -> f32 {
        if self.scoring_goal_last_touch_position_sample_count == 0 {
            0.0
        } else {
            self.cumulative_scoring_goal_last_touch_position_y
                / self.scoring_goal_last_touch_position_sample_count as f32
        }
    }

    fn average_scoring_goal_last_touch_position_z(&self) -> f32 {
        if self.scoring_goal_last_touch_position_sample_count == 0 {
            0.0
        } else {
            self.cumulative_scoring_goal_last_touch_position_z
                / self.scoring_goal_last_touch_position_sample_count as f32
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct CorePlayerStats {
    pub score: i32,
    pub goals: i32,
    pub assists: i32,
    pub saves: i32,
    pub shots: i32,
    #[serde(flatten)]
    pub scoring_context: PlayerScoringContextStats,
}

impl CorePlayerStats {
    pub fn shooting_percentage(&self) -> f32 {
        if self.shots == 0 {
            0.0
        } else {
            self.goals as f32 * 100.0 / self.shots as f32
        }
    }

    pub fn average_goal_time_after_kickoff(&self) -> f32 {
        self.scoring_context
            .goal_after_kickoff
            .average_goal_time_after_kickoff()
    }

    pub fn median_goal_time_after_kickoff(&self) -> f32 {
        self.scoring_context
            .goal_after_kickoff
            .median_goal_time_after_kickoff()
    }

    pub fn average_boost_on_goals_against(&self) -> f32 {
        self.scoring_context.average_boost_on_goals_against()
    }

    pub fn average_boost_in_goal_against_leadup(&self) -> f32 {
        self.scoring_context.average_boost_in_goal_against_leadup()
    }

    pub fn average_min_boost_in_goal_against_leadup(&self) -> f32 {
        self.scoring_context
            .average_min_boost_in_goal_against_leadup()
    }

    pub fn average_goal_against_position_x(&self) -> f32 {
        self.scoring_context.average_goal_against_position_x()
    }

    pub fn average_goal_against_position_y(&self) -> f32 {
        self.scoring_context.average_goal_against_position_y()
    }

    pub fn average_goal_against_position_z(&self) -> f32 {
        self.scoring_context.average_goal_against_position_z()
    }

    pub fn average_scoring_goal_last_touch_position_x(&self) -> f32 {
        self.scoring_context
            .average_scoring_goal_last_touch_position_x()
    }

    pub fn average_scoring_goal_last_touch_position_y(&self) -> f32 {
        self.scoring_context
            .average_scoring_goal_last_touch_position_y()
    }

    pub fn average_scoring_goal_last_touch_position_z(&self) -> f32 {
        self.scoring_context
            .average_scoring_goal_last_touch_position_z()
    }

    pub fn average_goal_ball_air_time(&self) -> f32 {
        self.scoring_context
            .goal_ball_air_time
            .average_goal_ball_air_time()
    }

    pub fn median_goal_ball_air_time(&self) -> f32 {
        self.scoring_context
            .goal_ball_air_time
            .median_goal_ball_air_time()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct TeamScoringContextStats {
    #[serde(flatten)]
    pub goal_after_kickoff: GoalAfterKickoffStats,
    #[serde(flatten)]
    pub goal_buildup: GoalBuildupStats,
    #[serde(default, flatten)]
    pub goal_ball_air_time: GoalBallAirTimeStats,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct CoreTeamStats {
    pub score: i32,
    pub goals: i32,
    pub assists: i32,
    pub saves: i32,
    pub shots: i32,
    #[serde(flatten)]
    pub scoring_context: TeamScoringContextStats,
}

impl CoreTeamStats {
    pub fn shooting_percentage(&self) -> f32 {
        if self.shots == 0 {
            0.0
        } else {
            self.goals as f32 * 100.0 / self.shots as f32
        }
    }

    pub fn average_goal_time_after_kickoff(&self) -> f32 {
        self.scoring_context
            .goal_after_kickoff
            .average_goal_time_after_kickoff()
    }

    pub fn median_goal_time_after_kickoff(&self) -> f32 {
        self.scoring_context
            .goal_after_kickoff
            .median_goal_time_after_kickoff()
    }

    pub fn average_goal_ball_air_time(&self) -> f32 {
        self.scoring_context
            .goal_ball_air_time
            .average_goal_ball_air_time()
    }

    pub fn median_goal_ball_air_time(&self) -> f32 {
        self.scoring_context
            .goal_ball_air_time
            .median_goal_ball_air_time()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct CorePlayerStatsEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub delta: CorePlayerStats,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct CoreTeamStatsEvent {
    pub time: f32,
    pub frame: usize,
    pub is_team_0: bool,
    pub delta: CoreTeamStats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum TimelineEventKind {
    Goal,
    Shot,
    Save,
    Assist,
    Kill,
    Death,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct TimelineEvent {
    pub time: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<usize>,
    pub kind: TimelineEventKind,
    #[ts(as = "Option<crate::ts_bindings::RemoteIdTs>")]
    pub player_id: Option<PlayerId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: Option<bool>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalContextPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<glam::Vec3> for GoalContextPosition {
    fn from(position: glam::Vec3) -> Self {
        Self {
            x: position.x,
            y: position.y,
            z: position.z,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalPlayerContext {
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub position: Option<GoalContextPosition>,
    pub boost_amount: Option<f32>,
    pub average_boost_in_leadup: Option<f32>,
    pub min_boost_in_leadup: Option<f32>,
    pub is_most_back: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalTouchContext {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub ball_position: Option<GoalContextPosition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ball_speed_after_touch: Option<f32>,
    pub player_position: Option<GoalContextPosition>,
    pub players: Vec<GoalPlayerContext>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalContextEvent {
    pub time: f32,
    pub frame: usize,
    pub scoring_team_is_team_0: bool,
    #[ts(as = "Option<crate::ts_bindings::RemoteIdTs>")]
    pub scorer: Option<PlayerId>,
    #[ts(as = "Option<crate::ts_bindings::RemoteIdTs>")]
    pub scoring_team_most_back_player: Option<PlayerId>,
    #[ts(as = "Option<crate::ts_bindings::RemoteIdTs>")]
    pub defending_team_most_back_player: Option<PlayerId>,
    pub ball_position: Option<GoalContextPosition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ball_speed_at_goal: Option<f32>,
    pub ball_air_time_before_goal: Option<f32>,
    #[serde(default)]
    pub goal_buildup: GoalBuildupKind,
    pub scorer_last_touch: Option<GoalTouchContext>,
    pub players: Vec<GoalPlayerContext>,
}

#[derive(Debug, Clone)]
struct PendingGoalEvent {
    event: GoalEvent,
    time_after_kickoff: Option<f32>,
    goal_buildup: GoalBuildupKind,
    ball_air_time_before_goal: Option<f32>,
}

#[derive(Debug, Clone)]
struct GoalBuildupSample {
    time: f32,
    dt: f32,
    ball_y: f32,
}

#[derive(Debug, Clone)]
struct GoalBuildupPressureEvent {
    time: f32,
    is_team_0: bool,
}

#[derive(Debug, Clone, Copy)]
struct BoostLeadupSample {
    time: f32,
    boost_amount: f32,
}

#[derive(Debug, Clone, Copy)]
struct BoostLeadupStats {
    average_boost: f32,
    min_boost: f32,
}

#[derive(Debug, Clone, Default)]
pub struct MatchStatsCalculator {
    player_stats: HashMap<PlayerId, CorePlayerStats>,
    player_teams: HashMap<PlayerId, bool>,
    previous_player_stats: HashMap<PlayerId, CorePlayerStats>,
    last_emitted_player_stats: HashMap<PlayerId, CorePlayerStats>,
    last_emitted_team_zero_stats: CoreTeamStats,
    last_emitted_team_one_stats: CoreTeamStats,
    core_player_events: Vec<CorePlayerStatsEvent>,
    core_team_events: Vec<CoreTeamStatsEvent>,
    timeline: Vec<TimelineEvent>,
    pending_goal_events: Vec<PendingGoalEvent>,
    previous_team_scores: Option<(i32, i32)>,
    kickoff_waiting_for_first_touch: bool,
    active_kickoff_touch_time: Option<f32>,
    goal_buildup_samples: Vec<GoalBuildupSample>,
    goal_buildup_pressure_events: Vec<GoalBuildupPressureEvent>,
    goal_context_events: Vec<GoalContextEvent>,
    last_touch_context_by_player: HashMap<PlayerId, GoalTouchContext>,
    boost_leadup_samples_by_player: HashMap<PlayerId, VecDeque<BoostLeadupSample>>,
    last_ball_ground_contact_time: Option<f32>,
}

impl MatchStatsCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, CorePlayerStats> {
        &self.player_stats
    }

    pub fn timeline(&self) -> &[TimelineEvent] {
        &self.timeline
    }

    pub fn goal_context_events(&self) -> &[GoalContextEvent] {
        &self.goal_context_events
    }

    pub fn core_player_events(&self) -> &[CorePlayerStatsEvent] {
        &self.core_player_events
    }

    pub fn core_team_events(&self) -> &[CoreTeamStatsEvent] {
        &self.core_team_events
    }
}

impl MatchStatsCalculator {
    #[allow(clippy::too_many_arguments)]
    pub fn update_parts(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        live_play_state: &LivePlayState,
        touch_state: &TouchState,
    ) -> SubtrActorResult<()> {
        self.update_kickoff_reference(gameplay, events);
        self.prune_goal_buildup_samples(frame.time);
        self.update_ball_ground_contact(frame, ball);
        if live_play_state.is_live_play {
            self.record_goal_buildup_sample(frame, ball);
            self.record_goal_buildup_pressure_events(events);
            self.update_boost_leadup_samples(frame, players);
        } else if events.goal_events.is_empty() {
            self.last_touch_context_by_player.clear();
            self.boost_leadup_samples_by_player.clear();
            self.last_ball_ground_contact_time = None;
        }
        self.update_last_touch_contexts(ball, players, &touch_state.touch_events);
        self.record_goal_context_events(ball, players, events);
        let pending_goal_events: Vec<_> = events
            .goal_events
            .iter()
            .cloned()
            .map(|event| PendingGoalEvent {
                time_after_kickoff: self
                    .active_kickoff_touch_time
                    .map(|kickoff_touch_time| (event.time - kickoff_touch_time).max(0.0)),
                goal_buildup: self.classify_goal_buildup(event.time, event.scoring_team_is_team_0),
                ball_air_time_before_goal: self.ball_air_time_before_goal(event.time),
                event,
            })
            .collect();
        self.pending_goal_events.extend(pending_goal_events);
        let mut processor_event_counts: HashMap<(PlayerId, TimelineEventKind), i32> =
            HashMap::new();
        for event in &events.player_stat_events {
            let kind = match event.kind {
                PlayerStatEventKind::Shot => TimelineEventKind::Shot,
                PlayerStatEventKind::Save => TimelineEventKind::Save,
                PlayerStatEventKind::Assist => TimelineEventKind::Assist,
            };
            self.timeline.push(TimelineEvent {
                time: event.time,
                frame: Some(event.frame),
                kind,
                player_id: Some(event.player.clone()),
                player_position: event
                    .player_position
                    .map(|position| vec_to_glam(&position).to_array())
                    .or_else(|| {
                        event
                            .shot
                            .as_ref()
                            .and_then(|shot| shot.player_position)
                            .map(|position| vec_to_glam(&position).to_array())
                    })
                    .or_else(|| players.player_position(&event.player)),
                is_team_0: Some(event.is_team_0),
            });
            *processor_event_counts
                .entry((event.player.clone(), kind))
                .or_default() += 1;
        }

        for player in &players.players {
            self.player_teams
                .insert(player.player_id.clone(), player.is_team_0);
            let mut current_stats = CorePlayerStats {
                score: player.match_score.unwrap_or(0),
                goals: player.match_goals.unwrap_or(0),
                assists: player.match_assists.unwrap_or(0),
                saves: player.match_saves.unwrap_or(0),
                shots: player.match_shots.unwrap_or(0),
                scoring_context: self
                    .player_stats
                    .get(&player.player_id)
                    .map(|stats| stats.scoring_context.clone())
                    .unwrap_or_default(),
            };

            let previous_stats = self
                .previous_player_stats
                .get(&player.player_id)
                .cloned()
                .unwrap_or_default();

            let shot_delta = current_stats.shots - previous_stats.shots;
            let save_delta = current_stats.saves - previous_stats.saves;
            let assist_delta = current_stats.assists - previous_stats.assists;
            let goal_delta = current_stats.goals - previous_stats.goals;
            let shot_fallback_delta = shot_delta
                - processor_event_counts
                    .get(&(player.player_id.clone(), TimelineEventKind::Shot))
                    .copied()
                    .unwrap_or(0);
            let save_fallback_delta = save_delta
                - processor_event_counts
                    .get(&(player.player_id.clone(), TimelineEventKind::Save))
                    .copied()
                    .unwrap_or(0);
            let assist_fallback_delta = assist_delta
                - processor_event_counts
                    .get(&(player.player_id.clone(), TimelineEventKind::Assist))
                    .copied()
                    .unwrap_or(0);

            if shot_fallback_delta > 0 {
                self.emit_timeline_events(
                    frame.time,
                    Some(frame.frame_number),
                    TimelineEventKind::Shot,
                    &player.player_id,
                    player.is_team_0,
                    player.position().map(|position| position.to_array()),
                    shot_fallback_delta,
                );
            }
            if save_fallback_delta > 0 {
                self.emit_timeline_events(
                    frame.time,
                    Some(frame.frame_number),
                    TimelineEventKind::Save,
                    &player.player_id,
                    player.is_team_0,
                    player.position().map(|position| position.to_array()),
                    save_fallback_delta,
                );
            }
            if assist_fallback_delta > 0 {
                self.emit_timeline_events(
                    frame.time,
                    Some(frame.frame_number),
                    TimelineEventKind::Assist,
                    &player.player_id,
                    player.is_team_0,
                    player.position().map(|position| position.to_array()),
                    assist_fallback_delta,
                );
            }
            if goal_delta > 0 {
                for _ in 0..goal_delta.max(0) {
                    let pending_goal_event =
                        self.take_pending_goal_event(&player.player_id, player.is_team_0);
                    if let Some(pending_goal_event) = pending_goal_event.as_ref() {
                        let scorer_last_touch = self.reconcile_goal_context_scorer(
                            &pending_goal_event.event,
                            &player.player_id,
                        );
                        if let Some(touch_position) =
                            scorer_last_touch.and_then(|touch| touch.ball_position)
                        {
                            current_stats
                                .scoring_context
                                .record_scoring_goal_last_touch_position(touch_position);
                        }
                        if let Some(ball_air_time_before_goal) =
                            pending_goal_event.ball_air_time_before_goal
                        {
                            current_stats
                                .scoring_context
                                .record_goal_ball_air_time(ball_air_time_before_goal);
                        }
                    }
                    let goal_time = pending_goal_event
                        .as_ref()
                        .map(|event| event.event.time)
                        .unwrap_or(frame.time);
                    let goal_buildup = pending_goal_event
                        .as_ref()
                        .map(|event| event.goal_buildup)
                        .unwrap_or_else(|| self.classify_goal_buildup(goal_time, player.is_team_0));
                    let goal_frame = pending_goal_event
                        .as_ref()
                        .map(|event| event.event.frame)
                        .unwrap_or(frame.frame_number);
                    let time_after_kickoff = pending_goal_event
                        .and_then(|event| event.time_after_kickoff)
                        .or_else(|| {
                            self.active_kickoff_touch_time
                                .map(|kickoff_touch_time| (goal_time - kickoff_touch_time).max(0.0))
                        });
                    if let Some(time_after_kickoff) = time_after_kickoff {
                        current_stats
                            .scoring_context
                            .goal_after_kickoff
                            .record_goal(time_after_kickoff);
                    }
                    current_stats
                        .scoring_context
                        .goal_buildup
                        .record(goal_buildup);
                    self.timeline.push(TimelineEvent {
                        time: goal_time,
                        frame: Some(goal_frame),
                        kind: TimelineEventKind::Goal,
                        player_id: Some(player.player_id.clone()),
                        player_position: player.position().map(|position| position.to_array()),
                        is_team_0: Some(player.is_team_0),
                    });
                }
            }

            self.previous_player_stats
                .insert(player.player_id.clone(), current_stats.clone());
            self.player_stats
                .insert(player.player_id.clone(), current_stats);
        }

        if let (Some(team_zero_score), Some(team_one_score)) =
            (gameplay.team_zero_score, gameplay.team_one_score)
        {
            if let Some((prev_team_zero_score, prev_team_one_score)) = self.previous_team_scores {
                let team_zero_delta = team_zero_score - prev_team_zero_score;
                let team_one_delta = team_one_score - prev_team_one_score;

                if team_zero_delta > 0 {
                    if let Some(last_defender) = self.last_defender(players, false) {
                        if let Some(stats) = self.player_stats.get_mut(&last_defender) {
                            stats.scoring_context.goals_conceded_while_last_defender +=
                                team_zero_delta as u32;
                        }
                    }
                }

                if team_one_delta > 0 {
                    if let Some(last_defender) = self.last_defender(players, true) {
                        if let Some(stats) = self.player_stats.get_mut(&last_defender) {
                            stats.scoring_context.goals_conceded_while_last_defender +=
                                team_one_delta as u32;
                        }
                    }
                }
            }

            self.previous_team_scores = Some((team_zero_score, team_one_score));
        }

        self.timeline.sort_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.emit_core_stats_events(frame, players);

        Ok(())
    }
}

#[cfg(test)]
#[path = "match_stats_tests.rs"]
mod tests;
