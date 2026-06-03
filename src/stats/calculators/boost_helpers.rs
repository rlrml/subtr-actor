use super::*;

pub(super) fn boost_transaction_label(kind: &'static str) -> StatLabel {
    StatLabel::new("transaction", kind)
}

pub(super) fn boost_pad_size_label(pad_size: Option<BoostPadSize>) -> StatLabel {
    match pad_size {
        Some(BoostPadSize::Big) => StatLabel::new("pad_size", "big"),
        Some(BoostPadSize::Small) => StatLabel::new("pad_size", "small"),
        None => StatLabel::new("pad_size", "unknown"),
    }
}

pub(super) fn boost_activity_label(activity: BoostPickupActivity) -> StatLabel {
    match activity {
        BoostPickupActivity::Active => StatLabel::new("activity", "active"),
        BoostPickupActivity::Inactive => StatLabel::new("activity", "inactive"),
        BoostPickupActivity::Unknown => StatLabel::new("activity", "unknown"),
    }
}

pub(super) fn boost_field_half_label(field_half: BoostPickupFieldHalf) -> StatLabel {
    match field_half {
        BoostPickupFieldHalf::Own => StatLabel::new("field_half", "own"),
        BoostPickupFieldHalf::Opponent => StatLabel::new("field_half", "opponent"),
        BoostPickupFieldHalf::Unknown => StatLabel::new("field_half", "unknown"),
    }
}

pub(super) fn boost_supersonic_label(supersonic: bool) -> StatLabel {
    if supersonic {
        StatLabel::new("supersonic", "true")
    } else {
        StatLabel::new("supersonic", "false")
    }
}

impl BoostCalculator {
    pub(super) fn estimated_pad_position(&self, pad_id: &str) -> Option<glam::Vec3> {
        self.observed_pad_positions
            .get(pad_id)
            .and_then(PadPositionEstimate::mean)
    }

    pub(super) fn observed_pad_positions(&self, pad_id: &str) -> &[glam::Vec3] {
        self.observed_pad_positions
            .get(pad_id)
            .map(PadPositionEstimate::observations)
            .unwrap_or(&[])
    }

    pub(super) fn pad_match_radius(pad_size: BoostPadSize) -> f32 {
        match pad_size {
            BoostPadSize::Big => STANDARD_PAD_MATCH_RADIUS_BIG,
            BoostPadSize::Small => STANDARD_PAD_MATCH_RADIUS_SMALL,
        }
    }

    pub fn resolved_boost_pads(&self) -> Vec<ResolvedBoostPad> {
        standard_soccar_boost_pad_layout()
            .iter()
            .enumerate()
            .map(|(index, (position, size))| ResolvedBoostPad {
                index,
                pad_id: self
                    .known_pad_indices
                    .iter()
                    .find_map(|(pad_id, pad_index)| (*pad_index == index).then(|| pad_id.clone())),
                size: *size,
                position: glam_to_vec(position),
            })
            .collect()
    }

