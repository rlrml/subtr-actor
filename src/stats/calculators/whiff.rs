use super::*;

const WHIFF_ENTER_DISTANCE: f32 = 185.0;
const WHIFF_EXIT_DISTANCE: f32 = 340.0;
const WHIFF_MAX_CANDIDATE_SECONDS: f32 = 0.65;
const WHIFF_MIN_APPROACH_SPEED: f32 = 300.0;
const WHIFF_MIN_FORWARD_ALIGNMENT: f32 = 0.15;
const WHIFF_MIN_DODGE_APPROACH_SPEED: f32 = 100.0;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct WhiffEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub closest_approach_distance: f32,
    pub forward_alignment: f32,
    pub approach_speed: f32,
    pub dodge_active: bool,
    pub aerial: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct WhiffStats {
    pub whiff_count: u32,
    pub grounded_whiff_count: u32,
    pub aerial_whiff_count: u32,
    pub dodge_whiff_count: u32,
    pub is_last_whiff: bool,
    pub last_whiff_time: Option<f32>,
    pub last_whiff_frame: Option<usize>,
    pub time_since_last_whiff: Option<f32>,
    pub frames_since_last_whiff: Option<usize>,
    pub last_closest_approach_distance: Option<f32>,
    pub best_closest_approach_distance: Option<f32>,
    pub cumulative_closest_approach_distance: f32,
}

