use super::legacy_types::LegacyRotationProbe;
use super::math::median;

pub(crate) fn print_velocity_summary(probe: &mut LegacyRotationProbe) {
    println!();
    println!("Velocity scale hypotheses:");
    for (scale, accumulator) in &mut probe.velocity_accumulators {
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
