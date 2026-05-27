use super::legacy_summary::{compare_desc, MIN_ROTATION_MODE_SAMPLE_COUNT};
use super::legacy_types::LegacyRotationProbe;
use super::math::{median, positive_fraction};
use super::rotation_types::EulerMode;

#[derive(Clone, Copy)]
struct AngularSummary {
    mode: EulerMode,
    samples: usize,
    median_direction_dot: f32,
    positive_fraction: f32,
}

pub(crate) fn print_euler_angular_summary(probe: &mut LegacyRotationProbe) {
    println!();
    println!("Top Euler interpretations by angular-velocity direction:");

    let mut ranked = angular_summaries(probe);
    ranked.sort_by(|left, right| {
        compare_desc(left.median_direction_dot, right.median_direction_dot)
            .then_with(|| compare_desc(left.positive_fraction, right.positive_fraction))
    });

    for (rank, summary) in ranked.iter().take(12).enumerate() {
        println!(
            "{:>2}. {:<72} samples={:<6} median_direction_dot={:>7.4} positive_fraction={:>7.4}",
            rank + 1,
            summary.mode.label(),
            summary.samples,
            summary.median_direction_dot,
            summary.positive_fraction
        );
    }
}

fn angular_summaries(probe: &mut LegacyRotationProbe) -> Vec<AngularSummary> {
    probe
        .euler_modes
        .iter()
        .filter_map(|mode| {
            let accumulator = probe.euler_angular_accumulators.get_mut(mode)?;
            let median_direction_dot = median(&mut accumulator.direction_dots)?;
            let positive_fraction = positive_fraction(&accumulator.direction_dots)?;
            Some(AngularSummary {
                mode: *mode,
                samples: accumulator.direction_dots.len(),
                median_direction_dot,
                positive_fraction,
            })
        })
        .filter(|summary| summary.samples >= MIN_ROTATION_MODE_SAMPLE_COUNT)
        .collect()
}
