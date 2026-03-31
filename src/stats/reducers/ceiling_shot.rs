use super::*;

const SOCCAR_CEILING_Z: f32 = 2044.0;
const CEILING_CONTACT_MAX_GAP: f32 = 90.0;
const CEILING_CONTACT_MIN_ROOF_ALIGNMENT: f32 = 0.72;
const CEILING_SHOT_MAX_TOUCH_AFTER_CONTACT_SECONDS: f32 = 1.35;
const CEILING_SHOT_MIN_TOUCH_SEPARATION: f32 = 120.0;
const CEILING_SHOT_MIN_PLAYER_HEIGHT: f32 = 260.0;
const CEILING_SHOT_MIN_BALL_HEIGHT: f32 = 220.0;
const CEILING_SHOT_MIN_FORWARD_ALIGNMENT: f32 = 0.12;
const CEILING_SHOT_MIN_FORWARD_APPROACH_SPEED: f32 = 90.0;
const CEILING_SHOT_MIN_BALL_SPEED_CHANGE: f32 = 120.0;
const CEILING_SHOT_MIN_CONFIDENCE: f32 = 0.54;
const CEILING_SHOT_HIGH_CONFIDENCE: f32 = 0.78;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CeilingShotEvent {
    pub time: f32,
    pub frame: usize,
    pub player: PlayerId,
    pub is_team_0: bool,
    pub ceiling_contact_time: f32,
    pub ceiling_contact_frame: usize,
    pub time_since_ceiling_contact: f32,
    pub ceiling_contact_position: [f32; 3],
    pub touch_position: [f32; 3],
    pub local_ball_position: [f32; 3],
    pub separation_from_ceiling: f32,
    pub roof_alignment: f32,
    pub forward_alignment: f32,
    pub forward_approach_speed: f32,
    pub ball_speed_change: f32,
    pub confidence: f32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct CeilingShotStats {
    pub count: u32,
    pub high_confidence_count: u32,
    pub is_last_ceiling_shot: bool,
    pub last_ceiling_shot_time: Option<f32>,
    pub last_ceiling_shot_frame: Option<usize>,
    pub time_since_last_ceiling_shot: Option<f32>,
    pub frames_since_last_ceiling_shot: Option<usize>,
    pub last_confidence: Option<f32>,
    pub best_confidence: f32,
    pub cumulative_confidence: f32,
}

impl CeilingShotStats {
    pub fn average_confidence(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.cumulative_confidence / self.count as f32
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct RecentCeilingContact {
    time: f32,
    frame: usize,
    position: [f32; 3],
    roof_alignment: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct CeilingContactObservation {
    position: glam::Vec3,
    roof_alignment: f32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CeilingShotReducer {
    player_stats: HashMap<PlayerId, CeilingShotStats>,
    events: Vec<CeilingShotEvent>,
    recent_ceiling_contacts: HashMap<PlayerId, RecentCeilingContact>,
    previous_ball_velocity: Option<glam::Vec3>,
    current_last_ceiling_shot_player: Option<PlayerId>,
    live_play_tracker: LivePlayTracker,
}

impl CeilingShotReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, CeilingShotStats> {
        &self.player_stats
    }

    pub fn events(&self) -> &[CeilingShotEvent] {
        &self.events
    }

    fn normalize_score(value: f32, min_value: f32, max_value: f32) -> f32 {
        if max_value <= min_value {
            return 0.0;
        }

        ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0)
    }

    fn ball_speed_change(sample: &StatsSample, previous_ball_velocity: Option<glam::Vec3>) -> f32 {
        const BALL_GRAVITY_Z: f32 = -650.0;

        let Some(ball) = sample.ball.as_ref() else {
            return 0.0;
        };
        let Some(previous_ball_velocity) = previous_ball_velocity else {
            return 0.0;
        };

        let expected_linear_delta = glam::Vec3::new(0.0, 0.0, BALL_GRAVITY_Z * sample.dt.max(0.0));
        let residual_linear_impulse =
            ball.velocity() - previous_ball_velocity - expected_linear_delta;
        residual_linear_impulse.length()
    }

    fn begin_sample(&mut self, sample: &StatsSample) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_ceiling_shot = false;
            stats.time_since_last_ceiling_shot = stats
                .last_ceiling_shot_time
                .map(|time| (sample.time - time).max(0.0));
            stats.frames_since_last_ceiling_shot = stats
                .last_ceiling_shot_frame
                .map(|frame| sample.frame_number.saturating_sub(frame));
        }

        if let Some(player_id) = self.current_last_ceiling_shot_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_ceiling_shot = true;
            }
        }
    }

    fn ceiling_contact_observation(player: &PlayerSample) -> Option<CeilingContactObservation> {
        let rigid_body = player.rigid_body.as_ref()?;
        let position = player.position()?;
        let gap_to_ceiling = SOCCAR_CEILING_Z - position.z;
        if !(0.0..=CEILING_CONTACT_MAX_GAP).contains(&gap_to_ceiling) {
            return None;
        }

        let up = quat_to_glam(&rigid_body.rotation) * glam::Vec3::Z;
        let roof_alignment = (-up).dot(glam::Vec3::Z);
        if roof_alignment < CEILING_CONTACT_MIN_ROOF_ALIGNMENT {
            return None;
        }

        Some(CeilingContactObservation {
            position,
            roof_alignment,
        })
    }

    fn update_recent_ceiling_contacts(&mut self, sample: &StatsSample) {
        for player in &sample.players {
            let observation = Self::ceiling_contact_observation(player);
            let Some(observation) = observation else {
                continue;
            };

            self.recent_ceiling_contacts.insert(
                player.player_id.clone(),
                RecentCeilingContact {
                    time: sample.time,
                    frame: sample.frame_number,
                    position: observation.position.to_array(),
                    roof_alignment: observation.roof_alignment,
                },
            );
        }
    }

    fn prune_recent_ceiling_contacts(&mut self, current_time: f32) {
        self.recent_ceiling_contacts.retain(|_, contact| {
            current_time - contact.time <= CEILING_SHOT_MAX_TOUCH_AFTER_CONTACT_SECONDS
        });
    }

    fn candidate_event(
        &self,
        sample: &StatsSample,
        player: &PlayerSample,
        touch_event: &TouchEvent,
        recent_contact: RecentCeilingContact,
        ball_speed_change: f32,
    ) -> Option<CeilingShotEvent> {
        let ball = sample.ball.as_ref()?;
        let player_position = player.position()?;
        let player_rigid_body = player.rigid_body.as_ref()?;
        let ball_position = ball.position();

        if player_position.z < CEILING_SHOT_MIN_PLAYER_HEIGHT
            || ball_position.z < CEILING_SHOT_MIN_BALL_HEIGHT
        {
            return None;
        }

        let time_since_ceiling_contact = touch_event.time - recent_contact.time;
        if !(0.0..=CEILING_SHOT_MAX_TOUCH_AFTER_CONTACT_SECONDS)
            .contains(&time_since_ceiling_contact)
        {
            return None;
        }

        let separation_from_ceiling = SOCCAR_CEILING_Z - player_position.z;
        if separation_from_ceiling < CEILING_SHOT_MIN_TOUCH_SEPARATION {
            return None;
        }

        let relative_ball_position = ball_position - player_position;
        if relative_ball_position.length_squared() <= f32::EPSILON {
            return None;
        }

        let player_rotation = quat_to_glam(&player_rigid_body.rotation);
        let local_ball_position = player_rotation.inverse() * relative_ball_position;
        if local_ball_position.x < -120.0
            || local_ball_position.y.abs() > 260.0
            || local_ball_position.z.abs() > 240.0
        {
            return None;
        }

        let to_ball = relative_ball_position.normalize_or_zero();
        let forward = player_rotation * glam::Vec3::X;
        let forward_alignment = forward.dot(to_ball);
        if forward_alignment < CEILING_SHOT_MIN_FORWARD_ALIGNMENT {
            return None;
        }

        let forward_approach_speed = player.velocity().unwrap_or(glam::Vec3::ZERO).dot(to_ball);
        if forward_approach_speed < CEILING_SHOT_MIN_FORWARD_APPROACH_SPEED {
            return None;
        }
        if ball_speed_change < CEILING_SHOT_MIN_BALL_SPEED_CHANGE {
            return None;
        }

        let timing_score = 1.0
            - Self::normalize_score(
                time_since_ceiling_contact,
                0.10,
                CEILING_SHOT_MAX_TOUCH_AFTER_CONTACT_SECONDS,
            );
        let separation_score = Self::normalize_score(separation_from_ceiling, 140.0, 520.0);
        let height_score = Self::normalize_score(
            player_position.z.max(ball_position.z),
            CEILING_SHOT_MIN_BALL_HEIGHT,
            900.0,
        );
        let alignment_score =
            Self::normalize_score(forward_alignment, CEILING_SHOT_MIN_FORWARD_ALIGNMENT, 0.92);
        let approach_score = Self::normalize_score(
            forward_approach_speed,
            CEILING_SHOT_MIN_FORWARD_APPROACH_SPEED,
            900.0,
        );
        let impulse_score =
            Self::normalize_score(ball_speed_change, CEILING_SHOT_MIN_BALL_SPEED_CHANGE, 900.0);
        let contact_score = Self::normalize_score(
            recent_contact.roof_alignment,
            CEILING_CONTACT_MIN_ROOF_ALIGNMENT,
            0.98,
        );

        let confidence = 0.20 * timing_score
            + 0.15 * separation_score
            + 0.12 * height_score
            + 0.17 * alignment_score
            + 0.16 * approach_score
            + 0.10 * impulse_score
            + 0.10 * contact_score;
        if confidence < CEILING_SHOT_MIN_CONFIDENCE {
            return None;
        }

        Some(CeilingShotEvent {
            time: touch_event.time,
            frame: touch_event.frame,
            player: player.player_id.clone(),
            is_team_0: player.is_team_0,
            ceiling_contact_time: recent_contact.time,
            ceiling_contact_frame: recent_contact.frame,
            time_since_ceiling_contact,
            ceiling_contact_position: recent_contact.position,
            touch_position: ball_position.to_array(),
            local_ball_position: local_ball_position.to_array(),
            separation_from_ceiling,
            roof_alignment: recent_contact.roof_alignment,
            forward_alignment,
            forward_approach_speed,
            ball_speed_change,
            confidence,
        })
    }

    fn apply_touch_events(&mut self, sample: &StatsSample, touch_events: &[TouchEvent]) {
        let ball_speed_change = Self::ball_speed_change(sample, self.previous_ball_velocity);

        for touch_event in touch_events {
            let Some(player_id) = touch_event.player.as_ref() else {
                continue;
            };
            let Some(player) = sample
                .players
                .iter()
                .find(|player| &player.player_id == player_id)
            else {
                continue;
            };
            let Some(recent_contact) = self.recent_ceiling_contacts.get(player_id).copied() else {
                continue;
            };
            let Some(event) = self.candidate_event(
                sample,
                player,
                touch_event,
                recent_contact,
                ball_speed_change,
            ) else {
                continue;
            };

            let stats = self.player_stats.entry(player_id.clone()).or_default();
            stats.count += 1;
            if event.confidence >= CEILING_SHOT_HIGH_CONFIDENCE {
                stats.high_confidence_count += 1;
            }
            stats.is_last_ceiling_shot = true;
            stats.last_ceiling_shot_time = Some(event.time);
            stats.last_ceiling_shot_frame = Some(event.frame);
            stats.time_since_last_ceiling_shot = Some((sample.time - event.time).max(0.0));
            stats.frames_since_last_ceiling_shot =
                Some(sample.frame_number.saturating_sub(event.frame));
            stats.last_confidence = Some(event.confidence);
            stats.best_confidence = stats.best_confidence.max(event.confidence);
            stats.cumulative_confidence += event.confidence;

            self.current_last_ceiling_shot_player = Some(player_id.clone());
            self.events.push(event);
        }

        if let Some(player_id) = self.current_last_ceiling_shot_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_ceiling_shot = true;
            }
        }
    }

    fn reset_live_play_state(&mut self, sample: &StatsSample) {
        self.current_last_ceiling_shot_player = None;
        self.recent_ceiling_contacts.clear();
        self.previous_ball_velocity = sample.ball.as_ref().map(BallSample::velocity);
    }

    fn on_sample_internal(
        &mut self,
        sample: &StatsSample,
        touch_events: &[TouchEvent],
    ) -> SubtrActorResult<()> {
        if !self.live_play_tracker.is_live_play(sample) {
            self.reset_live_play_state(sample);
            return Ok(());
        }

        self.begin_sample(sample);
        self.prune_recent_ceiling_contacts(sample.time);
        self.apply_touch_events(sample, touch_events);
        self.update_recent_ceiling_contacts(sample);
        self.previous_ball_velocity = sample.ball.as_ref().map(BallSample::velocity);
        Ok(())
    }
}