    pub(super) fn infer_pad_index(
        &self,
        pad_id: &str,
        pad_size: BoostPadSize,
        observed_position: glam::Vec3,
    ) -> Option<usize> {
        if let Some(index) = self.known_pad_indices.get(pad_id).copied() {
            return Some(index);
        }

        let observed_position = self
            .estimated_pad_position(pad_id)
            .unwrap_or(observed_position);
        let layout = &*STANDARD_SOCCAR_BOOST_PAD_LAYOUT;
        let used_indices: HashSet<usize> = self.known_pad_indices.values().copied().collect();
        let radius = Self::pad_match_radius(pad_size);
        let observed_positions = self.observed_pad_positions(pad_id);
        let best_candidate = |allow_used: bool| {
            layout
                .iter()
                .enumerate()
                .filter(|(index, (_, size))| {
                    *size == pad_size && (allow_used || !used_indices.contains(index))
                })
                .filter_map(|(index, (candidate_position, _))| {
                    let mut vote_count = 0usize;
                    let mut total_vote_distance = 0.0f32;
                    let mut best_vote_distance = f32::INFINITY;

                    for position in observed_positions {
                        let distance = position.distance(*candidate_position);
                        if distance <= radius {
                            vote_count += 1;
                            total_vote_distance += distance;
                            best_vote_distance = best_vote_distance.min(distance);
                        }
                    }

                    if vote_count == 0 {
                        return None;
                    }

                    let representative_distance = observed_position.distance(*candidate_position);
                    Some((
                        index,
                        vote_count,
                        total_vote_distance / vote_count as f32,
                        best_vote_distance,
                        representative_distance,
                    ))
                })
                .max_by(|left, right| {
                    left.1
                        .cmp(&right.1)
                        .then_with(|| right.2.partial_cmp(&left.2).unwrap())
                        .then_with(|| right.3.partial_cmp(&left.3).unwrap())
                        .then_with(|| right.4.partial_cmp(&left.4).unwrap())
                })
                .map(|(index, _, _, _, _)| index)
        };

        best_candidate(false)
            .or_else(|| best_candidate(true))
            .or_else(|| {
                layout
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
            })
            .or_else(|| {
                layout
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
            })
            .filter(|index| {
                observed_position.distance(standard_soccar_boost_pad_position(*index)) <= radius
            })
    }

    pub(super) fn infer_pad_details_from_position(
        &self,
        pad_id: &str,
        observed_position: glam::Vec3,
    ) -> Option<(usize, BoostPadSize)> {
        if let Some(index) = self.known_pad_indices.get(pad_id).copied() {
            let (_, size) = standard_soccar_boost_pad_layout().get(index)?;
            return Some((index, *size));
        }

        let observed_position = self
            .estimated_pad_position(pad_id)
            .unwrap_or(observed_position);
        let layout = &*STANDARD_SOCCAR_BOOST_PAD_LAYOUT;
        let used_indices: HashSet<usize> = self.known_pad_indices.values().copied().collect();
        let observed_positions = self.observed_pad_positions(pad_id);
        let best_candidate = |allow_used: bool| {
            layout
                .iter()
                .enumerate()
                .filter(|(index, _)| allow_used || !used_indices.contains(index))
                .filter_map(|(index, (candidate_position, size))| {
                    let radius = Self::pad_match_radius(*size);
                    let mut vote_count = 0usize;
                    let mut total_vote_distance = 0.0f32;
                    let mut best_vote_distance = f32::INFINITY;

                    for position in observed_positions {
                        let distance = position.distance(*candidate_position);
                        if distance <= radius {
                            vote_count += 1;
                            total_vote_distance += distance;
                            best_vote_distance = best_vote_distance.min(distance);
                        }
                    }

                    if vote_count == 0 {
                        return None;
                    }

                    let representative_distance = observed_position.distance(*candidate_position);
                    Some((
                        index,
                        *size,
                        vote_count,
                        total_vote_distance / vote_count as f32,
                        best_vote_distance,
                        representative_distance,
                    ))
                })
                .max_by(|left, right| {
                    left.2
                        .cmp(&right.2)
                        .then_with(|| right.3.partial_cmp(&left.3).unwrap())
                        .then_with(|| right.4.partial_cmp(&left.4).unwrap())
                        .then_with(|| right.5.partial_cmp(&left.5).unwrap())
                })
                .map(|(index, size, _, _, _, _)| (index, size))
        };

        best_candidate(false).or_else(|| best_candidate(true))
    }

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
        let observed_position = self
            .estimated_pad_position(pad_id)
            .unwrap_or(pending_pickup.player_position);
        let pad_position = self
            .infer_pad_index(pad_id, pad_size, observed_position)
            .map(|index| {
                self.known_pad_indices.insert(pad_id.to_string(), index);
                standard_soccar_boost_pad_position(index)
            })
            .unwrap_or(observed_position);
        let stolen = is_enemy_side(pending_pickup.is_team_0, pad_position);
        let stats = self
            .player_stats
            .entry(pending_pickup.player_id.clone())
            .or_default();
        let team_stats = if pending_pickup.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        let nominal_gain = match pad_size {
            BoostPadSize::Big => BOOST_MAX_AMOUNT,
            BoostPadSize::Small => SMALL_PAD_AMOUNT_RAW,
        };
        let collected_amount = (BOOST_MAX_AMOUNT - pending_pickup.previous_boost_amount)
            .min(nominal_gain)
            .max(pending_pickup.pre_applied_collected_amount);
        let collected_amount_delta = collected_amount - pending_pickup.pre_applied_collected_amount;
        let overfill = (nominal_gain - collected_amount).max(0.0);
        let field_half = if stolen {
            BoostPickupFieldHalf::Opponent
        } else {
            BoostPickupFieldHalf::Own
        };

