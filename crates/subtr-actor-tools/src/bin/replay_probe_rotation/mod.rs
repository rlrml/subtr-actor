use std::collections::HashMap;

use subtr_actor::{Collector, ProcessorView, TimeAdvance};

use super::{median, positive_fraction};

const MIN_FORWARD_ALIGNMENT_SPEED: f32 = 500.0;
const MAX_GROUNDED_HEIGHT: f32 = 60.0;
const MAX_GROUNDED_VERTICAL_SPEED: f32 = 200.0;
const MAX_PAIR_DT_SECONDS: f32 = 0.2;
const MIN_DISPLACEMENT_SPEED: f32 = 100.0;
const MIN_REPORTED_SPEED: f32 = 100.0;
const MIN_ROTATION_MODE_SAMPLE_COUNT: usize = 100;
const MIN_ANGULAR_VELOCITY_SPEED: f32 = 30.0;
const MIN_DERIVED_ORIENTATION_SPEED: f32 = 0.5;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct QuaternionMode {
    missing_slot: usize,
    order: [usize; 3],
    signs: [i8; 3],
    reconstruct_missing: bool,
}

impl QuaternionMode {
    fn label(&self) -> String {
        format!(
            "{}@{} order={:?} signs={:?}",
            if self.reconstruct_missing {
                "reconstruct"
            } else {
                "zero"
            },
            self.missing_slot,
            self.order,
            self.signs
        )
    }
}

