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
const DOUBLE_TAP_TOUCH_WINDOW_SECONDS: f32 = 2.5;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct GoalAfterKickoffStats {
    pub kickoff_goal_count: u32,
    pub short_goal_count: u32,
    pub medium_goal_count: u32,
    pub long_goal_count: u32,
    #[serde(skip)]
    goal_times: Vec<f32>,
}

impl GoalAfterKickoffStats {
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

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
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

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct CorePlayerStats {
    pub score: i32,
    pub goals: i32,
    pub assists: i32,
    pub saves: i32,
    pub shots: i32,
    pub attacking_backboard_hit_count: u32,
    pub double_tap_count: u32,
    pub goals_conceded_while_last_defender: u32,
    #[serde(flatten)]
    pub goal_after_kickoff: GoalAfterKickoffStats,
    #[serde(flatten)]
    pub goal_buildup: GoalBuildupStats,
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
        self.goal_after_kickoff.average_goal_time_after_kickoff()
    }

    pub fn median_goal_time_after_kickoff(&self) -> f32 {
        self.goal_after_kickoff.median_goal_time_after_kickoff()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct CoreTeamStats {
    pub score: i32,
    pub goals: i32,
    pub assists: i32,
    pub saves: i32,
    pub shots: i32,
    pub attacking_backboard_hit_count: u32,
    pub double_tap_count: u32,
    #[serde(flatten)]
    pub goal_after_kickoff: GoalAfterKickoffStats,
    #[serde(flatten)]
    pub goal_buildup: GoalBuildupStats,
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
        self.goal_after_kickoff.average_goal_time_after_kickoff()
    }

    pub fn median_goal_time_after_kickoff(&self) -> f32 {
        self.goal_after_kickoff.median_goal_time_after_kickoff()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum TimelineEventKind {
    Goal,
    Shot,
    Save,
    Assist,
    Kill,
    Death,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TimelineEvent {
    pub time: f32,
    pub kind: TimelineEventKind,
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

#[derive(Debug, Clone)]
struct PendingBackboardBounce {
    player_id: PlayerId,
    is_team_0: bool,
    time: f32,
}

#[derive(Debug, Clone, Default)]
pub struct MatchStatsReducer {
    player_stats: HashMap<PlayerId, CorePlayerStats>,
    player_teams: HashMap<PlayerId, bool>,
    previous_player_stats: HashMap<PlayerId, CorePlayerStats>,
    timeline: Vec<TimelineEvent>,
    pending_goal_events: Vec<PendingGoalEvent>,
    previous_team_scores: Option<(i32, i32)>,
    kickoff_waiting_for_first_touch: bool,
    active_kickoff_touch_time: Option<f32>,
    goal_buildup_samples: Vec<GoalBuildupSample>,
    pending_backboard_bounces: Vec<PendingBackboardBounce>,
    live_play_tracker: LivePlayTracker,
}

impl MatchStatsReducer {
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
        self.player_stats
            .iter()
            .filter(|(player_id, _)| self.player_teams.get(*player_id) == Some(&is_team_0))
            .fold(CoreTeamStats::default(), |mut stats, (_, player_stats)| {
                stats.score += player_stats.score;
                stats.goals += player_stats.goals;
                stats.assists += player_stats.assists;
                stats.saves += player_stats.saves;
                stats.shots += player_stats.shots;
                stats.attacking_backboard_hit_count += player_stats.attacking_backboard_hit_count;
                stats.double_tap_count += player_stats.double_tap_count;
                stats
                    .goal_after_kickoff
                    .merge(&player_stats.goal_after_kickoff);
                stats.goal_buildup.merge(&player_stats.goal_buildup);
                stats
            })
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

    fn kickoff_phase_active(sample: &StatsSample) -> bool {
        sample.game_state == Some(GAME_STATE_KICKOFF_COUNTDOWN)
            || sample.kickoff_countdown_time.is_some_and(|time| time > 0)
            || sample.ball_has_been_hit == Some(false)
    }

    fn update_kickoff_reference(&mut self, sample: &StatsSample) {
        if let Some(first_touch_time) = sample
            .touch_events
            .iter()
            .map(|event| event.time)
            .min_by(|a, b| a.total_cmp(b))
        {
            self.active_kickoff_touch_time = Some(first_touch_time);
            self.kickoff_waiting_for_first_touch = false;
            return;
        }

        if Self::kickoff_phase_active(sample) {
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
        sample: &StatsSample,
        defending_team_is_team_0: bool,
    ) -> Option<PlayerId> {
        sample
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

    fn prune_pending_backboard_bounces(&mut self, current_time: f32) {
        self.pending_backboard_bounces
            .retain(|entry| current_time - entry.time <= DOUBLE_TAP_TOUCH_WINDOW_SECONDS);
    }

    fn record_backboard_bounces(&mut self, state: &BackboardBounceState) {
        for event in &state.bounce_events {
            self.player_stats
                .entry(event.player.clone())
                .or_default()
                .attacking_backboard_hit_count += 1;
            if let Some(existing) = self
                .pending_backboard_bounces
                .iter_mut()
                .find(|pending| pending.player_id == event.player)
            {
                *existing = PendingBackboardBounce {
                    player_id: event.player.clone(),
                    is_team_0: event.is_team_0,
                    time: event.time,
                };
            } else {
                self.pending_backboard_bounces.push(PendingBackboardBounce {
                    player_id: event.player.clone(),
                    is_team_0: event.is_team_0,
                    time: event.time,
                });
            }
        }
    }

    fn resolve_double_tap_touches(&mut self, sample: &StatsSample) {
        if sample.touch_events.is_empty() || self.pending_backboard_bounces.is_empty() {
            return;
        }

        let mut completed_players = Vec::new();
        self.pending_backboard_bounces.retain(|pending| {
            if sample.time <= pending.time {
                return true;
            }

            let matching_touch = sample.touch_events.iter().any(|touch| {
                touch.team_is_team_0 == pending.is_team_0
                    && touch.player.as_ref() == Some(&pending.player_id)
            });
            let conflicting_touch = sample
                .touch_events
                .iter()
                .any(|touch| touch.player.as_ref() != Some(&pending.player_id));

            if matching_touch
                && !conflicting_touch
                && Self::followup_touch_is_goal_directed(sample, pending.is_team_0)
            {
                completed_players.push(pending.player_id.clone());
            }
            false
        });

        for player_id in completed_players {
            self.player_stats
                .entry(player_id)
                .or_default()
                .double_tap_count += 1;
        }
    }

    fn followup_touch_is_goal_directed(sample: &StatsSample, is_team_0: bool) -> bool {
        const GOAL_CENTER_Y: f32 = 5120.0;
        const MIN_GOAL_ALIGNMENT_COSINE: f32 = 0.6;

        let Some(ball) = sample.ball.as_ref() else {
            return false;
        };

        let ball_position = ball.position();
        let normalized_position_y = if is_team_0 {
            ball_position.y
        } else {
            -ball_position.y
        };
        let normalized_velocity = ball.velocity();
        let normalized_velocity = glam::Vec2::new(
            normalized_velocity.x,
            if is_team_0 {
                normalized_velocity.y
            } else {
                -normalized_velocity.y
            },
        );
        let goal_direction =
            glam::Vec2::new(-ball_position.x, GOAL_CENTER_Y - normalized_position_y);

        let Some(shot_direction) = normalized_velocity.try_normalize() else {
            return false;
        };
        let Some(goal_direction) = goal_direction.try_normalize() else {
            return false;
        };

        shot_direction.dot(goal_direction) >= MIN_GOAL_ALIGNMENT_COSINE
    }

    fn record_goal_buildup_sample(&mut self, sample: &StatsSample) {
        let Some(ball) = sample.ball.as_ref() else {
            return;
        };
        if sample.dt <= 0.0 {
            return;
        }
        self.goal_buildup_samples.push(GoalBuildupSample {
            time: sample.time,
            dt: sample.dt,
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

impl MatchStatsReducer {
    fn on_sample_internal(
        &mut self,
        sample: &StatsSample,
        backboard_bounce_state: &BackboardBounceState,
    ) -> SubtrActorResult<()> {
        self.update_kickoff_reference(sample);
        let live_play = self.live_play_tracker.is_live_play(sample);
        if !sample.is_live_play() {
            self.pending_backboard_bounces.clear();
        }
        self.prune_goal_buildup_samples(sample.time);
        self.prune_pending_backboard_bounces(sample.time);
        self.record_backboard_bounces(backboard_bounce_state);
        self.resolve_double_tap_touches(sample);
        if live_play {
            self.record_goal_buildup_sample(sample);
        }
        self.pending_goal_events
            .extend(sample.goal_events.iter().cloned().map(|event| {
                PendingGoalEvent {
                    time_after_kickoff: self
                        .active_kickoff_touch_time
                        .map(|kickoff_touch_time| (event.time - kickoff_touch_time).max(0.0)),
                    event,
                }
            }));
        let mut processor_event_counts: HashMap<(PlayerId, TimelineEventKind), i32> =
            HashMap::new();
        for event in &sample.player_stat_events {
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

        for player in &sample.players {
            self.player_teams
                .insert(player.player_id.clone(), player.is_team_0);
            let mut current_stats = CorePlayerStats {
                score: player.match_score.unwrap_or(0),
                goals: player.match_goals.unwrap_or(0),
                assists: player.match_assists.unwrap_or(0),
                saves: player.match_saves.unwrap_or(0),
                shots: player.match_shots.unwrap_or(0),
                attacking_backboard_hit_count: self
                    .player_stats
                    .get(&player.player_id)
                    .map(|stats| stats.attacking_backboard_hit_count)
                    .unwrap_or(0),
                double_tap_count: self
                    .player_stats
                    .get(&player.player_id)
                    .map(|stats| stats.double_tap_count)
                    .unwrap_or(0),
                goals_conceded_while_last_defender: self
                    .player_stats
                    .get(&player.player_id)
                    .map(|stats| stats.goals_conceded_while_last_defender)
                    .unwrap_or(0),
                goal_after_kickoff: self
                    .player_stats
                    .get(&player.player_id)
                    .map(|stats| stats.goal_after_kickoff.clone())
                    .unwrap_or_default(),
                goal_buildup: self
                    .player_stats
                    .get(&player.player_id)
                    .map(|stats| stats.goal_buildup.clone())
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
                    sample.time,
                    TimelineEventKind::Shot,
                    &player.player_id,
                    player.is_team_0,
                    shot_fallback_delta,
                );
            }
            if save_fallback_delta > 0 {
                self.emit_timeline_events(
                    sample.time,
                    TimelineEventKind::Save,
                    &player.player_id,
                    player.is_team_0,
                    save_fallback_delta,
                );
            }
            if assist_fallback_delta > 0 {
                self.emit_timeline_events(
                    sample.time,
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
                        .unwrap_or(sample.time);
                    let time_after_kickoff = pending_goal_event
                        .and_then(|event| event.time_after_kickoff)
                        .or_else(|| {
                            self.active_kickoff_touch_time
                                .map(|kickoff_touch_time| (goal_time - kickoff_touch_time).max(0.0))
                        });
                    if let Some(time_after_kickoff) = time_after_kickoff {
                        current_stats
                            .goal_after_kickoff
                            .record_goal(time_after_kickoff);
                    }
                    current_stats
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
            (sample.team_zero_score, sample.team_one_score)
        {
            if let Some((prev_team_zero_score, prev_team_one_score)) = self.previous_team_scores {
                let team_zero_delta = team_zero_score - prev_team_zero_score;
                let team_one_delta = team_one_score - prev_team_one_score;

                if team_zero_delta > 0 {
                    if let Some(last_defender) = self.last_defender(sample, false) {
                        if let Some(stats) = self.player_stats.get_mut(&last_defender) {
                            stats.goals_conceded_while_last_defender += team_zero_delta as u32;
                        }
                    }
                }

                if team_one_delta > 0 {
                    if let Some(last_defender) = self.last_defender(sample, true) {
                        if let Some(stats) = self.player_stats.get_mut(&last_defender) {
                            stats.goals_conceded_while_last_defender += team_one_delta as u32;
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

impl StatsReducer for MatchStatsReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        self.on_sample_internal(sample, &BackboardBounceState::default())
    }

    fn on_sample_with_context(
        &mut self,
        sample: &StatsSample,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<()> {
        let backboard_bounce_state = ctx
            .get::<BackboardBounceState>(BACKBOARD_BOUNCE_STATE_SIGNAL_ID)
            .cloned()
            .unwrap_or_default();
        self.on_sample_internal(sample, &backboard_bounce_state)
    }
}

#[cfg(test)]
mod tests {
    use boxcars::{Quaternion, RemoteId, RigidBody, Vector3f};

    use super::*;

    fn rigid_body(y: f32) -> RigidBody {
        RigidBody {
            sleeping: false,
            location: Vector3f { x: 0.0, y, z: 17.0 },
            rotation: Quaternion {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
            linear_velocity: Some(Vector3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
            angular_velocity: Some(Vector3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
        }
    }

    fn ball_rigid_body(x: f32, y: f32, z: f32, vx: f32, vy: f32) -> RigidBody {
        RigidBody {
            sleeping: false,
            location: Vector3f { x, y, z },
            rotation: Quaternion {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
            linear_velocity: Some(Vector3f {
                x: vx,
                y: vy,
                z: 0.0,
            }),
            angular_velocity: Some(Vector3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
        }
    }

    fn player(player_id: u64, is_team_0: bool, match_goals: i32) -> PlayerSample {
        PlayerSample {
            player_id: RemoteId::Steam(player_id),
            is_team_0,
            rigid_body: Some(rigid_body(if is_team_0 { -1000.0 } else { 1000.0 })),
            boost_amount: None,
            last_boost_amount: None,
            boost_active: false,
            dodge_active: false,
            powerslide_active: false,
            match_goals: Some(match_goals),
            match_assists: Some(0),
            match_saves: Some(0),
            match_shots: Some(match_goals.max(1)),
            match_score: Some(match_goals * 100),
        }
    }

    fn sample(
        frame_number: usize,
        time: f32,
        dt: f32,
        ball_y: f32,
        team_zero_goals: i32,
        goal_event: Option<GoalEvent>,
    ) -> StatsSample {
        StatsSample {
            frame_number,
            time,
            dt,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: Some(true),
            kickoff_countdown_time: None,
            team_zero_score: Some(team_zero_goals),
            team_one_score: Some(0),
            possession_team_is_team_0: Some(true),
            scored_on_team_is_team_0: goal_event
                .as_ref()
                .map(|event| !event.scoring_team_is_team_0),
            current_in_game_team_player_counts: Some([1, 1]),
            ball: Some(BallSample {
                rigid_body: rigid_body(ball_y),
            }),
            players: vec![player(1, true, team_zero_goals), player(2, false, 0)],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: goal_event.into_iter().collect(),
        }
    }

    fn touch_event(frame: usize, time: f32, player_id: u64, is_team_0: bool) -> TouchEvent {
        TouchEvent {
            time,
            frame,
            team_is_team_0: is_team_0,
            player: Some(RemoteId::Steam(player_id)),
            closest_approach_distance: Some(40.0),
        }
    }

    #[test]
    fn classifies_counter_attack_goals_from_recent_defensive_pressure() {
        let mut reducer = MatchStatsReducer::new();

        for (index, ball_y) in [-4200.0, -4000.0, -3600.0, -3200.0, -2600.0, -1800.0, 1200.0]
            .into_iter()
            .enumerate()
        {
            reducer
                .on_sample(&sample(index, index as f32 + 1.0, 1.0, ball_y, 0, None))
                .unwrap();
        }

        reducer
            .on_sample(&sample(
                8,
                8.0,
                1.0,
                4800.0,
                1,
                Some(GoalEvent {
                    time: 8.0,
                    frame: 8,
                    scoring_team_is_team_0: true,
                    player: Some(RemoteId::Steam(1)),
                    team_zero_score: Some(1),
                    team_one_score: Some(0),
                }),
            ))
            .unwrap();

        let stats = reducer.player_stats().get(&RemoteId::Steam(1)).unwrap();
        assert_eq!(stats.goal_buildup.counter_attack_goal_count, 1);
        assert_eq!(stats.goal_buildup.sustained_pressure_goal_count, 0);
        assert_eq!(stats.goal_buildup.other_buildup_goal_count, 0);
    }

    #[test]
    fn classifies_sustained_pressure_goals_after_long_attacking_spell() {
        let mut reducer = MatchStatsReducer::new();

        for (index, ball_y) in [
            800.0, 1400.0, 2200.0, 2800.0, 3200.0, 3600.0, 4100.0, 4600.0,
        ]
        .into_iter()
        .enumerate()
        {
            reducer
                .on_sample(&sample(index, index as f32 + 1.0, 1.0, ball_y, 0, None))
                .unwrap();
        }

        reducer
            .on_sample(&sample(
                9,
                9.0,
                1.0,
                5000.0,
                1,
                Some(GoalEvent {
                    time: 9.0,
                    frame: 9,
                    scoring_team_is_team_0: true,
                    player: Some(RemoteId::Steam(1)),
                    team_zero_score: Some(1),
                    team_one_score: Some(0),
                }),
            ))
            .unwrap();

        let stats = reducer.player_stats().get(&RemoteId::Steam(1)).unwrap();
        assert_eq!(stats.goal_buildup.counter_attack_goal_count, 0);
        assert_eq!(stats.goal_buildup.sustained_pressure_goal_count, 1);
        assert_eq!(stats.goal_buildup.other_buildup_goal_count, 0);
    }

    #[test]
    fn counts_double_taps_from_backboard_bounces_before_the_followup_touch() {
        let mut reducer = MatchStatsReducer::new();

        let mut setup_touch = sample(1, 1.0, 1.0, 4300.0, 0, None);
        setup_touch.ball = Some(BallSample {
            rigid_body: ball_rigid_body(0.0, 4300.0, 700.0, 0.0, 900.0),
        });
        setup_touch.touch_events = vec![touch_event(1, 1.0, 1, true)];
        reducer.on_sample(&setup_touch).unwrap();

        let bounce_sample = sample(2, 1.2, 0.2, 4800.0, 0, None);
        reducer
            .on_sample_internal(
                &bounce_sample,
                &BackboardBounceState {
                    bounce_events: vec![BackboardBounceEvent {
                        time: 1.2,
                        frame: 2,
                        player: RemoteId::Steam(1),
                        is_team_0: true,
                    }],
                    last_bounce_event: Some(BackboardBounceEvent {
                        time: 1.2,
                        frame: 2,
                        player: RemoteId::Steam(1),
                        is_team_0: true,
                    }),
                },
            )
            .unwrap();

        let mut followup_touch = sample(3, 1.6, 0.4, 4400.0, 0, None);
        followup_touch.ball = Some(BallSample {
            rigid_body: ball_rigid_body(0.0, 4400.0, 700.0, 0.0, 900.0),
        });
        followup_touch.touch_events = vec![touch_event(3, 1.6, 1, true)];
        reducer.on_sample(&followup_touch).unwrap();

        let stats = reducer.player_stats().get(&RemoteId::Steam(1)).unwrap();
        assert_eq!(stats.attacking_backboard_hit_count, 1);
        assert_eq!(stats.double_tap_count, 1);
        assert_eq!(reducer.team_zero_stats().attacking_backboard_hit_count, 1);
        assert_eq!(reducer.team_zero_stats().double_tap_count, 1);
    }

    #[test]
    fn does_not_count_double_taps_when_another_player_touches_first() {
        let mut reducer = MatchStatsReducer::new();

        let mut setup_touch = sample(1, 1.0, 1.0, 4300.0, 0, None);
        setup_touch.ball = Some(BallSample {
            rigid_body: ball_rigid_body(0.0, 4300.0, 700.0, 0.0, 900.0),
        });
        setup_touch.touch_events = vec![touch_event(1, 1.0, 1, true)];
        reducer.on_sample(&setup_touch).unwrap();

        let bounce_sample = sample(2, 1.2, 0.2, 4800.0, 0, None);
        reducer
            .on_sample_internal(
                &bounce_sample,
                &BackboardBounceState {
                    bounce_events: vec![BackboardBounceEvent {
                        time: 1.2,
                        frame: 2,
                        player: RemoteId::Steam(1),
                        is_team_0: true,
                    }],
                    last_bounce_event: Some(BackboardBounceEvent {
                        time: 1.2,
                        frame: 2,
                        player: RemoteId::Steam(1),
                        is_team_0: true,
                    }),
                },
            )
            .unwrap();

        let mut opponent_touch = sample(3, 1.4, 0.2, 4300.0, 0, None);
        opponent_touch.touch_events = vec![touch_event(3, 1.4, 2, false)];
        reducer.on_sample(&opponent_touch).unwrap();

        let mut late_followup = sample(4, 1.8, 0.4, 4200.0, 0, None);
        late_followup.touch_events = vec![touch_event(4, 1.8, 1, true)];
        reducer.on_sample(&late_followup).unwrap();

        let stats = reducer.player_stats().get(&RemoteId::Steam(1)).unwrap();
        assert_eq!(stats.attacking_backboard_hit_count, 1);
        assert_eq!(stats.double_tap_count, 0);
    }

    #[test]
    fn does_not_count_double_taps_when_followup_touch_is_not_goal_directed() {
        let mut reducer = MatchStatsReducer::new();

        let mut setup_touch = sample(1, 1.0, 1.0, 4300.0, 0, None);
        setup_touch.ball = Some(BallSample {
            rigid_body: ball_rigid_body(0.0, 4300.0, 700.0, 0.0, 900.0),
        });
        setup_touch.touch_events = vec![touch_event(1, 1.0, 1, true)];
        reducer.on_sample(&setup_touch).unwrap();

        let bounce_sample = sample(2, 1.2, 0.2, 4800.0, 0, None);
        reducer
            .on_sample_internal(
                &bounce_sample,
                &BackboardBounceState {
                    bounce_events: vec![BackboardBounceEvent {
                        time: 1.2,
                        frame: 2,
                        player: RemoteId::Steam(1),
                        is_team_0: true,
                    }],
                    last_bounce_event: Some(BackboardBounceEvent {
                        time: 1.2,
                        frame: 2,
                        player: RemoteId::Steam(1),
                        is_team_0: true,
                    }),
                },
            )
            .unwrap();

        let mut wide_followup = sample(3, 1.6, 0.4, 4400.0, 0, None);
        wide_followup.ball = Some(BallSample {
            rigid_body: ball_rigid_body(1400.0, 4400.0, 700.0, 1200.0, 150.0),
        });
        wide_followup.touch_events = vec![touch_event(3, 1.6, 1, true)];
        reducer.on_sample(&wide_followup).unwrap();

        let stats = reducer.player_stats().get(&RemoteId::Steam(1)).unwrap();
        assert_eq!(stats.attacking_backboard_hit_count, 1);
        assert_eq!(stats.double_tap_count, 0);
    }
}
