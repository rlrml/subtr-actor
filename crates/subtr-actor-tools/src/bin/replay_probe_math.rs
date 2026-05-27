pub(crate) fn vec_length(vector: boxcars::Vector3f) -> f32 {
    glam::Vec3::new(vector.x, vector.y, vector.z).length()
}

pub(crate) fn median(values: &mut [f32]) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    Some(values[values.len() / 2])
}

pub(crate) fn percentile_sorted(values: &[f32], percentile: f32) -> f32 {
    if values.is_empty() {
        return f32::NAN;
    }
    let clamped = percentile.clamp(0.0, 1.0);
    let index = ((values.len() - 1) as f32 * clamped).round() as usize;
    values[index]
}

pub(crate) fn positive_fraction(values: &[f32]) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    Some(values.iter().filter(|value| **value > 0.0).count() as f32 / values.len() as f32)
}
