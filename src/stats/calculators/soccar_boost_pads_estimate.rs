#[derive(Debug, Clone, Default)]
pub(crate) struct PadPositionEstimate {
    observations: Vec<glam::Vec3>,
}

impl PadPositionEstimate {
    pub(crate) fn observe(&mut self, position: glam::Vec3) {
        self.observations.push(position);
    }

    pub(crate) fn observations(&self) -> &[glam::Vec3] {
        self.observations.as_slice()
    }

    pub(crate) fn mean(&self) -> Option<glam::Vec3> {
        if self.observations.is_empty() {
            return None;
        }

        let sum = self
            .observations
            .iter()
            .copied()
            .fold(glam::Vec3::ZERO, |acc, position| acc + position);
        Some(sum / self.observations.len() as f32)
    }
}