impl StatsReducer for CeilingShotReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        self.on_sample_internal(sample, &sample.touch_events)
    }

    fn on_sample_with_context(
        &mut self,
        sample: &StatsSample,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<()> {
        let touch_state = ctx
            .get::<TouchState>(TOUCH_STATE_SIGNAL_ID)
            .cloned()
            .unwrap_or_default();
        self.on_sample_internal(sample, &touch_state.touch_events)
    }
}

#[cfg(test)]
mod tests {
    use boxcars::{Quaternion, RemoteId, RigidBody, Vector3f};

    use super::*;

    fn rigid_body(
        location: glam::Vec3,
        rotation: glam::Quat,
        linear_velocity: glam::Vec3,
        angular_velocity: glam::Vec3,
    ) -> RigidBody {
        RigidBody {
            sleeping: false,
            location: boxcars::Vector3f {
                x: location.x,
                y: location.y,
                z: location.z,
            },
            rotation: Quaternion {
                x: rotation.x,
                y: rotation.y,
                z: rotation.z,
                w: rotation.w,
            },
            linear_velocity: Some(Vector3f {
                x: linear_velocity.x,
                y: linear_velocity.y,
                z: linear_velocity.z,
            }),
            angular_velocity: Some(Vector3f {
                x: angular_velocity.x,
                y: angular_velocity.y,
                z: angular_velocity.z,
            }),
        }
    }

