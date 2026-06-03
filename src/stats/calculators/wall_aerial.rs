use super::*;

const WALL_AERIAL_MIN_CONTROL_DURATION: f32 = 0.30;
const WALL_AERIAL_MAX_CONTROL_BALL_DISTANCE: f32 = 380.0;
const WALL_AERIAL_MAX_WALL_CONTACT_TO_TAKEOFF_SECONDS: f32 = 1.25;
const WALL_AERIAL_MAX_TAKEOFF_TO_TOUCH_SECONDS: f32 = 2.25;
const WALL_AERIAL_MIN_SECONDS_BETWEEN_ATTEMPTS: f32 = 3.0;
pub(crate) const WALL_AERIAL_MIN_TOUCH_PLAYER_Z: f32 = AIR_DRIBBLE_MIN_PLAYER_Z;
const WALL_AERIAL_SETUP_SIDE_WALL_START_ABS_X: f32 = 3200.0;
const WALL_AERIAL_SETUP_BACK_WALL_START_ABS_Y: f32 = 4600.0;
const WALL_AERIAL_MIN_CONTINUATION_PLAYER_Z: f32 = 300.0;
pub(crate) const WALL_AERIAL_MIN_TOUCH_BALL_Z: f32 = 400.0;
const WALL_AERIAL_REFERENCE_BALL_SPEED_CHANGE: f32 = 80.0;
pub(crate) const WALL_AERIAL_HIGH_CONFIDENCE: f32 = 0.78;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum WallAerialWall {
    Side,
    Back,
}

impl WallAerialWall {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::Side => "side",
            Self::Back => "back",
        }
    }
}

pub(crate) fn wall_aerial_wall_for_position(position: glam::Vec3) -> Option<WallAerialWall> {
    if position.z < WALL_CONTACT_MIN_PLAYER_Z {
        return None;
    }
    if position.y.abs() >= BACK_WALL_CONTACT_ABS_Y
        && position.x.abs() > BACK_WALL_GOAL_MOUTH_HALF_WIDTH_X
    {
        return Some(WallAerialWall::Back);
    }
    if position.x.abs() >= SIDE_WALL_CONTACT_ABS_X {
        return Some(WallAerialWall::Side);
    }
    None
}

fn wall_aerial_setup_wall_for_position(position: glam::Vec3) -> Option<WallAerialWall> {
    if position.z < WALL_CONTACT_MIN_PLAYER_Z {
        return None;
    }
    if position.y.abs() >= WALL_AERIAL_SETUP_BACK_WALL_START_ABS_Y
        && position.x.abs() > BACK_WALL_GOAL_MOUTH_HALF_WIDTH_X
    {
        return Some(WallAerialWall::Back);
    }
    if position.x.abs() >= WALL_AERIAL_SETUP_SIDE_WALL_START_ABS_X {
        return Some(WallAerialWall::Side);
    }
    None
}

pub(crate) fn wall_aerial_normalize_score(value: f32, min_value: f32, max_value: f32) -> f32 {
    if max_value <= min_value {
        return 0.0;
    }
    ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0)
}

