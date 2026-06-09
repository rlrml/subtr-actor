use super::*;

const GOAL_BUILDUP_LOOKBACK_SECONDS: f32 = 12.0;
const COUNTER_ATTACK_MAX_ATTACK_SECONDS: f32 = 4.0;
const COUNTER_ATTACK_MIN_DEFENSIVE_HALF_SECONDS: f32 = 4.0;
const COUNTER_ATTACK_MIN_DEFENSIVE_THIRD_SECONDS: f32 = 1.0;
const SUSTAINED_PRESSURE_MIN_ATTACK_SECONDS: f32 = 6.0;
const SUSTAINED_PRESSURE_MIN_OFFENSIVE_HALF_SECONDS: f32 = 7.0;
const SUSTAINED_PRESSURE_MIN_OFFENSIVE_THIRD_SECONDS: f32 = 3.5;
const GOAL_CONTEXT_BOOST_LEADUP_SECONDS: f32 = 5.0;
const BALL_GROUND_CONTACT_MAX_Z: f32 = BALL_RADIUS_Z + 5.0;
// On the frame a goal is recorded, the ball's rigid body is the goal explosion
// rather than a normal physics update, so its interpolated velocity reads as
// zero. We carry forward the most recent meaningful ball velocity (anything
// above this threshold, in uu/s) so `ball_speed_at_goal` reflects the speed of
// the shot as it crossed the line instead of a spurious 0.
const MIN_TRACKED_BALL_SPEED: f32 = 1.0;
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GoalBuildupKind {
    CounterAttack,
    SustainedPressure,
    #[default]
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct CorePlayerScoreboardEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub score_delta: i32,
    pub goals_delta: i32,
    pub assists_delta: i32,
    pub saves_delta: i32,
    pub shots_delta: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct CorePlayerGoalContextEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub scoring_team_is_team_0: bool,
    pub goals_conceded_while_last_defender: bool,
    pub goals_for_while_most_back: bool,
    pub goals_against_while_most_back: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_against_boost_amount: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_against_average_boost_in_leadup: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_against_min_boost_in_leadup: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_against_position: Option<GoalContextPosition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scoring_goal_last_touch_position: Option<GoalContextPosition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_after_kickoff: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_buildup: Option<GoalBuildupKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ball_air_time_before_goal: Option<f32>,
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
    #[ts(as = "Option<crate::interop::ts_bindings::RemoteIdTs>")]
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
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
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
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
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
    #[ts(as = "Option<crate::interop::ts_bindings::RemoteIdTs>")]
    pub scorer: Option<PlayerId>,
    #[ts(as = "Option<crate::interop::ts_bindings::RemoteIdTs>")]
    pub scoring_team_most_back_player: Option<PlayerId>,
    #[ts(as = "Option<crate::interop::ts_bindings::RemoteIdTs>")]
    pub defending_team_most_back_player: Option<PlayerId>,
    pub ball_position: Option<GoalContextPosition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ball_speed_at_goal: Option<f32>,
    pub ball_air_time_before_goal: Option<f32>,
    /// How long the scoring team's established territorial-pressure session had
    /// been running when the goal was scored (goal_time - session.start_time).
    /// None for goals scored with no active pressure session (e.g. clean
    /// counter-attacks). Filled in at finish once pressure sessions are final.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pressure_duration_before_goal: Option<f32>,
    #[serde(default)]
    pub goal_buildup: GoalBuildupKind,
    pub scorer_last_touch: Option<GoalTouchContext>,
    pub players: Vec<GoalPlayerContext>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<GoalTag>,
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
struct BoostLeadupSummary {
    average_boost: f32,
    min_boost: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct PlayerScoreboardSample {
    score: i32,
    goals: i32,
    assists: i32,
    saves: i32,
    shots: i32,
}

impl PlayerScoreboardSample {
    fn from_player(player: &PlayerSample) -> Self {
        Self {
            score: player.match_score.unwrap_or(0),
            goals: player.match_goals.unwrap_or(0),
            assists: player.match_assists.unwrap_or(0),
            saves: player.match_saves.unwrap_or(0),
            shots: player.match_shots.unwrap_or(0),
        }
    }

    fn delta_from(self, previous: Self) -> Self {
        Self {
            score: self.score - previous.score,
            goals: self.goals - previous.goals,
            assists: self.assists - previous.assists,
            saves: self.saves - previous.saves,
            shots: self.shots - previous.shots,
        }
    }

    fn is_zero(self) -> bool {
        self == Self::default()
    }
}

#[derive(Debug, Clone, Default)]
pub struct MatchStatsCalculator {
    previous_player_scoreboard_samples: HashMap<PlayerId, PlayerScoreboardSample>,
    core_player_scoreboard_events: EventStream<CorePlayerScoreboardEvent>,
    core_player_goal_context_events: EventStream<CorePlayerGoalContextEvent>,
    timeline: EventStream<TimelineEvent>,
    pending_goal_events: Vec<PendingGoalEvent>,
    previous_team_scores: Option<(i32, i32)>,
    kickoff_waiting_for_first_touch: bool,
    active_kickoff_touch_time: Option<f32>,
    goal_buildup_samples: Vec<GoalBuildupSample>,
    goal_buildup_pressure_events: Vec<GoalBuildupPressureEvent>,
    goal_context_events: EventStream<GoalContextEvent>,
    last_touch_context_by_player: HashMap<PlayerId, GoalTouchContext>,
    boost_leadup_samples_by_player: HashMap<PlayerId, VecDeque<BoostLeadupSample>>,
    last_ball_ground_contact_time: Option<f32>,
    last_ball_velocity: Option<glam::Vec3>,
}

impl MatchStatsCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn timeline(&self) -> &[TimelineEvent] {
        self.timeline.all()
    }

    pub fn new_timeline_events(&self) -> &[TimelineEvent] {
        self.timeline.new_events()
    }

    pub fn goal_context_events(&self) -> &[GoalContextEvent] {
        self.goal_context_events.all()
    }

    pub fn new_goal_context_events(&self) -> &[GoalContextEvent] {
        self.goal_context_events.new_events()
    }

    pub fn core_player_events(&self) -> &[CorePlayerScoreboardEvent] {
        self.core_player_scoreboard_events.all()
    }

    pub fn new_core_player_events(&self) -> &[CorePlayerScoreboardEvent] {
        self.core_player_scoreboard_events.new_events()
    }

    pub fn core_player_goal_context_events(&self) -> &[CorePlayerGoalContextEvent] {
        self.core_player_goal_context_events.all()
    }

    pub fn new_core_player_goal_context_events(&self) -> &[CorePlayerGoalContextEvent] {
        self.core_player_goal_context_events.new_events()
    }

    pub fn finish(&mut self) -> SubtrActorResult<()> {
        self.timeline.begin_update();
        self.goal_context_events.begin_update();
        self.core_player_scoreboard_events.begin_update();
        self.core_player_goal_context_events.begin_update();
        let pending_goal_events = std::mem::take(&mut self.pending_goal_events);
        for pending_goal_event in pending_goal_events {
            let Some(scorer) = pending_goal_event.event.player.clone() else {
                continue;
            };
            let scorer_last_touch =
                self.reconcile_goal_context_scorer(&pending_goal_event.event, &scorer);
            self.emit_core_player_scoreboard_event_parts(
                pending_goal_event.event.time,
                pending_goal_event.event.frame,
                scorer.clone(),
                pending_goal_event
                    .event
                    .player_position
                    .map(|position| vec_to_glam(&position).to_array()),
                pending_goal_event.event.scoring_team_is_team_0,
                PlayerScoreboardSample {
                    goals: 1,
                    ..PlayerScoreboardSample::default()
                },
            );
            self.emit_scoring_goal_context_event(
                pending_goal_event.event.time,
                pending_goal_event.event.frame,
                scorer.clone(),
                pending_goal_event
                    .event
                    .player_position
                    .map(|position| vec_to_glam(&position).to_array()),
                pending_goal_event.event.scoring_team_is_team_0,
                scorer_last_touch.and_then(|touch| touch.ball_position),
                pending_goal_event.time_after_kickoff,
                pending_goal_event.goal_buildup,
                pending_goal_event.ball_air_time_before_goal,
            );

            self.timeline.push(TimelineEvent {
                time: pending_goal_event.event.time,
                frame: Some(pending_goal_event.event.frame),
                kind: TimelineEventKind::Goal,
                player_position: pending_goal_event
                    .event
                    .player_position
                    .map(|position| vec_to_glam(&position).to_array()),
                player_id: Some(scorer),
                is_team_0: Some(pending_goal_event.event.scoring_team_is_team_0),
            });
        }

        self.timeline.sort_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_timeline_events(
        &mut self,
        time: f32,
        frame: Option<usize>,
        kind: TimelineEventKind,
        player_id: &PlayerId,
        is_team_0: bool,
        player_position: Option<[f32; 3]>,
        delta: i32,
    ) {
        for _ in 0..delta.max(0) {
            self.timeline.push(TimelineEvent {
                time,
                frame,
                kind,
                player_id: Some(player_id.clone()),
                player_position,
                is_team_0: Some(is_team_0),
            });
        }
    }

    fn emit_core_player_scoreboard_event(
        &mut self,
        frame: &FrameInfo,
        player: PlayerId,
        player_position: Option<[f32; 3]>,
        is_team_0: bool,
        delta: PlayerScoreboardSample,
    ) {
        self.emit_core_player_scoreboard_event_parts(
            frame.time,
            frame.frame_number,
            player,
            player_position,
            is_team_0,
            delta,
        );
    }

    fn emit_core_player_scoreboard_event_parts(
        &mut self,
        time: f32,
        frame: usize,
        player: PlayerId,
        player_position: Option<[f32; 3]>,
        is_team_0: bool,
        delta: PlayerScoreboardSample,
    ) {
        if delta.is_zero() {
            return;
        }
        let event = CorePlayerScoreboardEvent {
            time,
            frame,
            player,
            player_position,
            is_team_0,
            score_delta: delta.score,
            goals_delta: delta.goals,
            assists_delta: delta.assists,
            saves_delta: delta.saves,
            shots_delta: delta.shots,
        };
        self.core_player_scoreboard_events.push(event);
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_core_player_goal_context_event(
        &mut self,
        time: f32,
        frame: usize,
        player: PlayerId,
        player_position: Option<[f32; 3]>,
        is_team_0: bool,
        scoring_team_is_team_0: bool,
        goals_conceded_while_last_defender: bool,
        goals_for_while_most_back: bool,
        goals_against_while_most_back: bool,
        goal_against_boost_amount: Option<f32>,
        goal_against_boost_leadup: Option<(f32, f32)>,
        goal_against_position: Option<GoalContextPosition>,
        scoring_goal_last_touch_position: Option<GoalContextPosition>,
        time_after_kickoff: Option<f32>,
        goal_buildup: Option<GoalBuildupKind>,
        ball_air_time_before_goal: Option<f32>,
    ) {
        self.core_player_goal_context_events
            .push(CorePlayerGoalContextEvent {
                time,
                frame,
                player,
                player_position,
                is_team_0,
                scoring_team_is_team_0,
                goals_conceded_while_last_defender,
                goals_for_while_most_back,
                goals_against_while_most_back,
                goal_against_boost_amount,
                goal_against_average_boost_in_leadup: goal_against_boost_leadup
                    .map(|leadup| leadup.0),
                goal_against_min_boost_in_leadup: goal_against_boost_leadup.map(|leadup| leadup.1),
                goal_against_position,
                scoring_goal_last_touch_position,
                time_after_kickoff,
                goal_buildup,
                ball_air_time_before_goal,
            });
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_scoring_goal_context_event(
        &mut self,
        time: f32,
        frame: usize,
        player: PlayerId,
        player_position: Option<[f32; 3]>,
        is_team_0: bool,
        scoring_goal_last_touch_position: Option<GoalContextPosition>,
        time_after_kickoff: Option<f32>,
        goal_buildup: GoalBuildupKind,
        ball_air_time_before_goal: Option<f32>,
    ) {
        self.emit_core_player_goal_context_event(
            time,
            frame,
            player,
            player_position,
            is_team_0,
            is_team_0,
            false,
            false,
            false,
            None,
            None,
            None,
            scoring_goal_last_touch_position,
            time_after_kickoff,
            Some(goal_buildup),
            ball_air_time_before_goal,
        );
    }

    fn kickoff_phase_active(gameplay: &GameplayState) -> bool {
        gameplay.kickoff_phase_active()
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
        let ball_speed_after_touch = ball.velocity().map(|velocity| velocity.length());
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
                    ball_speed_after_touch,
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

    fn update_ball_velocity(&mut self, ball: &BallFrameState) {
        if let Some(velocity) = ball.velocity() {
            if velocity.length() >= MIN_TRACKED_BALL_SPEED {
                self.last_ball_velocity = Some(velocity);
            }
        }
    }

    // Speed of the ball as it crossed the goal line. The explosion frame itself
    // carries no usable velocity, so fall back to the most recent tracked
    // velocity from just before the goal.
    fn ball_speed_at_goal(&self, ball: &BallFrameState) -> Option<f32> {
        ball.velocity()
            .filter(|velocity| velocity.length() >= MIN_TRACKED_BALL_SPEED)
            .or(self.last_ball_velocity)
            .map(|velocity| velocity.length())
    }

    fn ball_air_time_before_goal(&self, goal_time: f32) -> Option<f32> {
        self.last_ball_ground_contact_time
            .map(|ground_contact_time| (goal_time - ground_contact_time).max(0.0))
    }

    fn boost_leadup_for_player(&self, player_id: &PlayerId) -> Option<BoostLeadupSummary> {
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

        Some(BoostLeadupSummary {
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
    ) {
        if let Some(player_id) = scoring_team_most_back_player {
            self.emit_core_player_goal_context_event(
                goal_event.time,
                goal_event.frame,
                player_id.clone(),
                players.player_position(player_id),
                goal_event.scoring_team_is_team_0,
                goal_event.scoring_team_is_team_0,
                false,
                true,
                false,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            );
        }

        if let Some(player_id) = defending_team_most_back_player {
            self.emit_core_player_goal_context_event(
                goal_event.time,
                goal_event.frame,
                player_id.clone(),
                players.player_position(player_id),
                !goal_event.scoring_team_is_team_0,
                goal_event.scoring_team_is_team_0,
                false,
                false,
                true,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            );
        }

        for player in players
            .players
            .iter()
            .filter(|player| player.is_team_0 != goal_event.scoring_team_is_team_0)
        {
            let boost_leadup = self
                .boost_leadup_for_player(&player.player_id)
                .map(|stats| (stats.average_boost, stats.min_boost));
            self.emit_core_player_goal_context_event(
                goal_event.time,
                goal_event.frame,
                player.player_id.clone(),
                player.position().map(|position| position.to_array()),
                player.is_team_0,
                goal_event.scoring_team_is_team_0,
                false,
                false,
                false,
                player.boost_amount.or(player.last_boost_amount),
                boost_leadup,
                player.position().map(GoalContextPosition::from),
                None,
                None,
                None,
                None,
            );
        }
    }

    fn record_goal_context_events(
        &mut self,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
    ) {
        let ball_position = ball.position().map(GoalContextPosition::from);
        let ball_speed_at_goal = self.ball_speed_at_goal(ball);
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
            );

            self.goal_context_events.push(GoalContextEvent {
                time: goal_event.time,
                frame: goal_event.frame,
                scoring_team_is_team_0: goal_event.scoring_team_is_team_0,
                scorer: goal_event.player.clone(),
                scoring_team_most_back_player: scoring_team_most_back_player.clone(),
                defending_team_most_back_player: defending_team_most_back_player.clone(),
                ball_position,
                ball_speed_at_goal,
                ball_air_time_before_goal,
                // Filled in at finish once territorial-pressure sessions are final.
                pressure_duration_before_goal: None,
                goal_buildup,
                scorer_last_touch,
                players: self.goal_player_contexts(
                    players,
                    goal_event.scoring_team_is_team_0,
                    scoring_team_most_back_player.as_ref(),
                    defending_team_most_back_player.as_ref(),
                ),
                tags: Vec::new(),
            });
        }
    }

    fn reconcile_goal_context_scorer(
        &mut self,
        goal_event: &GoalEvent,
        scorer: &PlayerId,
    ) -> Option<GoalTouchContext> {
        let scorer_last_touch = self
            .last_touch_context_by_player
            .get(scorer)
            .filter(|touch| touch.is_team_0 == goal_event.scoring_team_is_team_0)
            .cloned();
        if let Some(context) = self.goal_context_events.iter_mut().rev().find(|context| {
            context.frame == goal_event.frame
                && context.time == goal_event.time
                && context.scoring_team_is_team_0 == goal_event.scoring_team_is_team_0
                && context.scorer.as_ref() != Some(scorer)
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

    /// Attach to each recorded goal how long the scoring team's established
    /// territorial-pressure session had been running when the goal was scored
    /// (`goal_time - session.start_time`). The session is the scoring team's
    /// pressure session that spans the goal frame. Goals scored with no active
    /// pressure session (e.g. clean counter-attacks) are left as None.
    ///
    /// Runs at finish, after territorial-pressure sessions are finalized, so it
    /// must be fed the pressure calculator's projected (final) sessions.
    pub fn attach_goal_pressure_durations(
        &mut self,
        pressure_sessions: &[TerritorialPressureEvent],
    ) {
        for goal in self.goal_context_events.iter_mut() {
            let goal_time = goal.time;
            let scoring_team_is_team_0 = goal.scoring_team_is_team_0;
            goal.pressure_duration_before_goal = pressure_sessions
                .iter()
                .find(|session| {
                    session.team_is_team_0 == scoring_team_is_team_0
                        && session.start_time <= goal_time
                        && goal_time <= session.end_time
                })
                .map(|session| (goal_time - session.start_time).max(0.0));
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
        self.timeline.begin_update();
        self.goal_context_events.begin_update();
        self.core_player_scoreboard_events.begin_update();
        self.core_player_goal_context_events.begin_update();
        self.update_kickoff_reference(gameplay, events);
        self.prune_goal_buildup_samples(frame.time);
        self.update_ball_ground_contact(frame, ball);
        self.update_ball_velocity(ball);
        if live_play_state.is_live_play {
            self.record_goal_buildup_sample(frame, ball);
            self.record_goal_buildup_pressure_events(events);
            self.update_boost_leadup_samples(frame, players);
        } else if events.goal_events.is_empty() {
            self.last_touch_context_by_player.clear();
            self.boost_leadup_samples_by_player.clear();
            self.last_ball_ground_contact_time = None;
            self.last_ball_velocity = None;
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
            let current_stats = PlayerScoreboardSample::from_player(player);

            let previous_stats = self
                .previous_player_scoreboard_samples
                .get(&player.player_id)
                .copied()
                .unwrap_or_default();
            let delta_stats = current_stats.delta_from(previous_stats);

            let shot_delta = delta_stats.shots;
            let save_delta = delta_stats.saves;
            let assist_delta = delta_stats.assists;
            let goal_delta = delta_stats.goals;
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
                    let (
                        goal_time,
                        goal_frame,
                        time_after_kickoff,
                        goal_buildup,
                        ball_air_time_before_goal,
                        scoring_goal_last_touch_position,
                    ) = if let Some(pending_goal_event) = pending_goal_event.as_ref() {
                        let scorer_last_touch = self.reconcile_goal_context_scorer(
                            &pending_goal_event.event,
                            &player.player_id,
                        );
                        (
                            pending_goal_event.event.time,
                            pending_goal_event.event.frame,
                            pending_goal_event.time_after_kickoff,
                            pending_goal_event.goal_buildup,
                            pending_goal_event.ball_air_time_before_goal,
                            scorer_last_touch.and_then(|touch| touch.ball_position),
                        )
                    } else {
                        let goal_time = frame.time;
                        (
                            goal_time,
                            frame.frame_number,
                            self.active_kickoff_touch_time.map(|kickoff_touch_time| {
                                (goal_time - kickoff_touch_time).max(0.0)
                            }),
                            self.classify_goal_buildup(goal_time, player.is_team_0),
                            self.ball_air_time_before_goal(goal_time),
                            None,
                        )
                    };
                    self.emit_scoring_goal_context_event(
                        goal_time,
                        goal_frame,
                        player.player_id.clone(),
                        player.position().map(|position| position.to_array()),
                        player.is_team_0,
                        scoring_goal_last_touch_position,
                        time_after_kickoff,
                        goal_buildup,
                        ball_air_time_before_goal,
                    );
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

            self.previous_player_scoreboard_samples
                .insert(player.player_id.clone(), current_stats);
            self.emit_core_player_scoreboard_event(
                frame,
                player.player_id.clone(),
                player.position().map(|position| position.to_array()),
                player.is_team_0,
                delta_stats,
            );
        }

        if let (Some(team_zero_score), Some(team_one_score)) =
            (gameplay.team_zero_score, gameplay.team_one_score)
        {
            if let Some((prev_team_zero_score, prev_team_one_score)) = self.previous_team_scores {
                let team_zero_delta = team_zero_score - prev_team_zero_score;
                let team_one_delta = team_one_score - prev_team_one_score;

                if team_zero_delta > 0 {
                    if let Some(last_defender) = self.last_defender(players, false) {
                        let player_position = players.player_position(&last_defender);
                        for _ in 0..team_zero_delta {
                            self.emit_core_player_goal_context_event(
                                frame.time,
                                frame.frame_number,
                                last_defender.clone(),
                                player_position,
                                false,
                                true,
                                true,
                                false,
                                false,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                            );
                        }
                    }
                }

                if team_one_delta > 0 {
                    if let Some(last_defender) = self.last_defender(players, true) {
                        let player_position = players.player_position(&last_defender);
                        for _ in 0..team_one_delta {
                            self.emit_core_player_goal_context_event(
                                frame.time,
                                frame.frame_number,
                                last_defender.clone(),
                                player_position,
                                true,
                                false,
                                true,
                                false,
                                false,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                            );
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
        Ok(())
    }
}

#[cfg(test)]
#[path = "match_stats_tests.rs"]
mod tests;