        stats.amount_collected += collected_amount_delta;
        team_stats.amount_collected += collected_amount_delta;
        let collected_labels = [
            boost_transaction_label("collected"),
            boost_pad_size_label(Some(pad_size)),
            boost_activity_label(BoostPickupActivity::Active),
            boost_field_half_label(field_half),
        ];
        stats.add_labeled_amount(collected_labels.clone(), collected_amount_delta);
        team_stats.add_labeled_amount(collected_labels.clone(), collected_amount_delta);
        stats.increment_labeled_count(collected_labels.clone());
        team_stats.increment_labeled_count(collected_labels.clone());

        match pending_pickup.pre_applied_pad_size {
            Some(pre_applied_pad_size) if pre_applied_pad_size == pad_size => {
                Self::apply_collected_bucket_amount(stats, pad_size, collected_amount_delta);
                Self::apply_collected_bucket_amount(team_stats, pad_size, collected_amount_delta);
            }
            Some(pre_applied_pad_size) => {
                Self::apply_collected_bucket_amount(
                    stats,
                    pre_applied_pad_size,
                    -pending_pickup.pre_applied_collected_amount,
                );
                Self::apply_collected_bucket_amount(
                    team_stats,
                    pre_applied_pad_size,
                    -pending_pickup.pre_applied_collected_amount,
                );
                Self::apply_collected_bucket_amount(stats, pad_size, collected_amount);
                Self::apply_collected_bucket_amount(team_stats, pad_size, collected_amount);
            }
            None => {
                Self::apply_collected_bucket_amount(stats, pad_size, collected_amount);
                Self::apply_collected_bucket_amount(team_stats, pad_size, collected_amount);
            }
        }

        if stolen {
            stats.amount_stolen += collected_amount;
            team_stats.amount_stolen += collected_amount;
            let stolen_labels = [
                boost_transaction_label("stolen"),
                boost_pad_size_label(Some(pad_size)),
                boost_activity_label(BoostPickupActivity::Active),
                boost_field_half_label(field_half),
            ];
            stats.add_labeled_amount(stolen_labels.clone(), collected_amount);
            team_stats.add_labeled_amount(stolen_labels, collected_amount);
        }

        match pad_size {
            BoostPadSize::Big => {
                stats.big_pads_collected += 1;
                team_stats.big_pads_collected += 1;
                if stolen {
                    stats.big_pads_stolen += 1;
                    team_stats.big_pads_stolen += 1;
                    stats.amount_stolen_big += collected_amount;
                    team_stats.amount_stolen_big += collected_amount;
                }
            }
            BoostPadSize::Small => {
                stats.small_pads_collected += 1;
                team_stats.small_pads_collected += 1;
                if stolen {
                    stats.small_pads_stolen += 1;
                    team_stats.small_pads_stolen += 1;
                    stats.amount_stolen_small += collected_amount;
                    team_stats.amount_stolen_small += collected_amount;
                }
            }
        }

