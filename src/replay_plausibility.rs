use crate::collector::replay_data::{BallFrame, PlayerFrame, ReplayData};
use crate::geometry::{quat_to_glam, vec_to_glam};

const MAX_PAIR_DT_SECONDS: f32 = 0.2;
const MIN_DISPLACEMENT_SPEED: f32 = 100.0;
const MIN_REPORTED_SPEED: f32 = 100.0;
const VELOCITY_RATIO_MIN: f32 = 0.5;
const VELOCITY_RATIO_MAX: f32 = 2.0;
const MIN_VELOCITY_PAIR_COUNT: usize = 100;
const MAX_PLAUSIBLE_LOCATION_ABS: f32 = 10_000.0;
const MAX_PLAUSIBLE_LINEAR_SPEED: f32 = 10_000.0;
const MAX_PLAUSIBLE_QUATERNION_NORM_ERROR: f32 = 0.01;
const MIN_FORWARD_ALIGNMENT_SPEED: f32 = 500.0;
const MAX_GROUNDED_HEIGHT: f32 = 60.0;
const MAX_GROUNDED_VERTICAL_SPEED: f32 = 200.0;

#[derive(Debug, Clone, PartialEq)]
pub struct ReplayPlausibilityReport {
    pub ball: RigidBodyPlausibilityReport,
    pub players: RigidBodyPlausibilityReport,
}

impl ReplayPlausibilityReport {
    pub fn all_motion_consistent(&self) -> bool {
        self.ball.motion_consistent() && self.players.motion_consistent()
    }

    pub fn all_field_bounds_plausible(&self) -> bool {
        self.ball.field_bounds_plausible() && self.players.field_bounds_plausible()
    }

    pub fn all_location_bounds_plausible(&self) -> bool {
        self.ball.location_bounds_plausible() && self.players.location_bounds_plausible()
    }

    pub fn all_linear_speed_bounds_plausible(&self) -> bool {
        self.ball.linear_speed_bounds_plausible() && self.players.linear_speed_bounds_plausible()
    }

