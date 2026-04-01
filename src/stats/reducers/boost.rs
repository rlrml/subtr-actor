use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BoostStats {
    pub tracked_time: f32,
    pub boost_integral: f32,
    pub time_zero_boost: f32,
    pub time_hundred_boost: f32,
    pub time_boost_0_25: f32,
    pub time_boost_25_50: f32,
    pub time_boost_50_75: f32,
    pub time_boost_75_100: f32,
    pub amount_collected: f32,
    pub amount_stolen: f32,
    pub big_pads_collected: u32,
    pub small_pads_collected: u32,
    pub big_pads_stolen: u32,
    pub small_pads_stolen: u32,
    pub amount_collected_big: f32,
    pub amount_stolen_big: f32,
    pub amount_collected_small: f32,
    pub amount_stolen_small: f32,
    pub amount_respawned: f32,
    pub overfill_total: f32,
    pub overfill_from_stolen: f32,
    pub amount_used: f32,
    pub amount_used_while_grounded: f32,
    pub amount_used_while_airborne: f32,
    pub amount_used_while_supersonic: f32,
}

impl BoostStats {
    pub fn average_boost_amount(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.boost_integral / self.tracked_time
        }
    }

    pub fn bpm(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.amount_collected * 60.0 / self.tracked_time
        }
    }

    fn pct(&self, value: f32) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            value * 100.0 / self.tracked_time
        }
    }

    pub fn zero_boost_pct(&self) -> f32 {
        self.pct(self.time_zero_boost)
    }

    pub fn hundred_boost_pct(&self) -> f32 {
        self.pct(self.time_hundred_boost)
    }

    pub fn boost_0_25_pct(&self) -> f32 {
        self.pct(self.time_boost_0_25)
    }

    pub fn boost_25_50_pct(&self) -> f32 {
        self.pct(self.time_boost_25_50)
    }

    pub fn boost_50_75_pct(&self) -> f32 {
        self.pct(self.time_boost_50_75)
    }

    pub fn boost_75_100_pct(&self) -> f32 {
        self.pct(self.time_boost_75_100)
    }

    pub fn amount_obtained(&self) -> f32 {
        self.amount_collected_big + self.amount_collected_small + self.amount_respawned
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BoostReducerConfig {
    pub include_non_live_pickups: bool,
}

#[derive(Debug, Clone, Default)]
pub struct BoostReducer {
    config: BoostReducerConfig,
    player_stats: HashMap<PlayerId, BoostStats>,
    team_zero_stats: BoostStats,
    team_one_stats: BoostStats,
    previous_boost_amounts: HashMap<PlayerId, f32>,
    previous_player_speeds: HashMap<PlayerId, f32>,
    observed_pad_positions: HashMap<String, PadPositionEstimate>,
    known_pad_sizes: HashMap<String, BoostPadSize>,
    known_pad_indices: HashMap<String, usize>,
    unavailable_pads: HashSet<String>,
    seen_pickup_sequences: HashSet<(String, u8)>,
    pickup_frames: HashMap<(String, PlayerId), usize>,
    last_pickup_times: HashMap<String, f32>,
    kickoff_phase_active_last_frame: bool,
    kickoff_respawn_awarded: HashSet<PlayerId>,
    initial_respawn_awarded: HashSet<PlayerId>,
    pending_demo_respawns: HashSet<PlayerId>,
    previous_boost_levels_live: Option<bool>,
    active_invariant_warnings: HashSet<BoostInvariantWarningKey>,
    live_play_tracker: LivePlayTracker,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BoostInvariantWarningKey {
    scope: String,
    kind: BoostInvariantKind,
}

#[derive(Debug, Clone)]
struct PendingBoostPickup {
    player_id: PlayerId,
    is_team_0: bool,
    previous_boost_amount: f32,
    pre_applied_collected_amount: f32,
    pre_applied_pad_size: Option<BoostPadSize>,
    player_position: glam::Vec3,
}

impl BoostReducer {
    pub fn new() -> Self {
        Self::with_config(BoostReducerConfig::default())
    }

    pub fn with_config(config: BoostReducerConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, BoostStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &BoostStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &BoostStats {
        &self.team_one_stats
    }

    fn estimated_pad_position(&self, pad_id: &str) -> Option<glam::Vec3> {
        self.observed_pad_positions
            .get(pad_id)
            .and_then(PadPositionEstimate::mean)
    }

    fn observed_pad_positions(&self, pad_id: &str) -> &[glam::Vec3] {
        self.observed_pad_positions
            .get(pad_id)
            .map(PadPositionEstimate::observations)
            .unwrap_or(&[])
    }

    fn pad_match_radius(pad_size: BoostPadSize) -> f32 {
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

    fn infer_pad_index(
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

    fn infer_pad_details_from_position(
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

    fn guess_pad_size_from_position(
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

    fn resolve_pickup(
        &mut self,
        pad_id: &str,
        pending_pickup: PendingBoostPickup,
        pad_size: BoostPadSize,
    ) {
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

        stats.amount_collected += collected_amount_delta;
        team_stats.amount_collected += collected_amount_delta;

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
        if stolen {
            stats.overfill_from_stolen += overfill;
            team_stats.overfill_from_stolen += overfill;
        }
    }

    fn apply_collected_bucket_amount(stats: &mut BoostStats, pad_size: BoostPadSize, amount: f32) {
        if amount == 0.0 {
            return;
        }

        match pad_size {
            BoostPadSize::Big => stats.amount_collected_big += amount,
            BoostPadSize::Small => stats.amount_collected_small += amount,
        }
    }

    fn apply_pickup_collected_amount(
        &mut self,
        player_id: &PlayerId,
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
        if let Some(pad_size) = pad_size {
            Self::apply_collected_bucket_amount(stats, pad_size, amount);
            Self::apply_collected_bucket_amount(team_stats, pad_size, amount);
        }
    }

    fn apply_respawn_amount(&mut self, player_id: &PlayerId, is_team_0: bool, amount: f32) {
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
    }

    fn warn_for_boost_invariant_violations(
        &mut self,
        scope: &str,
        frame_number: usize,
        time: f32,
        stats: &BoostStats,
        observed_boost_amount: Option<f32>,
    ) {
        let violations = boost_invariant_violations(stats, observed_boost_amount);
        let active_kinds: HashSet<BoostInvariantKind> =
            violations.iter().map(|violation| violation.kind).collect();

        for violation in violations {
            let key = BoostInvariantWarningKey {
                scope: scope.to_string(),
                kind: violation.kind,
            };
            if self.active_invariant_warnings.insert(key) {
                log::warn!(
                    "Boost invariant violation for {} at frame {} (t={:.3}): {}",
                    scope,
                    frame_number,
                    time,
                    violation.message(),
                );
            }
        }

        for kind in BoostInvariantKind::ALL {
            if active_kinds.contains(&kind) {
                continue;
            }
            self.active_invariant_warnings
                .remove(&BoostInvariantWarningKey {
                    scope: scope.to_string(),
                    kind,
                });
        }
    }

    fn warn_for_sample_boost_invariants(&mut self, sample: &StatsSample) {
        let team_zero_stats = self.team_zero_stats.clone();
        let team_one_stats = self.team_one_stats.clone();
        let player_scopes: Vec<(PlayerId, Option<f32>, BoostStats)> = sample
            .players
            .iter()
            .map(|player| {
                (
                    player.player_id.clone(),
                    player.boost_amount,
                    self.player_stats
                        .get(&player.player_id)
                        .cloned()
                        .unwrap_or_default(),
                )
            })
            .collect();

        self.warn_for_boost_invariant_violations(
            "team_zero",
            sample.frame_number,
            sample.time,
            &team_zero_stats,
            None,
        );
        self.warn_for_boost_invariant_violations(
            "team_one",
            sample.frame_number,
            sample.time,
            &team_one_stats,
            None,
        );
        for (player_id, observed_boost_amount, stats) in player_scopes {
            self.warn_for_boost_invariant_violations(
                &format!("player {player_id:?}"),
                sample.frame_number,
                sample.time,
                &stats,
                observed_boost_amount,
            );
        }
    }

    fn interval_fraction_in_boost_range(
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

    fn pad_respawn_time_seconds(pad_size: BoostPadSize) -> f32 {
        match pad_size {
            BoostPadSize::Big => 10.0,
            BoostPadSize::Small => 4.0,
        }
    }

    fn boost_levels_live(_sample: &StatsSample, live_play: bool) -> bool {
        live_play
    }

    fn tracks_boost_levels(boost_levels_live: bool) -> bool {
        boost_levels_live
    }

    fn tracks_boost_pickups(sample: &StatsSample, live_play: bool) -> bool {
        live_play
            || (sample.ball_has_been_hit == Some(false)
                && sample.game_state != Some(GAME_STATE_KICKOFF_COUNTDOWN)
                && sample.kickoff_countdown_time.is_none_or(|t| t <= 0))
    }
}

impl StatsReducer for BoostReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        let live_play = self.live_play_tracker.is_live_play(sample);
        let boost_levels_live = Self::boost_levels_live(sample, live_play);
        let track_boost_levels = Self::tracks_boost_levels(boost_levels_live);
        let track_boost_pickups = Self::tracks_boost_pickups(sample, live_play);
        let boost_levels_resumed_this_sample =
            boost_levels_live && !self.previous_boost_levels_live.unwrap_or(false);
        let kickoff_phase_active = sample.game_state == Some(GAME_STATE_KICKOFF_COUNTDOWN)
            || sample.kickoff_countdown_time.is_some_and(|t| t > 0)
            || sample.ball_has_been_hit == Some(false);
        let kickoff_phase_started = kickoff_phase_active && !self.kickoff_phase_active_last_frame;
        if kickoff_phase_started {
            self.kickoff_respawn_awarded.clear();
        }
        for demo in &sample.demo_events {
            self.pending_demo_respawns.insert(demo.victim.clone());
        }

        let mut current_boost_amounts = Vec::new();
        let mut pickup_counts_by_player = HashMap::<PlayerId, usize>::new();
        let mut respawn_amounts_by_player = HashMap::<PlayerId, f32>::new();

        for event in &sample.boost_pad_events {
            let BoostPadEventKind::PickedUp { .. } = event.kind else {
                continue;
            };
            let Some(player_id) = &event.player else {
                continue;
            };
            *pickup_counts_by_player
                .entry(player_id.clone())
                .or_default() += 1;
        }

        for player in &sample.players {
            let Some(boost_amount) = player.boost_amount else {
                continue;
            };
            let previous_boost_amount = player.last_boost_amount.unwrap_or_else(|| {
                self.previous_boost_amounts
                    .get(&player.player_id)
                    .copied()
                    .unwrap_or(boost_amount)
            });
            let previous_boost_amount = if boost_levels_resumed_this_sample {
                boost_amount
            } else {
                previous_boost_amount
            };
            let speed = player.speed();
            let previous_speed = self
                .previous_player_speeds
                .get(&player.player_id)
                .copied()
                .or(speed);
            let previous_speed = if boost_levels_resumed_this_sample {
                speed
            } else {
                previous_speed
            };

            if track_boost_levels {
                let average_boost_amount = (previous_boost_amount + boost_amount) * 0.5;
                let time_zero_boost = sample.dt
                    * Self::interval_fraction_in_boost_range(
                        previous_boost_amount,
                        boost_amount,
                        0.0,
                        BOOST_ZERO_BAND_RAW,
                    );
                let time_hundred_boost = sample.dt
                    * Self::interval_fraction_in_boost_range(
                        previous_boost_amount,
                        boost_amount,
                        BOOST_FULL_BAND_MIN_RAW,
                        BOOST_MAX_AMOUNT + 1.0,
                    );
                let time_boost_0_25 = sample.dt
                    * Self::interval_fraction_in_boost_range(
                        previous_boost_amount,
                        boost_amount,
                        0.0,
                        boost_percent_to_amount(25.0),
                    );
                let time_boost_25_50 = sample.dt
                    * Self::interval_fraction_in_boost_range(
                        previous_boost_amount,
                        boost_amount,
                        boost_percent_to_amount(25.0),
                        boost_percent_to_amount(50.0),
                    );
                let time_boost_50_75 = sample.dt
                    * Self::interval_fraction_in_boost_range(
                        previous_boost_amount,
                        boost_amount,
                        boost_percent_to_amount(50.0),
                        boost_percent_to_amount(75.0),
                    );
                let time_boost_75_100 = sample.dt
                    * Self::interval_fraction_in_boost_range(
                        previous_boost_amount,
                        boost_amount,
                        boost_percent_to_amount(75.0),
                        BOOST_MAX_AMOUNT + 1.0,
                    );
                let supersonic_usage = if player.boost_active
                    && speed.unwrap_or(0.0) >= SUPERSONIC_SPEED_THRESHOLD
                    && previous_speed.unwrap_or(0.0) >= SUPERSONIC_SPEED_THRESHOLD
                {
                    (previous_boost_amount - boost_amount)
                        .max(0.0)
                        .min(BOOST_USED_RAW_UNITS_PER_SECOND * sample.dt)
                } else {
                    0.0
                };
                let observed_usage = (previous_boost_amount - boost_amount).max(0.0);
                let grounded_usage = if player
                    .position()
                    .is_some_and(|position| position.z <= GROUND_Z_THRESHOLD)
                {
                    observed_usage
                } else {
                    0.0
                };
                let airborne_usage = observed_usage - grounded_usage;

                let stats = self
                    .player_stats
                    .entry(player.player_id.clone())
                    .or_default();
                let team_stats = if player.is_team_0 {
                    &mut self.team_zero_stats
                } else {
                    &mut self.team_one_stats
                };

                stats.tracked_time += sample.dt;
                stats.boost_integral += average_boost_amount * sample.dt;
                team_stats.tracked_time += sample.dt;
                team_stats.boost_integral += average_boost_amount * sample.dt;
                stats.time_zero_boost += time_zero_boost;
                team_stats.time_zero_boost += time_zero_boost;
                stats.time_hundred_boost += time_hundred_boost;
                team_stats.time_hundred_boost += time_hundred_boost;
                stats.time_boost_0_25 += time_boost_0_25;
                team_stats.time_boost_0_25 += time_boost_0_25;
                stats.time_boost_25_50 += time_boost_25_50;
                team_stats.time_boost_25_50 += time_boost_25_50;
                stats.time_boost_50_75 += time_boost_50_75;
                team_stats.time_boost_50_75 += time_boost_50_75;
                stats.time_boost_75_100 += time_boost_75_100;
                team_stats.time_boost_75_100 += time_boost_75_100;
                stats.amount_used_while_grounded += grounded_usage;
                team_stats.amount_used_while_grounded += grounded_usage;
                stats.amount_used_while_airborne += airborne_usage;
                team_stats.amount_used_while_airborne += airborne_usage;
                stats.amount_used_while_supersonic += supersonic_usage;
                team_stats.amount_used_while_supersonic += supersonic_usage;
            }

            let mut respawn_amount = 0.0;
            // Grant initial kickoff respawn the first time we see each player.
            // This handles replays that start after the kickoff countdown has
            // already ended (game_state != 55 on the first frame).
            let first_seen_player = self
                .initial_respawn_awarded
                .insert(player.player_id.clone());
            if first_seen_player
                || (kickoff_phase_active
                    && !self.kickoff_respawn_awarded.contains(&player.player_id))
            {
                respawn_amount += BOOST_KICKOFF_START_AMOUNT;
                self.kickoff_respawn_awarded
                    .insert(player.player_id.clone());
            }
            if self.pending_demo_respawns.contains(&player.player_id) && player.rigid_body.is_some()
            {
                respawn_amount += BOOST_KICKOFF_START_AMOUNT;
                self.pending_demo_respawns.remove(&player.player_id);
            }
            if respawn_amount > 0.0 {
                self.apply_respawn_amount(&player.player_id, player.is_team_0, respawn_amount);
            }
            respawn_amounts_by_player.insert(player.player_id.clone(), respawn_amount);

            current_boost_amounts.push((player.player_id.clone(), boost_amount));
        }

        for event in &sample.boost_pad_events {
            match event.kind {
                BoostPadEventKind::PickedUp { sequence } => {
                    if !track_boost_pickups && !self.config.include_non_live_pickups {
                        continue;
                    }
                    if self.unavailable_pads.contains(&event.pad_id) {
                        continue;
                    }
                    let Some(player_id) = &event.player else {
                        continue;
                    };
                    let pickup_key = (event.pad_id.clone(), player_id.clone());
                    if self.pickup_frames.get(&pickup_key).copied() == Some(event.frame) {
                        continue;
                    }
                    self.pickup_frames.insert(pickup_key, event.frame);
                    if !self
                        .seen_pickup_sequences
                        .insert((event.pad_id.clone(), sequence))
                    {
                        continue;
                    }
                    self.unavailable_pads.insert(event.pad_id.clone());
                    self.last_pickup_times
                        .insert(event.pad_id.clone(), event.time);
                    let Some(player) = sample
                        .players
                        .iter()
                        .find(|player| &player.player_id == player_id)
                    else {
                        continue;
                    };
                    if let Some(position) = player.position() {
                        self.observed_pad_positions
                            .entry(event.pad_id.clone())
                            .or_default()
                            .observe(position);
                    }
                    let previous_boost_amount = player.last_boost_amount.unwrap_or_else(|| {
                        self.previous_boost_amounts
                            .get(player_id)
                            .copied()
                            .unwrap_or_else(|| player.boost_amount.unwrap_or(0.0))
                    });
                    let pre_applied_collected_amount =
                        if pickup_counts_by_player.get(player_id).copied() == Some(1) {
                            self.previous_boost_amounts
                                .get(player_id)
                                .copied()
                                .map(|previous_sample_boost_amount| {
                                    let respawn_amount = respawn_amounts_by_player
                                        .get(player_id)
                                        .copied()
                                        .unwrap_or(0.0);
                                    (player.boost_amount.unwrap_or(previous_boost_amount)
                                        - previous_sample_boost_amount
                                        - respawn_amount)
                                        .max(0.0)
                                })
                                .unwrap_or(0.0)
                        } else {
                            0.0
                        };
                    let pre_applied_pad_size = (pre_applied_collected_amount > 0.0)
                        .then(|| {
                            self.guess_pad_size_from_position(
                                &event.pad_id,
                                player.position().unwrap_or(glam::Vec3::ZERO),
                            )
                        })
                        .flatten();
                    self.apply_pickup_collected_amount(
                        player_id,
                        player.is_team_0,
                        pre_applied_collected_amount,
                        pre_applied_pad_size,
                    );
                    let pending_pickup = PendingBoostPickup {
                        player_id: player_id.clone(),
                        is_team_0: player.is_team_0,
                        previous_boost_amount,
                        pre_applied_collected_amount,
                        pre_applied_pad_size,
                        player_position: player.position().unwrap_or(glam::Vec3::ZERO),
                    };

                    let pad_size = self
                        .known_pad_sizes
                        .get(&event.pad_id)
                        .copied()
                        .or_else(|| {
                            let mut size = self.guess_pad_size_from_position(
                                &event.pad_id,
                                player.position().unwrap_or(glam::Vec3::ZERO),
                            )?;
                            // Sanity check: if the observed boost gain clearly
                            // exceeds what a small pad can provide, the pad must
                            // be big.  Use a margin to avoid float imprecision.
                            if size == BoostPadSize::Small
                                && pre_applied_collected_amount > SMALL_PAD_AMOUNT_RAW * 1.5
                            {
                                size = BoostPadSize::Big;
                            }
                            self.known_pad_sizes.insert(event.pad_id.clone(), size);
                            Some(size)
                        });
                    if let Some(pad_size) = pad_size {
                        self.resolve_pickup(&event.pad_id, pending_pickup, pad_size);
                    }
                }
                BoostPadEventKind::Available => {
                    if let Some(pad_size) = self.known_pad_sizes.get(&event.pad_id).copied() {
                        let Some(last_pickup_time) = self.last_pickup_times.get(&event.pad_id)
                        else {
                            continue;
                        };
                        if event.time - *last_pickup_time < Self::pad_respawn_time_seconds(pad_size)
                        {
                            continue;
                        }
                    }
                    self.unavailable_pads.remove(&event.pad_id);
                }
            }
        }

        for (player_id, boost_amount) in current_boost_amounts {
            self.previous_boost_amounts.insert(player_id, boost_amount);
        }
        for player in &sample.players {
            if let Some(speed) = player.speed() {
                self.previous_player_speeds
                    .insert(player.player_id.clone(), speed);
            }
        }
        let mut team_zero_used = 0.0;
        let mut team_one_used = 0.0;
        for player in &sample.players {
            let Some(boost_amount) = player.boost_amount else {
                continue;
            };
            let stats = self
                .player_stats
                .entry(player.player_id.clone())
                .or_default();
            stats.amount_used = (stats.amount_obtained() - boost_amount).max(0.0);
            if player.is_team_0 {
                team_zero_used += stats.amount_used;
            } else {
                team_one_used += stats.amount_used;
            }
        }
        self.team_zero_stats.amount_used = team_zero_used;
        self.team_one_stats.amount_used = team_one_used;
        self.warn_for_sample_boost_invariants(sample);
        self.kickoff_phase_active_last_frame = kickoff_phase_active;
        self.previous_boost_levels_live = Some(boost_levels_live);

        Ok(())
    }
}

#[cfg(test)]
#[path = "boost_test.rs"]
mod tests;
