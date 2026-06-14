use super::*;

const DEFAULT_LEVEL_BALL_DEPTH_MARGIN: f32 = 150.0;
const DEFAULT_CLOSEST_TO_BALL_SWITCH_MARGIN: f32 = 100.0;
const DEFAULT_CLOSEST_TO_BALL_SWITCH_MIN_SECONDS: f32 = 0.2;

/// Whether the player is participating in the frame: on the field being
/// tracked, or waiting out a demolition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ActivityState {
    Tracked,
    Demolished,
}

/// Which third of the field (relative to the player's own goal) the player
/// occupies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum FieldThirdState {
    Defensive,
    Neutral,
    Offensive,
}

/// Which half of the field (relative to the player's own goal) the player
/// occupies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum FieldHalfState {
    Defensive,
    Offensive,
}

/// The player's depth relative to the ball along the attacking axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum BallDepthState {
    BehindBall,
    LevelWithBall,
    AheadOfBall,
}

/// The player's depth-ordered role within their team (most back / most
/// forward / mid by normalized field depth).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DepthRoleState {
    NoTeammates,
    MostBack,
    MostForward,
    Mid,
    Other,
}

/// Ball-distance designations the player currently holds. These are not
/// mutually exclusive (the absolute closest player is usually also their
/// team's closest), so they ride one event as flags rather than separate
/// per-designation streams.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct BallProximityState {
    pub closest_to_ball_team: bool,
    pub closest_to_ball_absolute: bool,
    pub farthest_from_ball: bool,
}

impl BallProximityState {
    fn any(self) -> bool {
        self.closest_to_ball_team || self.closest_to_ball_absolute || self.farthest_from_ball
    }
}

pub type PlayerActivityEvent = PlayerStateSpan<ActivityState>;
pub type FieldThirdEvent = PlayerStateSpan<FieldThirdState>;
pub type FieldHalfEvent = PlayerStateSpan<FieldHalfState>;
pub type BallDepthEvent = PlayerStateSpan<BallDepthState>;
pub type DepthRoleEvent = PlayerStateSpan<DepthRoleState>;
pub type BallProximityEvent = PlayerStateSpan<BallProximityState>;

/// Possession context used in positioning classification.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PositioningPossessionState {
    HasPossession,
    NoPossession,
    #[default]
    Neutral,
}

/// Cumulative per-player continuous totals.
///
/// Distance to the ball/teammates is a continuous magnitude, not a discrete occurrence, so
/// it cannot be reconstructed from an event stream; the calculator accumulates these running
/// totals as it processes frames and the timeline ships them directly. Possession is an input
/// to positioning rather than a positioning facet of its own, so the possession-split distance
/// sums and the possession *times* that denominate them both ride this continuous channel
/// rather than being emitted as positioning events.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PositioningSignalSnapshot {
    pub sum_distance_to_teammates: f32,
    pub sum_distance_to_ball: f32,
    pub sum_distance_to_ball_has_possession: f32,
    pub time_has_possession: f32,
    pub sum_distance_to_ball_no_possession: f32,
    pub time_no_possession: f32,
}

impl PositioningSignalSnapshot {
    fn accumulate(&mut self, facets: &PlayerFrameFacets, dt: f32) {
        if let Some(distance) = facets.distance_to_teammates {
            self.sum_distance_to_teammates += distance * dt;
        }
        let distance = facets.distance_to_ball;
        if let Some(distance) = distance {
            self.sum_distance_to_ball += distance * dt;
        }
        match facets.possession_state {
            PositioningPossessionState::HasPossession => {
                self.time_has_possession += dt;
                if let Some(distance) = distance {
                    self.sum_distance_to_ball_has_possession += distance * dt;
                }
            }
            PositioningPossessionState::NoPossession => {
                self.time_no_possession += dt;
                if let Some(distance) = distance {
                    self.sum_distance_to_ball_no_possession += distance * dt;
                }
            }
            PositioningPossessionState::Neutral => {}
        }
    }
}

/// Configuration thresholds for positioning classification.
#[derive(Debug, Clone)]
pub struct PositioningCalculatorConfig {
    pub most_back_forward_threshold_y: f32,
    pub level_ball_depth_margin: f32,
    pub closest_to_ball_switch_margin: f32,
    pub closest_to_ball_switch_min_seconds: f32,
}