    pub fn all_quaternion_norms_plausible(&self) -> bool {
        self.ball.quaternion_norms_plausible() && self.players.quaternion_norms_plausible()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RigidBodyPlausibilityReport {
    pub sample_count: usize,
    pub motion_pair_count: usize,
    pub rotation_pair_count: usize,
    pub max_abs_location: f32,
    pub max_linear_speed: f32,
    pub max_angular_speed: f32,
    pub max_quaternion_norm_error: f32,
    pub max_orientation_speed: f32,
    pub median_velocity_to_displacement_ratio: Option<f32>,
    pub median_velocity_log10_error: Option<f32>,
    pub velocity_pairs_within_factor_2_fraction: Option<f32>,
    pub median_rotation_angle_delta_radians: Option<f32>,
    pub median_orientation_speed: Option<f32>,
    pub grounded_forward_alignment_sample_count: usize,
    pub median_grounded_forward_alignment: Option<f32>,
    pub grounded_forward_alignment_positive_fraction: Option<f32>,
}

impl RigidBodyPlausibilityReport {
    pub fn field_bounds_plausible(&self) -> bool {
        self.location_bounds_plausible() && self.linear_speed_bounds_plausible()
    }

    pub fn location_bounds_plausible(&self) -> bool {
        self.max_abs_location <= MAX_PLAUSIBLE_LOCATION_ABS
    }

    pub fn linear_speed_bounds_plausible(&self) -> bool {
        self.max_linear_speed <= MAX_PLAUSIBLE_LINEAR_SPEED
    }

    pub fn quaternion_norms_plausible(&self) -> bool {
        self.max_quaternion_norm_error <= MAX_PLAUSIBLE_QUATERNION_NORM_ERROR
    }

    pub fn motion_consistent(&self) -> bool {
        if self.motion_pair_count < MIN_VELOCITY_PAIR_COUNT {
            return true;
        }

        self.median_velocity_to_displacement_ratio
            .is_some_and(|ratio| (VELOCITY_RATIO_MIN..=VELOCITY_RATIO_MAX).contains(&ratio))
            && self
                .velocity_pairs_within_factor_2_fraction
                .is_some_and(|fraction| fraction >= 0.6)
    }
}

#[derive(Debug, Default)]
struct RigidBodyPlausibilityAccumulator {
    sample_count: usize,
    motion_ratios: Vec<f32>,
    motion_log10_errors: Vec<f32>,
    rotation_angle_deltas: Vec<f32>,
    orientation_speeds: Vec<f32>,
    grounded_forward_alignments: Vec<f32>,
    max_abs_location: f32,
    max_linear_speed: f32,
    max_angular_speed: f32,
    max_quaternion_norm_error: f32,
    max_orientation_speed: f32,
}

impl RigidBodyPlausibilityAccumulator {
    fn add_sample(&mut self, rigid_body: &boxcars::RigidBody) {
        self.sample_count += 1;
        let location = vec_to_glam(&rigid_body.location);
        self.max_abs_location = self
            .max_abs_location
            .max(location.x.abs())
            .max(location.y.abs())
            .max(location.z.abs());

        if let Some(linear_velocity) = rigid_body.linear_velocity {
            self.max_linear_speed = self
                .max_linear_speed
                .max(vec_to_glam(&linear_velocity).length());
        }
        if let Some(angular_velocity) = rigid_body.angular_velocity {
            self.max_angular_speed = self
                .max_angular_speed
                .max(vec_to_glam(&angular_velocity).length());
        }

        let rotation = quat_to_glam(&rigid_body.rotation);
        self.max_quaternion_norm_error = self
            .max_quaternion_norm_error
            .max((rotation.length() - 1.0).abs());

        if let Some(linear_velocity) = rigid_body.linear_velocity {
            let velocity = vec_to_glam(&linear_velocity);
            let planar_speed = velocity.truncate().length();
            let grounded = rigid_body.location.z.abs() <= MAX_GROUNDED_HEIGHT
                && velocity.z.abs() <= MAX_GROUNDED_VERTICAL_SPEED;
            if grounded
                && planar_speed >= MIN_FORWARD_ALIGNMENT_SPEED
                && rotation.length_squared() > f32::EPSILON
            {
                let forward = rotation.normalize() * glam::Vec3::X;
                let forward_xy = forward.truncate().normalize_or_zero();
                let velocity_xy = velocity.truncate().normalize_or_zero();
                let alignment = forward_xy.dot(velocity_xy);
                if alignment.is_finite() {
                    self.grounded_forward_alignments.push(alignment);
                }
            }
        }
    }

    fn add_pair(
        &mut self,
        previous_time: f32,
        previous: &boxcars::RigidBody,
        current_time: f32,
        current: &boxcars::RigidBody,
    ) {
        let dt = current_time - previous_time;
        if !(0.0..=MAX_PAIR_DT_SECONDS).contains(&dt) {
            return;
        }

        let displacement_speed =
            (vec_to_glam(&current.location) - vec_to_glam(&previous.location)).length() / dt;

        if let Some(reported_speed) = previous
            .linear_velocity
            .or(current.linear_velocity)
            .map(|velocity| vec_to_glam(&velocity).length())
        {
            if displacement_speed >= MIN_DISPLACEMENT_SPEED && reported_speed >= MIN_REPORTED_SPEED
            {
                let ratio = reported_speed / displacement_speed;
                if ratio.is_finite() && ratio > 0.0 {
                    self.motion_ratios.push(ratio);
                    self.motion_log10_errors.push(ratio.log10().abs());
                }
            }
        }

        let previous_rotation = quat_to_glam(&previous.rotation);
        let current_rotation = quat_to_glam(&current.rotation);
        if previous_rotation.length_squared() > f32::EPSILON
            && current_rotation.length_squared() > f32::EPSILON
        {
            let previous_rotation = previous_rotation.normalize();
            let current_rotation = current_rotation.normalize();
            let dot = previous_rotation
                .dot(current_rotation)
                .abs()
                .clamp(0.0, 1.0);
            let angle_delta = 2.0 * dot.acos();
            let orientation_speed = angle_delta / dt;
            if angle_delta.is_finite() && orientation_speed.is_finite() {
                self.rotation_angle_deltas.push(angle_delta);
                self.orientation_speeds.push(orientation_speed);
                self.max_orientation_speed = self.max_orientation_speed.max(orientation_speed);
            }
        }
    }

    fn finish(mut self) -> RigidBodyPlausibilityReport {
        let within_factor_2 = if self.motion_ratios.is_empty() {
            None
        } else {
            Some(
                self.motion_ratios
                    .iter()
                    .filter(|ratio| (VELOCITY_RATIO_MIN..=VELOCITY_RATIO_MAX).contains(ratio))
                    .count() as f32
                    / self.motion_ratios.len() as f32,
            )
        };

        RigidBodyPlausibilityReport {
            sample_count: self.sample_count,
            motion_pair_count: self.motion_ratios.len(),
            rotation_pair_count: self.rotation_angle_deltas.len(),
            max_abs_location: self.max_abs_location,
            max_linear_speed: self.max_linear_speed,
            max_angular_speed: self.max_angular_speed,
            max_quaternion_norm_error: self.max_quaternion_norm_error,
            max_orientation_speed: self.max_orientation_speed,
            median_velocity_to_displacement_ratio: median(&mut self.motion_ratios),
            median_velocity_log10_error: median(&mut self.motion_log10_errors),
            velocity_pairs_within_factor_2_fraction: within_factor_2,
            median_rotation_angle_delta_radians: median(&mut self.rotation_angle_deltas),
            median_orientation_speed: median(&mut self.orientation_speeds),
            grounded_forward_alignment_sample_count: self.grounded_forward_alignments.len(),
            median_grounded_forward_alignment: median(&mut self.grounded_forward_alignments),
            grounded_forward_alignment_positive_fraction: positive_fraction(
                &self.grounded_forward_alignments,
            ),
        }
    }
}

pub fn evaluate_replay_plausibility(replay_data: &ReplayData) -> ReplayPlausibilityReport {
    let mut ball = RigidBodyPlausibilityAccumulator::default();
    let mut players = RigidBodyPlausibilityAccumulator::default();
    let times: Vec<f32> = replay_data
        .frame_data
        .metadata_frames
        .iter()
        .map(|frame| frame.time)
        .collect();

    let mut previous_ball: Option<(f32, &boxcars::RigidBody)> = None;
    for (time, frame) in times
        .iter()
        .copied()
        .zip(replay_data.frame_data.ball_data.frames())
    {
        if let BallFrame::Data { rigid_body } = frame {
            ball.add_sample(rigid_body);
            if let Some((previous_time, previous_rigid_body)) = previous_ball {
                ball.add_pair(previous_time, previous_rigid_body, time, rigid_body);
            }
            previous_ball = Some((time, rigid_body));
        }
    }

    for (_, player_data) in &replay_data.frame_data.players {
        let mut previous_player: Option<(f32, &boxcars::RigidBody)> = None;
        for (time, frame) in times.iter().copied().zip(player_data.frames()) {
            if let PlayerFrame::Data { rigid_body, .. } = frame {
                players.add_sample(rigid_body);
                if let Some((previous_time, previous_rigid_body)) = previous_player {
                    players.add_pair(previous_time, previous_rigid_body, time, rigid_body);
                }
                previous_player = Some((time, rigid_body));
            }
        }
    }

    ReplayPlausibilityReport {
        ball: ball.finish(),
        players: players.finish(),
    }
}

fn median(values: &mut [f32]) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f32::total_cmp);
    let middle = values.len() / 2;
    if values.len() % 2 == 0 {
        Some((values[middle - 1] + values[middle]) / 2.0)
    } else {
        Some(values[middle])
    }
}

fn positive_fraction(values: &[f32]) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    Some(values.iter().filter(|value| **value > 0.0).count() as f32 / values.len() as f32)
}