pub(crate) fn wall_aerial_goal_alignment(
    is_team_0: bool,
    ball_position: glam::Vec3,
    ball_velocity: glam::Vec3,
) -> f32 {
    const GOAL_CENTER_Y: f32 = 5120.0;

    let target_y = if is_team_0 {
        GOAL_CENTER_Y
    } else {
        -GOAL_CENTER_Y
    };
    let goal_direction =
        (glam::Vec3::new(0.0, target_y, ball_position.z) - ball_position).normalize_or_zero();
    goal_direction.dot(ball_velocity.normalize_or_zero())
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct WallAerialEvent {
    pub time: f32,
    pub frame: usize,
    pub sample_time: f32,
    pub sample_frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub wall: WallAerialWall,
    pub wall_contact_time: f32,
    pub wall_contact_frame: usize,
    pub takeoff_time: f32,
    pub takeoff_frame: usize,
    pub time_since_takeoff: f32,
    pub wall_contact_position: [f32; 3],
    pub takeoff_position: [f32; 3],
    pub player_position: [f32; 3],
    pub ball_position: [f32; 3],
    pub setup_start_time: f32,
    pub setup_start_frame: usize,
    pub setup_duration: f32,
    pub ball_speed: f32,
    pub ball_speed_change: f32,
    pub goal_alignment: f32,
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct WallControl {
    player_position: glam::Vec3,
    ball_position: glam::Vec3,
    wall: WallAerialWall,
}

#[derive(Debug, Clone, PartialEq)]
struct ActiveWallControl {
    player: PlayerId,
    is_team_0: bool,
    wall: WallAerialWall,
    start_time: f32,
    start_frame: usize,
    last_time: f32,
    last_frame: usize,
    start_position: glam::Vec3,
    last_position: glam::Vec3,
    last_ball_position: glam::Vec3,
}

#[derive(Debug, Clone, PartialEq)]
struct RecentWallContact {
    player: PlayerId,
    is_team_0: bool,
    wall: WallAerialWall,
    time: f32,
    frame: usize,
    position: glam::Vec3,
    controlled_setup: Option<CompletedWallSetup>,
}

#[derive(Debug, Clone, PartialEq)]
struct CompletedWallSetup {
    start_time: f32,
    start_frame: usize,
    duration: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct ArmedWallAerial {
    player: PlayerId,
    is_team_0: bool,
    wall: WallAerialWall,
    wall_contact_time: f32,
    wall_contact_frame: usize,
    wall_contact_position: glam::Vec3,
    takeoff_time: f32,
    takeoff_frame: usize,
    takeoff_position: glam::Vec3,
    controlled_setup: CompletedWallSetup,
    recorded: bool,
}

#[derive(Debug, Clone, Default)]
pub struct WallAerialCalculator {
    stats: WallAerialStatsAccumulator,
    events: EventStream<WallAerialEvent>,
    active_wall_controls: HashMap<PlayerId, ActiveWallControl>,
    recent_wall_contacts: HashMap<PlayerId, RecentWallContact>,
    armed_aerials: HashMap<PlayerId, ArmedWallAerial>,
    recent_event_times: HashMap<PlayerId, f32>,
    previous_ball_velocity: Option<glam::Vec3>,
}

impl WallAerialCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, WallAerialStats> {
        self.stats.player_stats()
    }

    pub fn events(&self) -> &[WallAerialEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[WallAerialEvent] {
        self.events.new_events()
    }

    fn control_observation(
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_state: &TouchState,
    ) -> Option<(PlayerId, bool, WallControl)> {
        let player_id = touch_state.last_touch_player.as_ref()?;
        let ball_position = ball.position()?;
        let player = players
            .players
            .iter()
            .find(|player| &player.player_id == player_id)?;
        let player_position = player.position()?;
        let wall = wall_aerial_setup_wall_for_position(player_position)?;
        if player_position.distance(ball_position) > WALL_AERIAL_MAX_CONTROL_BALL_DISTANCE {
            return None;
        }

        Some((
            player_id.clone(),
            player.is_team_0,
            WallControl {
                player_position,
                ball_position,
                wall,
            },
        ))
    }

    fn update_active_wall_control(
        &mut self,
        frame: &FrameInfo,
        control: Option<(PlayerId, bool, WallControl)>,
    ) {
        let Some((player_id, is_team_0, control)) = control else {
            self.active_wall_controls.clear();
            return;
        };

        self.active_wall_controls
            .retain(|active_player, _| active_player == &player_id);

        let same_sequence = self
            .active_wall_controls
            .get(&player_id)
            .is_some_and(|active| active.wall == control.wall);
        if same_sequence {
            if let Some(active) = self.active_wall_controls.get_mut(&player_id) {
                active.last_time = frame.time;
                active.last_frame = frame.frame_number;
                active.last_position = control.player_position;
                active.last_ball_position = control.ball_position;
            }
        } else {
            self.active_wall_controls.insert(
                player_id.clone(),
                ActiveWallControl {
                    player: player_id,
                    is_team_0,
                    wall: control.wall,
                    start_time: frame.time,
                    start_frame: frame.frame_number,
                    last_time: frame.time,
                    last_frame: frame.frame_number,
                    start_position: control.player_position,
                    last_position: control.player_position,
                    last_ball_position: control.ball_position,
                },
            );
        }
    }

    fn completed_setup(active: &ActiveWallControl) -> Option<CompletedWallSetup> {
        let duration = active.last_time - active.start_time;
        (duration >= WALL_AERIAL_MIN_CONTROL_DURATION).then_some(CompletedWallSetup {
            start_time: active.start_time,
            start_frame: active.start_frame,
            duration,
        })
    }

    fn update_wall_contacts_and_takeoffs(&mut self, frame: &FrameInfo, players: &PlayerFrameState) {
        for player in &players.players {
            let Some(position) = player.position() else {
                continue;
            };
            let setup_wall = wall_aerial_setup_wall_for_position(position);
            if let Some(wall) = setup_wall {
                let controlled_setup = self
                    .active_wall_controls
                    .get(&player.player_id)
                    .and_then(Self::completed_setup)
                    .or_else(|| {
                        self.recent_wall_contacts
                            .get(&player.player_id)
                            .and_then(|contact| contact.controlled_setup.clone())
                    });
                self.recent_wall_contacts.insert(
                    player.player_id.clone(),
                    RecentWallContact {
                        player: player.player_id.clone(),
                        is_team_0: player.is_team_0,
                        wall,
                        time: frame.time,
                        frame: frame.frame_number,
                        position,
                        controlled_setup,
                    },
                );
                if player_is_on_wall(position) {
                    continue;
                }
            }

            if position.z < WALL_AERIAL_MIN_TOUCH_PLAYER_Z {
                self.armed_aerials.remove(&player.player_id);
                continue;
            }

            let Some(contact) = self.recent_wall_contacts.remove(&player.player_id) else {
                continue;
            };
            if frame.time - contact.time > WALL_AERIAL_MAX_WALL_CONTACT_TO_TAKEOFF_SECONDS {
                continue;
            }
            let Some(controlled_setup) = contact.controlled_setup.clone() else {
                continue;
            };
            if self.armed_aerials.contains_key(&player.player_id) {
                continue;
            }
            if self
                .recent_event_times
                .get(&player.player_id)
                .is_some_and(|time| frame.time - time < WALL_AERIAL_MIN_SECONDS_BETWEEN_ATTEMPTS)
            {
                continue;
            }
            self.armed_aerials.insert(
                player.player_id.clone(),
                ArmedWallAerial {
                    player: contact.player,
                    is_team_0: contact.is_team_0,
                    wall: contact.wall,
                    wall_contact_time: contact.time,
                    wall_contact_frame: contact.frame,
                    wall_contact_position: contact.position,
                    takeoff_time: frame.time,
                    takeoff_frame: frame.frame_number,
                    takeoff_position: position,
                    controlled_setup,
                    recorded: false,
                },
            );
        }
    }

    fn prune_armed_aerials(&mut self, current_time: f32) {
        self.armed_aerials.retain(|_, armed| {
            current_time - armed.takeoff_time <= WALL_AERIAL_MAX_TAKEOFF_TO_TOUCH_SECONDS
        });
    }

    fn ball_speed_change(
        frame: &FrameInfo,
        ball: &BallFrameState,
        previous_ball_velocity: Option<glam::Vec3>,
    ) -> f32 {
        const BALL_GRAVITY_Z: f32 = -650.0;

        let Some(ball) = ball.sample() else {
            return 0.0;
        };
        let Some(previous_ball_velocity) = previous_ball_velocity else {
            return 0.0;
        };

        let expected_linear_delta = glam::Vec3::new(0.0, 0.0, BALL_GRAVITY_Z * frame.dt.max(0.0));
        let residual_linear_impulse =
            ball.velocity() - previous_ball_velocity - expected_linear_delta;
        residual_linear_impulse.length()
    }

    fn player_position(players: &PlayerFrameState, player_id: &PlayerId) -> Option<glam::Vec3> {
        players
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
            .and_then(PlayerSample::position)
    }

    fn controlled_play_event(
        &self,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch: &TouchEvent,
        ball_speed_change: f32,
    ) -> Option<WallAerialEvent> {
        let player_id = touch.player.as_ref()?;
        let armed = self.armed_aerials.get(player_id)?;
        if armed.recorded {
            return None;
        }
        let player_position = Self::player_position(players, player_id)?;
        if player_is_on_wall(player_position) || player_position.z < WALL_AERIAL_MIN_TOUCH_PLAYER_Z
        {
            return None;
        }
        let ball = ball.sample()?;
        let ball_position = ball.position();
        if ball_position.z < WALL_AERIAL_MIN_TOUCH_BALL_Z {
            return None;
        }
        if player_position.z < WALL_AERIAL_MIN_CONTINUATION_PLAYER_Z {
            return None;
        }
        let time_since_takeoff = touch.time - armed.takeoff_time;
        if !(0.0..=WALL_AERIAL_MAX_TAKEOFF_TO_TOUCH_SECONDS).contains(&time_since_takeoff) {
            return None;
        }
        let setup = &armed.controlled_setup;
        let confidence = 0.30
            + 0.20
                * wall_aerial_normalize_score(
                    setup.duration,
                    WALL_AERIAL_MIN_CONTROL_DURATION,
                    1.2,
                )
            + 0.18
                * (1.0
                    - wall_aerial_normalize_score(
                        time_since_takeoff,
                        0.15,
                        WALL_AERIAL_MAX_TAKEOFF_TO_TOUCH_SECONDS,
                    ))
            + 0.16
                * wall_aerial_normalize_score(
                    player_position.z,
                    WALL_AERIAL_MIN_TOUCH_PLAYER_Z,
                    850.0,
                )
            + 0.16
                * wall_aerial_normalize_score(
                    ball_speed_change,
                    WALL_AERIAL_REFERENCE_BALL_SPEED_CHANGE,
                    900.0,
                );

        Some(WallAerialEvent {
            time: touch.time,
            frame: touch.frame,
            sample_time: touch.time,
            sample_frame: touch.frame,
            player: player_id.clone(),
            is_team_0: touch.team_is_team_0,
            wall: armed.wall,
            wall_contact_time: armed.wall_contact_time,
            wall_contact_frame: armed.wall_contact_frame,
            takeoff_time: armed.takeoff_time,
            takeoff_frame: armed.takeoff_frame,
            time_since_takeoff,
            wall_contact_position: armed.wall_contact_position.to_array(),
            takeoff_position: armed.takeoff_position.to_array(),
            player_position: player_position.to_array(),
            ball_position: ball_position.to_array(),
            setup_start_time: setup.start_time,
            setup_start_frame: setup.start_frame,
            setup_duration: setup.duration,
            ball_speed: ball.velocity().length(),
            ball_speed_change,
            goal_alignment: wall_aerial_goal_alignment(
                touch.team_is_team_0,
                ball_position,
                ball.velocity(),
            ),
            confidence: confidence.clamp(0.0, 1.0),
        })
    }

    fn record_event(&mut self, frame: &FrameInfo, mut event: WallAerialEvent) {
        event.sample_time = frame.time;
        event.sample_frame = frame.frame_number;
        self.stats.apply_event(&event, frame);
        self.recent_event_times
            .insert(event.player.clone(), event.time);
        self.events.push(event);
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_state: &TouchState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        self.stats.begin_sample(frame);
        if !live_play {
            self.active_wall_controls.clear();
            self.recent_wall_contacts.clear();
            self.armed_aerials.clear();
            self.recent_event_times.clear();
            self.previous_ball_velocity = ball.velocity();
            self.stats.reset_current_last_event_marker();
            return Ok(());
        }

        self.update_active_wall_control(
            frame,
            Self::control_observation(ball, players, touch_state),
        );
        self.update_wall_contacts_and_takeoffs(frame, players);
        self.prune_armed_aerials(frame.time);

        let ball_speed_change = Self::ball_speed_change(frame, ball, self.previous_ball_velocity);
        for touch in &touch_state.touch_events {
            if let Some(event) = self.controlled_play_event(ball, players, touch, ball_speed_change)
            {
                if let Some(armed) = self.armed_aerials.get_mut(&event.player) {
                    armed.recorded = true;
                }
                self.record_event(frame, event);
            }
        }

        self.previous_ball_velocity = ball.velocity();
        self.stats.restore_current_last_event_marker();

        Ok(())
    }
}

#[cfg(test)]
#[path = "wall_aerial_tests.rs"]
mod tests;
