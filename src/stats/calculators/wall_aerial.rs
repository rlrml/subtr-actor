use super::*;

/// Minimum time a player must ride the wall before leaving it for the takeoff to
/// count as a wall aerial. A wall aerial is launched *off the wall*, so the
/// player must actually be on the wall surface (`wall_aerial_wall_for_position`)
/// for at least this long. Carrying the ball is *not* required — driving up the
/// wall and then hitting the ball in the air is still a wall aerial.
const WALL_AERIAL_MIN_WALL_CONTACT_DURATION: f32 = 0.30;
const WALL_AERIAL_MAX_WALL_CONTACT_TO_TAKEOFF_SECONDS: f32 = 1.25;
const WALL_AERIAL_MAX_TAKEOFF_TO_TOUCH_SECONDS: f32 = 2.25;
const WALL_AERIAL_MIN_SECONDS_BETWEEN_ATTEMPTS: f32 = 3.0;
pub(crate) const WALL_AERIAL_MIN_TOUCH_PLAYER_Z: f32 = AIR_DRIBBLE_MIN_PLAYER_Z;
const WALL_AERIAL_MIN_CONTINUATION_PLAYER_Z: f32 = 300.0;
pub(crate) const WALL_AERIAL_MIN_TOUCH_BALL_Z: f32 = 400.0;
const WALL_AERIAL_REFERENCE_BALL_SPEED_CHANGE: f32 = 80.0;
pub(crate) const WALL_AERIAL_HIGH_CONFIDENCE: f32 = 0.78;

/// Field coordinates where the rounded corner arcs begin: the flat walls
/// (side `|x| = 4096`, end `|y| = 5120`) curve into a quarter circle of radius
/// 1152 beyond these. Subtracting them from an on-wall position (clamping the
/// residual at zero) leaves the outward wall normal: axis-aligned on a flat
/// wall, radial on a corner arc.
const WALL_CORNER_ARC_START_ABS_X: f32 = 4096.0 - 1152.0;
const WALL_CORNER_ARC_START_ABS_Y: f32 = 5120.0 - 1152.0;
/// Ratio of the smaller to the larger attack-relative axis above which a wall
/// takeoff is treated as a (diagonal) corner. `tan(22.5°)` splits each quadrant
/// into three equal 45° sectors across the eight [`WallAerialWall`] directions.
const WALL_AERIAL_CORNER_AXIS_RATIO: f32 = 0.4142136;

/// Which wall a player took off from, relative to their attack direction.
///
/// `Front`/`Back` are the end walls (the opponent's net side vs. the player's
/// own net side); `Left`/`Right` are the side walls; the `*Left`/`*Right`
/// variants are the rounded corners where a side wall meets an end wall.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum WallAerialWall {
    Left,
    Right,
    Front,
    Back,
    FrontLeft,
    FrontRight,
    BackLeft,
    BackRight,
}

impl WallAerialWall {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Front => "front",
            Self::Back => "back",
            Self::FrontLeft => "front_left",
            Self::FrontRight => "front_right",
            Self::BackLeft => "back_left",
            Self::BackRight => "back_right",
        }
    }
}

/// Coarse wall surface used internally to keep setup/continuity tracking stable
/// while the player slides along a wall. The attack-relative [`WallAerialWall`]
/// is computed separately at the moment of the recorded takeoff.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WallSurface {
    Side,
    Back,
}

pub(crate) fn wall_aerial_wall_for_position(position: glam::Vec3) -> Option<WallSurface> {
    if position.z < WALL_CONTACT_MIN_PLAYER_Z {
        return None;
    }
    if position.y.abs() >= BACK_WALL_CONTACT_ABS_Y
        && position.x.abs() > BACK_WALL_GOAL_MOUTH_HALF_WIDTH_X
    {
        return Some(WallSurface::Back);
    }
    if position.x.abs() >= SIDE_WALL_CONTACT_ABS_X {
        return Some(WallSurface::Side);
    }
    None
}

