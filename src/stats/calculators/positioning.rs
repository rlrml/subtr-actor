use super::*;

const GOAL_CAUGHT_AHEAD_MAX_BALL_Y: f32 = -1200.0;
const GOAL_CAUGHT_AHEAD_MIN_PLAYER_Y: f32 = -250.0;
const GOAL_CAUGHT_AHEAD_MIN_BALL_DELTA_Y: f32 = 2200.0;
const DEFAULT_LEVEL_BALL_DEPTH_MARGIN: f32 = 150.0;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PositioningEvent {
    pub time: f32,
    pub frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub duration: f32,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub active: bool,
    pub tracked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distance_to_teammates: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distance_to_ball: Option<f32>,
    pub possession_state: PositioningPossessionState,
    pub demolished: bool,
    pub no_teammates: bool,
    pub teammate_role: PositioningTeammateRoleState,
    pub defensive_zone_fraction: f32,
    pub neutral_zone_fraction: f32,
    pub offensive_zone_fraction: f32,
    pub defensive_half_fraction: f32,
    pub offensive_half_fraction: f32,
    pub closest_to_ball: bool,
    pub farthest_from_ball: bool,
    pub behind_ball_fraction: f32,
    pub level_with_ball_fraction: f32,
    pub in_front_of_ball_fraction: f32,
    pub caught_ahead_of_play_on_conceded_goal: bool,
}

impl PositioningEvent {
    fn new(
        frame: &FrameInfo,
        player: PlayerId,
        player_position: Option<[f32; 3]>,
        is_team_0: bool,
    ) -> Self {
        Self {
            time: frame.time,
            frame: frame.frame_number,
            end_time: frame.time,
            end_frame: frame.frame_number,
            duration: 0.0,
            player,
            player_position,
            is_team_0,
            active: false,
            tracked: false,
            distance_to_teammates: None,
            distance_to_ball: None,
            possession_state: PositioningPossessionState::Neutral,
            demolished: false,
            no_teammates: false,
            teammate_role: PositioningTeammateRoleState::Unknown,
            defensive_zone_fraction: 0.0,
            neutral_zone_fraction: 0.0,
            offensive_zone_fraction: 0.0,
            defensive_half_fraction: 0.0,
            offensive_half_fraction: 0.0,
            closest_to_ball: false,
            farthest_from_ball: false,
            behind_ball_fraction: 0.0,
            level_with_ball_fraction: 0.0,
            in_front_of_ball_fraction: 0.0,
            caught_ahead_of_play_on_conceded_goal: false,
        }
    }

