use super::*;

const GOAL_CAUGHT_AHEAD_MAX_BALL_Y: f32 = -1200.0;
const GOAL_CAUGHT_AHEAD_MIN_PLAYER_Y: f32 = -250.0;
const GOAL_CAUGHT_AHEAD_MIN_BALL_DELTA_Y: f32 = 2200.0;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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
pub struct PositioningCalculatorConfig {
    pub most_back_forward_threshold_y: f32,
}

impl Default for PositioningCalculatorConfig {
    fn default() -> Self {
        Self {
            most_back_forward_threshold_y: DEFAULT_MOST_BACK_FORWARD_THRESHOLD_Y,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PositioningCalculator {
    config: PositioningCalculatorConfig,
    player_stats: HashMap<PlayerId, PositioningStats>,
    previous_ball_position: Option<glam::Vec3>,
    previous_player_positions: HashMap<PlayerId, glam::Vec3>,
}

impl PositioningCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: PositioningCalculatorConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn config(&self) -> &PositioningCalculatorConfig {
        &self.config
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, PositioningStats> {
        &self.player_stats
    }

    fn record_goal_positioning_events(
        &mut self,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        ball_position: glam::Vec3,
    ) {
        for goal_event in &events.goal_events {
            let defending_team_is_team_0 = !goal_event.scoring_team_is_team_0;
            let normalized_ball_y = normalized_y(defending_team_is_team_0, ball_position);
            if normalized_ball_y > GOAL_CAUGHT_AHEAD_MAX_BALL_Y {
                continue;
            }

            for player in players
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
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        live_play: bool,
        possession_player_before_sample: Option<&PlayerId>,
    ) -> SubtrActorResult<()> {
        if frame.dt == 0.0 {
            if let Some(ball) = ball.sample() {
                self.previous_ball_position = Some(ball.position());
            }
            for player in &players.players {
                if let Some(position) = player.position() {
                    self.previous_player_positions
                        .insert(player.player_id.clone(), position);
                }
            }
            return Ok(());
        }

        let Some(ball) = ball.sample() else {
            return Ok(());
        };
        let ball_position = ball.position();
        if !events.goal_events.is_empty() {
            self.record_goal_positioning_events(players, events, ball_position);
        }
        let demoed_players: HashSet<_> = events
            .active_demos
            .iter()
            .map(|demo| demo.victim.clone())
            .collect();

        for player in &players.players {
            let is_demoed = demoed_players.contains(&player.player_id);
            if live_play && is_demoed {
                let stats = self
                    .player_stats
                    .entry(player.player_id.clone())
                    .or_default();
                stats.active_game_time += frame.dt;
                stats.time_demolished += frame.dt;
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
                stats.active_game_time += frame.dt;
                stats.tracked_time += frame.dt;
                stats.sum_distance_to_ball += position.distance(ball_position) * frame.dt;

                if possession_player_before_sample == Some(&player.player_id) {
                    stats.time_has_possession += frame.dt;
                    stats.sum_distance_to_ball_has_possession +=
                        position.distance(ball_position) * frame.dt;
                } else if possession_player_before_sample.is_some() {
                    stats.time_no_possession += frame.dt;
                    stats.sum_distance_to_ball_no_possession +=
                        position.distance(ball_position) * frame.dt;
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
                stats.time_defensive_zone += frame.dt * defensive_zone_fraction;
                stats.time_neutral_zone += frame.dt * neutral_zone_fraction;
                stats.time_offensive_zone += frame.dt * offensive_zone_fraction;

                let defensive_half_fraction = interval_fraction_below_threshold(
                    normalized_previous_position_y,
                    normalized_position_y,
                    0.0,
                );
                stats.time_defensive_half += frame.dt * defensive_half_fraction;
                stats.time_offensive_half += frame.dt * (1.0 - defensive_half_fraction);

                let previous_ball_delta =
                    normalized_previous_position_y - normalized_previous_ball_y;
                let current_ball_delta = normalized_position_y - normalized_ball_y;
                let behind_ball_fraction =
                    interval_fraction_below_threshold(previous_ball_delta, current_ball_delta, 0.0);
                stats.time_behind_ball += frame.dt * behind_ball_fraction;
                stats.time_in_front_of_ball += frame.dt * (1.0 - behind_ball_fraction);
            }
        }

        if live_play {
            for is_team_0 in [true, false] {
                let team_present_player_count = players
                    .players
                    .iter()
                    .filter(|player| player.is_team_0 == is_team_0)
                    .count();
                let team_roster_count = gameplay.current_in_game_team_player_count(is_team_0).max(
                    players
                        .players
                        .iter()
                        .filter(|player| player.is_team_0 == is_team_0)
                        .count(),
                );
                let team_players: Vec<_> = players
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
                            teammate_distance_sum * frame.dt / teammate_count as f32;
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
                            .time_no_teammates += frame.dt;
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
                                .time_other_role += frame.dt;
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
                                    .time_most_back += frame.dt;
                            } else if near_front && !near_back {
                                self.player_stats
                                    .entry(player_id.clone())
                                    .or_default()
                                    .time_most_forward += frame.dt;
                            } else if can_assign_mid_role {
                                self.player_stats
                                    .entry(player_id.clone())
                                    .or_default()
                                    .time_mid_role += frame.dt;
                            } else {
                                self.player_stats
                                    .entry(player_id.clone())
                                    .or_default()
                                    .time_other_role += frame.dt;
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
                        .time_closest_to_ball += frame.dt;
                }

                if let Some((farthest_player, _)) = team_players.iter().max_by(|(_, a), (_, b)| {
                    a.distance(ball_position)
                        .partial_cmp(&b.distance(ball_position))
                        .unwrap()
                }) {
                    self.player_stats
                        .entry(farthest_player.player_id.clone())
                        .or_default()
                        .time_farthest_from_ball += frame.dt;
                }
            }
        }

        self.previous_ball_position = Some(ball_position);
        for player in &players.players {
            if let Some(position) = player.position() {
                self.previous_player_positions
                    .insert(player.player_id.clone(), position);
            }
        }

        Ok(())
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        live_play: bool,
        possession_player_before_sample: Option<&PlayerId>,
    ) -> SubtrActorResult<()> {
        self.process_sample(
            frame,
            gameplay,
            ball,
            players,
            events,
            live_play,
            possession_player_before_sample,
        )
    }
}
