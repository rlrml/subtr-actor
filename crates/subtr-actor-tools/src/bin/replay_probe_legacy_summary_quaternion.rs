use super::legacy_summary::{
    compare_desc, print_alignment_summaries, sample_filtered, AlignmentSummary,
};
use super::legacy_types::LegacyRotationProbe;
use super::math::{median, positive_fraction};
use super::rotation_types::QuaternionMode;

pub(crate) fn print_quaternion_summary(probe: &mut LegacyRotationProbe) {
    let summaries = quaternion_summaries(probe);
    print_baseline_summary(&summaries);

    let mut alignment_ranked = sample_filtered(&summaries);
    alignment_ranked.sort_by(|left, right| {
        compare_desc(left.median_alignment, right.median_alignment)
            .then_with(|| compare_desc(left.positive_fraction, right.positive_fraction))
            .then_with(|| compare_desc(left.median_up_z, right.median_up_z))
    });

    let mut upright_ranked = alignment_ranked.clone();
    upright_ranked.sort_by(|left, right| {
        compare_desc(left.median_up_z, right.median_up_z)
            .then_with(|| compare_desc(left.median_alignment, right.median_alignment))
    });

    let mut combined_ranked = alignment_ranked.clone();
    combined_ranked.sort_by(|left, right| {
        compare_desc(left.combined_score(), right.combined_score())
            .then_with(|| compare_desc(left.median_alignment, right.median_alignment))
    });

    print_alignment_summaries(
        "Top quaternion reinterpretations by forward alignment:",
        &alignment_ranked,
        48,
        |mode| mode.label(),
    );
    print_alignment_summaries(
        "Top quaternion reinterpretations by grounded uprightness:",
        &upright_ranked,
        48,
        |mode| mode.label(),
    );
    print_alignment_summaries(
        "Top quaternion reinterpretations by combined min(alignment, up_z):",
        &combined_ranked,
        48,
        |mode| mode.label(),
    );
}

fn quaternion_summaries(probe: &mut LegacyRotationProbe) -> Vec<AlignmentSummary<QuaternionMode>> {
    probe
        .modes
        .iter()
        .filter_map(|mode| {
            let accumulator = probe.accumulators.get_mut(mode)?;
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

fn print_baseline_summary(summaries: &[AlignmentSummary<QuaternionMode>]) {
    let baseline_mode = QuaternionMode {
        missing_slot: 3,
        order: [0, 1, 2],
        signs: [1, 1, 1],
        reconstruct_missing: false,
    };
    let Some(summary) = summaries
        .iter()
        .find(|summary| summary.mode == baseline_mode)
    else {
        return;
    };
    println!(
        "Baseline mode: {:<48} samples={:<6} median_alignment={:>7.4} positive_fraction={:>7.4} median_up_z={:>7.4}",
        summary.mode.label(),
        summary.samples,
        summary.median_alignment,
        summary.positive_fraction,
        summary.median_up_z
    );
}
