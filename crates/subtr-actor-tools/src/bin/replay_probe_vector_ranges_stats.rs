use super::super::{percentile_sorted, vec_length};

#[derive(Debug, Default)]
pub(super) struct VectorRangeStats {
    count: usize,
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    min_z: f32,
    max_z: f32,
    max_abs_axis: f32,
    magnitudes: Vec<f32>,
}

impl VectorRangeStats {
    pub(super) fn add(&mut self, vector: boxcars::Vector3f) {
        if !(vector.x.is_finite() && vector.y.is_finite() && vector.z.is_finite()) {
            return;
        }

        if self.count == 0 {
            self.min_x = vector.x;
            self.max_x = vector.x;
            self.min_y = vector.y;
            self.max_y = vector.y;
            self.min_z = vector.z;
            self.max_z = vector.z;
        } else {
            self.min_x = self.min_x.min(vector.x);
            self.max_x = self.max_x.max(vector.x);
            self.min_y = self.min_y.min(vector.y);
            self.max_y = self.max_y.max(vector.y);
            self.min_z = self.min_z.min(vector.z);
            self.max_z = self.max_z.max(vector.z);
        }
        self.count += 1;
        self.max_abs_axis = self
            .max_abs_axis
            .max(vector.x.abs())
            .max(vector.y.abs())
            .max(vector.z.abs());
        self.magnitudes.push(vec_length(vector));
    }

    pub(super) fn summary(&mut self) -> Option<VectorRangeSummary> {
        if self.count == 0 {
            return None;
        }
        self.magnitudes
            .sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
        Some(VectorRangeSummary {
            count: self.count,
            min_x: self.min_x,
            max_x: self.max_x,
            min_y: self.min_y,
            max_y: self.max_y,
            min_z: self.min_z,
            max_z: self.max_z,
            max_abs_axis: self.max_abs_axis,
            median_magnitude: percentile_sorted(&self.magnitudes, 0.5),
            p95_magnitude: percentile_sorted(&self.magnitudes, 0.95),
            max_magnitude: *self.magnitudes.last().unwrap_or(&f32::NAN),
        })
    }
}

#[derive(Debug)]
pub(super) struct VectorRangeSummary {
    pub(super) count: usize,
    pub(super) min_x: f32,
    pub(super) max_x: f32,
    pub(super) min_y: f32,
    pub(super) max_y: f32,
    pub(super) min_z: f32,
    pub(super) max_z: f32,
    pub(super) max_abs_axis: f32,
    pub(super) median_magnitude: f32,
    pub(super) p95_magnitude: f32,
    pub(super) max_magnitude: f32,
}
