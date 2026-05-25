use super::*;

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

fn optional_delta<T: Copy + PartialEq>(current: Option<T>, previous: Option<T>) -> Option<T> {
    if current == previous {
        None
    } else {
        current
    }
}

fn sample_delta<T: Copy + PartialEq>(current: &[T], previous: &[T]) -> Vec<T> {
    let mut unmatched_previous = previous.to_vec();
    let mut delta = Vec::new();
    for value in current {
        if let Some(index) = unmatched_previous
            .iter()
            .position(|previous_value| previous_value == value)
        {
            unmatched_previous.remove(index);
        } else {
            delta.push(*value);
        }
    }
    delta
}

fn goal_after_kickoff_delta(
    current: &GoalAfterKickoffStats,
    previous: &GoalAfterKickoffStats,
) -> GoalAfterKickoffStats {
    GoalAfterKickoffStats {
        kickoff_goal_count: current
            .kickoff_goal_count
            .saturating_sub(previous.kickoff_goal_count),
        short_goal_count: current
            .short_goal_count
            .saturating_sub(previous.short_goal_count),
        medium_goal_count: current
            .medium_goal_count
            .saturating_sub(previous.medium_goal_count),
        long_goal_count: current
            .long_goal_count
            .saturating_sub(previous.long_goal_count),
        goal_times: sample_delta(&current.goal_times, &previous.goal_times),
    }
}

fn goal_buildup_delta(current: &GoalBuildupStats, previous: &GoalBuildupStats) -> GoalBuildupStats {
    GoalBuildupStats {
        counter_attack_goal_count: current
            .counter_attack_goal_count
            .saturating_sub(previous.counter_attack_goal_count),
        sustained_pressure_goal_count: current
            .sustained_pressure_goal_count
            .saturating_sub(previous.sustained_pressure_goal_count),
        other_buildup_goal_count: current
            .other_buildup_goal_count
            .saturating_sub(previous.other_buildup_goal_count),
    }
}

fn goal_ball_air_time_delta(
    current: &GoalBallAirTimeStats,
    previous: &GoalBallAirTimeStats,
) -> GoalBallAirTimeStats {
    GoalBallAirTimeStats {
        goal_ball_air_time_sample_count: current
            .goal_ball_air_time_sample_count
            .saturating_sub(previous.goal_ball_air_time_sample_count),
        cumulative_goal_ball_air_time: current.cumulative_goal_ball_air_time
            - previous.cumulative_goal_ball_air_time,
        last_goal_ball_air_time: optional_delta(
            current.last_goal_ball_air_time,
            previous.last_goal_ball_air_time,
        ),
        goal_ball_air_times: sample_delta(
            &current.goal_ball_air_times,
            &previous.goal_ball_air_times,
        ),
    }
}

fn team_scoring_context_delta(
    current: &TeamScoringContextStats,
    previous: &TeamScoringContextStats,
) -> TeamScoringContextStats {
    TeamScoringContextStats {
        goal_after_kickoff: goal_after_kickoff_delta(
            &current.goal_after_kickoff,
            &previous.goal_after_kickoff,
        ),
        goal_buildup: goal_buildup_delta(&current.goal_buildup, &previous.goal_buildup),
        goal_ball_air_time: goal_ball_air_time_delta(
            &current.goal_ball_air_time,
            &previous.goal_ball_air_time,
        ),
    }
}