    fn sample(
        frame_number: usize,
        time: f32,
        player_body: RigidBody,
        ball_body: RigidBody,
        touch: bool,
    ) -> StatsSample {
        StatsSample {
            frame_number,
            time,
            dt: if frame_number == 0 { 0.0 } else { 1.0 / 120.0 },
            seconds_remaining: None,
            game_state: None,
            ball_has_been_hit: Some(true),
            kickoff_countdown_time: None,
            team_zero_score: Some(0),
            team_one_score: Some(0),
            possession_team_is_team_0: Some(true),
            scored_on_team_is_team_0: None,
            current_in_game_team_player_counts: Some([1, 1]),
            ball: Some(BallSample {
                rigid_body: ball_body,
            }),
            players: vec![
                PlayerSample {
                    player_id: RemoteId::Steam(1),
                    is_team_0: true,
                    rigid_body: Some(player_body),
                    boost_amount: None,
                    last_boost_amount: None,
                    boost_active: false,
                    dodge_active: false,
                    powerslide_active: false,
                    match_goals: None,
                    match_assists: None,
                    match_saves: None,
                    match_shots: None,
                    match_score: None,
                },
                PlayerSample {
                    player_id: RemoteId::Steam(2),
                    is_team_0: false,
                    rigid_body: Some(rigid_body(
                        glam::Vec3::new(3000.0, 0.0, 17.0),
                        glam::Quat::IDENTITY,
                        glam::Vec3::ZERO,
                        glam::Vec3::ZERO,
                    )),
                    boost_amount: None,
                    last_boost_amount: None,
                    boost_active: false,
                    dodge_active: false,
                    powerslide_active: false,
                    match_goals: None,
                    match_assists: None,
                    match_saves: None,
                    match_shots: None,
                    match_score: None,
                },
            ],
            active_demos: Vec::new(),
            demo_events: Vec::new(),
            boost_pad_events: Vec::new(),
            touch_events: if touch {
                vec![TouchEvent {
                    time,
                    frame: frame_number,
                    team_is_team_0: true,
                    player: Some(RemoteId::Steam(1)),
                    closest_approach_distance: Some(0.0),
                }]
            } else {
                Vec::new()
            },
            dodge_refreshed_events: Vec::new(),
            player_stat_events: Vec::new(),
            goal_events: Vec::new(),
        }
    }

