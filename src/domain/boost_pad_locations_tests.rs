use super::*;

#[test]
fn pad_position_estimate_ignores_nonfinite_samples() {
    let mut estimate = PadPositionEstimate::default();
    for invalid in [f32::NAN, f32::from_bits(0xffc0_0000), f32::INFINITY] {
        estimate.observe(glam::Vec3::splat(invalid));
    }
    assert_eq!(estimate.mean(), None);
    estimate.observe(glam::vec3(10.0, 20.0, 30.0));
    assert_eq!(estimate.mean(), Some(glam::vec3(10.0, 20.0, 30.0)));
}
