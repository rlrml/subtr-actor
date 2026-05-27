use super::*;

impl BoostCalculator {
    pub(super) fn guess_pad_size_from_position(
        &self,
        pad_id: &str,
        observed_position: glam::Vec3,
    ) -> Option<BoostPadSize> {
        if let Some(pad_size) = self.known_pad_sizes.get(pad_id).copied() {
            return Some(pad_size);
        }

        if let Some((_, pad_size)) = self.infer_pad_details_from_position(pad_id, observed_position)
        {
            return Some(pad_size);
        }

        let observed_position = self
            .estimated_pad_position(pad_id)
            .unwrap_or(observed_position);
        standard_soccar_boost_pad_layout()
            .iter()
            .min_by(|(left_position, _), (right_position, _)| {
                observed_position
                    .distance_squared(*left_position)
                    .partial_cmp(&observed_position.distance_squared(*right_position))
                    .unwrap()
            })
            .map(|(_, pad_size)| *pad_size)
    }

    pub(super) fn resolve_pickup(
        &mut self,
        pad_id: &str,
        pending_pickup: PendingBoostPickup,
        pad_size: BoostPadSize,
    ) -> BoostPickupFieldHalf {
        let pad_position = self.resolved_pickup_pad_position(pad_id, &pending_pickup, pad_size);
        let stolen = is_enemy_side(pending_pickup.is_team_0, pad_position);
        let field_half = if stolen {
            BoostPickupFieldHalf::Opponent
        } else {
            BoostPickupFieldHalf::Own
        };
        let amounts = ResolvedPickupAmounts::new(&pending_pickup, pad_size);

        {
            let stats = self
                .player_stats
                .entry(pending_pickup.player_id.clone())
                .or_default();
            let team_stats = if pending_pickup.is_team_0 {
                &mut self.team_zero_stats
            } else {
                &mut self.team_one_stats
            };
            Self::apply_resolved_pickup_stats(
                stats,
                team_stats,
                &pending_pickup,
                pad_size,
                field_half,
                stolen,
                amounts,
            );
        }

        self.record_resolved_pickup_ledger_events(
            &pending_pickup,
            pad_size,
            field_half,
            stolen,
            amounts,
        );
        field_half
    }

    fn resolved_pickup_pad_position(
        &mut self,
        pad_id: &str,
        pending_pickup: &PendingBoostPickup,
        pad_size: BoostPadSize,
    ) -> glam::Vec3 {
        let observed_position = self
            .estimated_pad_position(pad_id)
            .unwrap_or(pending_pickup.player_position);
        self.infer_pad_index(pad_id, pad_size, observed_position)
            .map(|index| {
                self.known_pad_indices.insert(pad_id.to_string(), index);
                standard_soccar_boost_pad_position(index)
            })
            .unwrap_or(observed_position)
    }
}
