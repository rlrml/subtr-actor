use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct CorePlayerStats {
    pub score: i32,
    pub goals: i32,
    pub assists: i32,
    pub saves: i32,
    pub shots: i32,
    pub goals_conceded_while_last_defender: u32,
}

impl CorePlayerStats {
    pub fn shooting_percentage(&self) -> f32 {
        if self.shots == 0 {
            0.0
        } else {
            self.goals as f32 * 100.0 / self.shots as f32
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct CoreTeamStats {
    pub score: i32,
    pub goals: i32,
    pub assists: i32,
    pub saves: i32,
    pub shots: i32,
}

impl CoreTeamStats {
    pub fn shooting_percentage(&self) -> f32 {
        if self.shots == 0 {
            0.0
        } else {
            self.goals as f32 * 100.0 / self.shots as f32
        }
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

#[derive(Debug, Clone, Default)]
pub struct MatchStatsReducer {
    player_stats: HashMap<PlayerId, CorePlayerStats>,
    player_teams: HashMap<PlayerId, bool>,
    previous_player_stats: HashMap<PlayerId, CorePlayerStats>,
    timeline: Vec<TimelineEvent>,
    pending_goal_events: Vec<GoalEvent>,
    previous_team_scores: Option<(i32, i32)>,
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

    fn take_goal_event_time(&mut self, player_id: &PlayerId, is_team_0: bool) -> Option<f32> {
        if let Some(index) = self.pending_goal_events.iter().position(|event| {
            event.scoring_team_is_team_0 == is_team_0 && event.player.as_ref() == Some(player_id)
        }) {
            return Some(self.pending_goal_events.remove(index).time);
        }

        self.pending_goal_events
            .iter()
            .position(|event| event.scoring_team_is_team_0 == is_team_0)
            .map(|index| self.pending_goal_events.remove(index).time)
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
}

impl StatsReducer for MatchStatsReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        self.pending_goal_events
            .extend(sample.goal_events.iter().cloned());
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
            let current_stats = CorePlayerStats {
                score: player.match_score.unwrap_or(0),
                goals: player.match_goals.unwrap_or(0),
                assists: player.match_assists.unwrap_or(0),
                saves: player.match_saves.unwrap_or(0),
                shots: player.match_shots.unwrap_or(0),
                goals_conceded_while_last_defender: self
                    .player_stats
                    .get(&player.player_id)
                    .map(|stats| stats.goals_conceded_while_last_defender)
                    .unwrap_or(0),
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
                    let goal_time = self
                        .take_goal_event_time(&player.player_id, player.is_team_0)
                        .unwrap_or(sample.time);
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
