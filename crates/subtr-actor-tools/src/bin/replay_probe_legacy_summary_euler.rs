use super::legacy_summary::{
    compare_desc, print_alignment_summaries, sample_filtered, AlignmentSummary,
};
use super::legacy_types::LegacyRotationProbe;
use super::math::{median, positive_fraction};
use super::rotation_types::EulerMode;

pub(crate) fn print_euler_summary(probe: &mut LegacyRotationProbe) {
    let summaries = euler_summaries(probe);
    let mut ranked = sample_filtered(&summaries);
    ranked.sort_by(|left, right| {
        compare_desc(left.combined_score(), right.combined_score())
            .then_with(|| compare_desc(left.median_alignment, right.median_alignment))
    });
    print_alignment_summaries(
        "Top Euler interpretations by combined min(alignment, up_z):",
        &ranked,
        72,
        |mode| mode.label(),
    );
}

fn euler_summaries(probe: &mut LegacyRotationProbe) -> Vec<AlignmentSummary<EulerMode>> {
    probe
        .euler_modes
        .iter()
        .filter_map(|mode| {
            let accumulator = probe.euler_accumulators.get_mut(mode)?;
            let median_alignment = median(&mut accumulator.alignments)?;
            let positive_fraction = positive_fraction(&accumulator.alignments)?;
            let median_up_z = median(&mut accumulator.up_zs)?;
            Some(AlignmentSummary {
                mode: *mode,
                samples: accumulator.alignments.len(),
                median_alignment,
                positive_fraction,
                median_up_z,
            })
        })
        .collect()
}
