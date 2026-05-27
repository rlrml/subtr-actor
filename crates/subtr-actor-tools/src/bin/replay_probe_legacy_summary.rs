use std::cmp::Ordering;

use super::legacy_summary_angular::print_euler_angular_summary;
use super::legacy_summary_euler::print_euler_summary;
use super::legacy_summary_quaternion::print_quaternion_summary;
use super::legacy_summary_velocity::print_velocity_summary;
use super::legacy_types::LegacyRotationProbe;

pub(crate) const MIN_ROTATION_MODE_SAMPLE_COUNT: usize = 100;

#[derive(Clone, Copy)]
pub(crate) struct AlignmentSummary<T> {
    pub(crate) mode: T,
    pub(crate) samples: usize,
    pub(crate) median_alignment: f32,
    pub(crate) positive_fraction: f32,
    pub(crate) median_up_z: f32,
}

impl<T> AlignmentSummary<T> {
    pub(crate) fn combined_score(&self) -> f32 {
        self.median_alignment.min(self.median_up_z)
    }
}

impl LegacyRotationProbe {
    pub(super) fn print_summary(&mut self) {
        print_quaternion_summary(self);
        print_euler_summary(self);
        print_euler_angular_summary(self);
        print_velocity_summary(self);
    }
}

pub(crate) fn compare_desc(left: f32, right: f32) -> Ordering {
    right.partial_cmp(&left).unwrap_or(Ordering::Equal)
}

pub(crate) fn sample_filtered<T: Copy>(
    summaries: &[AlignmentSummary<T>],
) -> Vec<AlignmentSummary<T>> {
    summaries
        .iter()
        .copied()
        .filter(|summary| summary.samples >= MIN_ROTATION_MODE_SAMPLE_COUNT)
        .collect()
}

pub(crate) fn print_alignment_summaries<T: Copy>(
    title: &str,
    summaries: &[AlignmentSummary<T>],
    label_width: usize,
    label: impl Fn(T) -> String,
) {
    println!();
    println!("{title}");
    for (rank, summary) in summaries.iter().take(12).enumerate() {
        println!(
            "{:>2}. {:<width$} samples={:<6} median_alignment={:>7.4} positive_fraction={:>7.4} median_up_z={:>7.4}",
            rank + 1,
            label(summary.mode),
            summary.samples,
            summary.median_alignment,
            summary.positive_fraction,
            summary.median_up_z,
            width = label_width
        );
    }
}
