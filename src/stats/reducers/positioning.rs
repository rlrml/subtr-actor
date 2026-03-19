use super::*;

const GOAL_CAUGHT_AHEAD_MAX_BALL_Y: f32 = -1200.0;
const GOAL_CAUGHT_AHEAD_MIN_PLAYER_Y: f32 = -250.0;
const GOAL_CAUGHT_AHEAD_MIN_BALL_DELTA_Y: f32 = 2200.0;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct PositioningStats {
    pub active_game_time: f32,
    pub tracked_time: f32,
    pub sum_distance_to_teammates: f32,
    pub sum_distance_to_ball: f32,
    pub sum_distance_to_ball_has_possession: f32,
    pub time_has_possession: f32,
    pub sum_distance_to_ball_no_possession: f32,
    pub time_no_possession: f32,
    pub time_demolished: f32,
    pub time_no_teammates: f32,
    pub time_most_back: f32,
    pub time_most_forward: f32,
    pub time_mid_role: f32,
    pub time_other_role: f32,
    #[serde(rename = "time_defensive_third")]
    pub time_defensive_zone: f32,
    #[serde(rename = "time_neutral_third")]
    pub time_neutral_zone: f32,
    #[serde(rename = "time_offensive_third")]
    pub time_offensive_zone: f32,
    pub time_defensive_half: f32,
    pub time_offensive_half: f32,
    pub time_closest_to_ball: f32,
    pub time_farthest_from_ball: f32,
    pub time_behind_ball: f32,
    pub time_in_front_of_ball: f32,
    pub times_caught_ahead_of_play_on_conceded_goals: u32,
}

impl PositioningStats {
    pub fn average_distance_to_teammates(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.sum_distance_to_teammates / self.tracked_time
        }
    }

    pub fn average_distance_to_ball(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.sum_distance_to_ball / self.tracked_time
        }
    }

    pub fn average_distance_to_ball_has_possession(&self) -> f32 {
        if self.time_has_possession == 0.0 {
            0.0
        } else {
            self.sum_distance_to_ball_has_possession / self.time_has_possession
        }
    }

    pub fn average_distance_to_ball_no_possession(&self) -> f32 {
        if self.time_no_possession == 0.0 {
            0.0
        } else {
            self.sum_distance_to_ball_no_possession / self.time_no_possession
        }
    }

    fn pct(&self, value: f32) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            value * 100.0 / self.tracked_time
        }
    }

    pub fn most_back_pct(&self) -> f32 {
        self.pct(self.time_most_back)
    }

    pub fn most_forward_pct(&self) -> f32 {
        self.pct(self.time_most_forward)
    }

    pub fn mid_role_pct(&self) -> f32 {
        self.pct(self.time_mid_role)
    }

    pub fn other_role_pct(&self) -> f32 {
        self.pct(self.time_other_role)
    }

    pub fn defensive_third_pct(&self) -> f32 {
        self.pct(self.time_defensive_zone)
    }

    pub fn neutral_third_pct(&self) -> f32 {
        self.pct(self.time_neutral_zone)
    }

    pub fn offensive_third_pct(&self) -> f32 {
        self.pct(self.time_offensive_zone)
    }

    pub fn defensive_zone_pct(&self) -> f32 {
        self.defensive_third_pct()
    }

    pub fn neutral_zone_pct(&self) -> f32 {
        self.neutral_third_pct()
    }

    pub fn offensive_zone_pct(&self) -> f32 {
        self.offensive_third_pct()
    }

    pub fn defensive_half_pct(&self) -> f32 {
        self.pct(self.time_defensive_half)
    }

    pub fn offensive_half_pct(&self) -> f32 {
        self.pct(self.time_offensive_half)
    }

    pub fn closest_to_ball_pct(&self) -> f32 {
        self.pct(self.time_closest_to_ball)
    }

    pub fn farthest_from_ball_pct(&self) -> f32 {
        self.pct(self.time_farthest_from_ball)
    }

    pub fn behind_ball_pct(&self) -> f32 {
        self.pct(self.time_behind_ball)
    }

    pub fn in_front_of_ball_pct(&self) -> f32 {
        self.pct(self.time_in_front_of_ball)
    }
}

