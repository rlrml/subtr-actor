use subtr_actor::{Collector, ProcessorView, TimeAdvance};

#[path = "replay_probe_args.rs"]
mod args;
#[path = "replay_probe_basic.rs"]
mod basic;
#[path = "replay_probe_constants.rs"]
mod constants;
#[path = "replay_probe_demolition.rs"]
mod demolition;
#[path = "replay_probe_legacy_sample.rs"]
mod legacy_sample;
#[path = "replay_probe_legacy_sample_alignment.rs"]
mod legacy_sample_alignment;
#[path = "replay_probe_legacy_summary.rs"]
mod legacy_summary;
#[path = "replay_probe_legacy_summary_angular.rs"]
mod legacy_summary_angular;
#[path = "replay_probe_legacy_summary_euler.rs"]
mod legacy_summary_euler;
#[path = "replay_probe_legacy_summary_quaternion.rs"]
mod legacy_summary_quaternion;
#[path = "replay_probe_legacy_summary_velocity.rs"]
mod legacy_summary_velocity;
#[path = "replay_probe_legacy_types.rs"]
mod legacy_types;
#[path = "replay_probe_math.rs"]
mod math;
#[path = "replay_probe_replay.rs"]
mod replay;
#[path = "replay_probe_rotation_interpret.rs"]
mod rotation_interpret;
#[path = "replay_probe_rotation_modes.rs"]
mod rotation_modes;
#[path = "replay_probe_rotation_types.rs"]
mod rotation_types;
#[path = "replay_probe_vector_ranges.rs"]
mod vector_ranges;

use args::{parse_args, ProbeCommand};
use basic::{print_mechanics, print_metadata, print_plausibility};
use demolition::print_demolition;
use legacy_types::LegacyRotationProbe;
use replay::parse_replay;
use vector_ranges::print_vector_ranges;

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

fn main() {
    let (command, path) = parse_args();

    match command {
        ProbeCommand::Metadata => print_metadata(&path),
        ProbeCommand::Plausibility => print_plausibility(&path),
        ProbeCommand::LegacyRotation => print_legacy_rotation(&path),
        ProbeCommand::Demolition => print_demolition(&path),
        ProbeCommand::VectorRanges => print_vector_ranges(&path),
        ProbeCommand::Mechanics => print_mechanics(&path),
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