/// Classify which wall (relative to the player's attack direction) an on-wall
/// position is on. The residual of each axis beyond the corner-arc start is the
/// outward wall normal — zero along an axis whose flat wall is out of reach, and
/// the radial direction of the arc in a corner — so field position alone
/// determines the wall, independent of car orientation.
pub(crate) fn wall_aerial_wall_classification(
    is_team_0: bool,
    position: glam::Vec3,
) -> WallAerialWall {
    let x = position.x.signum() * (position.x.abs() - WALL_CORNER_ARC_START_ABS_X).max(0.0);
    let y = position.y.signum() * (position.y.abs() - WALL_CORNER_ARC_START_ABS_Y).max(0.0);
    wall_aerial_wall_from_axes(is_team_0, x, y)
}

/// Map a horizontal direction toward the wall (any positive scale) into an
/// attack-relative [`WallAerialWall`]. `x`/`y` are field-axis components; they
/// are normalized for team so `+x` is the player's right and `+y` points at the
/// opponent's (front) end wall.
fn wall_aerial_wall_from_axes(is_team_0: bool, x: f32, y: f32) -> WallAerialWall {
    let right = if is_team_0 { x } else { -x };
    let front = if is_team_0 { y } else { -y };
    let abs_right = right.abs();
    let abs_front = front.abs();
    let dominant = abs_right.max(abs_front);
    if dominant <= f32::EPSILON {
        return WallAerialWall::Back;
    }
    let toward_right = right >= 0.0;
    let toward_front = front >= 0.0;
    if abs_right.min(abs_front) / dominant >= WALL_AERIAL_CORNER_AXIS_RATIO {
        match (toward_front, toward_right) {
            (true, true) => WallAerialWall::FrontRight,
            (true, false) => WallAerialWall::FrontLeft,
            (false, true) => WallAerialWall::BackRight,
            (false, false) => WallAerialWall::BackLeft,
        }
    } else if abs_right >= abs_front {
        if toward_right {
            WallAerialWall::Right
        } else {
            WallAerialWall::Left
        }
    } else if toward_front {
        WallAerialWall::Front
    } else {
        WallAerialWall::Back
    }
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

/// An aerial launched off a side or back wall: the player rides the wall, leaves
/// it while airborne, and hits the ball in the air. Carrying the ball up the wall
/// is not required.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct WallAerialEvent {
    pub time: f32,
    pub frame: usize,
    pub sample_time: f32,
    pub sample_frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
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

/// A continuous span of frames during which a player is on the wall surface
/// (`wall_aerial_wall_for_position`). This is the wall-aerial "setup": the player
/// riding the wall before launching off it. Ball control is intentionally not
/// tracked here — a wall aerial does not require carrying the ball.
#[derive(Debug, Clone, PartialEq)]
struct WallContactSpan {
    /// Coarse surface used to detect that the player stayed on the *same* wall.
    surface: WallSurface,
    /// Attack-relative wall classification at the most recent on-wall frame; this
    /// is the label the recorded takeoff carries.
    wall_direction: WallAerialWall,
    start_time: f32,
    start_frame: usize,
    last_time: f32,
    last_frame: usize,
    last_position: glam::Vec3,
}

#[derive(Debug, Clone, PartialEq)]
struct ArmedWallAerial {
    player: PlayerId,
    wall_direction: WallAerialWall,
    wall_contact_time: f32,
    wall_contact_frame: usize,
    wall_contact_position: glam::Vec3,
    takeoff_time: f32,
    takeoff_frame: usize,
    takeoff_position: glam::Vec3,
    setup_start_time: f32,
    setup_start_frame: usize,
    setup_duration: f32,
    recorded: bool,
}

/// Detects wall aerials during live play.
#[derive(Debug, Clone, Default)]
pub struct WallAerialCalculator {
    events: EventStream<WallAerialEvent>,
    wall_contacts: HashMap<PlayerId, WallContactSpan>,
    armed_aerials: HashMap<PlayerId, ArmedWallAerial>,
    recent_event_times: HashMap<PlayerId, f32>,
    previous_ball_velocity: Option<glam::Vec3>,
}

impl WallAerialCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[WallAerialEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[WallAerialEvent] {
        self.events.new_events()
    }

    /// Tracks each player's on-wall presence and arms a takeoff when a player
    /// who rode the wall long enough leaves it while airborne.
    ///
    /// A wall aerial is "ride the wall, leave it, hit the ball in the air." We
    /// detect the first two phases here, keyed purely on the player being on the
    /// wall surface (`wall_aerial_wall_for_position`) — no ball control required.
    /// The attack-relative wall label is read from the field position at the
    /// last on-wall frame (`wall_aerial_wall_classification`).
    fn update_wall_contacts_and_takeoffs(&mut self, frame: &FrameInfo, players: &PlayerFrameState) {
        for player in &players.players {
            let Some(position) = player.position() else {
                continue;
            };

            // On the wall: start or extend the contact span, and defer takeoff.
            if let Some(surface) = wall_aerial_wall_for_position(position) {
                let wall_direction = wall_aerial_wall_classification(player.is_team_0, position);
                match self.wall_contacts.get_mut(&player.player_id) {
                    Some(span) if span.surface == surface => {
                        span.last_time = frame.time;
                        span.last_frame = frame.frame_number;
                        span.last_position = position;
                        span.wall_direction = wall_direction;
                    }
                    _ => {
                        self.wall_contacts.insert(
                            player.player_id.clone(),
                            WallContactSpan {
                                surface,
                                wall_direction,
                                start_time: frame.time,
                                start_frame: frame.frame_number,
                                last_time: frame.time,
                                last_frame: frame.frame_number,
                                last_position: position,
                            },
                        );
                    }
                }
                continue;
            }

            // Off the wall and back near the ground: the player landed rather than
            // launched, so drop any pending contact/takeoff.
            if position.z < WALL_AERIAL_MIN_TOUCH_PLAYER_Z {
                self.wall_contacts.remove(&player.player_id);
                self.armed_aerials.remove(&player.player_id);
                continue;
            }

            if self.armed_aerials.contains_key(&player.player_id) {
                continue;
            }

            // Off the wall and airborne: a takeoff. Arm it if the player rode the
            // wall long enough and left it recently.
            let Some(span) = self.wall_contacts.remove(&player.player_id) else {
                continue;
            };
            if frame.time - span.last_time > WALL_AERIAL_MAX_WALL_CONTACT_TO_TAKEOFF_SECONDS {
                continue;
            }
            let setup_duration = span.last_time - span.start_time;
            if setup_duration < WALL_AERIAL_MIN_WALL_CONTACT_DURATION {
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
                    player: player.player_id.clone(),
                    wall_direction: span.wall_direction,
                    wall_contact_time: span.last_time,
                    wall_contact_frame: span.last_frame,
                    wall_contact_position: span.last_position,
                    takeoff_time: frame.time,
                    takeoff_frame: frame.frame_number,
                    takeoff_position: position,
                    setup_start_time: span.start_time,
                    setup_start_frame: span.start_frame,
                    setup_duration,
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
        let confidence = 0.30
            + 0.20
                * wall_aerial_normalize_score(
                    armed.setup_duration,
                    WALL_AERIAL_MIN_WALL_CONTACT_DURATION,
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
            wall: armed.wall_direction,
            wall_contact_time: armed.wall_contact_time,
            wall_contact_frame: armed.wall_contact_frame,
            takeoff_time: armed.takeoff_time,
            takeoff_frame: armed.takeoff_frame,
            time_since_takeoff,
            wall_contact_position: armed.wall_contact_position.to_array(),
            takeoff_position: armed.takeoff_position.to_array(),
            player_position: player_position.to_array(),
            ball_position: ball_position.to_array(),
            setup_start_time: armed.setup_start_time,
            setup_start_frame: armed.setup_start_frame,
            setup_duration: armed.setup_duration,
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
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        if !live_play_state.is_live_play {
            self.wall_contacts.clear();
            self.armed_aerials.clear();
            self.recent_event_times.clear();
            self.previous_ball_velocity = ball.velocity();
            return Ok(());
        }

        self.update_wall_contacts_and_takeoffs(frame, players);
        self.prune_armed_aerials(frame.time);

        let ball_speed_change = Self::ball_speed_change(frame, ball, self.previous_ball_velocity);
        for touch in chronological_touch_events(&touch_state.touch_events) {
            if let Some(event) = self.controlled_play_event(ball, players, touch, ball_speed_change)
            {
                if let Some(armed) = self.armed_aerials.get_mut(&event.player) {
                    armed.recorded = true;
                }
                self.record_event(frame, event);
            }
        }

        self.previous_ball_velocity = ball.velocity();

        Ok(())
    }
}

#[cfg(test)]
#[path = "wall_aerial_tests.rs"]
mod tests;
