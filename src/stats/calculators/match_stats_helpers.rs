use super::*;

pub(super) fn optional_delta<T: Copy + PartialEq>(
    current: Option<T>,
    previous: Option<T>,
) -> Option<T> {
    if current == previous {
        None
    } else {
        current
    }
}

pub(super) fn sample_delta<T: Copy + PartialEq>(current: &[T], previous: &[T]) -> Vec<T> {
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

pub(super) fn goal_after_kickoff_delta(
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

pub(super) fn goal_buildup_delta(
    current: &GoalBuildupStats,
    previous: &GoalBuildupStats,
) -> GoalBuildupStats {
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

pub(super) fn goal_ball_air_time_delta(
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

pub(super) fn team_scoring_context_delta(
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

pub(super) fn player_scoring_context_delta(
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

pub(super) fn core_player_stats_delta(
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

pub(super) fn core_team_stats_delta(
    current: &CoreTeamStats,
    previous: &CoreTeamStats,
) -> CoreTeamStats {
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

pub(super) fn player_id_sort_key(player_id: &PlayerId) -> String {
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

impl MatchStatsCalculator {
    pub fn finish(&mut self) -> SubtrActorResult<()> {
        self.timeline.begin_update();
        self.goal_context_events.begin_update();
        self.core_player_events.begin_update();
        self.core_team_events.begin_update();
        let pending_goal_events = std::mem::take(&mut self.pending_goal_events);
        for pending_goal_event in pending_goal_events {
            let Some(scorer) = pending_goal_event.event.player.clone() else {
                continue;
            };
            let scorer_last_touch =
                self.reconcile_goal_context_scorer(&pending_goal_event.event, &scorer);
            let scorer_stats = self.player_stats.entry(scorer.clone()).or_default();
            scorer_stats.goals += 1;
            if let Some(touch_position) = scorer_last_touch.and_then(|touch| touch.ball_position) {
                scorer_stats
                    .scoring_context
                    .record_scoring_goal_last_touch_position(touch_position);
            }
            if let Some(time_after_kickoff) = pending_goal_event.time_after_kickoff {
                scorer_stats
                    .scoring_context
                    .goal_after_kickoff
                    .record_goal(time_after_kickoff);
            }
            scorer_stats
                .scoring_context
                .goal_buildup
                .record(pending_goal_event.goal_buildup);
            if let Some(ball_air_time_before_goal) = pending_goal_event.ball_air_time_before_goal {
                scorer_stats
                    .scoring_context
                    .record_goal_ball_air_time(ball_air_time_before_goal);
            }

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

    pub fn team_zero_stats(&self) -> CoreTeamStats {
        self.team_stats_for_side(true)
    }

    pub fn team_one_stats(&self) -> CoreTeamStats {
        self.team_stats_for_side(false)
    }

    pub(super) fn team_stats_for_side(&self, is_team_0: bool) -> CoreTeamStats {
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

    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_timeline_events(
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

    pub(super) fn emit_core_stats_events(&mut self, frame: &FrameInfo, players: &PlayerFrameState) {
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
                player_position: players.player_position(&player_id),
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

    pub(super) fn kickoff_phase_active(gameplay: &GameplayState) -> bool {
        gameplay.kickoff_phase_active()
    }

    pub(super) fn update_kickoff_reference(
        &mut self,
        gameplay: &GameplayState,
        events: &FrameEventsState,
    ) {
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

    pub(super) fn take_pending_goal_event(
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

    pub(super) fn last_defender(
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

    pub(super) fn most_back_player(
        players: &PlayerFrameState,
        team_is_team_0: bool,
    ) -> Option<PlayerId> {
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

    pub(super) fn player_position(
        players: &PlayerFrameState,
        player_id: &PlayerId,
    ) -> Option<glam::Vec3> {
        players
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
            .and_then(PlayerSample::position)
    }

    pub(super) fn update_last_touch_contexts(
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

    pub(super) fn update_boost_leadup_samples(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
    ) {
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

    pub(super) fn update_ball_ground_contact(&mut self, frame: &FrameInfo, ball: &BallFrameState) {
        if ball
            .position()
            .is_some_and(|position| position.z <= BALL_GROUND_CONTACT_MAX_Z)
        {
            self.last_ball_ground_contact_time = Some(frame.time);
        }
    }

    pub(super) fn ball_air_time_before_goal(&self, goal_time: f32) -> Option<f32> {
        self.last_ball_ground_contact_time
            .map(|ground_contact_time| (goal_time - ground_contact_time).max(0.0))
    }

    pub(super) fn boost_leadup_for_player(&self, player_id: &PlayerId) -> Option<BoostLeadupStats> {
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

    pub(super) fn goal_player_contexts(
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

    pub(super) fn record_goal_context_stats(
        &mut self,
        players: &PlayerFrameState,
        goal_event: &GoalEvent,
        scoring_team_most_back_player: Option<&PlayerId>,
        defending_team_most_back_player: Option<&PlayerId>,
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
    }

    pub(super) fn record_goal_context_events(
        &mut self,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
    ) {
        let ball_position = ball.position().map(GoalContextPosition::from);
        let ball_speed_at_goal = ball.velocity().map(|velocity| velocity.length());
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

    pub(super) fn reconcile_goal_context_scorer(
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

    pub(super) fn prune_goal_buildup_samples(&mut self, current_time: f32) {
        self.goal_buildup_samples
            .retain(|entry| current_time - entry.time <= GOAL_BUILDUP_LOOKBACK_SECONDS);
        self.goal_buildup_pressure_events
            .retain(|entry| current_time - entry.time <= GOAL_BUILDUP_LOOKBACK_SECONDS);
    }

    pub(super) fn record_goal_buildup_sample(&mut self, frame: &FrameInfo, ball: &BallFrameState) {
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

    pub(super) fn record_goal_buildup_pressure_events(&mut self, events: &FrameEventsState) {
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

    pub(super) fn classify_goal_buildup(
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
