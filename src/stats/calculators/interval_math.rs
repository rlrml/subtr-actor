pub(crate) fn interval_fraction_in_scalar_range(
    start: f32,
    end: f32,
    min_value: f32,
    max_value: f32,
) -> f32 {
    if (end - start).abs() <= f32::EPSILON {
        return ((start >= min_value) && (start < max_value)) as i32 as f32;
    }

    let t_at_min = (min_value - start) / (end - start);
    let t_at_max = (max_value - start) / (end - start);
    let interval_start = t_at_min.min(t_at_max).max(0.0);
    let interval_end = t_at_min.max(t_at_max).min(1.0);
    (interval_end - interval_start).max(0.0)
}

pub(crate) fn interval_fraction_below_threshold(start: f32, end: f32, threshold: f32) -> f32 {
    if (end - start).abs() <= f32::EPSILON {
        return (start < threshold) as i32 as f32;
    }

    let threshold_time = ((threshold - start) / (end - start)).clamp(0.0, 1.0);
    if start < threshold {
        if end < threshold {
            1.0
        } else {
            threshold_time
        }
    } else if end < threshold {
        1.0 - threshold_time
    } else {
        0.0
    }
}

pub(crate) fn interval_fraction_above_threshold(start: f32, end: f32, threshold: f32) -> f32 {
    if (end - start).abs() <= f32::EPSILON {
        return (start > threshold) as i32 as f32;
    }

    let threshold_time = ((threshold - start) / (end - start)).clamp(0.0, 1.0);
    if start > threshold {
        if end > threshold {
            1.0
        } else {
            threshold_time
        }
    } else if end > threshold {
        1.0 - threshold_time
    } else {
        0.0
    }
}
