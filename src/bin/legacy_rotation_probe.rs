use std::collections::HashMap;

use subtr_actor::{Collector, ReplayProcessor, TimeAdvance};

const MIN_FORWARD_ALIGNMENT_SPEED: f32 = 500.0;
const MAX_GROUNDED_HEIGHT: f32 = 60.0;
const MAX_GROUNDED_VERTICAL_SPEED: f32 = 200.0;
const MAX_PAIR_DT_SECONDS: f32 = 0.2;
const MIN_DISPLACEMENT_SPEED: f32 = 100.0;
const MIN_REPORTED_SPEED: f32 = 100.0;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct QuaternionMode {
    zero_slot: usize,
    order: [usize; 3],
    signs: [i8; 3],
}

impl QuaternionMode {
    fn label(&self) -> String {
        format!(
            "zero@{} order={:?} signs={:?}",
            self.zero_slot, self.order, self.signs
        )
    }
}

#[derive(Debug, Default)]
struct ModeAccumulator {
    alignments: Vec<f32>,
    up_zs: Vec<f32>,
}

#[derive(Debug, Default)]
struct VelocityScaleAccumulator {
    ratios: Vec<f32>,
}

#[derive(Debug)]
struct LegacyRotationProbe {
    modes: Vec<QuaternionMode>,
    accumulators: HashMap<QuaternionMode, ModeAccumulator>,
    velocity_accumulators: Vec<(f32, VelocityScaleAccumulator)>,
    previous_bodies: HashMap<subtr_actor::PlayerId, (f32, boxcars::RigidBody)>,
}

impl LegacyRotationProbe {
    fn new() -> Self {
        let modes = build_modes();
        let accumulators = modes
            .iter()
            .copied()
            .map(|mode| (mode, ModeAccumulator::default()))
            .collect();
        let velocity_accumulators = [1.0, 0.1, 0.01]
            .into_iter()
            .map(|scale| (scale, VelocityScaleAccumulator::default()))
            .collect();
        Self {
            modes,
            accumulators,
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
                        let forward = quaternion * glam::Vec3::X;
                        let forward_xy = forward.truncate().normalize_or_zero();
                        let velocity_xy = glam::Vec2::new(linear_velocity.x, linear_velocity.y)
                            .normalize_or_zero();
                        let alignment = forward_xy.dot(velocity_xy);
                        if alignment.is_finite() {
                            let accumulator = self.accumulators.get_mut(mode).unwrap();
                            accumulator.alignments.push(alignment);
                            accumulator.up_zs.push((quaternion * glam::Vec3::Z).z);
                        }
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
            }
        }

        self.previous_bodies
            .insert(player_id.clone(), (time, rigid_body));
    }

    fn print_summary(&mut self) {
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
            zero_slot: 3,
            order: [0, 1, 2],
            signs: [1, 1, 1],
        };

        let baseline_summary = mode_summaries
            .iter()
            .find(|(mode, ..)| *mode == baseline_mode)
            .copied();

        let mut alignment_ranked = mode_summaries.clone();
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

        let mut upright_ranked = mode_summaries.clone();
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

        let mut combined_ranked = mode_summaries.clone();
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
        processor: &ReplayProcessor,
        _frame: &boxcars::Frame,
        _frame_number: usize,
        current_time: f32,
    ) -> subtr_actor::SubtrActorResult<TimeAdvance> {
        let player_ids: Vec<_> = processor.iter_player_ids_in_order().cloned().collect();
        for player_id in &player_ids {
            if let Ok(rigid_body) =
                processor.get_interpolated_player_rigid_body(player_id, current_time, 0.0)
            {
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
    for zero_slot in 0..4 {
        for order in orders {
            for sign in signs {
                modes.push(QuaternionMode {
                    zero_slot,
                    order,
                    signs: sign,
                });
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
        if slot == mode.zero_slot {
            continue;
        }
        *component = values[value_index];
        value_index += 1;
    }
    let quaternion =
        glam::Quat::from_xyzw(components[0], components[1], components[2], components[3]);
    (quaternion.length_squared() > f32::EPSILON).then(|| quaternion.normalize())
}

fn median(values: &mut [f32]) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    Some(values[values.len() / 2])
}

fn positive_fraction(values: &[f32]) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    Some(values.iter().filter(|value| **value > 0.0).count() as f32 / values.len() as f32)
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "assets/rlcs.replay".to_string());
    let data =
        std::fs::read(&path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"));
    let replay = boxcars::ParserBuilder::new(&data[..])
        .must_parse_network_data()
        .on_error_check_crc()
        .parse()
        .unwrap_or_else(|error| panic!("failed to parse {path}: {error}"));

    println!(
        "replay={path} major_version={} minor_version={} net_version={:?}",
        replay.major_version, replay.minor_version, replay.net_version
    );

    let mut probe = LegacyRotationProbe::new()
        .process_replay(&replay)
        .unwrap_or_else(|error| panic!("failed to process {path}: {error:?}"));
    probe.print_summary();
}