fn player_scoring_context_delta(
    current: &PlayerScoringContextStats,
    previous: &PlayerScoringContextStats,
) -> PlayerScoringContextStats {
    PlayerScoringContextStats {
        goals_conceded_while_last_defender: current
            .goals_conceded_while_last_defender
            .saturating_sub(previous.goals_conceded_while_last_defender),
        goals_for_while_most_back: current
            .goals_for_while_most_back
            .saturating_sub(previous.goals_for_while_most_back),
        goals_against_while_most_back: current
            .goals_against_while_most_back
            .saturating_sub(previous.goals_against_while_most_back),
        goal_against_boost_sample_count: current
            .goal_against_boost_sample_count
            .saturating_sub(previous.goal_against_boost_sample_count),
        cumulative_boost_on_goals_against: current.cumulative_boost_on_goals_against
            - previous.cumulative_boost_on_goals_against,
        last_boost_on_goal_against: optional_delta(
            current.last_boost_on_goal_against,
            previous.last_boost_on_goal_against,
        ),
        goal_against_boost_leadup_sample_count: current
            .goal_against_boost_leadup_sample_count
            .saturating_sub(previous.goal_against_boost_leadup_sample_count),
        cumulative_average_boost_in_goal_against_leadup: current
            .cumulative_average_boost_in_goal_against_leadup
            - previous.cumulative_average_boost_in_goal_against_leadup,
        cumulative_min_boost_in_goal_against_leadup: current
            .cumulative_min_boost_in_goal_against_leadup
            - previous.cumulative_min_boost_in_goal_against_leadup,
        last_average_boost_in_goal_against_leadup: optional_delta(
            current.last_average_boost_in_goal_against_leadup,
            previous.last_average_boost_in_goal_against_leadup,
        ),
        last_min_boost_in_goal_against_leadup: optional_delta(
            current.last_min_boost_in_goal_against_leadup,
            previous.last_min_boost_in_goal_against_leadup,
        ),
        goal_against_position_sample_count: current
            .goal_against_position_sample_count
            .saturating_sub(previous.goal_against_position_sample_count),
        cumulative_goal_against_position_x: current.cumulative_goal_against_position_x
            - previous.cumulative_goal_against_position_x,
        cumulative_goal_against_position_y: current.cumulative_goal_against_position_y
            - previous.cumulative_goal_against_position_y,
        cumulative_goal_against_position_z: current.cumulative_goal_against_position_z
            - previous.cumulative_goal_against_position_z,
        last_goal_against_position: optional_delta(
            current.last_goal_against_position,
            previous.last_goal_against_position,
        ),
        scoring_goal_last_touch_position_sample_count: current
            .scoring_goal_last_touch_position_sample_count
            .saturating_sub(previous.scoring_goal_last_touch_position_sample_count),
        cumulative_scoring_goal_last_touch_position_x: current
            .cumulative_scoring_goal_last_touch_position_x
            - previous.cumulative_scoring_goal_last_touch_position_x,
        cumulative_scoring_goal_last_touch_position_y: current
            .cumulative_scoring_goal_last_touch_position_y
            - previous.cumulative_scoring_goal_last_touch_position_y,
        cumulative_scoring_goal_last_touch_position_z: current
            .cumulative_scoring_goal_last_touch_position_z
            - previous.cumulative_scoring_goal_last_touch_position_z,
        last_scoring_goal_last_touch_position: optional_delta(
            current.last_scoring_goal_last_touch_position,
            previous.last_scoring_goal_last_touch_position,
        ),
        goal_after_kickoff: goal_after_kickoff_delta(
            &current.goal_after_kickoff,
            &previous.goal_after_kickoff,
        ),
        goal_buildup: goal_buildup_delta(&current.goal_buildup, &previous.goal_buildup),
        goal_ball_air_time: goal_ball_air_time_delta(
            &current.goal_ball_air_time,
            &previous.goal_ball_air_time,
        ),
    }
}

fn core_player_stats_delta(
    current: &CorePlayerStats,
    previous: &CorePlayerStats,
) -> CorePlayerStats {
    CorePlayerStats {
        score: current.score - previous.score,
        goals: current.goals - previous.goals,
        assists: current.assists - previous.assists,
        saves: current.saves - previous.saves,
        shots: current.shots - previous.shots,
        scoring_context: player_scoring_context_delta(
            &current.scoring_context,
            &previous.scoring_context,
        ),
    }
}

fn core_team_stats_delta(current: &CoreTeamStats, previous: &CoreTeamStats) -> CoreTeamStats {
    CoreTeamStats {
        score: current.score - previous.score,
        goals: current.goals - previous.goals,
        assists: current.assists - previous.assists,
        saves: current.saves - previous.saves,
        shots: current.shots - previous.shots,
        scoring_context: team_scoring_context_delta(
            &current.scoring_context,
            &previous.scoring_context,
        ),
    }
}

