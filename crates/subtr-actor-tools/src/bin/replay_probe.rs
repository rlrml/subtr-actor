use std::collections::BTreeMap;

use clap::{Parser, ValueEnum};
mod replay_probe_rotation;
use replay_probe_rotation::LegacyRotationProbe;

use subtr_actor::{
    evaluate_replay_plausibility, Collector, PlayerFrame, ReplayDataCollector,
    StatsTimelineCollector,
};

const DEFAULT_REPLAY_PATH: &str =
    "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay";
const DEFAULT_DEMOLITION_REPLAY_PATH: &str =
    "assets/replay-format-2026-01-14-v868-32-net10-demolish-extended.replay";

#[derive(Clone, Copy, Debug, ValueEnum)]
#[value(rename_all = "kebab-case")]
enum ProbeCommand {
    Metadata,
    Plausibility,
    LegacyRotation,
    Demolition,
    VectorRanges,
    Mechanics,
}

impl ProbeCommand {
    fn default_path(self) -> &'static str {
        match self {
            Self::Demolition => DEFAULT_DEMOLITION_REPLAY_PATH,
            Self::Metadata
            | Self::Plausibility
            | Self::LegacyRotation
            | Self::VectorRanges
            | Self::Mechanics => DEFAULT_REPLAY_PATH,
        }
    }
}

#[derive(Debug, Parser)]
#[command(about = "Probe replay metadata, plausibility, rotation, demolition, and vector ranges.")]
struct Args {
    /// Probe to run.
    command: ProbeCommand,

    /// Replay path. Defaults to a built-in fixture for the selected probe.
    replay_path: Option<String>,
}

fn main() {
    let Args {
        command,
        replay_path,
    } = Args::parse();
    let path = replay_path.unwrap_or_else(|| command.default_path().to_string());

    match command {
        ProbeCommand::Metadata => print_metadata(&path),
        ProbeCommand::Plausibility => print_plausibility(&path),
        ProbeCommand::LegacyRotation => print_legacy_rotation(&path),
        ProbeCommand::Demolition => print_demolition(&path),
        ProbeCommand::VectorRanges => print_vector_ranges(&path),
        ProbeCommand::Mechanics => print_mechanics(&path),
    }
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

fn print_mechanics(path: &str) {
    let replay = parse_replay(path);
    let timeline = StatsTimelineCollector::new()
        .get_legacy_replay_stats_timeline(&replay)
        .unwrap_or_else(|error| panic!("failed to collect stats timeline for {path}: {error:?}"));
    for event in &timeline.events.flick {
        println!(
            "flick {}",
            serde_json::to_string(event).expect("flick event should serialize")
        );
    }
    for event in &timeline.events.dodge_reset {
        println!(
            "dodge_reset {}",
            serde_json::to_string(event).expect("dodge-reset event should serialize")
        );
    }
    for event in &timeline.events.mechanics {
        if event.kind == "flip_reset" || event.kind == "flick" {
            println!(
                "mechanic {}",
                serde_json::to_string(event).expect("mechanic event should serialize")
            );
        }
    }
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
