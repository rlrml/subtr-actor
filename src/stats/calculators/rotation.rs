use super::*;

const DEFAULT_ROLE_DEPTH_MARGIN: f32 = 150.0;
const DEFAULT_FIRST_MAN_AMBIGUITY_MARGIN: f32 = 250.0;
const DEFAULT_FIRST_MAN_DEBOUNCE_SECONDS: f32 = 0.35;
const DEFAULT_FIRST_MAN_STINT_END_GRACE_SECONDS: f32 = 0.35;

/// A player's rotational role (first/second/third man).
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

pub const ALL_ROLE_STATES: [RoleState; 5] = [
    RoleState::Unknown,
    RoleState::FirstMan,
    RoleState::SecondMan,
    RoleState::ThirdMan,
    RoleState::Ambiguous,
];

impl RoleState {
    pub fn as_label(self) -> StatLabel {
        let value = match self {
            Self::Unknown => "unknown",
            Self::FirstMan => "first_man",
            Self::SecondMan => "second_man",
            Self::ThirdMan => "third_man",
            Self::Ambiguous => "ambiguous",
        };
        StatLabel::new("role", value)
    }
}

/// Depth relative to the play, used to tag touches with the toucher's rotation
/// context. This is no longer emitted as its own event stream — the unified
/// `ball_depth` positioning facet covers depth spans on the timeline — but the
/// rotation calculator still computes it per frame for touch classification.
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

pub const ALL_PLAY_DEPTH_STATES: [PlayDepthState; 4] = [
    PlayDepthState::Unknown,
    PlayDepthState::BehindPlay,
    PlayDepthState::LevelWithPlay,
    PlayDepthState::AheadOfPlay,
];

impl PlayDepthState {
    pub fn as_label(self) -> StatLabel {
        let value = match self {
            Self::Unknown => "unknown",
            Self::BehindPlay => "behind_play",
            Self::LevelWithPlay => "level_with_play",
            Self::AheadOfPlay => "ahead_of_play",
        };
        StatLabel::new("play_depth", value)
    }
}

/// A span of game time during which a player held one rotation role. Spans are
/// only emitted while rotation tracking is active (live play, full 2v2/3v3
/// rosters), so they never carry [`RoleState::Unknown`].
pub type RotationRoleEvent = PlayerStateSpan<RoleState>;

/// The debounced first man for a team changed from one player to another.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct FirstManChangeEvent {
    pub time: f32,
    pub frame: usize,
    pub is_team_0: bool,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub previous_first_man: PlayerId,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub next_first_man: PlayerId,
}

/// Configuration thresholds for rotation classification.
#[derive(Debug, Clone)]
pub struct RotationCalculatorConfig {
    pub role_depth_margin: f32,
    pub first_man_ambiguity_margin: f32,
    pub first_man_debounce_seconds: f32,
    pub first_man_stint_end_grace_seconds: f32,
}

impl Default for RotationCalculatorConfig {
    fn default() -> Self {
        Self {
            role_depth_margin: DEFAULT_ROLE_DEPTH_MARGIN,
            first_man_ambiguity_margin: DEFAULT_FIRST_MAN_AMBIGUITY_MARGIN,
            first_man_debounce_seconds: DEFAULT_FIRST_MAN_DEBOUNCE_SECONDS,
            first_man_stint_end_grace_seconds: DEFAULT_FIRST_MAN_STINT_END_GRACE_SECONDS,
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

/// Tracks rotational roles over time.
#[derive(Debug, Clone, Default)]
pub struct RotationCalculator {
    config: RotationCalculatorConfig,
    team_zero_tracker: TeamFirstManTracker,
    team_one_tracker: TeamFirstManTracker,
    role_spans: PlayerSpanTracker<RoleState>,
    first_man_changes: EventStream<FirstManChangeEvent>,
    current_states: HashMap<PlayerId, (RoleState, PlayDepthState)>,
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

    pub fn role_events(&self) -> Vec<RotationRoleEvent> {
        self.role_spans.projected_events()
    }

    pub fn first_man_change_events(&self) -> &[FirstManChangeEvent] {
        self.first_man_changes.all()
    }

    /// Role spans closed during the current frame's update.
    pub fn new_role_events(&self) -> &[RotationRoleEvent] {
        self.role_spans.new_events()
    }

    /// Close every open role span so the projected event stream is final.
    pub fn flush_pending_events(&mut self) {
        self.role_spans.close_all();
    }

    /// The rotation role and play-depth the player currently holds, as of the
    /// most recently processed frame. Used by downstream consumers (e.g. touch
    /// classification) to tag events with the toucher's rotation context.
    pub fn current_role_and_depth(&self, player_id: &PlayerId) -> (RoleState, PlayDepthState) {
        self.current_states
            .get(player_id)
            .copied()
            .unwrap_or_default()
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.role_spans.begin_update();
        self.first_man_changes.begin_update();
        if frame.dt == 0.0 {
            return Ok(());
        }

        let Some(ball) = ball.sample() else {
            self.reset_trackers();
            self.role_spans.close_all();
            return Ok(());
        };

        if !live_play_state.is_live_play || !events.goal_events.is_empty() {
            self.reset_trackers();
            self.role_spans.close_all();
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

    fn reset_trackers(&mut self) {
        self.team_zero_tracker.reset();
        self.team_one_tracker.reset();
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

        let mut team_players: Vec<_> = players
            .players
            .iter()
            .filter(|player| player.is_team_0 == is_team_0)
            .filter(|player| !demoed_players.contains(&player.player_id))
            .filter_map(|player| player.position().map(|position| (player, position)))
            .collect();
        team_players.sort_by_key(|(player, _)| format!("{:?}", player.player_id));

        if !(2..=3).contains(&team_size) || team_players.len() != team_size {
            self.team_tracker_mut(is_team_0).reset();
            for player in players
                .players
                .iter()
                .filter(|player| player.is_team_0 == is_team_0)
            {
                self.role_spans.close(&player.player_id);
                let state = self
                    .current_states
                    .entry(player.player_id.clone())
                    .or_default();
                state.0 = RoleState::Unknown;
            }
            return;
        }

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
            let event = FirstManChangeEvent {
                time: frame.time,
                frame: frame.frame_number,
                is_team_0,
                previous_first_man: previous,
                next_first_man: next,
            };
            self.first_man_changes.push(event);
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
            self.role_spans.record(
                frame.frame_number,
                frame.time - frame.dt,
                frame.time,
                frame.dt,
                &player.player_id,
                player.position().map(|position| position.to_array()),
                player.is_team_0,
                role_state,
            );
            self.current_states
                .insert(player.player_id.clone(), (role_state, depth_state));
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