#[derive(Debug, Default)]
struct ModeAccumulator {
    alignments: Vec<f32>,
    up_zs: Vec<f32>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct EulerMode {
    order: [usize; 3],
    signs: [i8; 3],
    scale: EulerScale,
    rotation_order: EulerRotationOrder,
}

impl EulerMode {
    fn label(&self) -> String {
        format!(
            "euler order={:?} signs={:?} scale={:?} rot={:?}",
            self.order, self.signs, self.scale, self.rotation_order
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum EulerScale {
    Pi,
    TwoPi,
    HalfPi,
}

impl EulerScale {
    fn factor(self) -> f32 {
        match self {
            Self::Pi => std::f32::consts::PI,
            Self::TwoPi => std::f32::consts::TAU,
            Self::HalfPi => std::f32::consts::FRAC_PI_2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum EulerRotationOrder {
    Xyz,
    Xzy,
    Yxz,
    Yzx,
    Zxy,
    Zyx,
}

impl EulerRotationOrder {
    fn to_glam(self) -> glam::EulerRot {
        match self {
            Self::Xyz => glam::EulerRot::XYZ,
            Self::Xzy => glam::EulerRot::XZY,
            Self::Yxz => glam::EulerRot::YXZ,
            Self::Yzx => glam::EulerRot::YZX,
            Self::Zxy => glam::EulerRot::ZXY,
            Self::Zyx => glam::EulerRot::ZYX,
        }
    }
}

#[derive(Debug, Default)]
struct VelocityScaleAccumulator {
    ratios: Vec<f32>,
}

#[derive(Debug, Default)]
struct AngularVelocityAccumulator {
    direction_dots: Vec<f32>,
}

#[derive(Debug)]
pub(super) struct LegacyRotationProbe {
    modes: Vec<QuaternionMode>,
    accumulators: HashMap<QuaternionMode, ModeAccumulator>,
    euler_modes: Vec<EulerMode>,
    euler_accumulators: HashMap<EulerMode, ModeAccumulator>,
    euler_angular_accumulators: HashMap<EulerMode, AngularVelocityAccumulator>,
    velocity_accumulators: Vec<(f32, VelocityScaleAccumulator)>,
    previous_bodies: HashMap<subtr_actor::PlayerId, (f32, boxcars::RigidBody)>,
}

impl LegacyRotationProbe {
    pub(super) fn new() -> Self {
        let modes = build_modes();
        let accumulators = modes
            .iter()
            .copied()
            .map(|mode| (mode, ModeAccumulator::default()))
            .collect();
        let euler_modes = build_euler_modes();
        let euler_accumulators = euler_modes
            .iter()
            .copied()
            .map(|mode| (mode, ModeAccumulator::default()))
            .collect();
        let euler_angular_accumulators = euler_modes
            .iter()
            .copied()
            .map(|mode| (mode, AngularVelocityAccumulator::default()))
            .collect();
        let velocity_accumulators = [1.0, 0.1, 0.01]
            .into_iter()
            .map(|scale| (scale, VelocityScaleAccumulator::default()))
            .collect();
        Self {
            modes,
            accumulators,
            euler_modes,
            euler_accumulators,
            euler_angular_accumulators,
            velocity_accumulators,
            previous_bodies: HashMap::new(),
        }
    }

    fn sample_player(
        &mut self,
        player_id: &subtr_actor::PlayerId,
        time: f32,
        rigid_body: boxcars::RigidBody,
    ) {
        if let Some(linear_velocity) = rigid_body.linear_velocity {
            let planar_speed = glam::Vec2::new(linear_velocity.x, linear_velocity.y).length();
            let grounded = rigid_body.location.z.abs() <= MAX_GROUNDED_HEIGHT
                && linear_velocity.z.abs() <= MAX_GROUNDED_VERTICAL_SPEED;
            if grounded && planar_speed >= MIN_FORWARD_ALIGNMENT_SPEED {
                for mode in &self.modes {
                    if let Some(quaternion) = reinterpret_quaternion(rigid_body.rotation, *mode) {
                        if let Some((alignment, up_z)) =
                            rotation_alignment(quaternion, linear_velocity)
                        {
                            let accumulator = self.accumulators.get_mut(mode).unwrap();
                            accumulator.alignments.push(alignment);
                            accumulator.up_zs.push(up_z);
                        }
                    }
                }
                for mode in &self.euler_modes {
                    let quaternion = reinterpret_euler_rotation(rigid_body.rotation, *mode);
                    if let Some((alignment, up_z)) = rotation_alignment(quaternion, linear_velocity)
                    {
                        let accumulator = self.euler_accumulators.get_mut(mode).unwrap();
                        accumulator.alignments.push(alignment);
                        accumulator.up_zs.push(up_z);
                    }
                }
            }
        }

        if let Some((previous_time, previous_body)) = self.previous_bodies.get(player_id) {
            let dt = time - previous_time;
            if (0.0..=MAX_PAIR_DT_SECONDS).contains(&dt) {
                let displacement = glam::Vec3::new(
                    rigid_body.location.x - previous_body.location.x,
                    rigid_body.location.y - previous_body.location.y,
                    rigid_body.location.z - previous_body.location.z,
                );
                let displacement_speed = displacement.length() / dt;
                if displacement_speed >= MIN_DISPLACEMENT_SPEED {
                    let reported_velocity = previous_body
                        .linear_velocity
                        .or(rigid_body.linear_velocity)
                        .map(|velocity| {
                            glam::Vec3::new(velocity.x, velocity.y, velocity.z).length()
                        });
                    if let Some(reported_speed) = reported_velocity {
                        if reported_speed >= MIN_REPORTED_SPEED {
                            for (scale, accumulator) in &mut self.velocity_accumulators {
                                let ratio = (reported_speed * *scale) / displacement_speed;
                                if ratio.is_finite() && ratio > 0.0 {
                                    accumulator.ratios.push(ratio);
                                }
                            }
                        }
                    }
                }

                if let Some(reported_angular_velocity) = previous_body
                    .angular_velocity
                    .or(rigid_body.angular_velocity)
                {
                    let reported_angular_velocity = glam::Vec3::new(
                        reported_angular_velocity.x,
                        reported_angular_velocity.y,
                        reported_angular_velocity.z,
                    );
                    let reported_angular_speed = reported_angular_velocity.length();
                    if reported_angular_speed >= MIN_ANGULAR_VELOCITY_SPEED {
                        for mode in &self.euler_modes {
                            let previous_rotation =
                                reinterpret_euler_rotation(previous_body.rotation, *mode);
                            let current_rotation =
                                reinterpret_euler_rotation(rigid_body.rotation, *mode);
                            if let Some(derived_angular_velocity) = derive_world_angular_velocity(
                                previous_rotation,
                                current_rotation,
                                dt,
                            ) {
                                if derived_angular_velocity.length()
                                    >= MIN_DERIVED_ORIENTATION_SPEED
                                {
                                    let direction_dot = derived_angular_velocity
                                        .normalize()
                                        .dot(reported_angular_velocity.normalize());
                                    if direction_dot.is_finite() {
                                        self.euler_angular_accumulators
                                            .get_mut(mode)
                                            .unwrap()
                                            .direction_dots
                                            .push(direction_dot);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        self.previous_bodies
            .insert(player_id.clone(), (time, rigid_body));
    }

    pub(super) fn print_summary(&mut self) {
        let mode_summaries: Vec<_> = self
            .modes
            .iter()
            .filter_map(|mode| {
                let accumulator = self.accumulators.get_mut(mode)?;
                let median_alignment = median(&mut accumulator.alignments)?;
                let positive_fraction = positive_fraction(&accumulator.alignments)?;
                let median_up_z = median(&mut accumulator.up_zs)?;
                Some((
                    *mode,
                    accumulator.alignments.len(),
                    median_alignment,
                    positive_fraction,
                    median_up_z,
                ))
            })
            .collect();
        let baseline_mode = QuaternionMode {
            missing_slot: 3,
            order: [0, 1, 2],
            signs: [1, 1, 1],
            reconstruct_missing: false,
        };

        let baseline_summary = mode_summaries
            .iter()
            .find(|(mode, ..)| *mode == baseline_mode)
            .copied();

        let mut alignment_ranked: Vec<_> = mode_summaries
            .iter()
            .copied()
            .filter(|(_, samples, ..)| *samples >= MIN_ROTATION_MODE_SAMPLE_COUNT)
            .collect();
        alignment_ranked.sort_by(|left, right| {
            right
                .2
                .partial_cmp(&left.2)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    right
                        .3
                        .partial_cmp(&left.3)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| {
                    right
                        .4
                        .partial_cmp(&left.4)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        let mut upright_ranked = alignment_ranked.clone();
        upright_ranked.sort_by(|left, right| {
            right
                .4
                .partial_cmp(&left.4)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    right
                        .2
                        .partial_cmp(&left.2)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        let mut combined_ranked = alignment_ranked.clone();
        combined_ranked.sort_by(|left, right| {
            let left_score = left.2.min(left.4);
            let right_score = right.2.min(right.4);
            right_score
                .partial_cmp(&left_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    right
                        .2
                        .partial_cmp(&left.2)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        if let Some((mode, samples, median_alignment, positive_fraction, median_up_z)) =
            baseline_summary
        {
            println!(
                "Baseline mode: {:<48} samples={:<6} median_alignment={:>7.4} positive_fraction={:>7.4} median_up_z={:>7.4}",
                mode.label(),
                samples,
                median_alignment,
                positive_fraction,
                median_up_z
            );
            println!();
        }

        println!("Top quaternion reinterpretations by forward alignment:");
        for (rank, (mode, samples, median_alignment, positive_fraction, median_up_z)) in
            alignment_ranked.iter().take(12).enumerate()
        {
            println!(
                "{:>2}. {:<48} samples={:<6} median_alignment={:>7.4} positive_fraction={:>7.4} median_up_z={:>7.4}",
                rank + 1,
                mode.label(),
                samples,
                median_alignment,
                positive_fraction,
                median_up_z
            );
        }

        println!();
        println!("Top quaternion reinterpretations by grounded uprightness:");
        for (rank, (mode, samples, median_alignment, positive_fraction, median_up_z)) in
            upright_ranked.iter().take(12).enumerate()
        {
            println!(
                "{:>2}. {:<48} samples={:<6} median_alignment={:>7.4} positive_fraction={:>7.4} median_up_z={:>7.4}",
                rank + 1,
                mode.label(),
                samples,
                median_alignment,
                positive_fraction,
                median_up_z
            );
        }

        println!();
        println!("Top quaternion reinterpretations by combined min(alignment, up_z):");
        for (rank, (mode, samples, median_alignment, positive_fraction, median_up_z)) in
            combined_ranked.iter().take(12).enumerate()
        {
            println!(
                "{:>2}. {:<48} samples={:<6} median_alignment={:>7.4} positive_fraction={:>7.4} median_up_z={:>7.4}",
                rank + 1,
                mode.label(),
                samples,
                median_alignment,
                positive_fraction,
                median_up_z
            );
        }

        println!();
        println!("Top Euler interpretations by combined min(alignment, up_z):");
        let mut euler_ranked: Vec<_> = self
            .euler_modes
            .iter()
            .filter_map(|mode| {
                let accumulator = self.euler_accumulators.get_mut(mode)?;
                let median_alignment = median(&mut accumulator.alignments)?;
                let positive_fraction = positive_fraction(&accumulator.alignments)?;
                let median_up_z = median(&mut accumulator.up_zs)?;
                Some((
                    *mode,
                    accumulator.alignments.len(),
                    median_alignment,
                    positive_fraction,
                    median_up_z,
                ))
            })
            .filter(|(_, samples, ..)| *samples >= MIN_ROTATION_MODE_SAMPLE_COUNT)
            .collect();
        euler_ranked.sort_by(|left, right| {
            let left_score = left.2.min(left.4);
            let right_score = right.2.min(right.4);
            right_score
                .partial_cmp(&left_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    right
                        .2
                        .partial_cmp(&left.2)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });
        for (rank, (mode, samples, median_alignment, positive_fraction, median_up_z)) in
            euler_ranked.iter().take(12).enumerate()
        {
            println!(
                "{:>2}. {:<72} samples={:<6} median_alignment={:>7.4} positive_fraction={:>7.4} median_up_z={:>7.4}",
                rank + 1,
                mode.label(),
                samples,
                median_alignment,
                positive_fraction,
                median_up_z
            );
        }

        println!();
        println!("Top Euler interpretations by angular-velocity direction:");
        let mut euler_angular_ranked: Vec<_> = self
            .euler_modes
            .iter()
            .filter_map(|mode| {
                let accumulator = self.euler_angular_accumulators.get_mut(mode)?;
                let median_direction_dot = median(&mut accumulator.direction_dots)?;
                let positive_fraction = positive_fraction(&accumulator.direction_dots)?;
                Some((
                    *mode,
                    accumulator.direction_dots.len(),
                    median_direction_dot,
                    positive_fraction,
                ))
            })
            .filter(|(_, samples, ..)| *samples >= MIN_ROTATION_MODE_SAMPLE_COUNT)
            .collect();
        euler_angular_ranked.sort_by(|left, right| {
            right
                .2
                .partial_cmp(&left.2)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    right
                        .3
                        .partial_cmp(&left.3)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });
        for (rank, (mode, samples, median_direction_dot, positive_fraction)) in
            euler_angular_ranked.iter().take(12).enumerate()
        {
            println!(
                "{:>2}. {:<72} samples={:<6} median_direction_dot={:>7.4} positive_fraction={:>7.4}",
                rank + 1,
                mode.label(),
                samples,
                median_direction_dot,
                positive_fraction
            );
        }

        println!();
        println!("Velocity scale hypotheses:");
        for (scale, accumulator) in &mut self.velocity_accumulators {
            let median_ratio = median(&mut accumulator.ratios).unwrap_or(f32::NAN);
            let within_factor_2 = accumulator
                .ratios
                .iter()
                .filter(|ratio| (0.5..=2.0).contains(*ratio))
                .count() as f32
                / accumulator.ratios.len().max(1) as f32;
            println!(
                "scale={scale:<4} samples={:<6} median_velocity_to_displacement_ratio={:>7.4} within_factor_2_fraction={:>7.4}",
                accumulator.ratios.len(),
                median_ratio,
                within_factor_2
            );
        }
    }
}

impl Collector for LegacyRotationProbe {
    fn process_frame(
        &mut self,
        processor: &dyn ProcessorView,
        _frame: &boxcars::Frame,
        _frame_number: usize,
        current_time: f32,
    ) -> subtr_actor::SubtrActorResult<TimeAdvance> {
        let player_ids: Vec<_> = processor.iter_player_ids_in_order().cloned().collect();
        for player_id in &player_ids {
            if let Ok(rigid_body) = processor.get_normalized_player_rigid_body(player_id) {
                if !rigid_body.sleeping {
                    self.sample_player(player_id, current_time, rigid_body);
                }
            }
        }
        Ok(TimeAdvance::NextFrame)
    }
}

fn build_modes() -> Vec<QuaternionMode> {
    let orders = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    let signs = [
        [1, 1, 1],
        [1, 1, -1],
        [1, -1, 1],
        [1, -1, -1],
        [-1, 1, 1],
        [-1, 1, -1],
        [-1, -1, 1],
        [-1, -1, -1],
    ];

    let mut modes = Vec::new();
    for missing_slot in 0..4 {
        for order in orders {
            for sign in signs {
                for reconstruct_missing in [false, true] {
                    modes.push(QuaternionMode {
                        missing_slot,
                        order,
                        signs: sign,
                        reconstruct_missing,
                    });
                }
            }
        }
    }
    modes
}

fn build_euler_modes() -> Vec<EulerMode> {
    let orders = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    let signs = [
        [1, 1, 1],
        [1, 1, -1],
        [1, -1, 1],
        [1, -1, -1],
        [-1, 1, 1],
        [-1, 1, -1],
        [-1, -1, 1],
        [-1, -1, -1],
    ];
    let scales = [EulerScale::Pi, EulerScale::TwoPi, EulerScale::HalfPi];
    let rotation_orders = [
        EulerRotationOrder::Xyz,
        EulerRotationOrder::Xzy,
        EulerRotationOrder::Yxz,
        EulerRotationOrder::Yzx,
        EulerRotationOrder::Zxy,
        EulerRotationOrder::Zyx,
    ];

    let mut modes = Vec::new();
    for order in orders {
        for sign in signs {
            for scale in scales {
                for rotation_order in rotation_orders {
                    modes.push(EulerMode {
                        order,
                        signs: sign,
                        scale,
                        rotation_order,
                    });
                }
            }
        }
    }
    modes
}

fn reinterpret_quaternion(raw: boxcars::Quaternion, mode: QuaternionMode) -> Option<glam::Quat> {
    let source = [raw.x, raw.y, raw.z];
    let values = [
        source[mode.order[0]] * f32::from(mode.signs[0]),
        source[mode.order[1]] * f32::from(mode.signs[1]),
        source[mode.order[2]] * f32::from(mode.signs[2]),
    ];
    let mut components = [0.0; 4];
    let mut value_index = 0;
    for (slot, component) in components.iter_mut().enumerate() {
        if slot == mode.missing_slot {
            continue;
        }
        *component = values[value_index];
        value_index += 1;
    }
    if mode.reconstruct_missing {
        let sum_squares: f32 = components
            .iter()
            .map(|component| component * component)
            .sum();
        if sum_squares > 1.0 + 0.001 {
            return None;
        }
        components[mode.missing_slot] = (1.0 - sum_squares.min(1.0)).sqrt();
    }
    let quaternion =
        glam::Quat::from_xyzw(components[0], components[1], components[2], components[3]);
    (quaternion.length_squared() > f32::EPSILON).then(|| quaternion.normalize())
}

fn reinterpret_euler_rotation(raw: boxcars::Quaternion, mode: EulerMode) -> glam::Quat {
    let source = [raw.x, raw.y, raw.z];
    let factor = mode.scale.factor();
    let values = [
        source[mode.order[0]] * f32::from(mode.signs[0]) * factor,
        source[mode.order[1]] * f32::from(mode.signs[1]) * factor,
        source[mode.order[2]] * f32::from(mode.signs[2]) * factor,
    ];
    glam::Quat::from_euler(
        mode.rotation_order.to_glam(),
        values[0],
        values[1],
        values[2],
    )
}

fn rotation_alignment(
    quaternion: glam::Quat,
    linear_velocity: boxcars::Vector3f,
) -> Option<(f32, f32)> {
    let forward = quaternion * glam::Vec3::X;
    let forward_xy = forward.truncate().normalize_or_zero();
    let velocity_xy = glam::Vec2::new(linear_velocity.x, linear_velocity.y).normalize_or_zero();
    let alignment = forward_xy.dot(velocity_xy);
    alignment
        .is_finite()
        .then_some((alignment, (quaternion * glam::Vec3::Z).z))
}

fn derive_world_angular_velocity(
    previous_rotation: glam::Quat,
    mut current_rotation: glam::Quat,
    dt: f32,
) -> Option<glam::Vec3> {
    if dt <= 0.0 {
        return None;
    }
    if previous_rotation.dot(current_rotation) < 0.0 {
        current_rotation = glam::Quat::from_xyzw(
            -current_rotation.x,
            -current_rotation.y,
            -current_rotation.z,
            -current_rotation.w,
        );
    }
    let delta = current_rotation * previous_rotation.inverse();
    let (axis, angle) = delta.to_axis_angle();
    let angular_velocity = axis * (angle / dt);
    angular_velocity.is_finite().then_some(angular_velocity)
}