impl Default for PositioningCalculatorConfig {
    fn default() -> Self {
        Self {
            most_back_forward_threshold_y: DEFAULT_MOST_BACK_FORWARD_THRESHOLD_Y,
            level_ball_depth_margin: DEFAULT_LEVEL_BALL_DEPTH_MARGIN,
            closest_to_ball_switch_margin: DEFAULT_CLOSEST_TO_BALL_SWITCH_MARGIN,
            closest_to_ball_switch_min_seconds: DEFAULT_CLOSEST_TO_BALL_SWITCH_MIN_SECONDS,
        }
    }
}

#[derive(Debug, Clone)]
struct ClosestToBallCandidate {
    player_id: PlayerId,
    distance: f32,
}

impl ClosestToBallCandidate {
    fn from_player(player: &PlayerSample, position: glam::Vec3, ball_position: glam::Vec3) -> Self {
        Self {
            player_id: player.player_id.clone(),
            distance: position.distance(ball_position),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct ClosestToBallDebouncer {
    current_player: Option<PlayerId>,
    pending_player: Option<PlayerId>,
    pending_seconds: f32,
}

impl ClosestToBallDebouncer {
    fn select(
        &mut self,
        candidates: &[ClosestToBallCandidate],
        dt: f32,
        switch_margin: f32,
        switch_min_seconds: f32,
    ) -> Option<ClosestToBallCandidate> {
        let raw_closest = candidates
            .iter()
            .min_by(|left, right| left.distance.partial_cmp(&right.distance).unwrap())?;
        let Some(current_player) = self.current_player.as_ref() else {
            self.current_player = Some(raw_closest.player_id.clone());
            self.pending_player = None;
            self.pending_seconds = 0.0;
            return Some(raw_closest.clone());
        };
        let Some(current) = candidates
            .iter()
            .find(|candidate| candidate.player_id == *current_player)
        else {
            self.current_player = Some(raw_closest.player_id.clone());
            self.pending_player = None;
            self.pending_seconds = 0.0;
            return Some(raw_closest.clone());
        };
        if raw_closest.player_id == current.player_id {
            self.pending_player = None;
            self.pending_seconds = 0.0;
            return Some(current.clone());
        }
        if raw_closest.distance + switch_margin >= current.distance {
            self.pending_player = None;
            self.pending_seconds = 0.0;
            return Some(current.clone());
        }
        if self.pending_player.as_ref() == Some(&raw_closest.player_id) {
            self.pending_seconds += dt;
        } else {
            self.pending_player = Some(raw_closest.player_id.clone());
            self.pending_seconds = dt;
        }
        if self.pending_seconds >= switch_min_seconds.max(0.0) {
            self.current_player = Some(raw_closest.player_id.clone());
            self.pending_player = None;
            self.pending_seconds = 0.0;
            Some(raw_closest.clone())
        } else {
            Some(current.clone())
        }
    }

    fn clear(&mut self) {
        self.current_player = None;
        self.pending_player = None;
        self.pending_seconds = 0.0;
    }
}

/// Everything one frame contributes for one player, gathered before any spans
/// are recorded so emission can run in a deterministic player order.
#[derive(Debug, Clone, Default)]
struct PlayerFrameFacets {
    player_position: Option<[f32; 3]>,
    is_team_0: bool,
    activity: Option<ActivityState>,
    field_third_segments: Vec<(FieldThirdState, f32)>,
    field_half_segments: Vec<(FieldHalfState, f32)>,
    ball_depth_segments: Vec<(BallDepthState, f32)>,
    depth_role: Option<DepthRoleState>,
    proximity: BallProximityState,
    distance_to_ball: Option<f32>,
    distance_to_teammates: Option<f32>,
    possession_state: PositioningPossessionState,
}

/// Tracks per-player field positioning over time.
#[derive(Debug, Clone, Default)]
pub struct PositioningCalculator {
    config: PositioningCalculatorConfig,
    previous_ball_position: Option<glam::Vec3>,
    previous_player_positions: HashMap<PlayerId, glam::Vec3>,
    absolute_closest_to_ball: ClosestToBallDebouncer,
    team_zero_closest_to_ball: ClosestToBallDebouncer,
    team_one_closest_to_ball: ClosestToBallDebouncer,
    activity: PlayerSpanTracker<ActivityState>,
    field_third: PlayerSpanTracker<FieldThirdState>,
    field_half: PlayerSpanTracker<FieldHalfState>,
    ball_depth: PlayerSpanTracker<BallDepthState>,
    depth_role: PlayerSpanTracker<DepthRoleState>,
    ball_proximity: PlayerSpanTracker<BallProximityState>,
    signal: HashMap<PlayerId, PositioningSignalSnapshot>,
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

    pub fn activity_events(&self) -> Vec<PlayerActivityEvent> {
        self.activity.projected_events()
    }

    pub fn field_third_events(&self) -> Vec<FieldThirdEvent> {
        self.field_third.projected_events()
    }

    pub fn field_half_events(&self) -> Vec<FieldHalfEvent> {
        self.field_half.projected_events()
    }

    pub fn ball_depth_events(&self) -> Vec<BallDepthEvent> {
        self.ball_depth.projected_events()
    }

    pub fn depth_role_events(&self) -> Vec<DepthRoleEvent> {
        self.depth_role.projected_events()
    }

    pub fn ball_proximity_events(&self) -> Vec<BallProximityEvent> {
        self.ball_proximity.projected_events()
    }

    /// Players with a span (any facet) closed during the current frame's update.
    pub fn new_event_players(&self) -> Vec<PlayerId> {
        let mut players: Vec<PlayerId> = self
            .activity
            .new_events()
            .iter()
            .map(|span| span.player.clone())
            .chain(
                self.field_third
                    .new_events()
                    .iter()
                    .map(|span| span.player.clone()),
            )
            .chain(
                self.field_half
                    .new_events()
                    .iter()
                    .map(|span| span.player.clone()),
            )
            .chain(
                self.ball_depth
                    .new_events()
                    .iter()
                    .map(|span| span.player.clone()),
            )
            .chain(
                self.depth_role
                    .new_events()
                    .iter()
                    .map(|span| span.player.clone()),
            )
            .chain(
                self.ball_proximity
                    .new_events()
                    .iter()
                    .map(|span| span.player.clone()),
            )
            .collect();
        players.dedup();
        players
    }

    /// Close every open span so the projected event streams are final.
    pub fn flush_pending_events(&mut self) {
        self.activity.close_all();
        self.field_third.close_all();
        self.field_half.close_all();
        self.ball_depth.close_all();
        self.depth_role.close_all();
        self.ball_proximity.close_all();
    }

    fn close_all_spans(&mut self) {
        self.flush_pending_events();
    }

    fn store_previous_positions(
        &mut self,
        ball_position: Option<glam::Vec3>,
        players: &PlayerFrameState,
    ) {
        if let Some(ball_position) = ball_position {
            self.previous_ball_position = Some(ball_position);
        }
        for player in &players.players {
            if let Some(position) = player.position() {
                self.previous_player_positions
                    .insert(player.player_id.clone(), position);
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
            self.store_previous_positions(ball.sample().map(|ball| ball.position()), players);
            return Ok(());
        }

        let Some(ball) = ball.sample() else {
            self.close_all_spans();
            return Ok(());
        };
        let ball_position = ball.position();

        if !live_play {
            self.absolute_closest_to_ball.clear();
            self.team_zero_closest_to_ball.clear();
            self.team_one_closest_to_ball.clear();
            self.close_all_spans();
            self.store_previous_positions(Some(ball_position), players);
            return Ok(());
        }

        let mut facets: HashMap<PlayerId, PlayerFrameFacets> = HashMap::new();
        let demoed_players: HashSet<_> = events
            .active_demos
            .iter()
            .map(|demo| demo.victim.clone())
            .collect();

        for player in &players.players {
            let is_demoed = demoed_players.contains(&player.player_id);
            let entry = facets.entry(player.player_id.clone()).or_default();
            entry.is_team_0 = player.is_team_0;
            entry.player_position = player.position().map(|position| position.to_array());

            if is_demoed {
                entry.activity = Some(ActivityState::Demolished);
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

            entry.activity = Some(ActivityState::Tracked);
            entry.distance_to_ball = Some(position.distance(ball_position));

            if possession_player_before_sample == Some(&player.player_id) {
                entry.possession_state = PositioningPossessionState::HasPossession;
            } else if possession_player_before_sample.is_some() {
                entry.possession_state = PositioningPossessionState::NoPossession;
            }

            entry.field_third_segments = scalar_state_segments(
                normalized_previous_position_y,
                normalized_position_y,
                &[-FIELD_ZONE_BOUNDARY_Y, FIELD_ZONE_BOUNDARY_Y],
                &[
                    FieldThirdState::Defensive,
                    FieldThirdState::Neutral,
                    FieldThirdState::Offensive,
                ],
            );
            entry.field_half_segments = scalar_state_segments(
                normalized_previous_position_y,
                normalized_position_y,
                &[0.0],
                &[FieldHalfState::Defensive, FieldHalfState::Offensive],
            );
            entry.ball_depth_segments = scalar_state_segments(
                normalized_previous_position_y - normalized_previous_ball_y,
                normalized_position_y - normalized_ball_y,
                &[
                    -self.config.level_ball_depth_margin,
                    self.config.level_ball_depth_margin,
                ],
                &[
                    BallDepthState::BehindBall,
                    BallDepthState::LevelWithBall,
                    BallDepthState::AheadOfBall,
                ],
            );
        }

        let positioned_players: Vec<_> = players
            .players
            .iter()
            .filter(|player| !demoed_players.contains(&player.player_id))
            .filter_map(|player| {
                player.position().map(|position| {
                    ClosestToBallCandidate::from_player(player, position, ball_position)
                })
            })
            .collect();
        if let Some(closest_player) = self.absolute_closest_to_ball.select(
            &positioned_players,
            frame.dt,
            self.config.closest_to_ball_switch_margin,
            self.config.closest_to_ball_switch_min_seconds,
        ) {
            if let Some(entry) = facets.get_mut(&closest_player.player_id) {
                entry.proximity.closest_to_ball_absolute = true;
            }
        } else {
            self.absolute_closest_to_ball.clear();
        }

        for is_team_0 in [true, false] {
            let team_present_player_count = players
                .players
                .iter()
                .filter(|player| player.is_team_0 == is_team_0)
                .count();
            let team_roster_count = gameplay
                .current_in_game_team_player_count(is_team_0)
                .max(team_present_player_count);
            let team_players: Vec<_> = players
                .players
                .iter()
                .filter(|player| player.is_team_0 == is_team_0)
                .filter(|player| !demoed_players.contains(&player.player_id))
                .filter_map(|player| player.position().map(|position| (player, position)))
                .collect();

            if team_players.is_empty() {
                if is_team_0 {
                    self.team_zero_closest_to_ball.clear();
                } else {
                    self.team_one_closest_to_ball.clear();
                }
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
                    if let Some(entry) = facets.get_mut(&player.player_id) {
                        entry.distance_to_teammates =
                            Some(teammate_distance_sum / teammate_count as f32);
                    }
                }
            }

            if team_roster_count < 2
                || team_present_player_count < team_roster_count
                || team_players.len() < 2
            {
                for (player, _) in &team_players {
                    if let Some(entry) = facets.get_mut(&player.player_id) {
                        entry.depth_role = Some(DepthRoleState::NoTeammates);
                    }
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
                        if let Some(entry) = facets.get_mut(player_id) {
                            entry.depth_role = Some(DepthRoleState::Other);
                        }
                    }
                } else {
                    let min_y = sorted_team.first().map(|(_, y)| *y).unwrap_or(0.0);
                    let max_y = sorted_team.last().map(|(_, y)| *y).unwrap_or(0.0);
                    let can_assign_mid_role = sorted_team.len() == 3;

                    for (player_id, y) in &sorted_team {
                        let near_back = (*y - min_y) <= self.config.most_back_forward_threshold_y;
                        let near_front = (max_y - *y) <= self.config.most_back_forward_threshold_y;
                        let role = if near_back && !near_front {
                            DepthRoleState::MostBack
                        } else if near_front && !near_back {
                            DepthRoleState::MostForward
                        } else if can_assign_mid_role {
                            DepthRoleState::Mid
                        } else {
                            DepthRoleState::Other
                        };
                        if let Some(entry) = facets.get_mut(player_id) {
                            entry.depth_role = Some(role);
                        }
                    }
                }
            }

            let team_candidates: Vec<_> = team_players
                .iter()
                .map(|(player, position)| {
                    ClosestToBallCandidate::from_player(player, *position, ball_position)
                })
                .collect();
            let team_debouncer = if is_team_0 {
                &mut self.team_zero_closest_to_ball
            } else {
                &mut self.team_one_closest_to_ball
            };
            if let Some(closest_player) = team_debouncer.select(
                &team_candidates,
                frame.dt,
                self.config.closest_to_ball_switch_margin,
                self.config.closest_to_ball_switch_min_seconds,
            ) {
                if let Some(entry) = facets.get_mut(&closest_player.player_id) {
                    entry.proximity.closest_to_ball_team = true;
                }
            }

            if let Some((farthest_player, _)) = team_players.iter().max_by(|(_, a), (_, b)| {
                a.distance(ball_position)
                    .partial_cmp(&b.distance(ball_position))
                    .unwrap()
            }) {
                if let Some(entry) = facets.get_mut(&farthest_player.player_id) {
                    entry.proximity.farthest_from_ball = true;
                }
            }
        }

        self.record_frame_facets(frame, &facets);
        self.store_previous_positions(Some(ball_position), players);

        Ok(())
    }

    /// Emit span contributions for the frame in a deterministic player order,
    /// closing facets that stopped applying so spans never bridge gaps.
    fn record_frame_facets(
        &mut self,
        frame: &FrameInfo,
        facets: &HashMap<PlayerId, PlayerFrameFacets>,
    ) {
        let mut players: Vec<_> = facets.iter().collect();
        players.sort_by_key(|(player, _)| format!("{player:?}"));

        let frame_start = frame.time - frame.dt;
        for (player, entry) in players {
            let tracked = entry.activity == Some(ActivityState::Tracked);
            if tracked {
                self.signal
                    .entry(player.clone())
                    .or_default()
                    .accumulate(entry, frame.dt);
            }

            match entry.activity {
                Some(state) => self.activity.record(
                    frame.frame_number,
                    frame_start,
                    frame.time,
                    frame.dt,
                    player,
                    entry.player_position,
                    entry.is_team_0,
                    state,
                ),
                None => self.activity.close(player),
            }

            record_segments(
                &mut self.field_third,
                frame,
                player,
                entry,
                &entry.field_third_segments,
            );
            record_segments(
                &mut self.field_half,
                frame,
                player,
                entry,
                &entry.field_half_segments,
            );
            record_segments(
                &mut self.ball_depth,
                frame,
                player,
                entry,
                &entry.ball_depth_segments,
            );

            match entry.depth_role {
                Some(role) if tracked => self.depth_role.record(
                    frame.frame_number,
                    frame_start,
                    frame.time,
                    frame.dt,
                    player,
                    entry.player_position,
                    entry.is_team_0,
                    role,
                ),
                _ => self.depth_role.close(player),
            }

            if tracked && entry.proximity.any() {
                self.ball_proximity.record(
                    frame.frame_number,
                    frame_start,
                    frame.time,
                    frame.dt,
                    player,
                    entry.player_position,
                    entry.is_team_0,
                    entry.proximity,
                );
            } else {
                self.ball_proximity.close(player);
            }
        }
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
        self.activity.begin_update();
        self.field_third.begin_update();
        self.field_half.begin_update();
        self.ball_depth.begin_update();
        self.depth_role.begin_update();
        self.ball_proximity.begin_update();
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

    /// Cumulative distance/possession signal for `player` through the frames processed so
    /// far. Continuous distance is shipped as this per-frame snapshot rather than as events.
    pub fn player_signal(&self, player: &PlayerId) -> PositioningSignalSnapshot {
        self.signal.get(player).copied().unwrap_or_default()
    }

    pub fn signals(&self) -> &HashMap<PlayerId, PositioningSignalSnapshot> {
        &self.signal
    }
}

/// Record a frame's ordered sub-frame segments for one fraction-derived facet,
/// or close the player's span when the facet does not apply this frame.
fn record_segments<S: Copy + PartialEq>(
    tracker: &mut PlayerSpanTracker<S>,
    frame: &FrameInfo,
    player: &PlayerId,
    entry: &PlayerFrameFacets,
    segments: &[(S, f32)],
) {
    if segments.is_empty() {
        tracker.close(player);
        return;
    }
    let frame_start = frame.time - frame.dt;
    let mut cumulative = 0.0f32;
    for (state, fraction) in segments {
        let start_time = frame_start + cumulative * frame.dt;
        cumulative += fraction;
        let end_time = if cumulative >= 1.0 {
            frame.time
        } else {
            frame_start + cumulative * frame.dt
        };
        tracker.record(
            frame.frame_number,
            start_time,
            end_time,
            fraction * frame.dt,
            player,
            entry.player_position,
            entry.is_team_0,
            *state,
        );
    }
}

#[cfg(test)]
#[path = "positioning_tests.rs"]
mod tests;
