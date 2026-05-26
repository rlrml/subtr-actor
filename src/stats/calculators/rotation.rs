use super::*;

const DEFAULT_ROLE_DEPTH_MARGIN: f32 = 150.0;
const DEFAULT_FIRST_MAN_AMBIGUITY_MARGIN: f32 = 250.0;
const DEFAULT_FIRST_MAN_DEBOUNCE_SECONDS: f32 = 0.35;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RoleState {
    #[default]
    Unknown,
    FirstMan,
    SecondMan,
    ThirdMan,
    Ambiguous,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PlayDepthState {
    #[default]
    Unknown,
    BehindPlay,
    LevelWithPlay,
    AheadOfPlay,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct RotationPlayerStats {
    pub active_game_time: f32,
    pub tracked_time: f32,
    pub time_first_man: f32,
    pub time_second_man: f32,
    pub time_third_man: f32,
    pub time_ambiguous_role: f32,
    pub time_behind_play: f32,
    pub time_level_with_play: f32,
    pub time_ahead_of_play: f32,
    pub longest_first_man_stint_time: f32,
    pub first_man_stint_count: u32,
    pub became_first_man_count: u32,
    pub lost_first_man_count: u32,
    pub current_role_state: RoleState,
    pub current_depth_state: PlayDepthState,
}

impl RotationPlayerStats {
    fn role_pct(&self, value: f32) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            value * 100.0 / self.tracked_time
        }
    }

    pub fn first_man_pct(&self) -> f32 {
        self.role_pct(self.time_first_man)
    }

    pub fn second_man_pct(&self) -> f32 {
        self.role_pct(self.time_second_man)
    }

    pub fn third_man_pct(&self) -> f32 {
        self.role_pct(self.time_third_man)
    }

    pub fn ambiguous_role_pct(&self) -> f32 {
        self.role_pct(self.time_ambiguous_role)
    }

    pub fn behind_play_pct(&self) -> f32 {
        self.role_pct(self.time_behind_play)
    }

    pub fn level_with_play_pct(&self) -> f32 {
        self.role_pct(self.time_level_with_play)
    }

    pub fn ahead_of_play_pct(&self) -> f32 {
        self.role_pct(self.time_ahead_of_play)
    }

    pub fn average_first_man_stint_time(&self) -> f32 {
        if self.first_man_stint_count == 0 {
            0.0
        } else {
            self.time_first_man / self.first_man_stint_count as f32
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct RotationTeamStats {
    pub first_man_changes_for_team: u32,
    pub rotation_count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RotationPlayerEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub active: bool,
    pub became_first_man_count: u32,
    pub lost_first_man_count: u32,
    pub current_role_state: RoleState,
    pub current_depth_state: PlayDepthState,
}

impl RotationPlayerEvent {
    fn new(
        frame: &FrameInfo,
        player: PlayerId,
        is_team_0: bool,
        active: bool,
        current_role_state: RoleState,
        current_depth_state: PlayDepthState,
    ) -> Self {
        Self {
            time: frame.time,
            frame: frame.frame_number,
            player,
            is_team_0,
            active,
            became_first_man_count: 0,
            lost_first_man_count: 0,
            current_role_state,
            current_depth_state,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RotationTeamEvent {
    pub time: f32,
    pub frame: usize,
    pub is_team_0: bool,
    pub first_man_changes_for_team: u32,
    pub rotation_count: u32,
}

#[derive(Debug, Clone)]
pub struct RotationCalculatorConfig {
    pub role_depth_margin: f32,
    pub first_man_ambiguity_margin: f32,
    pub first_man_debounce_seconds: f32,
}

impl Default for RotationCalculatorConfig {
    fn default() -> Self {
        Self {
            role_depth_margin: DEFAULT_ROLE_DEPTH_MARGIN,
            first_man_ambiguity_margin: DEFAULT_FIRST_MAN_AMBIGUITY_MARGIN,
            first_man_debounce_seconds: DEFAULT_FIRST_MAN_DEBOUNCE_SECONDS,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct TeamFirstManTracker {
    stable_first_man: Option<PlayerId>,
    pending_first_man: Option<PlayerId>,
    pending_seconds: f32,
}

impl TeamFirstManTracker {
    fn reset(&mut self) {
        self.stable_first_man = None;
        self.pending_first_man = None;
        self.pending_seconds = 0.0;
    }

    fn update(
        &mut self,
        raw_first_man: Option<&PlayerId>,
        dt: f32,
        debounce_seconds: f32,
    ) -> Option<(PlayerId, PlayerId)> {
        let Some(raw_first_man) = raw_first_man else {
            self.pending_first_man = None;
            self.pending_seconds = 0.0;
            return None;
        };

        match self.stable_first_man.as_ref() {
            None => {
                self.stable_first_man = Some(raw_first_man.clone());
                self.pending_first_man = None;
                self.pending_seconds = 0.0;
                None
            }
            Some(stable_first_man) if stable_first_man == raw_first_man => {
                self.pending_first_man = None;
                self.pending_seconds = 0.0;
                None
            }
            Some(stable_first_man) => {
                if self.pending_first_man.as_ref() == Some(raw_first_man) {
                    self.pending_seconds += dt;
                } else {
                    self.pending_first_man = Some(raw_first_man.clone());
                    self.pending_seconds = dt;
                }

                if self.pending_seconds >= debounce_seconds {
                    let previous = stable_first_man.clone();
                    let next = raw_first_man.clone();
                    self.stable_first_man = Some(next.clone());
                    self.pending_first_man = None;
                    self.pending_seconds = 0.0;
                    Some((previous, next))
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RotationPlayerEventState {
    active: bool,
    current_role_state: RoleState,
    current_depth_state: PlayDepthState,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct FirstManStintState {
    active: bool,
    current_first_man_time: f32,
    non_first_man_seconds: f32,
}

#[derive(Debug, Clone, Default)]
pub struct RotationCalculator {
    config: RotationCalculatorConfig,
    player_stats: HashMap<PlayerId, RotationPlayerStats>,
    team_zero_stats: RotationTeamStats,
    team_one_stats: RotationTeamStats,
    team_zero_tracker: TeamFirstManTracker,
    team_one_tracker: TeamFirstManTracker,
    player_events: Vec<RotationPlayerEvent>,
    team_events: Vec<RotationTeamEvent>,
    last_emitted_player_states: HashMap<PlayerId, RotationPlayerEventState>,
    first_man_stints: HashMap<PlayerId, FirstManStintState>,
}

impl RotationCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: RotationCalculatorConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn config(&self) -> &RotationCalculatorConfig {
        &self.config
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, RotationPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &RotationTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &RotationTeamStats {
        &self.team_one_stats
    }

    pub fn player_events(&self) -> &[RotationPlayerEvent] {
        &self.player_events
    }

    pub fn team_events(&self) -> &[RotationTeamEvent] {
        &self.team_events
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        if frame.dt == 0.0 {
            return Ok(());
        }

        let Some(ball) = ball.sample() else {
            self.reset_trackers();
            self.emit_inactive_player_events(frame, players);
            return Ok(());
        };

        if !live_play || !events.goal_events.is_empty() {
            self.reset_trackers();
            self.emit_inactive_player_events(frame, players);
            return Ok(());
        }

        let demoed_players: HashSet<_> = events
            .active_demos
            .iter()
            .map(|demo| demo.victim.clone())
            .collect();
        let ball_position = ball.position();

        self.update_team(
            true,
            frame,
            gameplay,
            ball_position,
            players,
            &demoed_players,
        );
        self.update_team(
            false,
            frame,
            gameplay,
            ball_position,
            players,
            &demoed_players,
        );

        Ok(())
    }

    fn emit_inactive_player_events(&mut self, frame: &FrameInfo, players: &PlayerFrameState) {
        for player in &players.players {
            self.close_first_man_stint(&player.player_id);
            let stats = self
                .player_stats
                .entry(player.player_id.clone())
                .or_default();
            let current_role_state = stats.current_role_state;
            let current_depth_state = stats.current_depth_state;
            self.emit_player_event_if_changed(
                frame,
                &player.player_id,
                player.is_team_0,
                false,
                current_role_state,
                current_depth_state,
                0,
                0,
            );
        }
    }

    fn reset_trackers(&mut self) {
        self.team_zero_tracker.reset();
        self.team_one_tracker.reset();
    }

    fn close_first_man_stint(&mut self, player_id: &PlayerId) {
        if let Some(state) = self.first_man_stints.get_mut(player_id) {
            state.active = false;
            state.current_first_man_time = 0.0;
            state.non_first_man_seconds = 0.0;
        }
    }

    fn update_first_man_stint(
        &mut self,
        player_id: &PlayerId,
        stats: &mut RotationPlayerStats,
        role_state: RoleState,
        dt: f32,
    ) {
        let state = self.first_man_stints.entry(player_id.clone()).or_default();
        if role_state == RoleState::FirstMan {
            if !state.active {
                state.active = true;
                state.current_first_man_time = 0.0;
                stats.first_man_stint_count += 1;
            }
            state.current_first_man_time += dt;
            stats.longest_first_man_stint_time = stats
                .longest_first_man_stint_time
                .max(state.current_first_man_time);
            state.non_first_man_seconds = 0.0;
            return;
        }

        if state.active {
            state.non_first_man_seconds += dt;
            if state.non_first_man_seconds > self.config.first_man_debounce_seconds {
                state.active = false;
                state.current_first_man_time = 0.0;
                state.non_first_man_seconds = 0.0;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_player_event_if_changed(
        &mut self,
        frame: &FrameInfo,
        player_id: &PlayerId,
        is_team_0: bool,
        active: bool,
        current_role_state: RoleState,
        current_depth_state: PlayDepthState,
        became_first_man_count: u32,
        lost_first_man_count: u32,
    ) {
        let state = RotationPlayerEventState {
            active,
            current_role_state,
            current_depth_state,
        };
        let state_changed = self.last_emitted_player_states.get(player_id) != Some(&state);
        if !state_changed && became_first_man_count == 0 && lost_first_man_count == 0 {
            return;
        }

        let mut event = RotationPlayerEvent::new(
            frame,
            player_id.clone(),
            is_team_0,
            active,
            current_role_state,
            current_depth_state,
        );
        event.became_first_man_count = became_first_man_count;
        event.lost_first_man_count = lost_first_man_count;
        self.player_events.push(event);
        self.last_emitted_player_states
            .insert(player_id.clone(), state);
    }

    fn update_team(
        &mut self,
        is_team_0: bool,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball_position: glam::Vec3,
        players: &PlayerFrameState,
        demoed_players: &HashSet<PlayerId>,
    ) {
        let present_team_count = players
            .players
            .iter()
            .filter(|player| player.is_team_0 == is_team_0)
            .count();
        let team_size = gameplay
            .current_in_game_team_player_count(is_team_0)
            .max(present_team_count);

        let team_players: Vec<_> = players
            .players
            .iter()
            .filter(|player| player.is_team_0 == is_team_0)
            .filter(|player| !demoed_players.contains(&player.player_id))
            .filter_map(|player| player.position().map(|position| (player, position)))
            .collect();

        if !(2..=3).contains(&team_size) || team_players.len() != team_size {
            self.team_tracker_mut(is_team_0).reset();
            for player in players
                .players
                .iter()
                .filter(|player| player.is_team_0 == is_team_0)
            {
                self.close_first_man_stint(&player.player_id);
                let (current_role_state, current_depth_state) = {
                    let stats = self
                        .player_stats
                        .entry(player.player_id.clone())
                        .or_default();
                    stats.current_role_state = RoleState::Unknown;
                    (stats.current_role_state, stats.current_depth_state)
                };
                self.emit_player_event_if_changed(
                    frame,
                    &player.player_id,
                    player.is_team_0,
                    false,
                    current_role_state,
                    current_depth_state,
                    0,
                    0,
                );
            }
            return;
        }

        let mut became_first_man_counts = HashMap::<PlayerId, u32>::new();
        let mut lost_first_man_counts = HashMap::<PlayerId, u32>::new();
        let mut scored_players: Vec<_> = team_players
            .iter()
            .map(|(player, position)| {
                (
                    player.player_id.clone(),
                    first_man_score(*position, ball_position),
                )
            })
            .collect();
        scored_players.sort_by(|(_, left_score), (_, right_score)| {
            left_score.partial_cmp(right_score).unwrap()
        });

        let raw_first_man = raw_first_man(&scored_players, self.config.first_man_ambiguity_margin);
        let debounce_seconds = self.config.first_man_debounce_seconds;
        let change =
            self.team_tracker_mut(is_team_0)
                .update(raw_first_man, frame.dt, debounce_seconds);
        if let Some((previous, next)) = change {
            let team_stats = self.team_stats_mut(is_team_0);
            team_stats.first_man_changes_for_team += 1;
            team_stats.rotation_count += 1;
            self.team_events.push(RotationTeamEvent {
                time: frame.time,
                frame: frame.frame_number,
                is_team_0,
                first_man_changes_for_team: 1,
                rotation_count: 1,
            });
            let previous_stats = self.player_stats.entry(previous.clone()).or_default();
            previous_stats.lost_first_man_count += 1;
            *lost_first_man_counts.entry(previous).or_default() += 1;
            let next_stats = self.player_stats.entry(next.clone()).or_default();
            next_stats.became_first_man_count += 1;
            *became_first_man_counts.entry(next).or_default() += 1;
        }

        let stable_first_man = raw_first_man
            .and_then(|_| self.team_tracker(is_team_0).stable_first_man.as_ref())
            .cloned();
        let role_assignments = role_assignments(stable_first_man.as_ref(), &scored_players);

        for (player, position) in team_players {
            let role_state = role_assignments
                .get(&player.player_id)
                .copied()
                .unwrap_or(RoleState::Ambiguous);
            let depth_state = play_depth_state(
                is_team_0,
                position,
                ball_position,
                self.config.role_depth_margin,
            );
            let (current_role_state, current_depth_state) = {
                let mut stats = self
                    .player_stats
                    .remove(&player.player_id)
                    .unwrap_or_default();
                stats.active_game_time += frame.dt;
                stats.tracked_time += frame.dt;
                stats.current_role_state = role_state;
                stats.current_depth_state = depth_state;
                self.update_first_man_stint(&player.player_id, &mut stats, role_state, frame.dt);

                match role_state {
                    RoleState::FirstMan => {
                        stats.time_first_man += frame.dt;
                    }
                    RoleState::SecondMan => {
                        stats.time_second_man += frame.dt;
                    }
                    RoleState::ThirdMan => {
                        stats.time_third_man += frame.dt;
                    }
                    RoleState::Ambiguous => {
                        stats.time_ambiguous_role += frame.dt;
                    }
                    RoleState::Unknown => {}
                }

                match depth_state {
                    PlayDepthState::BehindPlay => {
                        stats.time_behind_play += frame.dt;
                    }
                    PlayDepthState::LevelWithPlay => {
                        stats.time_level_with_play += frame.dt;
                    }
                    PlayDepthState::AheadOfPlay => {
                        stats.time_ahead_of_play += frame.dt;
                    }
                    PlayDepthState::Unknown => {}
                }

                let current_role_state = stats.current_role_state;
                let current_depth_state = stats.current_depth_state;
                self.player_stats.insert(player.player_id.clone(), stats);
                (current_role_state, current_depth_state)
            };
            let became_first_man_count = became_first_man_counts
                .remove(&player.player_id)
                .unwrap_or_default();
            let lost_first_man_count = lost_first_man_counts
                .remove(&player.player_id)
                .unwrap_or_default();
            self.emit_player_event_if_changed(
                frame,
                &player.player_id,
                player.is_team_0,
                true,
                current_role_state,
                current_depth_state,
                became_first_man_count,
                lost_first_man_count,
            );
        }

        for (player_id, count) in became_first_man_counts {
            let (current_role_state, current_depth_state) = {
                let stats = self.player_stats.entry(player_id.clone()).or_default();
                (stats.current_role_state, stats.current_depth_state)
            };
            self.emit_player_event_if_changed(
                frame,
                &player_id,
                is_team_0,
                false,
                current_role_state,
                current_depth_state,
                count,
                0,
            );
        }
        for (player_id, count) in lost_first_man_counts {
            let (current_role_state, current_depth_state) = {
                let stats = self.player_stats.entry(player_id.clone()).or_default();
                (stats.current_role_state, stats.current_depth_state)
            };
            self.emit_player_event_if_changed(
                frame,
                &player_id,
                is_team_0,
                false,
                current_role_state,
                current_depth_state,
                0,
                count,
            );
        }
    }

    fn team_tracker(&self, is_team_0: bool) -> &TeamFirstManTracker {
        if is_team_0 {
            &self.team_zero_tracker
        } else {
            &self.team_one_tracker
        }
    }

    fn team_tracker_mut(&mut self, is_team_0: bool) -> &mut TeamFirstManTracker {
        if is_team_0 {
            &mut self.team_zero_tracker
        } else {
            &mut self.team_one_tracker
        }
    }

    fn team_stats_mut(&mut self, is_team_0: bool) -> &mut RotationTeamStats {
        if is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        }
    }
}

fn first_man_score(player_position: glam::Vec3, ball_position: glam::Vec3) -> f32 {
    player_position
        .truncate()
        .distance(ball_position.truncate())
}

fn raw_first_man(scored_players: &[(PlayerId, f32)], ambiguity_margin: f32) -> Option<&PlayerId> {
    let [(first_id, first_score), (_, second_score), ..] = scored_players else {
        return None;
    };

    if second_score - first_score <= ambiguity_margin {
        None
    } else {
        Some(first_id)
    }
}

fn role_assignments(
    stable_first_man: Option<&PlayerId>,
    scored_players: &[(PlayerId, f32)],
) -> HashMap<PlayerId, RoleState> {
    let mut assignments = HashMap::new();
    let Some(stable_first_man) = stable_first_man else {
        for (player_id, _) in scored_players {
            assignments.insert(player_id.clone(), RoleState::Ambiguous);
        }
        return assignments;
    };

    assignments.insert(stable_first_man.clone(), RoleState::FirstMan);
    let mut support_rank = 0;
    for (player_id, _) in scored_players {
        if player_id == stable_first_man {
            continue;
        }
        support_rank += 1;
        let role = match support_rank {
            1 => RoleState::SecondMan,
            2 => RoleState::ThirdMan,
            _ => RoleState::Ambiguous,
        };
        assignments.insert(player_id.clone(), role);
    }
    assignments
}

fn play_depth_state(
    is_team_0: bool,
    player_position: glam::Vec3,
    ball_position: glam::Vec3,
    margin: f32,
) -> PlayDepthState {
    let player_y = normalized_y(is_team_0, player_position);
    let ball_y = normalized_y(is_team_0, ball_position);
    let delta = player_y - ball_y;
    if delta < -margin {
        PlayDepthState::BehindPlay
    } else if delta > margin {
        PlayDepthState::AheadOfPlay
    } else {
        PlayDepthState::LevelWithPlay
    }
}

#[cfg(test)]
#[path = "rotation_tests.rs"]
mod tests;
