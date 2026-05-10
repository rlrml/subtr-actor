use std::collections::{BTreeMap, HashMap};

use subtr_actor::{evaluate_replay_plausibility, Collector, PlayerFrame, ReplayDataCollector};
use subtr_actor::{ReplayProcessor, TimeAdvance};

const DEFAULT_REPLAY_PATH: &str = "assets/rlcs.replay";
const DEFAULT_DEMOLITION_REPLAY_PATH: &str = "assets/new_demolition_format.replay";

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
struct LegacyRotationProbe {
    modes: Vec<QuaternionMode>,
    accumulators: HashMap<QuaternionMode, ModeAccumulator>,
    euler_modes: Vec<EulerMode>,
    euler_accumulators: HashMap<EulerMode, ModeAccumulator>,
    euler_angular_accumulators: HashMap<EulerMode, AngularVelocityAccumulator>,
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
        processor: &ReplayProcessor,
        _frame: &boxcars::Frame,
        _frame_number: usize,
        current_time: f32,
    ) -> subtr_actor::SubtrActorResult<TimeAdvance> {
        let player_ids: Vec<_> = processor.iter_player_ids_in_order().cloned().collect();
        for player_id in &player_ids {
            if let Ok(rigid_body) = processor.get_player_rigid_body(player_id) {
                let rigid_body = normalize_probe_rigid_body_vectors(processor, *rigid_body);
                if !rigid_body.sleeping {
                    self.sample_player(player_id, current_time, rigid_body);
                }
            }
        }
        Ok(TimeAdvance::NextFrame)
    }
}

#[derive(Clone, Copy, Debug)]
enum ProbeCommand {
    Metadata,
    Plausibility,
    LegacyRotation,
    Demolition,
    VectorRanges,
}

impl ProbeCommand {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "metadata" => Some(Self::Metadata),
            "plausibility" => Some(Self::Plausibility),
            "legacy-rotation" => Some(Self::LegacyRotation),
            "demolition" => Some(Self::Demolition),
            "vector-ranges" => Some(Self::VectorRanges),
            _ => None,
        }
    }

    fn default_path(self) -> &'static str {
        match self {
            Self::Demolition => DEFAULT_DEMOLITION_REPLAY_PATH,
            Self::Metadata | Self::Plausibility | Self::LegacyRotation | Self::VectorRanges => {
                DEFAULT_REPLAY_PATH
            }
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(command_text) = args.next() else {
        print_usage_and_exit();
    };
    let Some(command) = ProbeCommand::parse(&command_text) else {
        eprintln!("unknown subcommand: {command_text}");
        print_usage_and_exit();
    };
    let path = args
        .next()
        .unwrap_or_else(|| command.default_path().to_string());

    match command {
        ProbeCommand::Metadata => print_metadata(&path),
        ProbeCommand::Plausibility => print_plausibility(&path),
        ProbeCommand::LegacyRotation => print_legacy_rotation(&path),
        ProbeCommand::Demolition => print_demolition(&path),
        ProbeCommand::VectorRanges => print_vector_ranges(&path),
    }
}

fn print_usage_and_exit() -> ! {
    eprintln!(
        "usage: replay_probe <metadata|plausibility|legacy-rotation|demolition|vector-ranges> [replay_path]"
    );
    std::process::exit(2);
}