#[derive(Debug, Clone)]
pub struct PositioningReducerConfig {
    pub most_back_forward_threshold_y: f32,
}

impl Default for PositioningReducerConfig {
    fn default() -> Self {
        Self {
            most_back_forward_threshold_y: DEFAULT_MOST_BACK_FORWARD_THRESHOLD_Y,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PositioningReducer {
    config: PositioningReducerConfig,
    player_stats: HashMap<PlayerId, PositioningStats>,
    previous_ball_position: Option<glam::Vec3>,
    previous_player_positions: HashMap<PlayerId, glam::Vec3>,
    possession_tracker: PossessionTracker,
    live_play_tracker: LivePlayTracker,
}

impl PositioningReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: PositioningReducerConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn config(&self) -> &PositioningReducerConfig {
        &self.config
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, PositioningStats> {
        &self.player_stats
    }

    fn record_goal_positioning_events(&mut self, sample: &StatsSample, ball_position: glam::Vec3) {
        for goal_event in &sample.goal_events {
            let defending_team_is_team_0 = !goal_event.scoring_team_is_team_0;
            let normalized_ball_y = normalized_y(defending_team_is_team_0, ball_position);
            if normalized_ball_y > GOAL_CAUGHT_AHEAD_MAX_BALL_Y {
                continue;
            }

            for player in sample
                .players
                .iter()
                .filter(|player| player.is_team_0 == defending_team_is_team_0)
            {
                let Some(position) = player.position() else {
                    continue;
                };
                let normalized_player_y = normalized_y(defending_team_is_team_0, position);
                if normalized_player_y < GOAL_CAUGHT_AHEAD_MIN_PLAYER_Y {
                    continue;
                }
                if normalized_player_y - normalized_ball_y < GOAL_CAUGHT_AHEAD_MIN_BALL_DELTA_Y {
                    continue;
                }

                self.player_stats
                    .entry(player.player_id.clone())
                    .or_default()
                    .times_caught_ahead_of_play_on_conceded_goals += 1;
            }
        }
    }

    fn process_sample(
        &mut self,
        sample: &StatsSample,
        live_play: bool,
        possession_player_before_sample: Option<&PlayerId>,
    ) -> SubtrActorResult<()> {
        if sample.dt == 0.0 {
            if let Some(ball) = &sample.ball {
                self.previous_ball_position = Some(ball.position());
            }
            for player in &sample.players {
                if let Some(position) = player.position() {
                    self.previous_player_positions
                        .insert(player.player_id.clone(), position);
                }
            }
            return Ok(());
        }

        let Some(ball) = &sample.ball else {
            return Ok(());
        };
        let ball_position = ball.position();
        if !sample.goal_events.is_empty() {
            self.record_goal_positioning_events(sample, ball_position);
        }
        let demoed_players: HashSet<_> = sample
            .active_demos
            .iter()
            .map(|demo| demo.victim.clone())
            .collect();

        for player in &sample.players {
            let is_demoed = demoed_players.contains(&player.player_id);
            if live_play && is_demoed {
                let stats = self
                    .player_stats
                    .entry(player.player_id.clone())
                    .or_default();
                stats.active_game_time += sample.dt;
                stats.time_demolished += sample.dt;
                continue;
            }

            let Some(position) = player.position() else {
                continue;
            };
            let previous_position = self
                .previous_player_positions
                .get(&player.player_id)
                .copied()
                .unwrap_or(position);
            let previous_ball_position = self.previous_ball_position.unwrap_or(ball_position);
            let normalized_position_y = normalized_y(player.is_team_0, position);
            let normalized_previous_position_y = normalized_y(player.is_team_0, previous_position);
            let normalized_ball_y = normalized_y(player.is_team_0, ball_position);
            let normalized_previous_ball_y = normalized_y(player.is_team_0, previous_ball_position);
            let stats = self
                .player_stats
                .entry(player.player_id.clone())
                .or_default();

            if live_play {
                stats.active_game_time += sample.dt;
                stats.tracked_time += sample.dt;
                stats.sum_distance_to_ball += position.distance(ball_position) * sample.dt;

                if possession_player_before_sample == Some(&player.player_id) {
                    stats.time_has_possession += sample.dt;
                    stats.sum_distance_to_ball_has_possession +=
                        position.distance(ball_position) * sample.dt;
                } else if possession_player_before_sample.is_some() {
                    stats.time_no_possession += sample.dt;
                    stats.sum_distance_to_ball_no_possession +=
                        position.distance(ball_position) * sample.dt;
                }

                let defensive_zone_fraction = interval_fraction_below_threshold(
                    normalized_previous_position_y,
                    normalized_position_y,
                    -FIELD_ZONE_BOUNDARY_Y,
                );
                let offensive_zone_fraction = interval_fraction_above_threshold(
                    normalized_previous_position_y,
                    normalized_position_y,
                    FIELD_ZONE_BOUNDARY_Y,
                );
                let neutral_zone_fraction = interval_fraction_in_scalar_range(
                    normalized_previous_position_y,
                    normalized_position_y,
                    -FIELD_ZONE_BOUNDARY_Y,
                    FIELD_ZONE_BOUNDARY_Y,
                );
                stats.time_defensive_zone += sample.dt * defensive_zone_fraction;
                stats.time_neutral_zone += sample.dt * neutral_zone_fraction;
                stats.time_offensive_zone += sample.dt * offensive_zone_fraction;

                let defensive_half_fraction = interval_fraction_below_threshold(
                    normalized_previous_position_y,
                    normalized_position_y,
                    0.0,
                );
                stats.time_defensive_half += sample.dt * defensive_half_fraction;
                stats.time_offensive_half += sample.dt * (1.0 - defensive_half_fraction);

                let previous_ball_delta =
                    normalized_previous_position_y - normalized_previous_ball_y;
                let current_ball_delta = normalized_position_y - normalized_ball_y;
                let behind_ball_fraction =
                    interval_fraction_below_threshold(previous_ball_delta, current_ball_delta, 0.0);
                stats.time_behind_ball += sample.dt * behind_ball_fraction;
                stats.time_in_front_of_ball += sample.dt * (1.0 - behind_ball_fraction);
            }
        }

        if live_play {
            for is_team_0 in [true, false] {
                let team_present_player_count = sample
                    .players
                    .iter()
                    .filter(|player| player.is_team_0 == is_team_0)
                    .count();
                let team_roster_count = sample.current_in_game_team_player_count(is_team_0).max(
                    sample
                        .players
                        .iter()
                        .filter(|player| player.is_team_0 == is_team_0)
                        .count(),
                );
                let team_players: Vec<_> = sample
                    .players
                    .iter()
                    .filter(|player| player.is_team_0 == is_team_0)
                    .filter(|player| !demoed_players.contains(&player.player_id))
                    .filter_map(|player| player.position().map(|position| (player, position)))
                    .collect();

                if team_players.is_empty() {
                    continue;
                }

                for (player, position) in &team_players {
                    let teammate_distance_sum: f32 = team_players
                        .iter()
                        .filter(|(other_player, _)| other_player.player_id != player.player_id)
                        .map(|(_, other_position)| position.distance(*other_position))
                        .sum();
                    let teammate_count = team_players.len().saturating_sub(1);
                    if teammate_count > 0 {
                        let stats = self
                            .player_stats
                            .entry(player.player_id.clone())
                            .or_default();
                        stats.sum_distance_to_teammates +=
                            teammate_distance_sum * sample.dt / teammate_count as f32;
                    }
                }

                if team_roster_count < 2
                    || team_present_player_count < team_roster_count
                    || team_players.len() < 2
                {
                    for (player, _) in &team_players {
                        self.player_stats
                            .entry(player.player_id.clone())
                            .or_default()
                            .time_no_teammates += sample.dt;
                    }
                } else {
                    let mut sorted_team: Vec<_> = team_players
                        .iter()
                        .map(|(info, pos)| (info.player_id.clone(), normalized_y(is_team_0, *pos)))
                        .collect();
                    sorted_team.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());

                    let team_spread = sorted_team.last().map(|(_, y)| *y).unwrap_or(0.0)
                        - sorted_team.first().map(|(_, y)| *y).unwrap_or(0.0);

                    if team_spread <= self.config.most_back_forward_threshold_y {
                        for (player_id, _) in &sorted_team {
                            self.player_stats
                                .entry(player_id.clone())
                                .or_default()
                                .time_other_role += sample.dt;
                        }
                    } else {
                        let min_y = sorted_team.first().map(|(_, y)| *y).unwrap_or(0.0);
                        let max_y = sorted_team.last().map(|(_, y)| *y).unwrap_or(0.0);
                        let can_assign_mid_role = sorted_team.len() == 3;

                        for (player_id, y) in &sorted_team {
                            let near_back =
                                (*y - min_y) <= self.config.most_back_forward_threshold_y;
                            let near_front =
                                (max_y - *y) <= self.config.most_back_forward_threshold_y;

                            if near_back && !near_front {
                                self.player_stats
                                    .entry(player_id.clone())
                                    .or_default()
                                    .time_most_back += sample.dt;
                            } else if near_front && !near_back {
                                self.player_stats
                                    .entry(player_id.clone())
                                    .or_default()
                                    .time_most_forward += sample.dt;
                            } else if can_assign_mid_role {
                                self.player_stats
                                    .entry(player_id.clone())
                                    .or_default()
                                    .time_mid_role += sample.dt;
                            } else {
                                self.player_stats
                                    .entry(player_id.clone())
                                    .or_default()
                                    .time_other_role += sample.dt;
                            }
                        }
                    }
                }

                if let Some((closest_player, _)) = team_players.iter().min_by(|(_, a), (_, b)| {
                    a.distance(ball_position)
                        .partial_cmp(&b.distance(ball_position))
                        .unwrap()
                }) {
                    self.player_stats
                        .entry(closest_player.player_id.clone())
                        .or_default()
                        .time_closest_to_ball += sample.dt;
                }

                if let Some((farthest_player, _)) = team_players.iter().max_by(|(_, a), (_, b)| {
                    a.distance(ball_position)
                        .partial_cmp(&b.distance(ball_position))
                        .unwrap()
                }) {
                    self.player_stats
                        .entry(farthest_player.player_id.clone())
                        .or_default()
                        .time_farthest_from_ball += sample.dt;
                }
            }
        }

        self.previous_ball_position = Some(ball_position);
        for player in &sample.players {
            if let Some(position) = player.position() {
                self.previous_player_positions
                    .insert(player.player_id.clone(), position);
            }
        }

        Ok(())
    }
}

impl StatsReducer for PositioningReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        let live_play = self.live_play_tracker.is_live_play(sample);
        let possession_player_before_sample = if live_play {
            let possession_state = self.possession_tracker.update(sample, &sample.touch_events);
            possession_state.active_player_before_sample
        } else {
            self.possession_tracker.reset();
            None
        };
        self.process_sample(sample, live_play, possession_player_before_sample.as_ref())?;
        Ok(())
    }