        stats.overfill_total += overfill;
        team_stats.overfill_total += overfill;
        let overfill_labels = [
            boost_transaction_label("overfill"),
            boost_pad_size_label(Some(pad_size)),
            boost_activity_label(BoostPickupActivity::Active),
            boost_field_half_label(field_half),
        ];
        stats.add_labeled_amount(overfill_labels.clone(), overfill);
        team_stats.add_labeled_amount(overfill_labels.clone(), overfill);
        if stolen {
            stats.overfill_from_stolen += overfill;
            team_stats.overfill_from_stolen += overfill;
        }

        self.record_ledger_event(BoostLedgerEvent {
            frame: pending_pickup.frame,
            time: pending_pickup.time,
            player_id: pending_pickup.player_id.clone(),
            player_position: Some(pending_pickup.player_position.to_array()),
            is_team_0: pending_pickup.is_team_0,
            transaction: BoostLedgerTransactionKind::Collected,
            amount: collected_amount_delta,
            count: 1,
            labels: collected_labels.into_iter().collect(),
            boost_before: pending_pickup.boost_before,
            boost_after: pending_pickup.boost_after,
        });
        if stolen {
            let stolen_labels = [
                boost_transaction_label("stolen"),
                boost_pad_size_label(Some(pad_size)),
                boost_activity_label(BoostPickupActivity::Active),
                boost_field_half_label(field_half),
            ];
            self.record_ledger_event(BoostLedgerEvent {
                frame: pending_pickup.frame,
                time: pending_pickup.time,
                player_id: pending_pickup.player_id.clone(),
                player_position: Some(pending_pickup.player_position.to_array()),
                is_team_0: pending_pickup.is_team_0,
                transaction: BoostLedgerTransactionKind::Stolen,
                amount: collected_amount,
                count: 1,
                labels: stolen_labels.into_iter().collect(),
                boost_before: pending_pickup.boost_before,
                boost_after: pending_pickup.boost_after,
            });
        }
        self.record_ledger_event(BoostLedgerEvent {
            frame: pending_pickup.frame,
            time: pending_pickup.time,
            player_id: pending_pickup.player_id.clone(),
            player_position: Some(pending_pickup.player_position.to_array()),
            is_team_0: pending_pickup.is_team_0,
            transaction: BoostLedgerTransactionKind::Overfill,
            amount: overfill,
            count: 0,
            labels: overfill_labels.into_iter().collect(),
            boost_before: pending_pickup.boost_before,
            boost_after: pending_pickup.boost_after,
        });

