use super::*;
use std::collections::HashSet;

impl BoostCalculator {
    pub(super) fn nearest_unused_pad_index(
        &self,
        pad_size: BoostPadSize,
        observed_position: glam::Vec3,
    ) -> Option<usize> {
        let used_indices: HashSet<usize> = self.known_pad_indices.values().copied().collect();
        STANDARD_SOCCAR_BOOST_PAD_LAYOUT
            .iter()
            .enumerate()
            .filter(|(index, (_, size))| *size == pad_size && !used_indices.contains(index))
            .min_by(|(_, (a, _)), (_, (b, _))| {
                observed_position
                    .distance_squared(*a)
                    .partial_cmp(&observed_position.distance_squared(*b))
                    .unwrap()
            })
            .map(|(index, _)| index)
    }

    pub(super) fn nearest_pad_index(
        &self,
        pad_size: BoostPadSize,
        observed_position: glam::Vec3,
    ) -> Option<usize> {
        STANDARD_SOCCAR_BOOST_PAD_LAYOUT
            .iter()
            .enumerate()
            .filter(|(_, (_, size))| *size == pad_size)
            .min_by(|(_, (a, _)), (_, (b, _))| {
                observed_position
                    .distance_squared(*a)
                    .partial_cmp(&observed_position.distance_squared(*b))
                    .unwrap()
            })
            .map(|(index, _)| index)
    }
}
