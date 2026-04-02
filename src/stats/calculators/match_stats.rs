use super::*;

const GOAL_AFTER_KICKOFF_BUCKET_KICKOFF_MAX_SECONDS: f32 = 10.0;
const GOAL_AFTER_KICKOFF_BUCKET_SHORT_MAX_SECONDS: f32 = 20.0;
const GOAL_AFTER_KICKOFF_BUCKET_MEDIUM_MAX_SECONDS: f32 = 40.0;
const GOAL_BUILDUP_LOOKBACK_SECONDS: f32 = 12.0;
const COUNTER_ATTACK_MAX_ATTACK_SECONDS: f32 = 4.0;
const COUNTER_ATTACK_MIN_DEFENSIVE_HALF_SECONDS: f32 = 6.0;
const COUNTER_ATTACK_MIN_DEFENSIVE_THIRD_SECONDS: f32 = 2.5;
const SUSTAINED_PRESSURE_MIN_ATTACK_SECONDS: f32 = 6.0;
const SUSTAINED_PRESSURE_MIN_OFFENSIVE_HALF_SECONDS: f32 = 7.0;
const SUSTAINED_PRESSURE_MIN_OFFENSIVE_THIRD_SECONDS: f32 = 3.5;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GoalBuildupKind {
    CounterAttack,
    SustainedPressure,
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
    #[serde(flatten)]
    pub goal_after_kickoff: GoalAfterKickoffStats,
    #[serde(flatten)]
    pub goal_buildup: GoalBuildupStats,
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
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct TeamScoringContextStats {
    #[serde(flatten)]
    pub goal_after_kickoff: GoalAfterKickoffStats,
    #[serde(flatten)]
    pub goal_buildup: GoalBuildupStats,
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
    pub kind: TimelineEventKind,
    #[ts(as = "Option<crate::ts_bindings::RemoteIdTs>")]
    pub player_id: Option<PlayerId>,
    pub is_team_0: Option<bool>,
}

#[derive(Debug, Clone)]
struct PendingGoalEvent {
    event: GoalEvent,
    time_after_kickoff: Option<f32>,
}

#[derive(Debug, Clone)]
struct GoalBuildupSample {
    time: f32,
    dt: f32,
    ball_y: f32,
}

#[derive(Debug, Clone, Default)]
pub struct MatchStatsCalculator {
    player_stats: HashMap<PlayerId, CorePlayerStats>,
    player_teams: HashMap<PlayerId, bool>,
    previous_player_stats: HashMap<PlayerId, CorePlayerStats>,
    timeline: Vec<TimelineEvent>,
    pending_goal_events: Vec<PendingGoalEvent>,
    previous_team_scores: Option<(i32, i32)>,
    kickoff_waiting_for_first_touch: bool,
    active_kickoff_touch_time: Option<f32>,
    goal_buildup_samples: Vec<GoalBuildupSample>,
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

    pub fn team_zero_stats(&self) -> CoreTeamStats {
        self.team_stats_for_side(true)
    }

    pub fn team_one_stats(&self) -> CoreTeamStats {
        self.team_stats_for_side(false)
    }

    fn team_stats_for_side(&self, is_team_0: bool) -> CoreTeamStats {
        let mut stats = self
            .player_stats
            .iter()
            .filter(|(player_id, _)| self.player_teams.get(*player_id) == Some(&is_team_0))
            .fold(CoreTeamStats::default(), |mut stats, (_, player_stats)| {
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
            });
        stats
            .scoring_context
            .goal_after_kickoff
            .goal_times
            .sort_by(|left, right| left.total_cmp(right));
        stats
    }

    fn emit_timeline_events(
        &mut self,
        time: f32,
        kind: TimelineEventKind,
        player_id: &PlayerId,
        is_team_0: bool,
        delta: i32,
    ) {
        for _ in 0..delta.max(0) {
            self.timeline.push(TimelineEvent {
                time,
                kind,
                player_id: Some(player_id.clone()),
                is_team_0: Some(is_team_0),
            });
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

    fn prune_goal_buildup_samples(&mut self, current_time: f32) {
        self.goal_buildup_samples
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

    fn classify_goal_buildup(
        &self,
        goal_time: f32,
        scoring_team_is_team_0: bool,
    ) -> GoalBuildupKind {
        let relevant_samples: Vec<_> = self
            .goal_buildup_samples
            .iter()
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

        if current_attack_time <= COUNTER_ATTACK_MAX_ATTACK_SECONDS
            && defensive_half_time >= COUNTER_ATTACK_MIN_DEFENSIVE_HALF_SECONDS
            && defensive_third_time >= COUNTER_ATTACK_MIN_DEFENSIVE_THIRD_SECONDS
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
    pub fn update_parts(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        self.update_kickoff_reference(gameplay, events);
        self.prune_goal_buildup_samples(frame.time);
        if live_play {
            self.record_goal_buildup_sample(frame, ball);
        }
        self.pending_goal_events
            .extend(events.goal_events.iter().cloned().map(|event| {
                PendingGoalEvent {
                    time_after_kickoff: self
                        .active_kickoff_touch_time
                        .map(|kickoff_touch_time| (event.time - kickoff_touch_time).max(0.0)),
                    event,
                }
            }));
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
                    TimelineEventKind::Shot,
                    &player.player_id,
                    player.is_team_0,
                    shot_fallback_delta,
                );
            }
            if save_fallback_delta > 0 {
                self.emit_timeline_events(
                    frame.time,
                    TimelineEventKind::Save,
                    &player.player_id,
                    player.is_team_0,
                    save_fallback_delta,
                );
            }
            if assist_fallback_delta > 0 {
                self.emit_timeline_events(
                    frame.time,
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
                    let goal_time = pending_goal_event
                        .as_ref()
                        .map(|event| event.event.time)
                        .unwrap_or(frame.time);
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
                        .record(self.classify_goal_buildup(goal_time, player.is_team_0));
                    self.timeline.push(TimelineEvent {
                        time: goal_time,
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

        Ok(())
    }
}