    #[test]
    fn counts_touch_shortly_after_ceiling_contact_as_ceiling_shot() {
        let mut reducer = CeilingShotReducer::new();

        let on_ceiling = sample(
            0,
            0.0,
            rigid_body(
                glam::Vec3::new(0.0, -900.0, 1988.0),
                glam::Quat::from_rotation_y(std::f32::consts::PI),
                glam::Vec3::new(0.0, 400.0, 0.0),
                glam::Vec3::ZERO,
            ),
            rigid_body(
                glam::Vec3::new(220.0, -720.0, 1350.0),
                glam::Quat::IDENTITY,
                glam::Vec3::new(0.0, 0.0, 0.0),
                glam::Vec3::ZERO,
            ),
            false,
        );
        reducer.on_sample(&on_ceiling).unwrap();

        let ceiling_shot_touch = sample(
            1,
            1.0 / 120.0,
            rigid_body(
                glam::Vec3::new(0.0, -760.0, 1780.0),
                glam::Quat::IDENTITY,
                glam::Vec3::new(700.0, 260.0, -520.0),
                glam::Vec3::ZERO,
            ),
            rigid_body(
                glam::Vec3::new(115.0, -735.0, 1835.0),
                glam::Quat::IDENTITY,
                glam::Vec3::new(1250.0, 540.0, 420.0),
                glam::Vec3::ZERO,
            ),
            true,
        );
        reducer.on_sample(&ceiling_shot_touch).unwrap();

        assert_eq!(reducer.events().len(), 1);
        assert_eq!(reducer.player_stats()[&RemoteId::Steam(1)].count, 1);
        assert!(reducer.events()[0].confidence >= CEILING_SHOT_MIN_CONFIDENCE);
        assert!(
            reducer.events()[0].time_since_ceiling_contact
                <= CEILING_SHOT_MAX_TOUCH_AFTER_CONTACT_SECONDS
        );
    }

    #[test]
    fn rejects_touch_without_prior_ceiling_contact() {
        let mut reducer = CeilingShotReducer::new();

        let ordinary_touch = sample(
            0,
            0.0,
            rigid_body(
                glam::Vec3::new(0.0, -760.0, 540.0),
                glam::Quat::IDENTITY,
                glam::Vec3::new(700.0, 260.0, 0.0),
                glam::Vec3::ZERO,
            ),
            rigid_body(
                glam::Vec3::new(115.0, -735.0, 585.0),
                glam::Quat::IDENTITY,
                glam::Vec3::new(1250.0, 540.0, 420.0),
                glam::Vec3::ZERO,
            ),
            true,
        );
        reducer.on_sample(&ordinary_touch).unwrap();

        assert!(reducer.events().is_empty());
    }
}