    fn has_delta(&self) -> bool {
        self.duration != 0.0 || self.caught_ahead_of_play_on_conceded_goal
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PositioningPossessionState {
    HasPossession,
    NoPossession,
    #[default]
    Neutral,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PositioningTeammateRoleState {
    NoTeammates,
    MostBack,
    MostForward,
    Mid,
    Other,
    #[default]
    Unknown,
}

#[derive(Debug, Clone)]
pub struct PositioningCalculatorConfig {
    pub most_back_forward_threshold_y: f32,
    pub level_ball_depth_margin: f32,
}

impl Default for PositioningCalculatorConfig {
    fn default() -> Self {
        Self {
            most_back_forward_threshold_y: DEFAULT_MOST_BACK_FORWARD_THRESHOLD_Y,
            level_ball_depth_margin: DEFAULT_LEVEL_BALL_DEPTH_MARGIN,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PositioningCalculator {
    config: PositioningCalculatorConfig,
    previous_ball_position: Option<glam::Vec3>,
    previous_player_positions: HashMap<PlayerId, glam::Vec3>,
    events: EventStream<PositioningEvent>,
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

    pub fn events(&self) -> &[PositioningEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[PositioningEvent] {
        self.events.new_events()
    }

    pub fn projected_events(&self) -> Vec<PositioningEvent> {
        self.events.all().to_vec()
    }

    pub fn flush_pending_events(&mut self) {}

    fn event_delta<'a>(
        deltas: &'a mut HashMap<PlayerId, PositioningEvent>,
        frame: &FrameInfo,
        player_id: &PlayerId,
        player_position: Option<[f32; 3]>,
        is_team_0: bool,
    ) -> &'a mut PositioningEvent {
        deltas.entry(player_id.clone()).or_insert_with(|| {
            PositioningEvent::new(frame, player_id.clone(), player_position, is_team_0)
        })
    }

    fn record_goal_positioning_events(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        ball_position: glam::Vec3,
        event_deltas: &mut HashMap<PlayerId, PositioningEvent>,
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

                Self::event_delta(
                    event_deltas,
                    frame,
                    &player.player_id,
                    Some(position.to_array()),
                    player.is_team_0,
                )
                .caught_ahead_of_play_on_conceded_goal = true;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn process_sample(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        live_play_state: &LivePlayState,
        possession_player_before_sample: Option<&PlayerId>,
    ) -> SubtrActorResult<()> {
        let live_play = live_play_state.is_live_play;
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
        let mut event_deltas = HashMap::new();
        if !events.goal_events.is_empty() {
            self.record_goal_positioning_events(
                frame,
                players,
                events,
                ball_position,
                &mut event_deltas,
            );
        }
        let demoed_players: HashSet<_> = events
            .active_demos
            .iter()
            .map(|demo| demo.victim.clone())
            .collect();

        for player in &players.players {
            let is_demoed = demoed_players.contains(&player.player_id);
            if live_play && is_demoed {
                let delta = Self::event_delta(
                    &mut event_deltas,
                    frame,
                    &player.player_id,
                    player.position().map(|position| position.to_array()),
                    player.is_team_0,
                );
                delta.duration = frame.dt;
                delta.active = true;
                delta.demolished = true;
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

            if live_play {
                let distance_to_ball = position.distance(ball_position);
                let delta = Self::event_delta(
                    &mut event_deltas,
                    frame,
                    &player.player_id,
                    Some(position.to_array()),
                    player.is_team_0,
                );
                delta.duration = frame.dt;
                delta.active = true;
                delta.tracked = true;
                delta.distance_to_ball = Some(distance_to_ball);

                if possession_player_before_sample == Some(&player.player_id) {
                    delta.possession_state = PositioningPossessionState::HasPossession;
                } else if possession_player_before_sample.is_some() {
                    delta.possession_state = PositioningPossessionState::NoPossession;
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
                delta.defensive_zone_fraction = defensive_zone_fraction;
                delta.neutral_zone_fraction = neutral_zone_fraction;
                delta.offensive_zone_fraction = offensive_zone_fraction;

                let defensive_half_fraction = interval_fraction_below_threshold(
                    normalized_previous_position_y,
                    normalized_position_y,
                    0.0,
                );
                delta.defensive_half_fraction = defensive_half_fraction;
                delta.offensive_half_fraction = 1.0 - defensive_half_fraction;

                let previous_ball_delta =
                    normalized_previous_position_y - normalized_previous_ball_y;
                let current_ball_delta = normalized_position_y - normalized_ball_y;
                let (behind_ball_fraction, level_ball_fraction, in_front_ball_fraction) =
                    ball_depth_fractions(
                        self.config.level_ball_depth_margin,
                        previous_ball_delta,
                        current_ball_delta,
                    );
                delta.behind_ball_fraction = behind_ball_fraction;
                delta.level_with_ball_fraction = level_ball_fraction;
                delta.in_front_of_ball_fraction = in_front_ball_fraction;
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
                        Self::event_delta(
                            &mut event_deltas,
                            frame,
                            &player.player_id,
                            Some(position.to_array()),
                            player.is_team_0,
                        )
                        .distance_to_teammates =
                            Some(teammate_distance_sum / teammate_count as f32);
                    }
                }

                if team_roster_count < 2
                    || team_present_player_count < team_roster_count
                    || team_players.len() < 2
                {
                    for (player, position) in &team_players {
                        Self::event_delta(
                            &mut event_deltas,
                            frame,
                            &player.player_id,
                            Some(position.to_array()),
                            player.is_team_0,
                        )
                        .teammate_role = PositioningTeammateRoleState::NoTeammates;
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
                            let player_position = players.player_position(player_id);
                            Self::event_delta(
                                &mut event_deltas,
                                frame,
                                player_id,
                                player_position,
                                is_team_0,
                            )
                            .teammate_role = PositioningTeammateRoleState::Other;
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
                                Self::event_delta(
                                    &mut event_deltas,
                                    frame,
                                    player_id,
                                    players.player_position(player_id),
                                    is_team_0,
                                )
                                .teammate_role = PositioningTeammateRoleState::MostBack;
                            } else if near_front && !near_back {
                                Self::event_delta(
                                    &mut event_deltas,
                                    frame,
                                    player_id,
                                    players.player_position(player_id),
                                    is_team_0,
                                )
                                .teammate_role = PositioningTeammateRoleState::MostForward;
                            } else if can_assign_mid_role {
                                Self::event_delta(
                                    &mut event_deltas,
                                    frame,
                                    player_id,
                                    players.player_position(player_id),
                                    is_team_0,
                                )
                                .teammate_role = PositioningTeammateRoleState::Mid;
                            } else {
                                Self::event_delta(
                                    &mut event_deltas,
                                    frame,
                                    player_id,
                                    players.player_position(player_id),
                                    is_team_0,
                                )
                                .teammate_role = PositioningTeammateRoleState::Other;
                            }
                        }
                    }
                }

                if let Some((closest_player, _)) = team_players.iter().min_by(|(_, a), (_, b)| {
                    a.distance(ball_position)
                        .partial_cmp(&b.distance(ball_position))
                        .unwrap()
                }) {
                    Self::event_delta(
                        &mut event_deltas,
                        frame,
                        &closest_player.player_id,
                        closest_player
                            .position()
                            .map(|position| position.to_array()),
                        closest_player.is_team_0,
                    )
                    .closest_to_ball = true;
                }

                if let Some((farthest_player, _)) = team_players.iter().max_by(|(_, a), (_, b)| {
                    a.distance(ball_position)
                        .partial_cmp(&b.distance(ball_position))
                        .unwrap()
                }) {
                    Self::event_delta(
                        &mut event_deltas,
                        frame,
                        &farthest_player.player_id,
                        farthest_player
                            .position()
                            .map(|position| position.to_array()),
                        farthest_player.is_team_0,
                    )
                    .farthest_from_ball = true;
                }
            }
        }

        let frame_events: Vec<_> = event_deltas
            .into_values()
            .filter(PositioningEvent::has_delta)
            .collect();
        self.record_positioning_delta_events(frame_events);

        self.previous_ball_position = Some(ball_position);
        for player in &players.players {
            if let Some(position) = player.position() {
                self.previous_player_positions
                    .insert(player.player_id.clone(), position);
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        live_play_state: &LivePlayState,
        possession_player_before_sample: Option<&PlayerId>,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        self.process_sample(
            frame,
            gameplay,
            ball,
            players,
            events,
            live_play_state,
            possession_player_before_sample,
        )
    }

    fn record_positioning_delta_events(&mut self, mut frame_events: Vec<PositioningEvent>) {
        frame_events.sort_by(|left, right| {
            format!("{:?}", left.player).cmp(&format!("{:?}", right.player))
        });
        self.events.extend(frame_events);
    }
}

fn ball_depth_fractions(level_margin: f32, start_delta: f32, end_delta: f32) -> (f32, f32, f32) {
    let behind_fraction = interval_fraction_below_threshold(start_delta, end_delta, -level_margin);
    let level_fraction =
        interval_fraction_in_scalar_range(start_delta, end_delta, -level_margin, level_margin);
    let in_front_fraction = (1.0 - behind_fraction - level_fraction).clamp(0.0, 1.0);
    (behind_fraction, level_fraction, in_front_fraction)
}

#[cfg(test)]
#[path = "positioning_tests.rs"]
mod tests;