fn parse_replay(path: &str) -> boxcars::Replay {
    let data = std::fs::read(path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"));
    boxcars::ParserBuilder::new(&data[..])
        .must_parse_network_data()
        .on_error_check_crc()
        .parse()
        .unwrap_or_else(|error| panic!("failed to parse {path}: {error}"))
}

fn collect_replay_data(path: &str) -> subtr_actor::ReplayData {
    let replay = parse_replay(path);
    ReplayDataCollector::new()
        .get_replay_data(&replay)
        .unwrap_or_else(|error| panic!("failed to collect replay data for {path}: {error:?}"))
}

fn print_metadata(path: &str) {
    let replay = parse_replay(path);
    let build_version = replay
        .properties
        .iter()
        .find(|(key, _)| key == "BuildVersion")
        .and_then(|(_, value)| value.as_string());
    let num_frames = replay
        .properties
        .iter()
        .find(|(key, _)| key == "NumFrames")
        .and_then(|(_, value)| value.as_i32());
    let match_type = replay
        .properties
        .iter()
        .find(|(key, _)| key == "MatchType")
        .and_then(|(_, value)| value.as_string());

    println!(
        "replay={path} major_version={} minor_version={} net_version={:?} build_version={:?} match_type={:?} num_frames={:?}",
        replay.major_version,
        replay.minor_version,
        replay.net_version,
        build_version,
        match_type,
        num_frames
    );
}

fn print_plausibility(path: &str) {
    let replay_data = collect_replay_data(path);
    let report = evaluate_replay_plausibility(&replay_data);
    println!("{report:#?}");
}

fn print_legacy_rotation(path: &str) {
    let replay = parse_replay(path);
    println!(
        "replay={path} major_version={} minor_version={} net_version={:?}",
        replay.major_version, replay.minor_version, replay.net_version
    );
    let mut probe = LegacyRotationProbe::new()
        .process_replay(&replay)
        .unwrap_or_else(|error| panic!("failed to process {path}: {error:?}"));
    probe.print_summary();
}

fn print_demolition(path: &str) {
    let replay_data = collect_replay_data(path);
    let mut attacker_ratios = Vec::new();
    let mut victim_ratios = Vec::new();

    for demolish in &replay_data.demolish_infos {
        if let Some(player_data) = replay_data
            .frame_data
            .players
            .iter()
            .find(|(player_id, _)| player_id == &demolish.attacker)
            .map(|(_, player_data)| player_data)
        {
            if let Some(PlayerFrame::Data { rigid_body, .. }) =
                player_data.frames().get(demolish.frame)
            {
                if let Some(linear_velocity) = rigid_body.linear_velocity {
                    let demo_speed = vec_length(demolish.attacker_velocity);
                    let rigid_body_speed = vec_length(linear_velocity);
                    if demo_speed.is_finite()
                        && rigid_body_speed.is_finite()
                        && demo_speed > 0.0
                        && rigid_body_speed > 0.0
                    {
                        attacker_ratios.push(demo_speed / rigid_body_speed);
                    }
                }
            }
        }

        if let Some(player_data) = replay_data
            .frame_data
            .players
            .iter()
            .find(|(player_id, _)| player_id == &demolish.victim)
            .map(|(_, player_data)| player_data)
        {
            if let Some(PlayerFrame::Data { rigid_body, .. }) =
                player_data.frames().get(demolish.frame)
            {
                if let Some(linear_velocity) = rigid_body.linear_velocity {
                    let demo_speed = vec_length(demolish.victim_velocity);
                    let rigid_body_speed = vec_length(linear_velocity);
                    if demo_speed.is_finite()
                        && rigid_body_speed.is_finite()
                        && demo_speed > 0.0
                        && rigid_body_speed > 0.0
                    {
                        victim_ratios.push(demo_speed / rigid_body_speed);
                    }
                }
            }
        }
    }

    println!(
        "replay={path} demolishes={} attacker_ratio_median={:?} victim_ratio_median={:?}",
        replay_data.demolish_infos.len(),
        median(&mut attacker_ratios),
        median(&mut victim_ratios)
    );
}

#[derive(Debug, Default)]
struct VectorRangeStats {
    count: usize,
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    min_z: f32,
    max_z: f32,
    max_abs_axis: f32,
    magnitudes: Vec<f32>,
}

impl VectorRangeStats {
    fn add(&mut self, vector: boxcars::Vector3f) {
        if !(vector.x.is_finite() && vector.y.is_finite() && vector.z.is_finite()) {
            return;
        }

        if self.count == 0 {
            self.min_x = vector.x;
            self.max_x = vector.x;
            self.min_y = vector.y;
            self.max_y = vector.y;
            self.min_z = vector.z;
            self.max_z = vector.z;
        } else {
            self.min_x = self.min_x.min(vector.x);
            self.max_x = self.max_x.max(vector.x);
            self.min_y = self.min_y.min(vector.y);
            self.max_y = self.max_y.max(vector.y);
            self.min_z = self.min_z.min(vector.z);
            self.max_z = self.max_z.max(vector.z);
        }
        self.count += 1;
        self.max_abs_axis = self
            .max_abs_axis
            .max(vector.x.abs())
            .max(vector.y.abs())
            .max(vector.z.abs());
        self.magnitudes.push(vec_length(vector));
    }

    fn summary(&mut self) -> Option<VectorRangeSummary> {
        if self.count == 0 {
            return None;
        }
        self.magnitudes
            .sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
        Some(VectorRangeSummary {
            count: self.count,
            min_x: self.min_x,
            max_x: self.max_x,
            min_y: self.min_y,
            max_y: self.max_y,
            min_z: self.min_z,
            max_z: self.max_z,
            max_abs_axis: self.max_abs_axis,
            median_magnitude: percentile_sorted(&self.magnitudes, 0.5),
            p95_magnitude: percentile_sorted(&self.magnitudes, 0.95),
            max_magnitude: *self.magnitudes.last().unwrap_or(&f32::NAN),
        })
    }
}

#[derive(Debug)]
struct VectorRangeSummary {
    count: usize,
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    min_z: f32,
    max_z: f32,
    max_abs_axis: f32,
    median_magnitude: f32,
    p95_magnitude: f32,
    max_magnitude: f32,
}

fn print_vector_ranges(path: &str) {
    let replay = parse_replay(path);
    println!(
        "replay={path} major_version={} minor_version={} net_version={:?}",
        replay.major_version, replay.minor_version, replay.net_version
    );

    let mut ranges = BTreeMap::<&'static str, VectorRangeStats>::new();
    if let Some(network_frames) = &replay.network_frames {
        for frame in &network_frames.frames {
            for actor in &frame.new_actors {
                if let Some(location) = actor.initial_trajectory.location {
                    add_vector3i(
                        &mut ranges,
                        "NewActor.initial_trajectory.location",
                        location,
                    );
                }
            }

            for update in &frame.updated_actors {
                record_attribute_vectors(&mut ranges, &update.attribute);
            }
        }
    }

    println!(
        "{:<44} {:>8} {:>10} {:>10} {:>10} {:>10} {:>19} {:>19} {:>19}",
        "field",
        "count",
        "axis_max",
        "mag_p50",
        "mag_p95",
        "mag_max",
        "x_range",
        "y_range",
        "z_range"
    );
    for (field, stats) in &mut ranges {
        if let Some(summary) = stats.summary() {
            println!(
                "{:<44} {:>8} {:>10.2} {:>10.2} {:>10.2} {:>10.2} {:>9.2}..{:<9.2} {:>9.2}..{:<9.2} {:>9.2}..{:<9.2}",
                field,
                summary.count,
                summary.max_abs_axis,
                summary.median_magnitude,
                summary.p95_magnitude,
                summary.max_magnitude,
                summary.min_x,
                summary.max_x,
                summary.min_y,
                summary.max_y,
                summary.min_z,
                summary.max_z
            );
        }
    }
}

fn record_attribute_vectors(
    ranges: &mut BTreeMap<&'static str, VectorRangeStats>,
    attribute: &boxcars::Attribute,
) {
    match attribute {
        boxcars::Attribute::AppliedDamage(damage) => {
            add_vector(ranges, "AppliedDamage.position", damage.position);
        }
        boxcars::Attribute::DamageState(state) => {
            add_vector(ranges, "DamageState.ball_position", state.ball_position);
        }
        boxcars::Attribute::Demolish(demo) => {
            add_vector(ranges, "Demolish.attack_velocity", demo.attack_velocity);
            add_vector(ranges, "Demolish.victim_velocity", demo.victim_velocity);
        }
        boxcars::Attribute::DemolishExtended(demo) => {
            add_vector(
                ranges,
                "DemolishExtended.attacker_velocity",
                demo.attacker_velocity,
            );
            add_vector(
                ranges,
                "DemolishExtended.victim_velocity",
                demo.victim_velocity,
            );
        }
        boxcars::Attribute::DemolishFx(demo) => {
            add_vector(ranges, "DemolishFx.attack_velocity", demo.attack_velocity);
            add_vector(ranges, "DemolishFx.victim_velocity", demo.victim_velocity);
        }
        boxcars::Attribute::Explosion(explosion) => {
            add_vector(ranges, "Explosion.location", explosion.location);
        }
        boxcars::Attribute::ExtendedExplosion(explosion) => {
            add_vector(
                ranges,
                "ExtendedExplosion.explosion.location",
                explosion.explosion.location,
            );
        }
        boxcars::Attribute::Location(location) => {
            add_vector(ranges, "Attribute::Location", *location);
        }
        boxcars::Attribute::Welded(welded) => {
            add_vector(ranges, "Welded.offset", welded.offset);
        }
        boxcars::Attribute::RigidBody(rigid_body) => {
            add_vector(ranges, "RigidBody.location", rigid_body.location);
            if let Some(linear_velocity) = rigid_body.linear_velocity {
                add_vector(ranges, "RigidBody.linear_velocity", linear_velocity);
            }
            if let Some(angular_velocity) = rigid_body.angular_velocity {
                add_vector(ranges, "RigidBody.angular_velocity", angular_velocity);
            }
        }
        _ => {}
    }
}

fn add_vector(
    ranges: &mut BTreeMap<&'static str, VectorRangeStats>,
    field: &'static str,
    vector: boxcars::Vector3f,
) {
    ranges.entry(field).or_default().add(vector);
}

fn add_vector3i(
    ranges: &mut BTreeMap<&'static str, VectorRangeStats>,
    field: &'static str,
    vector: boxcars::Vector3i,
) {
    add_vector(
        ranges,
        field,
        boxcars::Vector3f {
            x: vector.x as f32,
            y: vector.y as f32,
            z: vector.z as f32,
        },
    );
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

fn normalize_probe_rigid_body_vectors(
    processor: &ReplayProcessor,
    rigid_body: boxcars::RigidBody,
) -> boxcars::RigidBody {
    boxcars::RigidBody {
        sleeping: rigid_body.sleeping,
        location: scale_vector(
            rigid_body.location,
            processor.spatial_normalization_factor(),
        ),
        rotation: rigid_body.rotation,
        linear_velocity: rigid_body.linear_velocity.map(|vector| {
            scale_vector(vector, processor.rigid_body_velocity_normalization_factor())
        }),
        angular_velocity: rigid_body.angular_velocity.map(|vector| {
            scale_vector(vector, processor.rigid_body_velocity_normalization_factor())
        }),
    }
}

fn scale_vector(vector: boxcars::Vector3f, factor: f32) -> boxcars::Vector3f {
    boxcars::Vector3f {
        x: vector.x * factor,
        y: vector.y * factor,
        z: vector.z * factor,
    }
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

fn vec_length(vector: boxcars::Vector3f) -> f32 {
    glam::Vec3::new(vector.x, vector.y, vector.z).length()
}

fn median(values: &mut [f32]) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    Some(values[values.len() / 2])
}

fn percentile_sorted(values: &[f32], percentile: f32) -> f32 {
    if values.is_empty() {
        return f32::NAN;
    }
    let clamped = percentile.clamp(0.0, 1.0);
    let index = ((values.len() - 1) as f32 * clamped).round() as usize;
    values[index]
}

fn positive_fraction(values: &[f32]) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    Some(values.iter().filter(|value| **value > 0.0).count() as f32 / values.len() as f32)
}
