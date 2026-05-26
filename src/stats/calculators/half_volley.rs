use super::*;

const DEFAULT_HALF_VOLLEY_MAX_BOUNCE_TO_TOUCH_SECONDS: f32 = 0.45;
const DEFAULT_HALF_VOLLEY_MIN_BALL_SPEED: f32 = 1000.0;
const HALF_VOLLEY_FLOOR_BOUNCE_MAX_BALL_Z: f32 = BALL_RADIUS_Z + 45.0;
const HALF_VOLLEY_FLOOR_BOUNCE_MIN_APPROACH_SPEED_Z: f32 = 250.0;
const HALF_VOLLEY_FLOOR_BOUNCE_MIN_REBOUND_SPEED_Z: f32 = 150.0;
const HALF_VOLLEY_MAX_DODGE_TO_TOUCH_SECONDS: f32 = 0.35;
const HALF_VOLLEY_MAX_GROUND_TO_DODGE_SECONDS: f32 = 0.45;
const HALF_VOLLEY_GOAL_CENTER_Y: f32 = 5120.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct HalfVolleyCalculatorConfig {
    pub max_bounce_to_touch_seconds: f32,
    pub min_ball_speed: f32,
}

impl Default for HalfVolleyCalculatorConfig {
    fn default() -> Self {
        Self {
            max_bounce_to_touch_seconds: DEFAULT_HALF_VOLLEY_MAX_BOUNCE_TO_TOUCH_SECONDS,
            min_ball_speed: DEFAULT_HALF_VOLLEY_MIN_BALL_SPEED,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct HalfVolleyEvent {
    pub time: f32,
    pub frame: usize,
    pub sample_time: f32,
    pub sample_frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub bounce_time: f32,
    pub bounce_frame: usize,
    pub bounce_to_touch_seconds: f32,
    pub ball_speed: f32,
    pub goal_alignment: f32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct HalfVolleyPlayerStats {
    pub count: u32,
    pub total_ball_speed: f32,
    pub fastest_ball_speed: f32,
    pub is_last_half_volley: bool,
    pub last_half_volley_time: Option<f32>,
    pub last_half_volley_frame: Option<usize>,
    pub time_since_last_half_volley: Option<f32>,
    pub frames_since_last_half_volley: Option<usize>,
}

impl HalfVolleyPlayerStats {
    pub fn average_ball_speed(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_ball_speed / self.count as f32
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct HalfVolleyTeamStats {
    pub count: u32,
    pub total_ball_speed: f32,
    pub fastest_ball_speed: f32,
}

impl HalfVolleyTeamStats {
    pub fn average_ball_speed(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_ball_speed / self.count as f32
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct FloorBounce {
    time: f32,
    frame: usize,
}

#[derive(Debug, Clone, PartialEq)]
struct GroundContact {
    time: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct DodgeStart {
    time: f32,
    ground_contact: GroundContact,
}

#[derive(Debug, Clone, Default)]
pub struct HalfVolleyCalculator {
    config: HalfVolleyCalculatorConfig,
    player_stats: HashMap<PlayerId, HalfVolleyPlayerStats>,
    team_zero_stats: HalfVolleyTeamStats,
    team_one_stats: HalfVolleyTeamStats,
    events: Vec<HalfVolleyEvent>,
    last_floor_bounce: Option<FloorBounce>,
    last_ground_contacts: HashMap<PlayerId, GroundContact>,
    recent_dodge_starts: HashMap<PlayerId, DodgeStart>,
    previous_dodge_active: HashMap<PlayerId, bool>,
    previous_ball_velocity: Option<glam::Vec3>,
    current_last_half_volley_player: Option<PlayerId>,
}

impl HalfVolleyCalculator {
    pub fn new() -> Self {
        Self::with_config(HalfVolleyCalculatorConfig::default())
    }

    pub fn with_config(config: HalfVolleyCalculatorConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn config(&self) -> &HalfVolleyCalculatorConfig {
        &self.config
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, HalfVolleyPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &HalfVolleyTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &HalfVolleyTeamStats {
        &self.team_one_stats
    }

    pub fn events(&self) -> &[HalfVolleyEvent] {
        &self.events
    }

    fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_half_volley = false;
            stats.time_since_last_half_volley = stats
                .last_half_volley_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_half_volley = stats
                .last_half_volley_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }
    }

    fn detect_floor_bounce(
        frame: &FrameInfo,
        ball: Option<&BallSample>,
        previous_ball_velocity: Option<glam::Vec3>,
        touch_events: &[TouchEvent],
    ) -> Option<FloorBounce> {
        if !touch_events.is_empty() {
            return None;
        }
        let ball = ball?;
        let previous_ball_velocity = previous_ball_velocity?;
        let ball_position = ball.position();
        let ball_velocity = ball.velocity();
        if ball_position.z > HALF_VOLLEY_FLOOR_BOUNCE_MAX_BALL_Z {
            return None;
        }
        if previous_ball_velocity.z > -HALF_VOLLEY_FLOOR_BOUNCE_MIN_APPROACH_SPEED_Z {
            return None;
        }
        if ball_velocity.z < HALF_VOLLEY_FLOOR_BOUNCE_MIN_REBOUND_SPEED_Z {
            return None;
        }

        Some(FloorBounce {
            time: frame.time,
            frame: frame.frame_number,
        })
    }

    fn event_for_touch(
        &self,
        ball: &BallFrameState,
        touch: &TouchEvent,
    ) -> Option<HalfVolleyEvent> {
        let player = touch.player.clone()?;
        let bounce = self.last_floor_bounce.as_ref()?;
        let bounce_to_touch_seconds = touch.time - bounce.time;
        if !(0.0..=self.config.max_bounce_to_touch_seconds).contains(&bounce_to_touch_seconds) {
            return None;
        }
        let dodge_start = self.recent_dodge_starts.get(&player)?;
        let dodge_to_touch_seconds = touch.time - dodge_start.time;
        if !(0.0..=HALF_VOLLEY_MAX_DODGE_TO_TOUCH_SECONDS).contains(&dodge_to_touch_seconds) {
            return None;
        }
        let ground_to_dodge_seconds = dodge_start.time - dodge_start.ground_contact.time;
        if !(0.0..=HALF_VOLLEY_MAX_GROUND_TO_DODGE_SECONDS).contains(&ground_to_dodge_seconds) {
            return None;
        }

        let ball = ball.sample()?;
        let ball_position = ball.position();
        let ball_velocity = ball.velocity();
        let ball_speed = ball_velocity.length();
        if ball_speed < self.config.min_ball_speed {
            return None;
        }

        let target_y = if touch.team_is_team_0 {
            HALF_VOLLEY_GOAL_CENTER_Y
        } else {
            -HALF_VOLLEY_GOAL_CENTER_Y
        };
        let goal_direction = glam::Vec3::new(0.0, target_y, ball_position.z) - ball_position;
        let goal_alignment = goal_direction
            .normalize_or_zero()
            .dot(ball_velocity.normalize_or_zero());

        Some(HalfVolleyEvent {
            time: touch.time,
            frame: touch.frame,
            sample_time: touch.time,
            sample_frame: touch.frame,
            player,
            is_team_0: touch.team_is_team_0,
            bounce_time: bounce.time,
            bounce_frame: bounce.frame,
            bounce_to_touch_seconds,
            ball_speed,
            goal_alignment,
        })
    }

    fn record_half_volley(&mut self, frame: &FrameInfo, mut event: HalfVolleyEvent) {
        event.sample_time = frame.time;
        event.sample_frame = frame.frame_number;
        let player_stats = self.player_stats.entry(event.player.clone()).or_default();
        player_stats.count += 1;
        player_stats.total_ball_speed += event.ball_speed;
        player_stats.fastest_ball_speed = player_stats.fastest_ball_speed.max(event.ball_speed);
        player_stats.last_half_volley_time = Some(event.time);
        player_stats.last_half_volley_frame = Some(event.frame);
        player_stats.time_since_last_half_volley = Some((frame.time - event.time).max(0.0));
        player_stats.frames_since_last_half_volley =
            Some(frame.frame_number.saturating_sub(event.frame));

        let team_stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        team_stats.count += 1;
        team_stats.total_ball_speed += event.ball_speed;
        team_stats.fastest_ball_speed = team_stats.fastest_ball_speed.max(event.ball_speed);

        self.current_last_half_volley_player = Some(event.player.clone());
        self.events.push(event);
    }

    fn update_player_movement_state(&mut self, frame: &FrameInfo, players: &PlayerFrameState) {
        for player in &players.players {
            if player
                .position()
                .is_some_and(|position| position.z <= PLAYER_GROUND_Z_THRESHOLD)
            {
                self.last_ground_contacts
                    .insert(player.player_id.clone(), GroundContact { time: frame.time });
            }

            let was_dodge_active = self
                .previous_dodge_active
                .insert(player.player_id.clone(), player.dodge_active)
                .unwrap_or(false);
            if !player.dodge_active || was_dodge_active {
                continue;
            }

            if let Some(ground_contact) = self.last_ground_contacts.get(&player.player_id) {
                self.recent_dodge_starts.insert(
                    player.player_id.clone(),
                    DodgeStart {
                        time: frame.time,
                        ground_contact: ground_contact.clone(),
                    },
                );
            }
        }

        self.recent_dodge_starts.retain(|_, dodge_start| {
            frame.time - dodge_start.time <= HALF_VOLLEY_MAX_DODGE_TO_TOUCH_SECONDS
        });
        self.last_ground_contacts.retain(|_, ground_contact| {
            frame.time - ground_contact.time
                <= HALF_VOLLEY_MAX_GROUND_TO_DODGE_SECONDS + HALF_VOLLEY_MAX_DODGE_TO_TOUCH_SECONDS
        });
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_state: &TouchState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        self.begin_sample(frame);
        if !live_play {
            self.last_floor_bounce = None;
            self.last_ground_contacts.clear();
            self.recent_dodge_starts.clear();
            self.previous_dodge_active.clear();
            self.previous_ball_velocity = ball.velocity();
            self.current_last_half_volley_player = None;
            return Ok(());
        }

        self.update_player_movement_state(frame, players);

        if let Some(bounce) = Self::detect_floor_bounce(
            frame,
            ball.sample(),
            self.previous_ball_velocity,
            &touch_state.touch_events,
        ) {
            self.last_floor_bounce = Some(bounce);
        }

        for touch in &touch_state.touch_events {
            if let Some(event) = self.event_for_touch(ball, touch) {
                self.record_half_volley(frame, event);
            }
        }

        self.previous_ball_velocity = ball.velocity();
        if let Some(player_id) = self.current_last_half_volley_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_half_volley = true;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "half_volley_tests.rs"]
mod tests;