        field_half
    }

    pub(super) fn apply_collected_bucket_amount(
        stats: &mut BoostStats,
        pad_size: BoostPadSize,
        amount: f32,
    ) {
        if amount == 0.0 {
            return;
        }

        match pad_size {
            BoostPadSize::Big => stats.amount_collected_big += amount,
            BoostPadSize::Small => stats.amount_collected_small += amount,
        }
    }

    pub(super) fn apply_pickup_collected_amount(
        &mut self,
        ledger_context: BoostLedgerContext,
        player_id: &PlayerId,
        player_position: Option<[f32; 3]>,
        is_team_0: bool,
        amount: f32,
        pad_size: Option<BoostPadSize>,
    ) {
        if amount <= 0.0 {
            return;
        }

        let stats = self.player_stats.entry(player_id.clone()).or_default();
        let team_stats = if is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        stats.amount_collected += amount;
        team_stats.amount_collected += amount;
        let collected_labels = [
            boost_transaction_label("collected"),
            boost_pad_size_label(pad_size),
            boost_activity_label(BoostPickupActivity::Active),
            boost_field_half_label(BoostPickupFieldHalf::Unknown),
        ];
        stats.add_labeled_amount(collected_labels.clone(), amount);
        team_stats.add_labeled_amount(collected_labels.clone(), amount);
        stats.increment_labeled_count(collected_labels.clone());
        team_stats.increment_labeled_count(collected_labels.clone());
        if let Some(pad_size) = pad_size {
            Self::apply_collected_bucket_amount(stats, pad_size, amount);
            Self::apply_collected_bucket_amount(team_stats, pad_size, amount);
        }
        self.record_ledger_event(BoostLedgerEvent {
            frame: ledger_context.frame,
            time: ledger_context.time,
            player_id: player_id.clone(),
            player_position,
            is_team_0,
            transaction: BoostLedgerTransactionKind::Collected,
            amount,
            count: 0,
            labels: collected_labels.into_iter().collect(),
            boost_before: ledger_context.boost_before,
            boost_after: ledger_context.boost_after,
        });
    }

    pub(super) fn apply_inactive_pickup(
        &mut self,
        ledger_context: BoostLedgerContext,
        player_id: &PlayerId,
        player_position: Option<[f32; 3]>,
        is_team_0: bool,
        amount: f32,
        pad_size: BoostPadSize,
    ) {
        let stats = self.player_stats.entry(player_id.clone()).or_default();
        let team_stats = if is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        stats.amount_collected_inactive += amount;
        team_stats.amount_collected_inactive += amount;
        let collected_labels = [
            boost_transaction_label("collected"),
            boost_pad_size_label(Some(pad_size)),
            boost_activity_label(BoostPickupActivity::Inactive),
            boost_field_half_label(BoostPickupFieldHalf::Unknown),
        ];
        stats.add_labeled_amount(collected_labels.clone(), amount);
        team_stats.add_labeled_amount(collected_labels.clone(), amount);
        stats.increment_labeled_count(collected_labels.clone());
        team_stats.increment_labeled_count(collected_labels.clone());
        match pad_size {
            BoostPadSize::Big => {
                stats.big_pads_collected_inactive += 1;
                team_stats.big_pads_collected_inactive += 1;
            }
            BoostPadSize::Small => {
                stats.small_pads_collected_inactive += 1;
                team_stats.small_pads_collected_inactive += 1;
            }
        }
        self.record_ledger_event(BoostLedgerEvent {
            frame: ledger_context.frame,
            time: ledger_context.time,
            player_id: player_id.clone(),
            player_position,
            is_team_0,
            transaction: BoostLedgerTransactionKind::Collected,
            amount,
            count: 1,
            labels: collected_labels.into_iter().collect(),
            boost_before: ledger_context.boost_before,
            boost_after: ledger_context.boost_after,
        });
    }

    pub(super) fn apply_respawn_amount(
        &mut self,
        ledger_context: BoostLedgerContext,
        player_id: &PlayerId,
        player_position: Option<[f32; 3]>,
        is_team_0: bool,
        amount: f32,
    ) {
        if amount <= 0.0 {
            return;
        }

        let stats = self.player_stats.entry(player_id.clone()).or_default();
        let team_stats = if is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        stats.amount_respawned += amount;
        team_stats.amount_respawned += amount;
        let respawn_labels = [boost_transaction_label("respawn")];
        stats.add_labeled_amount(respawn_labels.clone(), amount);
        team_stats.add_labeled_amount(respawn_labels.clone(), amount);
        self.record_ledger_event(BoostLedgerEvent {
            frame: ledger_context.frame,
            time: ledger_context.time,
            player_id: player_id.clone(),
            player_position,
            is_team_0,
            transaction: BoostLedgerTransactionKind::Respawn,
            amount,
            count: 0,
            labels: respawn_labels.into_iter().collect(),
            boost_before: ledger_context.boost_before,
            boost_after: ledger_context.boost_after,
        });
    }

    pub(super) fn interval_fraction_in_boost_range(
        start_boost: f32,
        end_boost: f32,
        min_boost: f32,
        max_boost: f32,
    ) -> f32 {
        if (end_boost - start_boost).abs() <= f32::EPSILON {
            return ((start_boost >= min_boost) && (start_boost < max_boost)) as i32 as f32;
        }

        let t_at_min = (min_boost - start_boost) / (end_boost - start_boost);
        let t_at_max = (max_boost - start_boost) / (end_boost - start_boost);
        let interval_start = t_at_min.min(t_at_max).max(0.0);
        let interval_end = t_at_min.max(t_at_max).min(1.0);
        (interval_end - interval_start).max(0.0)
    }

    pub(super) fn pad_respawn_time_seconds(pad_size: BoostPadSize) -> f32 {
        match pad_size {
            BoostPadSize::Big => 10.0,
            BoostPadSize::Small => 4.0,
        }
    }

    pub(super) fn seen_pickup_sequence_is_recent(
        &self,
        pad_id: &str,
        sequence: u8,
        event_time: f32,
        player_position: Option<glam::Vec3>,
    ) -> bool {
        let Some(last_time) = self
            .seen_pickup_sequence_times
            .get(&(pad_id.to_string(), sequence))
            .copied()
        else {
            return false;
        };
        let Some(pad_size) = self.known_pad_sizes.get(pad_id).copied().or_else(|| {
            player_position.and_then(|position| self.guess_pad_size_from_position(pad_id, position))
        }) else {
            return false;
        };
        event_time - last_time < Self::pad_respawn_time_seconds(pad_size)
    }

    pub(super) fn unavailable_pad_is_recent(
        &self,
        pad_id: &str,
        event_time: f32,
        player_position: Option<glam::Vec3>,
    ) -> bool {
        if !self.unavailable_pads.contains(pad_id) {
            return false;
        }
        let Some(last_time) = self.last_pickup_times.get(pad_id).copied() else {
            return true;
        };
        let Some(pad_size) = self.known_pad_sizes.get(pad_id).copied().or_else(|| {
            player_position.and_then(|position| self.guess_pad_size_from_position(pad_id, position))
        }) else {
            return true;
        };
        event_time - last_time < Self::pad_respawn_time_seconds(pad_size)
    }

    pub(super) fn boost_levels_live(live_play: bool) -> bool {
        live_play
    }

    pub(super) fn tracks_boost_levels(boost_levels_live: bool) -> bool {
        boost_levels_live
    }

    pub(super) fn tracks_boost_pickups(gameplay: &GameplayState, live_play: bool) -> bool {
        live_play
            || (gameplay.ball_has_been_hit == Some(false) && !gameplay.kickoff_countdown_active())
    }

    pub(super) fn activity_label(active: bool) -> BoostPickupActivity {
        if active {
            BoostPickupActivity::Active
        } else {
            BoostPickupActivity::Inactive
        }
    }

    pub(super) fn field_half_from_position(
        is_team_0: bool,
        position: Option<glam::Vec3>,
    ) -> BoostPickupFieldHalf {
        match position {
            Some(position) if is_enemy_side(is_team_0, position) => BoostPickupFieldHalf::Opponent,
            Some(_) => BoostPickupFieldHalf::Own,
            None => BoostPickupFieldHalf::Unknown,
        }
    }

    pub(super) fn classify_boost_increase_reasons(
        previous_boost: f32,
        boost: f32,
        kickoff_phase_active: bool,
        demo_respawn_supported: bool,
    ) -> Vec<BoostIncreaseReason> {
        const TOLERANCE: f32 = 1.0;
        let delta = boost - previous_boost;
        if delta <= TOLERANCE {
            return vec![BoostIncreaseReason::Unknown];
        }

        let is_respawn_value = (boost - BOOST_KICKOFF_START_AMOUNT).abs() <= TOLERANCE;
        if demo_respawn_supported && is_respawn_value {
            return vec![BoostIncreaseReason::DemoRespawn];
        }
        if kickoff_phase_active && is_respawn_value {
            return vec![BoostIncreaseReason::KickoffRespawn];
        }
        if is_respawn_value {
            return vec![BoostIncreaseReason::Respawn];
        }

        let small_pad_floor = SMALL_PAD_AMOUNT_RAW - 3.0;
        let big_pad_floor = SMALL_PAD_AMOUNT_RAW + 5.0;
        if boost < BOOST_FULL_BAND_MIN_RAW && delta >= small_pad_floor {
            const SMALL_PICKUP_COUNT_TOLERANCE: f32 = 3.0;
            let inferred_small_pickups = ((delta - SMALL_PICKUP_COUNT_TOLERANCE)
                / SMALL_PAD_AMOUNT_RAW)
                .ceil()
                .max(1.0) as usize;
            return vec![BoostIncreaseReason::SmallPad; inferred_small_pickups];
        }

        if delta > big_pad_floor {
            return vec![BoostIncreaseReason::BigPad];
        }
        if boost >= BOOST_MAX_AMOUNT - TOLERANCE {
            return vec![BoostIncreaseReason::AmbiguousPad];
        }
        if delta >= small_pad_floor {
            return vec![BoostIncreaseReason::SmallPad];
        }
        vec![BoostIncreaseReason::Unknown]
    }

    pub(super) fn emit_pickup_comparison_event(
        &mut self,
        comparison: BoostPickupComparison,
        inferred: Option<PendingBoostPickupEvent>,
        reported: Option<PendingBoostPickupEvent>,
    ) {
        let reference = inferred.as_ref().or(reported.as_ref()).unwrap();
        let pad_type = reported
            .as_ref()
            .map(|event| event.pad_type)
            .or_else(|| inferred.as_ref().map(|event| event.pad_type))
            .unwrap_or(reference.pad_type);
        let field_half = reported
            .as_ref()
            .map(|event| event.field_half)
            .or_else(|| inferred.as_ref().map(|event| event.field_half))
            .unwrap_or(reference.field_half);
        let activity = reported
            .as_ref()
            .map(|event| event.activity)
            .or_else(|| inferred.as_ref().map(|event| event.activity))
            .unwrap_or(reference.activity);
        let event_frame = inferred
            .as_ref()
            .map(|event| event.frame)
            .or_else(|| reported.as_ref().map(|event| event.frame))
            .unwrap_or(reference.frame);
        let event_time = inferred
            .as_ref()
            .map(|event| event.time)
            .or_else(|| reported.as_ref().map(|event| event.time))
            .unwrap_or(reference.time);
        let comparison_event = BoostPickupComparisonEvent {
            comparison,
            frame: event_frame,
            time: event_time,
            player_id: reference.player_id.clone(),
            player_position: reference.player_position,
            is_team_0: reference.is_team_0,
            pad_type,
            field_half,
            activity,
            reported_frame: reported.as_ref().map(|event| event.frame),
            reported_time: reported.as_ref().map(|event| event.time),
            inferred_frame: inferred.as_ref().map(|event| event.frame),
            inferred_time: inferred.as_ref().map(|event| event.time),
            boost_before: inferred.as_ref().and_then(|event| event.boost_before),
            boost_after: inferred.as_ref().and_then(|event| event.boost_after),
        };
        self.pickup_comparison_events.push(comparison_event);
    }

    pub(super) fn matching_pending_pickup_index(
        pending: &VecDeque<PendingBoostPickupEvent>,
        event: &PendingBoostPickupEvent,
        pending_is_inferred: bool,
    ) -> Option<usize> {
        pending
            .iter()
            .enumerate()
            .filter(|(_, pending_event)| {
                pending_event.player_id == event.player_id
                    && if pending_is_inferred {
                        pending_event.pad_type.is_compatible_with(event.pad_type)
                    } else {
                        event.pad_type.is_compatible_with(pending_event.pad_type)
                    }
                    && pending_event.frame.abs_diff(event.frame) <= Self::PICKUP_MATCH_FRAME_WINDOW
            })
            .min_by_key(|(_, pending_event)| pending_event.frame.abs_diff(event.frame))
            .map(|(index, _)| index)
    }

    pub(super) fn record_inferred_pickup(&mut self, event: PendingBoostPickupEvent) {
        self.pending_inferred_pickups.push_back(event);
    }

    pub(super) fn record_reported_pickup(&mut self, event: PendingBoostPickupEvent) {
        if let Some(index) =
            Self::matching_pending_pickup_index(&self.pending_inferred_pickups, &event, true)
        {
            let inferred = self
                .pending_inferred_pickups
                .remove(index)
                .expect("matched inferred pickup index should exist");
            self.emit_pickup_comparison_event(
                BoostPickupComparison::Both,
                Some(inferred),
                Some(event),
            );
        } else {
            self.emit_pickup_comparison_event(BoostPickupComparison::Both, None, Some(event));
        }
    }

    pub(super) fn resolve_deferred_reported_pickups(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
    ) {
        const TOLERANCE: f32 = 1.0;

        let mut remaining_pickups = VecDeque::new();
        for mut deferred in std::mem::take(&mut self.pending_reported_pickups) {
            let player = players
                .players
                .iter()
                .find(|player| player.player_id == deferred.pending_pickup.player_id);
            let observed_boost_amount = player.and_then(|player| player.boost_amount);
            let previous_sample_boost_amount = self
                .previous_boost_amounts
                .get(&deferred.pending_pickup.player_id)
                .copied()
                .unwrap_or(deferred.pending_pickup.previous_boost_amount);
            let gain_is_visible = observed_boost_amount.is_some_and(|boost_amount| {
                boost_amount > previous_sample_boost_amount + TOLERANCE
            });
            let pickup_expired = deferred.pending_pickup.frame + Self::PICKUP_MATCH_FRAME_WINDOW
                < frame.frame_number;

            if !gain_is_visible && !pickup_expired {
                remaining_pickups.push_back(deferred);
                continue;
            }

            if gain_is_visible || pickup_expired {
                deferred.pending_pickup.frame = frame.frame_number;
                deferred.pending_pickup.time = frame.time;
                deferred.pending_pickup.boost_after = observed_boost_amount;
                if let Some(position) = player.and_then(|player| player.position()) {
                    deferred.pending_pickup.player_position = position;
                }
            }

            let field_half =
                self.resolve_pickup(&deferred.pad_id, deferred.pending_pickup, deferred.pad_size);
            deferred.reported_event.field_half = field_half;
            self.record_reported_pickup(deferred.reported_event);
        }
        self.pending_reported_pickups = remaining_pickups;
    }

    pub(super) fn flush_deferred_reported_pickups(&mut self) {
        while let Some(mut deferred) = self.pending_reported_pickups.pop_front() {
            let field_half =
                self.resolve_pickup(&deferred.pad_id, deferred.pending_pickup, deferred.pad_size);
            deferred.reported_event.field_half = field_half;
            self.record_reported_pickup(deferred.reported_event);
        }
    }

    pub(super) fn flush_stale_pickup_comparisons(&mut self, current_frame: usize) {
        while self
            .pending_inferred_pickups
            .front()
            .is_some_and(|event| event.frame + Self::PICKUP_MATCH_FRAME_WINDOW < current_frame)
        {
            self.pending_inferred_pickups.pop_front();
        }
    }

    pub fn finish_calculation(&mut self) -> SubtrActorResult<()> {
        self.flush_deferred_reported_pickups();
        self.pending_inferred_pickups.clear();
        Ok(())
    }

    pub(super) fn inactive_pickup_stats(
        &self,
        player: &PlayerSample,
        pad_id: &str,
        previous_boost_amount: f32,
        respawn_amount: f32,
    ) -> Option<(f32, BoostPadSize)> {
        let pad_size = self
            .known_pad_sizes
            .get(pad_id)
            .copied()
            .or_else(|| self.guess_pad_size_from_position(pad_id, player.position()?))?;
        let nominal_gain = match pad_size {
            BoostPadSize::Big => BOOST_MAX_AMOUNT,
            BoostPadSize::Small => SMALL_PAD_AMOUNT_RAW,
        };
        let capacity_limited_gain = (BOOST_MAX_AMOUNT - previous_boost_amount)
            .min(nominal_gain)
            .max(0.0);
        let observed_gain = player
            .boost_amount
            .map(|boost_amount| (boost_amount - previous_boost_amount - respawn_amount).max(0.0))
            .unwrap_or(0.0);
        if observed_gain <= 1.0 {
            return None;
        }
        Some((
            capacity_limited_gain.max(observed_gain).min(nominal_gain),
            pad_size,
        ))
    }
}
