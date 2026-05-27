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
use math::{median, positive_fraction};
use replay::parse_replay;
use rotation_types::QuaternionMode;
use vector_ranges::print_vector_ranges;

const MIN_ROTATION_MODE_SAMPLE_COUNT: usize = 100;

impl LegacyRotationProbe {
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
