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
    pub time_level_with_ball: f32,
    pub time_in_front_of_ball: f32,
    pub times_caught_ahead_of_play_on_conceded_goals: u32,
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
            active_game_time: 0.0,
            tracked_time: 0.0,
            sum_distance_to_teammates: 0.0,
            sum_distance_to_ball: 0.0,
            sum_distance_to_ball_has_possession: 0.0,
            time_has_possession: 0.0,
            sum_distance_to_ball_no_possession: 0.0,
            time_no_possession: 0.0,
            time_demolished: 0.0,
            time_no_teammates: 0.0,
            time_most_back: 0.0,
            time_most_forward: 0.0,
            time_mid_role: 0.0,
            time_other_role: 0.0,
            time_defensive_zone: 0.0,
            time_neutral_zone: 0.0,
            time_offensive_zone: 0.0,
            time_defensive_half: 0.0,
            time_offensive_half: 0.0,
            time_closest_to_ball: 0.0,
            time_farthest_from_ball: 0.0,
            time_behind_ball: 0.0,
            time_level_with_ball: 0.0,
            time_in_front_of_ball: 0.0,
            times_caught_ahead_of_play_on_conceded_goals: 0,
        }
    }

    fn has_delta(&self) -> bool {
        self.active_game_time != 0.0
            || self.tracked_time != 0.0
            || self.sum_distance_to_teammates != 0.0
            || self.sum_distance_to_ball != 0.0
            || self.sum_distance_to_ball_has_possession != 0.0
            || self.time_has_possession != 0.0
            || self.sum_distance_to_ball_no_possession != 0.0
            || self.time_no_possession != 0.0
            || self.time_demolished != 0.0
            || self.time_no_teammates != 0.0
            || self.time_most_back != 0.0
            || self.time_most_forward != 0.0
            || self.time_mid_role != 0.0
            || self.time_other_role != 0.0
            || self.time_defensive_zone != 0.0
            || self.time_neutral_zone != 0.0
            || self.time_offensive_zone != 0.0
            || self.time_defensive_half != 0.0
            || self.time_offensive_half != 0.0
            || self.time_closest_to_ball != 0.0
            || self.time_farthest_from_ball != 0.0
            || self.time_behind_ball != 0.0
            || self.time_level_with_ball != 0.0
            || self.time_in_front_of_ball != 0.0
            || self.times_caught_ahead_of_play_on_conceded_goals != 0
    }

    fn timing_state(&self) -> PositioningEventState {
        PositioningEventState {
            active: self.active_game_time > 0.0,
            demolished: self.time_demolished > 0.0,
            possession: PositioningPossessionState::from_event(self),
            teammate_role: PositioningTeammateRoleState::from_event(self),
            field_zone: PositioningFieldZoneState::from_event(self),
            field_half: PositioningFieldHalfState::from_event(self),
            ball_depth: PositioningBallDepthState::from_event(self),
            closest_to_ball: self.time_closest_to_ball > 0.0,
            farthest_from_ball: self.time_farthest_from_ball > 0.0,
        }
    }

    fn absorb_delta(&mut self, delta: Self) {
        self.end_time = delta.time;
        self.end_frame = delta.frame;
        self.duration += delta.sample_duration();
        self.player_position = delta.player_position;
        self.active_game_time += delta.active_game_time;
        self.tracked_time += delta.tracked_time;
        self.sum_distance_to_teammates += delta.sum_distance_to_teammates;
        self.sum_distance_to_ball += delta.sum_distance_to_ball;
        self.sum_distance_to_ball_has_possession += delta.sum_distance_to_ball_has_possession;
        self.time_has_possession += delta.time_has_possession;
        self.sum_distance_to_ball_no_possession += delta.sum_distance_to_ball_no_possession;
        self.time_no_possession += delta.time_no_possession;
        self.time_demolished += delta.time_demolished;
        self.time_no_teammates += delta.time_no_teammates;
        self.time_most_back += delta.time_most_back;
        self.time_most_forward += delta.time_most_forward;
        self.time_mid_role += delta.time_mid_role;
        self.time_other_role += delta.time_other_role;
        self.time_defensive_zone += delta.time_defensive_zone;
        self.time_neutral_zone += delta.time_neutral_zone;
        self.time_offensive_zone += delta.time_offensive_zone;
        self.time_defensive_half += delta.time_defensive_half;
        self.time_offensive_half += delta.time_offensive_half;
        self.time_closest_to_ball += delta.time_closest_to_ball;
        self.time_farthest_from_ball += delta.time_farthest_from_ball;
        self.time_behind_ball += delta.time_behind_ball;
        self.time_level_with_ball += delta.time_level_with_ball;
        self.time_in_front_of_ball += delta.time_in_front_of_ball;
        self.times_caught_ahead_of_play_on_conceded_goals +=
            delta.times_caught_ahead_of_play_on_conceded_goals;
    }

    fn sample_duration(&self) -> f32 {
        [
            self.active_game_time,
            self.tracked_time,
            self.time_has_possession,
            self.time_no_possession,
            self.time_demolished,
            self.time_no_teammates,
            self.time_most_back,
            self.time_most_forward,
            self.time_mid_role,
            self.time_other_role,
            self.time_defensive_zone,
            self.time_neutral_zone,
            self.time_offensive_zone,
            self.time_defensive_half,
            self.time_offensive_half,
            self.time_closest_to_ball,
            self.time_farthest_from_ball,
            self.time_behind_ball,
            self.time_level_with_ball,
            self.time_in_front_of_ball,
        ]
        .into_iter()
        .fold(0.0, f32::max)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct PositioningEventState {
    active: bool,
    demolished: bool,
    possession: PositioningPossessionState,
    teammate_role: PositioningTeammateRoleState,
    field_zone: PositioningFieldZoneState,
    field_half: PositioningFieldHalfState,
    ball_depth: PositioningBallDepthState,
    closest_to_ball: bool,
    farthest_from_ball: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum PositioningPossessionState {
    HasPossession,
    NoPossession,
    #[default]
    Neutral,
}

impl PositioningPossessionState {
    fn from_event(event: &PositioningEvent) -> Self {
        if event.time_has_possession > 0.0 {
            Self::HasPossession
        } else if event.time_no_possession > 0.0 {
            Self::NoPossession
        } else {
            Self::Neutral
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum PositioningTeammateRoleState {
    NoTeammates,
    MostBack,
    MostForward,
    Mid,
    Other,
    #[default]
    Unknown,
}

impl PositioningTeammateRoleState {
    fn from_event(event: &PositioningEvent) -> Self {
        if event.time_no_teammates > 0.0 {
            Self::NoTeammates
        } else if event.time_most_back > 0.0 {
            Self::MostBack
        } else if event.time_most_forward > 0.0 {
            Self::MostForward
        } else if event.time_mid_role > 0.0 {
            Self::Mid
        } else if event.time_other_role > 0.0 {
            Self::Other
        } else {
            Self::Unknown
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum PositioningFieldZoneState {
    Defensive,
    Neutral,
    Offensive,
    #[default]
    Unknown,
}

impl PositioningFieldZoneState {
    fn from_event(event: &PositioningEvent) -> Self {
        max_duration_state(
            [
                (event.time_defensive_zone, Self::Defensive),
                (event.time_neutral_zone, Self::Neutral),
                (event.time_offensive_zone, Self::Offensive),
            ],
            Self::Unknown,
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum PositioningFieldHalfState {
    Defensive,
    Offensive,
    #[default]
    Unknown,
}

impl PositioningFieldHalfState {
    fn from_event(event: &PositioningEvent) -> Self {
        max_duration_state(
            [
                (event.time_defensive_half, Self::Defensive),
                (event.time_offensive_half, Self::Offensive),
            ],
            Self::Unknown,
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum PositioningBallDepthState {
    Behind,
    Level,
    InFront,
    #[default]
    Unknown,
}

impl PositioningBallDepthState {
    fn from_event(event: &PositioningEvent) -> Self {
        max_duration_state(
            [
                (event.time_behind_ball, Self::Behind),
                (event.time_level_with_ball, Self::Level),
                (event.time_in_front_of_ball, Self::InFront),
            ],
            Self::Unknown,
        )
    }
}

fn max_duration_state<T: Copy>(values: impl IntoIterator<Item = (f32, T)>, default: T) -> T {
    values
        .into_iter()
        .filter(|(duration, _)| *duration > 0.0)
        .max_by(|(left, _), (right, _)| left.partial_cmp(right).unwrap())
        .map(|(_, state)| state)
        .unwrap_or(default)
}

#[derive(Debug, Clone, PartialEq)]
struct PendingPositioningEvent {
    state: PositioningEventState,
    event: PositioningEvent,
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
    pending_events: HashMap<PlayerId, PendingPositioningEvent>,
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
        let mut events = self.events.all().to_vec();
        let mut pending = self
            .pending_events
            .iter()
            .map(|(player, pending)| (player.clone(), pending.event.clone()))
            .collect::<Vec<_>>();
        pending.sort_by(|(left, _), (right, _)| format!("{left:?}").cmp(&format!("{right:?}")));
        events.extend(pending.into_iter().map(|(_, event)| event));
        events
    }

    pub fn flush_pending_events(&mut self) {
        let mut pending = self.pending_events.drain().collect::<Vec<_>>();
        pending.sort_by(|(left, _), (right, _)| format!("{left:?}").cmp(&format!("{right:?}")));
        self.events
            .extend(pending.into_iter().map(|(_, pending)| pending.event));
    }

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
                .times_caught_ahead_of_play_on_conceded_goals += 1;
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
                delta.active_game_time += frame.dt;
                delta.time_demolished += frame.dt;
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
                delta.active_game_time += frame.dt;
                delta.tracked_time += frame.dt;
                delta.sum_distance_to_ball += distance_to_ball * frame.dt;

                if possession_player_before_sample == Some(&player.player_id) {
                    delta.time_has_possession += frame.dt;
                    delta.sum_distance_to_ball_has_possession += distance_to_ball * frame.dt;
                } else if possession_player_before_sample.is_some() {
                    delta.time_no_possession += frame.dt;
                    delta.sum_distance_to_ball_no_possession += distance_to_ball * frame.dt;
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
                delta.time_defensive_zone += frame.dt * defensive_zone_fraction;
                delta.time_neutral_zone += frame.dt * neutral_zone_fraction;
                delta.time_offensive_zone += frame.dt * offensive_zone_fraction;

                let defensive_half_fraction = interval_fraction_below_threshold(
                    normalized_previous_position_y,
                    normalized_position_y,
                    0.0,
                );
                delta.time_defensive_half += frame.dt * defensive_half_fraction;
                delta.time_offensive_half += frame.dt * (1.0 - defensive_half_fraction);

                let previous_ball_delta =
                    normalized_previous_position_y - normalized_previous_ball_y;
                let current_ball_delta = normalized_position_y - normalized_ball_y;
                let (behind_ball_fraction, level_ball_fraction, in_front_ball_fraction) =
                    ball_depth_fractions(
                        self.config.level_ball_depth_margin,
                        previous_ball_delta,
                        current_ball_delta,
                    );
                delta.time_behind_ball += frame.dt * behind_ball_fraction;
                delta.time_level_with_ball += frame.dt * level_ball_fraction;
                delta.time_in_front_of_ball += frame.dt * in_front_ball_fraction;
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
                        .sum_distance_to_teammates +=
                            teammate_distance_sum * frame.dt / teammate_count as f32;
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
                            let player_position = players.player_position(player_id);
                            Self::event_delta(
                                &mut event_deltas,
                                frame,
                                player_id,
                                player_position,
                                is_team_0,
                            )
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
                                Self::event_delta(
                                    &mut event_deltas,
                                    frame,
                                    player_id,
                                    players.player_position(player_id),
                                    is_team_0,
                                )
                                .time_most_back += frame.dt;
                            } else if near_front && !near_back {
                                Self::event_delta(
                                    &mut event_deltas,
                                    frame,
                                    player_id,
                                    players.player_position(player_id),
                                    is_team_0,
                                )
                                .time_most_forward += frame.dt;
                            } else if can_assign_mid_role {
                                Self::event_delta(
                                    &mut event_deltas,
                                    frame,
                                    player_id,
                                    players.player_position(player_id),
                                    is_team_0,
                                )
                                .time_mid_role += frame.dt;
                            } else {
                                Self::event_delta(
                                    &mut event_deltas,
                                    frame,
                                    player_id,
                                    players.player_position(player_id),
                                    is_team_0,
                                )
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
                    Self::event_delta(
                        &mut event_deltas,
                        frame,
                        &closest_player.player_id,
                        closest_player
                            .position()
                            .map(|position| position.to_array()),
                        closest_player.is_team_0,
                    )
                    .time_closest_to_ball += frame.dt;
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
                    .time_farthest_from_ball += frame.dt;
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
        live_play: bool,
        possession_player_before_sample: Option<&PlayerId>,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
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

    fn record_positioning_delta_events(&mut self, mut frame_events: Vec<PositioningEvent>) {
        frame_events.sort_by(|left, right| {
            format!("{:?}", left.player).cmp(&format!("{:?}", right.player))
        });
        let active_players = frame_events
            .iter()
            .map(|event| event.player.clone())
            .collect::<HashSet<_>>();
        self.flush_pending_events_for_missing_players(&active_players);

        for event in frame_events {
            let state = event.timing_state();
            let player = event.player.clone();
            let Some(pending) = self.pending_events.get_mut(&player) else {
                self.pending_events.insert(
                    player,
                    PendingPositioningEvent {
                        state,
                        event: Self::new_pending_event(event),
                    },
                );
                continue;
            };

            if pending.state == state {
                pending.event.absorb_delta(event);
            } else {
                let previous = self.pending_events.insert(
                    player,
                    PendingPositioningEvent {
                        state,
                        event: Self::new_pending_event(event),
                    },
                );
                let Some(previous) = previous else {
                    continue;
                };
                self.events.push(previous.event);
            }
        }
    }

    fn new_pending_event(mut event: PositioningEvent) -> PositioningEvent {
        event.duration = event.sample_duration();
        event
    }

    fn flush_pending_events_for_missing_players(&mut self, active_players: &HashSet<PlayerId>) {
        let mut inactive_players = self
            .pending_events
            .keys()
            .filter(|player| !active_players.contains(*player))
            .cloned()
            .collect::<Vec<_>>();
        inactive_players.sort_by(|left, right| format!("{left:?}").cmp(&format!("{right:?}")));
        for player in inactive_players {
            if let Some(pending) = self.pending_events.remove(&player) {
                self.events.push(pending.event);
            }
        }
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