impl WhiffStats {
    pub fn average_closest_approach_distance(&self) -> f32 {
        if self.whiff_count == 0 {
            0.0
        } else {
            self.cumulative_closest_approach_distance / self.whiff_count as f32
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ActiveWhiffCandidate {
    player: PlayerId,
    is_team_0: bool,
    start_time: f32,
    closest_time: f32,
    closest_frame: usize,
    closest_approach_distance: f32,
    forward_alignment: f32,
    approach_speed: f32,
    dodge_active: bool,
    aerial: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WhiffCalculator {
    player_stats: HashMap<PlayerId, WhiffStats>,
    active_candidates: HashMap<PlayerId, ActiveWhiffCandidate>,
    events: Vec<WhiffEvent>,
    current_last_whiff_player: Option<PlayerId>,
}

impl WhiffCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, WhiffStats> {
        &self.player_stats
    }

    pub fn events(&self) -> &[WhiffEvent] {
        &self.events
    }

    fn hitbox_distance(ball_position: glam::Vec3, player: &PlayerSample) -> Option<f32> {
        const OCTANE_HITBOX_LENGTH: f32 = 118.01;
        const OCTANE_HITBOX_WIDTH: f32 = 84.2;
        const OCTANE_HITBOX_HEIGHT: f32 = 36.16;
        const OCTANE_HITBOX_OFFSET: f32 = 13.88;
        const OCTANE_HITBOX_ELEVATION: f32 = 17.05;

        let rigid_body = player.rigid_body.as_ref()?;
        let player_position = player.position()?;
        let local_ball_position =
            quat_to_glam(&rigid_body.rotation).inverse() * (ball_position - player_position);

        let x_min = -OCTANE_HITBOX_LENGTH / 2.0 + OCTANE_HITBOX_OFFSET;
        let x_max = OCTANE_HITBOX_LENGTH / 2.0 + OCTANE_HITBOX_OFFSET;
        let y_min = -OCTANE_HITBOX_WIDTH / 2.0;
        let y_max = OCTANE_HITBOX_WIDTH / 2.0;
        let z_min = -OCTANE_HITBOX_HEIGHT / 2.0 + OCTANE_HITBOX_ELEVATION;
        let z_max = OCTANE_HITBOX_HEIGHT / 2.0 + OCTANE_HITBOX_ELEVATION;

        let x_distance = if local_ball_position.x < x_min {
            x_min - local_ball_position.x
        } else if local_ball_position.x > x_max {
            local_ball_position.x - x_max
        } else {
            0.0
        };
        let y_distance = if local_ball_position.y < y_min {
            y_min - local_ball_position.y
        } else if local_ball_position.y > y_max {
            local_ball_position.y - y_max
        } else {
            0.0
        };
        let z_distance = if local_ball_position.z < z_min {
            z_min - local_ball_position.z
        } else if local_ball_position.z > z_max {
            local_ball_position.z - z_max
        } else {
            0.0
        };

        Some(glam::Vec3::new(x_distance, y_distance, z_distance).length())
    }

    fn whiff_candidate(
        frame: &FrameInfo,
        ball_position: glam::Vec3,
        player: &PlayerSample,
    ) -> Option<ActiveWhiffCandidate> {
        let distance = Self::hitbox_distance(ball_position, player)?;
        if distance > WHIFF_ENTER_DISTANCE {
            return None;
        }

        let rigid_body = player.rigid_body.as_ref()?;
        let player_position = player.position()?;
        let to_ball = (ball_position - player_position).normalize_or_zero();
        if to_ball.length_squared() <= f32::EPSILON {
            return None;
        }

        let rotation = quat_to_glam(&rigid_body.rotation);
        let forward_alignment = (rotation * glam::Vec3::X).dot(to_ball);
        let approach_speed = player.velocity().unwrap_or(glam::Vec3::ZERO).dot(to_ball);
        let committed_approach = approach_speed >= WHIFF_MIN_APPROACH_SPEED
            && forward_alignment >= WHIFF_MIN_FORWARD_ALIGNMENT;
        let committed_dodge =
            player.dodge_active && approach_speed >= WHIFF_MIN_DODGE_APPROACH_SPEED;
        if !committed_approach && !committed_dodge {
            return None;
        }

        Some(ActiveWhiffCandidate {
            player: player.player_id.clone(),
            is_team_0: player.is_team_0,
            start_time: frame.time,
            closest_time: frame.time,
            closest_frame: frame.frame_number,
            closest_approach_distance: distance,
            forward_alignment,
            approach_speed,
            dodge_active: player.dodge_active,
            aerial: player_position.z > POWERSLIDE_MAX_Z_THRESHOLD,
        })
    }

    fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_whiff = false;
            stats.time_since_last_whiff = stats
                .last_whiff_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_whiff = stats
                .last_whiff_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }
    }

    fn cancel_touched_candidates(&mut self, touch_state: &TouchState) {
        for touch in &touch_state.touch_events {
            if let Some(player_id) = touch.player.as_ref() {
                self.active_candidates.remove(player_id);
            }
        }
    }

    fn emit_whiff(&mut self, candidate: ActiveWhiffCandidate, frame: &FrameInfo) {
        let event = WhiffEvent {
            time: candidate.closest_time,
            frame: candidate.closest_frame,
            player: candidate.player.clone(),
            is_team_0: candidate.is_team_0,
            closest_approach_distance: candidate.closest_approach_distance,
            forward_alignment: candidate.forward_alignment,
            approach_speed: candidate.approach_speed,
            dodge_active: candidate.dodge_active,
            aerial: candidate.aerial,
        };

        let stats = self
            .player_stats
            .entry(candidate.player.clone())
            .or_default();
        stats.whiff_count += 1;
        if event.aerial {
            stats.aerial_whiff_count += 1;
        } else {
            stats.grounded_whiff_count += 1;
        }
        if event.dodge_active {
            stats.dodge_whiff_count += 1;
        }
        stats.is_last_whiff = true;
        stats.last_whiff_time = Some(event.time);
        stats.last_whiff_frame = Some(event.frame);
        stats.time_since_last_whiff = Some((frame.time - event.time).max(0.0));
        stats.frames_since_last_whiff = Some(frame.frame_number.saturating_sub(event.frame));
        stats.last_closest_approach_distance = Some(event.closest_approach_distance);
        stats.best_closest_approach_distance = Some(
            stats
                .best_closest_approach_distance
                .map(|distance| distance.min(event.closest_approach_distance))
                .unwrap_or(event.closest_approach_distance),
        );
        stats.cumulative_closest_approach_distance += event.closest_approach_distance;

        self.current_last_whiff_player = Some(candidate.player);
        self.events.push(event);
    }

    fn update_active_candidates(
        &mut self,
        frame: &FrameInfo,
        ball_position: glam::Vec3,
        players: &PlayerFrameState,
    ) {
        let mut visible_players = HashSet::new();

        for player in &players.players {
            let player_id = player.player_id.clone();
            visible_players.insert(player_id.clone());
            let distance = Self::hitbox_distance(ball_position, player);

            if let (Some(candidate), Some(distance)) =
                (self.active_candidates.get_mut(&player_id), distance)
            {
                if distance < candidate.closest_approach_distance {
                    candidate.closest_approach_distance = distance;
                    candidate.closest_time = frame.time;
                    candidate.closest_frame = frame.frame_number;
                    if let Some(updated) = Self::whiff_candidate(frame, ball_position, player) {
                        candidate.forward_alignment = updated.forward_alignment;
                        candidate.approach_speed = updated.approach_speed;
                        candidate.dodge_active |= updated.dodge_active;
                        candidate.aerial |= updated.aerial;
                    }
                }

                if distance > WHIFF_EXIT_DISTANCE
                    || frame.time - candidate.start_time > WHIFF_MAX_CANDIDATE_SECONDS
                {
                    if let Some(candidate) = self.active_candidates.remove(&player_id) {
                        self.emit_whiff(candidate, frame);
                    }
                }
                continue;
            }

            if let Some(candidate) = Self::whiff_candidate(frame, ball_position, player) {
                self.active_candidates.insert(player_id, candidate);
            }
        }

        let missing_players = self
            .active_candidates
            .keys()
            .filter(|player_id| !visible_players.contains(*player_id))
            .cloned()
            .collect::<Vec<_>>();
        for player_id in missing_players {
            self.active_candidates.remove(&player_id);
        }
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_state: &TouchState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        if !live_play {
            self.active_candidates.clear();
            self.current_last_whiff_player = None;
            return Ok(());
        }

        self.begin_sample(frame);
        self.cancel_touched_candidates(touch_state);
        if let Some(ball_position) = ball.position() {
            self.update_active_candidates(frame, ball_position, players);
        }

        if let Some(player_id) = self.current_last_whiff_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_whiff = true;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rigid_body(position: glam::Vec3, velocity: glam::Vec3) -> boxcars::RigidBody {
        boxcars::RigidBody {
            sleeping: false,
            location: glam_to_vec(&position),
            rotation: boxcars::Quaternion {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
            linear_velocity: Some(glam_to_vec(&velocity)),
            angular_velocity: Some(glam_to_vec(&glam::Vec3::ZERO)),
        }
    }

    fn player(id: u64, x: f32, dodge_active: bool) -> PlayerSample {
        PlayerSample {
            player_id: boxcars::RemoteId::Steam(id),
            is_team_0: true,
            rigid_body: Some(rigid_body(
                glam::Vec3::new(x, 0.0, 17.0),
                glam::Vec3::new(900.0, 0.0, 0.0),
            )),
            boost_amount: None,
            last_boost_amount: None,
            boost_active: false,
            dodge_active,
            powerslide_active: false,
            match_goals: None,
            match_assists: None,
            match_saves: None,
            match_shots: None,
            match_score: None,
        }
    }

    fn frame(frame_number: usize, time: f32) -> FrameInfo {
        FrameInfo {
            frame_number,
            time,
            dt: 0.1,
            seconds_remaining: None,
        }
    }

    fn ball() -> BallFrameState {
        BallFrameState::Present(BallSample {
            rigid_body: rigid_body(glam::Vec3::new(0.0, 0.0, 92.75), glam::Vec3::ZERO),
        })
    }

    #[test]
    fn counts_near_miss_after_player_exits_ball_area() {
        let player_id = boxcars::RemoteId::Steam(1);
        let mut calculator = WhiffCalculator::new();
        let ball = ball();
        let touch_state = TouchState::default();

        calculator
            .update(
                &frame(1, 0.1),
                &ball,
                &PlayerFrameState {
                    players: vec![player(1, -210.0, false)],
                },
                &touch_state,
                true,
            )
            .unwrap();
        calculator
            .update(
                &frame(2, 0.2),
                &ball,
                &PlayerFrameState {
                    players: vec![player(1, 460.0, false)],
                },
                &touch_state,
                true,
            )
            .unwrap();

        let stats = calculator.player_stats().get(&player_id).unwrap();
        assert_eq!(stats.whiff_count, 1);
        assert_eq!(stats.grounded_whiff_count, 1);
        assert_eq!(calculator.events().len(), 1);
    }

    #[test]
    fn touch_cancels_active_whiff_candidate() {
        let player_id = boxcars::RemoteId::Steam(1);
        let mut calculator = WhiffCalculator::new();
        let ball = ball();

        calculator
            .update(
                &frame(1, 0.1),
                &ball,
                &PlayerFrameState {
                    players: vec![player(1, -210.0, false)],
                },
                &TouchState::default(),
                true,
            )
            .unwrap();
        calculator
            .update(
                &frame(2, 0.2),
                &ball,
                &PlayerFrameState {
                    players: vec![player(1, 460.0, false)],
                },
                &TouchState {
                    touch_events: vec![TouchEvent {
                        time: 0.2,
                        frame: 2,
                        team_is_team_0: true,
                        player: Some(player_id.clone()),
                        closest_approach_distance: Some(0.0),
                    }],
                    ..TouchState::default()
                },
                true,
            )
            .unwrap();

        assert!(calculator.player_stats().get(&player_id).is_none());
        assert!(calculator.events().is_empty());
    }
}