    fn on_sample_with_context(
        &mut self,
        sample: &StatsSample,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<()> {
        let live_play = self.live_play_tracker.is_live_play(sample);
        let possession_player_before_sample = ctx
            .get::<PossessionState>(POSSESSION_STATE_SIGNAL_ID)
            .and_then(|state| state.active_player_before_sample.as_ref());
        self.process_sample(sample, live_play, possession_player_before_sample)?;
        Ok(())
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

    fn player(player_id: u64, is_team_0: bool, y: f32) -> PlayerSample {
        PlayerSample {
            player_id: RemoteId::Steam(player_id),
            is_team_0,
            rigid_body: Some(rigid_body(y)),
            boost_amount: None,
            last_boost_amount: None,
            boost_active: false,
            dodge_active: false,
            powerslide_active: false,
            match_goals: Some(0),
            match_assists: Some(0),
            match_saves: Some(0),
            match_shots: Some(0),
            match_score: Some(0),
        }
    }

    fn sample(
        frame_number: usize,
        time: f32,
        touch_players: &[(u64, bool)],
        kickoff_phase_active: bool,
    ) -> StatsSample {
        StatsSample {
            frame_number,
            time,
            dt: 1.0,
            seconds_remaining: None,
            game_state: kickoff_phase_active.then_some(55),
            ball_has_been_hit: Some(!kickoff_phase_active),
            kickoff_countdown_time: kickoff_phase_active.then_some(3),
            team_zero_score: Some(0),
            team_one_score: Some(0),
            possession_team_is_team_0: None,
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: Some([2, 1]),
            ball: Some(BallSample {
                rigid_body: rigid_body(0.0),
            }),
            players: vec![
                player(1, true, -400.0),
                player(2, true, -100.0),
                player(3, false, 300.0),
            ],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: touch_players
                .iter()
                .map(|(player_id, team_is_team_0)| TouchEvent {
                    time,
                    frame: frame_number,
                    player: Some(RemoteId::Steam(*player_id)),
                    team_is_team_0: *team_is_team_0,
                    closest_approach_distance: None,
                })
                .collect(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        }
    }

    #[test]
    fn counts_defenders_caught_ahead_of_play_on_goal_frames() {
        let mut reducer = PositioningReducer::new();
        let sample = StatsSample {
            frame_number: 10,
            time: 10.0,
            dt: 1.0,
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: Some(true),
            kickoff_countdown_time: None,
            team_zero_score: Some(1),
            team_one_score: Some(0),
            possession_team_is_team_0: Some(true),
            scored_on_team_is_team_0: Some(false),
            current_in_game_team_player_counts: Some([1, 3]),
            ball: Some(BallSample {
                rigid_body: rigid_body(4800.0),
            }),
            players: vec![
                player(1, true, 0.0),
                player(2, false, -1800.0),
                player(3, false, -700.0),
                player(4, false, 3200.0),
            ],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: vec![GoalEvent {
                time: 10.0,
                frame: 10,
                scoring_team_is_team_0: true,
                player: Some(RemoteId::Steam(1)),
                team_zero_score: Some(1),
                team_one_score: Some(0),
            }],
        };

        reducer.on_sample(&sample).unwrap();

        assert_eq!(
            reducer
                .player_stats()
                .get(&RemoteId::Steam(2))
                .unwrap()
                .times_caught_ahead_of_play_on_conceded_goals,
            1
        );
        assert_eq!(
            reducer
                .player_stats()
                .get(&RemoteId::Steam(3))
                .unwrap()
                .times_caught_ahead_of_play_on_conceded_goals,
            1
        );
        assert_eq!(
            reducer
                .player_stats()
                .get(&RemoteId::Steam(4))
                .unwrap()
                .times_caught_ahead_of_play_on_conceded_goals,
            0
        );
    }

    #[test]
    fn player_possession_is_exclusive_and_resets_on_kickoff() {
        let mut reducer = PositioningReducer::new();

        reducer.on_sample(&sample(0, 0.0, &[], false)).unwrap();
        reducer
            .on_sample(&sample(1, 1.0, &[(1, true)], false))
            .unwrap();
        reducer.on_sample(&sample(2, 2.0, &[], false)).unwrap();
        reducer
            .on_sample(&sample(3, 3.0, &[(2, true)], false))
            .unwrap();
        reducer.on_sample(&sample(4, 4.0, &[], false)).unwrap();
        reducer.on_sample(&sample(5, 5.0, &[], true)).unwrap();
        reducer.on_sample(&sample(6, 6.0, &[], false)).unwrap();

        let player_one = reducer.player_stats().get(&RemoteId::Steam(1)).unwrap();
        let player_two = reducer.player_stats().get(&RemoteId::Steam(2)).unwrap();
        let player_three = reducer.player_stats().get(&RemoteId::Steam(3)).unwrap();

        assert_eq!(player_one.time_has_possession, 2.0);
        assert_eq!(player_two.time_has_possession, 1.0);
        assert_eq!(player_three.time_has_possession, 0.0);
        assert_eq!(
            player_one.time_has_possession
                + player_two.time_has_possession
                + player_three.time_has_possession,
            3.0
        );
        assert_eq!(player_one.time_no_possession, 1.0);
        assert_eq!(player_two.time_no_possession, 2.0);
        assert_eq!(player_three.time_no_possession, 3.0);
    }
}