fn player_id_sort_key(player_id: &PlayerId) -> String {
    match player_id {
        boxcars::RemoteId::PlayStation(id) => {
            format!("playstation:{}:{}:{:?}", id.online_id, id.name, id.unknown1)
        }
        boxcars::RemoteId::PsyNet(id) => format!("psynet:{}:{:?}", id.online_id, id.unknown1),
        boxcars::RemoteId::SplitScreen(id) => format!("splitscreen:{id}"),
        boxcars::RemoteId::Steam(id) => format!("steam:{id}"),
        boxcars::RemoteId::Switch(id) => format!("switch:{}:{:?}", id.online_id, id.unknown1),
        boxcars::RemoteId::Xbox(id) => format!("xbox:{id}"),
        boxcars::RemoteId::QQ(id) => format!("qq:{id}"),
        boxcars::RemoteId::Epic(id) => format!("epic:{id}"),
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

    pub fn team_zero_stats(&self) -> CoreTeamStats {
        self.team_stats_for_side(true)
    }

    pub fn team_one_stats(&self) -> CoreTeamStats {
        self.team_stats_for_side(false)
    }

    fn team_stats_for_side(&self, is_team_0: bool) -> CoreTeamStats {
        let mut player_stats: Vec<_> = self
            .player_stats
            .iter()
            .filter(|(player_id, _)| self.player_teams.get(*player_id) == Some(&is_team_0))
            .collect();
        player_stats.sort_by_cached_key(|(player_id, _)| player_id_sort_key(player_id));

        let mut stats = player_stats.into_iter().fold(
            CoreTeamStats::default(),
            |mut stats, (_, player_stats)| {
                stats.score += player_stats.score;
                stats.goals += player_stats.goals;
                stats.assists += player_stats.assists;
                stats.saves += player_stats.saves;
                stats.shots += player_stats.shots;
                stats
                    .scoring_context
                    .goal_after_kickoff
                    .merge(&player_stats.scoring_context.goal_after_kickoff);
                stats
                    .scoring_context
                    .goal_buildup
                    .merge(&player_stats.scoring_context.goal_buildup);
                stats
                    .scoring_context
                    .goal_ball_air_time
                    .merge(&player_stats.scoring_context.goal_ball_air_time);
                stats
            },
        );
        stats
            .scoring_context
            .goal_after_kickoff
            .goal_times
            .sort_by(|left, right| left.total_cmp(right));
        stats
            .scoring_context
            .goal_ball_air_time
            .goal_ball_air_times
            .sort_by(|left, right| left.total_cmp(right));
        stats
    }

    fn emit_timeline_events(
        &mut self,
        time: f32,
        frame: Option<usize>,
        kind: TimelineEventKind,
        player_id: &PlayerId,
        is_team_0: bool,
        delta: i32,
    ) {
        for _ in 0..delta.max(0) {
            self.timeline.push(TimelineEvent {
                time,
                frame,
                kind,
                player_id: Some(player_id.clone()),
                is_team_0: Some(is_team_0),
            });
        }
    }

    fn emit_core_stats_events(&mut self, frame: &FrameInfo) {
        let mut player_ids: Vec<_> = self.player_stats.keys().cloned().collect();
        player_ids.sort_by(|left, right| format!("{left:?}").cmp(&format!("{right:?}")));
        for player_id in player_ids {
            let Some(stats) = self.player_stats.get(&player_id) else {
                continue;
            };
            let previous_stats = self
                .last_emitted_player_stats
                .get(&player_id)
                .cloned()
                .unwrap_or_default();
            if previous_stats == *stats {
                continue;
            }
            let Some(is_team_0) = self.player_teams.get(&player_id).copied() else {
                continue;
            };
            self.core_player_events.push(CorePlayerStatsEvent {
                time: frame.time,
                frame: frame.frame_number,
                player: player_id.clone(),
                is_team_0,
                delta: core_player_stats_delta(stats, &previous_stats),
            });
            self.last_emitted_player_stats
                .insert(player_id, stats.clone());
        }

        let team_zero_stats = self.team_zero_stats();
        if team_zero_stats != self.last_emitted_team_zero_stats {
            self.core_team_events.push(CoreTeamStatsEvent {
                time: frame.time,
                frame: frame.frame_number,
                is_team_0: true,
                delta: core_team_stats_delta(&team_zero_stats, &self.last_emitted_team_zero_stats),
            });
            self.last_emitted_team_zero_stats = team_zero_stats;
        }

        let team_one_stats = self.team_one_stats();
        if team_one_stats != self.last_emitted_team_one_stats {
            self.core_team_events.push(CoreTeamStatsEvent {
                time: frame.time,
                frame: frame.frame_number,
                is_team_0: false,
                delta: core_team_stats_delta(&team_one_stats, &self.last_emitted_team_one_stats),
            });
            self.last_emitted_team_one_stats = team_one_stats;
        }
    }

    fn kickoff_phase_active(gameplay: &GameplayState) -> bool {
        gameplay.game_state == Some(GAME_STATE_KICKOFF_COUNTDOWN)
            || gameplay.kickoff_countdown_time.is_some_and(|time| time > 0)
            || gameplay.ball_has_been_hit == Some(false)
    }

    fn update_kickoff_reference(&mut self, gameplay: &GameplayState, events: &FrameEventsState) {
        if let Some(first_touch_time) = events
            .touch_events
            .iter()
            .map(|event| event.time)
            .min_by(|a, b| a.total_cmp(b))
        {
            self.active_kickoff_touch_time = Some(first_touch_time);
            self.kickoff_waiting_for_first_touch = false;
            return;
        }

        if Self::kickoff_phase_active(gameplay) {
            self.kickoff_waiting_for_first_touch = true;
            self.active_kickoff_touch_time = None;
        }
    }

    fn take_pending_goal_event(
        &mut self,
        player_id: &PlayerId,
        is_team_0: bool,
    ) -> Option<PendingGoalEvent> {
        if let Some(index) = self.pending_goal_events.iter().position(|event| {
            event.event.scoring_team_is_team_0 == is_team_0
                && event.event.player.as_ref() == Some(player_id)
        }) {
            return Some(self.pending_goal_events.remove(index));
        }

        self.pending_goal_events
            .iter()
            .position(|event| event.event.scoring_team_is_team_0 == is_team_0)
            .map(|index| self.pending_goal_events.remove(index))
    }

    fn last_defender(
        &self,
        players: &PlayerFrameState,
        defending_team_is_team_0: bool,
    ) -> Option<PlayerId> {
        players
            .players
            .iter()
            .filter(|player| player.is_team_0 == defending_team_is_team_0)
            .filter_map(|player| {
                player
                    .position()
                    .map(|position| (player.player_id.clone(), position.y))
            })
            .reduce(|current, candidate| {
                if defending_team_is_team_0 {
                    if candidate.1 < current.1 {
                        candidate
                    } else {
                        current
                    }
                } else if candidate.1 > current.1 {
                    candidate
                } else {
                    current
                }
            })
            .map(|(player_id, _)| player_id)
    }

    fn most_back_player(players: &PlayerFrameState, team_is_team_0: bool) -> Option<PlayerId> {
        players
            .players
            .iter()
            .filter(|player| player.is_team_0 == team_is_team_0)
            .filter_map(|player| {
                player.position().map(|position| {
                    (
                        player.player_id.clone(),
                        normalized_y(team_is_team_0, position),
                    )
                })
            })
            .min_by(|left, right| left.1.total_cmp(&right.1))
            .map(|(player_id, _)| player_id)
    }

    fn player_position(players: &PlayerFrameState, player_id: &PlayerId) -> Option<glam::Vec3> {
        players
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
            .and_then(PlayerSample::position)
    }

    fn update_last_touch_contexts(
        &mut self,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_events: &[TouchEvent],
    ) {
        let ball_position = ball.position().map(GoalContextPosition::from);
        for touch in touch_events {
            let Some(player_id) = touch.player.clone() else {
                continue;
            };
            let touch_team_most_back_player = Self::most_back_player(players, touch.team_is_team_0);
            let other_team_most_back_player =
                Self::most_back_player(players, !touch.team_is_team_0);
            let touch_players = self.goal_player_contexts(
                players,
                touch.team_is_team_0,
                touch_team_most_back_player.as_ref(),
                other_team_most_back_player.as_ref(),
            );
            self.last_touch_context_by_player.insert(
                player_id.clone(),
                GoalTouchContext {
                    time: touch.time,
                    frame: touch.frame,
                    player: player_id.clone(),
                    is_team_0: touch.team_is_team_0,
                    ball_position,
                    player_position: Self::player_position(players, &player_id)
                        .map(GoalContextPosition::from),
                    players: touch_players,
                },
            );
        }
    }

    fn update_boost_leadup_samples(&mut self, frame: &FrameInfo, players: &PlayerFrameState) {
        let cutoff_time = frame.time - GOAL_CONTEXT_BOOST_LEADUP_SECONDS;
        for player in &players.players {
            let Some(boost_amount) = player.boost_amount.or(player.last_boost_amount) else {
                continue;
            };
            let samples = self
                .boost_leadup_samples_by_player
                .entry(player.player_id.clone())
                .or_default();
            samples.push_back(BoostLeadupSample {
                time: frame.time,
                boost_amount,
            });
            while samples
                .front()
                .is_some_and(|sample| sample.time < cutoff_time)
            {
                samples.pop_front();
            }
        }

        self.boost_leadup_samples_by_player
            .retain(|_, samples| !samples.is_empty());
    }

    fn update_ball_ground_contact(&mut self, frame: &FrameInfo, ball: &BallFrameState) {
        if ball
            .position()
            .is_some_and(|position| position.z <= BALL_GROUND_CONTACT_MAX_Z)
        {
            self.last_ball_ground_contact_time = Some(frame.time);
        }
    }

    fn ball_air_time_before_goal(&self, goal_time: f32) -> Option<f32> {
        self.last_ball_ground_contact_time
            .map(|ground_contact_time| (goal_time - ground_contact_time).max(0.0))
    }

    fn boost_leadup_for_player(&self, player_id: &PlayerId) -> Option<BoostLeadupStats> {
        let samples = self.boost_leadup_samples_by_player.get(player_id)?;
        if samples.is_empty() {
            return None;
        }

        let mut sum = 0.0;
        let mut min_boost = f32::INFINITY;
        for sample in samples {
            sum += sample.boost_amount;
            min_boost = min_boost.min(sample.boost_amount);
        }

        Some(BoostLeadupStats {
            average_boost: sum / samples.len() as f32,
            min_boost,
        })
    }

    fn goal_player_contexts(
        &self,
        players: &PlayerFrameState,
        scoring_team_is_team_0: bool,
        scoring_team_most_back_player: Option<&PlayerId>,
        defending_team_most_back_player: Option<&PlayerId>,
    ) -> Vec<GoalPlayerContext> {
        players
            .players
            .iter()
            .map(|player| {
                let most_back_player = if player.is_team_0 == scoring_team_is_team_0 {
                    scoring_team_most_back_player
                } else {
                    defending_team_most_back_player
                };
                let boost_leadup = self.boost_leadup_for_player(&player.player_id);
                GoalPlayerContext {
                    player: player.player_id.clone(),
                    is_team_0: player.is_team_0,
                    position: player.position().map(GoalContextPosition::from),
                    boost_amount: player.boost_amount.or(player.last_boost_amount),
                    average_boost_in_leadup: boost_leadup.map(|stats| stats.average_boost),
                    min_boost_in_leadup: boost_leadup.map(|stats| stats.min_boost),
                    is_most_back: most_back_player == Some(&player.player_id),
                }
            })
            .collect()
    }

    fn record_goal_context_stats(
        &mut self,
        players: &PlayerFrameState,
        goal_event: &GoalEvent,
        scoring_team_most_back_player: Option<&PlayerId>,
        defending_team_most_back_player: Option<&PlayerId>,
        scorer_last_touch: Option<&GoalTouchContext>,
        ball_air_time_before_goal: Option<f32>,
    ) {
        if let Some(player_id) = scoring_team_most_back_player {
            self.player_stats
                .entry(player_id.clone())
                .or_default()
                .scoring_context
                .goals_for_while_most_back += 1;
        }

        if let Some(player_id) = defending_team_most_back_player {
            self.player_stats
                .entry(player_id.clone())
                .or_default()
                .scoring_context
                .goals_against_while_most_back += 1;
        }

        for player in players
            .players
            .iter()
            .filter(|player| player.is_team_0 != goal_event.scoring_team_is_team_0)
        {
            let boost_leadup = self.boost_leadup_for_player(&player.player_id);
            self.player_stats
                .entry(player.player_id.clone())
                .or_default()
                .scoring_context
                .record_goal_against_snapshot(
                    player.boost_amount.or(player.last_boost_amount),
                    player.position().map(GoalContextPosition::from),
                    boost_leadup,
                );
        }

        if let Some(scorer) = goal_event.player.as_ref() {
            if let Some(touch_position) = scorer_last_touch.and_then(|touch| touch.ball_position) {
                self.player_stats
                    .entry(scorer.clone())
                    .or_default()
                    .scoring_context
                    .record_scoring_goal_last_touch_position(touch_position);
            }
            if let Some(ball_air_time_before_goal) = ball_air_time_before_goal {
                self.player_stats
                    .entry(scorer.clone())
                    .or_default()
                    .scoring_context
                    .record_goal_ball_air_time(ball_air_time_before_goal);
            }
        }
    }

    fn record_goal_context_events(
        &mut self,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
    ) {
        let ball_position = ball.position().map(GoalContextPosition::from);
        for goal_event in &events.goal_events {
            let scoring_team_most_back_player =
                Self::most_back_player(players, goal_event.scoring_team_is_team_0);
            let defending_team_most_back_player =
                Self::most_back_player(players, !goal_event.scoring_team_is_team_0);
            let scorer_last_touch = goal_event
                .player
                .as_ref()
                .and_then(|player_id| self.last_touch_context_by_player.get(player_id))
                .filter(|touch| touch.is_team_0 == goal_event.scoring_team_is_team_0)
                .cloned();
            let ball_air_time_before_goal = self.ball_air_time_before_goal(goal_event.time);
            let goal_buildup =
                self.classify_goal_buildup(goal_event.time, goal_event.scoring_team_is_team_0);

            self.record_goal_context_stats(
                players,
                goal_event,
                scoring_team_most_back_player.as_ref(),
                defending_team_most_back_player.as_ref(),
                scorer_last_touch.as_ref(),
                ball_air_time_before_goal,
            );

            self.goal_context_events.push(GoalContextEvent {
                time: goal_event.time,
                frame: goal_event.frame,
                scoring_team_is_team_0: goal_event.scoring_team_is_team_0,
                scorer: goal_event.player.clone(),
                scoring_team_most_back_player: scoring_team_most_back_player.clone(),
                defending_team_most_back_player: defending_team_most_back_player.clone(),
                ball_position,
                ball_air_time_before_goal,
                goal_buildup,
                scorer_last_touch,
                players: self.goal_player_contexts(
                    players,
                    goal_event.scoring_team_is_team_0,
                    scoring_team_most_back_player.as_ref(),
                    defending_team_most_back_player.as_ref(),
                ),
            });
        }
    }

    fn fill_missing_goal_context_scorer(
        &mut self,
        goal_event: &GoalEvent,
        scorer: &PlayerId,
    ) -> Option<GoalTouchContext> {
        if goal_event.player.is_some() {
            return None;
        }

        let scorer_last_touch = self
            .last_touch_context_by_player
            .get(scorer)
            .filter(|touch| touch.is_team_0 == goal_event.scoring_team_is_team_0)
            .cloned();
        if let Some(context) = self.goal_context_events.iter_mut().rev().find(|context| {
            context.frame == goal_event.frame
                && context.time == goal_event.time
                && context.scoring_team_is_team_0 == goal_event.scoring_team_is_team_0
                && context.scorer.is_none()
        }) {
            context.scorer = Some(scorer.clone());
            context.scorer_last_touch = scorer_last_touch.clone();
        }
        scorer_last_touch
    }

    fn prune_goal_buildup_samples(&mut self, current_time: f32) {
        self.goal_buildup_samples
            .retain(|entry| current_time - entry.time <= GOAL_BUILDUP_LOOKBACK_SECONDS);
        self.goal_buildup_pressure_events
            .retain(|entry| current_time - entry.time <= GOAL_BUILDUP_LOOKBACK_SECONDS);
    }

    fn record_goal_buildup_sample(&mut self, frame: &FrameInfo, ball: &BallFrameState) {
        let Some(ball) = ball.sample() else {
            return;
        };
        if frame.dt <= 0.0 {
            return;
        }
        self.goal_buildup_samples.push(GoalBuildupSample {
            time: frame.time,
            dt: frame.dt,
            ball_y: ball.position().y,
        });
    }

    fn record_goal_buildup_pressure_events(&mut self, events: &FrameEventsState) {
        self.goal_buildup_pressure_events.extend(
            events
                .player_stat_events
                .iter()
                .filter(|event| event.kind == PlayerStatEventKind::Shot)
                .map(|event| GoalBuildupPressureEvent {
                    time: event.time,
                    is_team_0: event.is_team_0,
                }),
        );
    }

    fn classify_goal_buildup(
        &self,
        goal_time: f32,
        scoring_team_is_team_0: bool,
    ) -> GoalBuildupKind {
        let relevant_samples: Vec<_> = self
            .goal_buildup_samples
            .iter()
            .filter(|entry| entry.time <= goal_time)
            .filter(|entry| goal_time - entry.time <= GOAL_BUILDUP_LOOKBACK_SECONDS)
            .collect();
        if relevant_samples.is_empty() {
            return GoalBuildupKind::Other;
        }

        let mut defensive_half_time = 0.0;
        let mut defensive_third_time = 0.0;
        let mut offensive_half_time = 0.0;
        let mut offensive_third_time = 0.0;
        let mut current_attack_time = 0.0;

        for entry in &relevant_samples {
            let normalized_ball_y = if scoring_team_is_team_0 {
                entry.ball_y
            } else {
                -entry.ball_y
            };
            if normalized_ball_y < 0.0 {
                defensive_half_time += entry.dt;
            } else {
                offensive_half_time += entry.dt;
            }
            if normalized_ball_y < -FIELD_ZONE_BOUNDARY_Y {
                defensive_third_time += entry.dt;
            }
            if normalized_ball_y > FIELD_ZONE_BOUNDARY_Y {
                offensive_third_time += entry.dt;
            }
        }

        for entry in relevant_samples.iter().rev() {
            let normalized_ball_y = if scoring_team_is_team_0 {
                entry.ball_y
            } else {
                -entry.ball_y
            };
            if normalized_ball_y > 0.0 {
                current_attack_time += entry.dt;
            } else {
                break;
            }
        }

        let opponent_shot_in_lookback = self.goal_buildup_pressure_events.iter().any(|entry| {
            entry.time <= goal_time
                && goal_time - entry.time <= GOAL_BUILDUP_LOOKBACK_SECONDS
                && entry.is_team_0 != scoring_team_is_team_0
        });
        let has_defensive_pressure_signal = defensive_half_time
            >= COUNTER_ATTACK_MIN_DEFENSIVE_HALF_SECONDS
            || defensive_third_time >= COUNTER_ATTACK_MIN_DEFENSIVE_THIRD_SECONDS
            || opponent_shot_in_lookback;

        if current_attack_time <= COUNTER_ATTACK_MAX_ATTACK_SECONDS && has_defensive_pressure_signal
        {
            GoalBuildupKind::CounterAttack
        } else if current_attack_time >= SUSTAINED_PRESSURE_MIN_ATTACK_SECONDS
            && offensive_half_time >= SUSTAINED_PRESSURE_MIN_OFFENSIVE_HALF_SECONDS
            && offensive_third_time >= SUSTAINED_PRESSURE_MIN_OFFENSIVE_THIRD_SECONDS
        {
            GoalBuildupKind::SustainedPressure
        } else {
            GoalBuildupKind::Other
        }
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
                    assist_fallback_delta,
                );
            }
            if goal_delta > 0 {
                for _ in 0..goal_delta.max(0) {
                    let pending_goal_event =
                        self.take_pending_goal_event(&player.player_id, player.is_team_0);
                    if let Some(pending_goal_event) = pending_goal_event.as_ref() {
                        if pending_goal_event.event.player.is_none() {
                            let scorer_last_touch = self.fill_missing_goal_context_scorer(
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
        self.emit_core_stats_events(frame);

        Ok(())
    }
}

#[cfg(test)]
#[path = "match_stats_tests.rs"]
mod tests;
