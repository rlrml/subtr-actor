use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
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
    pub amount_collected_inactive: f32,
    pub big_pads_collected_inactive: u32,
    pub small_pads_collected_inactive: u32,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoostIncreaseReason {
    KickoffRespawn,
    DemoRespawn,
    Respawn,
    BigPad,
    SmallPad,
    AmbiguousPad,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum BoostPickupPadType {
    Big,
    Small,
    Ambiguous,
}

impl BoostPickupPadType {
    fn is_compatible_with(self, reported: Self) -> bool {
        match self {
            Self::Ambiguous => matches!(reported, Self::Big | Self::Small),
            _ => self == reported,
        }
    }
}

impl From<BoostPadSize> for BoostPickupPadType {
    fn from(pad_size: BoostPadSize) -> Self {
        match pad_size {
            BoostPadSize::Big => Self::Big,
            BoostPadSize::Small => Self::Small,
        }
    }
}

impl TryFrom<BoostIncreaseReason> for BoostPickupPadType {
    type Error = ();

    fn try_from(reason: BoostIncreaseReason) -> Result<Self, Self::Error> {
        match reason {
            BoostIncreaseReason::BigPad => Ok(Self::Big),
            BoostIncreaseReason::SmallPad => Ok(Self::Small),
            BoostIncreaseReason::AmbiguousPad => Ok(Self::Ambiguous),
            BoostIncreaseReason::KickoffRespawn
            | BoostIncreaseReason::DemoRespawn
            | BoostIncreaseReason::Respawn
            | BoostIncreaseReason::Unknown => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum BoostPickupFieldHalf {
    Own,
    Opponent,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum BoostPickupActivity {
    Active,
    Inactive,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum BoostPickupComparison {
    Both,
    Ghost,
    Missed,
}

#[derive(Clone, Debug)]
struct PendingBoostPickupEvent {
    frame: usize,
    time: f32,
    player_id: PlayerId,
    is_team_0: bool,
    pad_type: BoostPickupPadType,
    field_half: BoostPickupFieldHalf,
    activity: BoostPickupActivity,
    boost_before: Option<f32>,
    boost_after: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BoostPickupComparisonEvent {
    pub comparison: BoostPickupComparison,
    pub frame: usize,
    pub time: f32,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player_id: PlayerId,
    pub is_team_0: bool,
    pub pad_type: BoostPickupPadType,
    pub field_half: BoostPickupFieldHalf,
    pub activity: BoostPickupActivity,
    pub reported_frame: Option<usize>,
    pub reported_time: Option<f32>,
    pub inferred_frame: Option<usize>,
    pub inferred_time: Option<f32>,
    pub boost_before: Option<f32>,
    pub boost_after: Option<f32>,
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

    pub fn amount_used_by_vertical_band(&self) -> f32 {
        self.amount_used_while_grounded + self.amount_used_while_airborne
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BoostCalculatorConfig {
    pub include_non_live_pickups: bool,
}

#[derive(Debug, Clone, Default)]
pub struct BoostCalculator {
    config: BoostCalculatorConfig,
    player_stats: HashMap<PlayerId, BoostStats>,
    team_zero_stats: BoostStats,
    team_one_stats: BoostStats,
    previous_boost_amounts: HashMap<PlayerId, f32>,
    previous_player_speeds: HashMap<PlayerId, f32>,
    observed_pad_positions: HashMap<String, PadPositionEstimate>,
    known_pad_sizes: HashMap<String, BoostPadSize>,
    known_pad_indices: HashMap<String, usize>,
    unavailable_pads: HashSet<String>,
    seen_pickup_sequence_times: HashMap<(String, u8), f32>,
    pickup_frames: HashMap<(String, PlayerId), usize>,
    inactive_pickup_frames: HashSet<(PlayerId, usize, BoostPadSize)>,
    last_pickup_times: HashMap<String, f32>,
    pending_inferred_pickups: VecDeque<PendingBoostPickupEvent>,
    pickup_comparison_events: Vec<BoostPickupComparisonEvent>,
    kickoff_phase_active_last_frame: bool,
    kickoff_respawn_awarded: HashSet<PlayerId>,
    initial_respawn_awarded: HashSet<PlayerId>,
    pending_demo_respawns: HashSet<PlayerId>,
    previous_boost_levels_live: Option<bool>,
    active_invariant_warnings: HashSet<BoostInvariantWarningKey>,
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

impl BoostCalculator {
    const PICKUP_MATCH_FRAME_WINDOW: usize = 3;

    pub fn new() -> Self {
        Self::with_config(BoostCalculatorConfig::default())
    }

    pub fn with_config(config: BoostCalculatorConfig) -> Self {
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

    pub fn pickup_comparison_events(&self) -> &[BoostPickupComparisonEvent] {
        &self.pickup_comparison_events
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

        if stolen {
            BoostPickupFieldHalf::Opponent
        } else {
            BoostPickupFieldHalf::Own
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

    fn apply_inactive_pickup(
        &mut self,
        player_id: &PlayerId,
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

    fn warn_for_sample_boost_invariants(&mut self, frame: &FrameInfo, players: &PlayerFrameState) {
        let team_zero_stats = self.team_zero_stats.clone();
        let team_one_stats = self.team_one_stats.clone();
        let player_scopes: Vec<(PlayerId, Option<f32>, BoostStats)> = players
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
            frame.frame_number,
            frame.time,
            &team_zero_stats,
            None,
        );
        self.warn_for_boost_invariant_violations(
            "team_one",
            frame.frame_number,
            frame.time,
            &team_one_stats,
            None,
        );
        for (player_id, observed_boost_amount, stats) in player_scopes {
            self.warn_for_boost_invariant_violations(
                &format!("player {player_id:?}"),
                frame.frame_number,
                frame.time,
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

    fn seen_pickup_sequence_is_recent(
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

    fn unavailable_pad_is_recent(
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

    fn boost_levels_live(live_play: bool) -> bool {
        live_play
    }

    fn tracks_boost_levels(boost_levels_live: bool) -> bool {
        boost_levels_live
    }

    fn tracks_boost_pickups(gameplay: &GameplayState, live_play: bool) -> bool {
        live_play
            || (gameplay.ball_has_been_hit == Some(false)
                && gameplay.game_state != Some(GAME_STATE_KICKOFF_COUNTDOWN)
                && gameplay.kickoff_countdown_time.is_none_or(|t| t <= 0))
    }

    fn activity_label(active: bool) -> BoostPickupActivity {
        if active {
            BoostPickupActivity::Active
        } else {
            BoostPickupActivity::Inactive
        }
    }

    fn field_half_from_position(
        is_team_0: bool,
        position: Option<glam::Vec3>,
    ) -> BoostPickupFieldHalf {
        match position {
            Some(position) if is_enemy_side(is_team_0, position) => BoostPickupFieldHalf::Opponent,
            Some(_) => BoostPickupFieldHalf::Own,
            None => BoostPickupFieldHalf::Unknown,
        }
    }

    fn classify_boost_increase_reasons(
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

    fn emit_pickup_comparison_event(
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

    fn matching_pending_pickup_index(
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

    fn record_inferred_pickup(&mut self, event: PendingBoostPickupEvent) {
        self.pending_inferred_pickups.push_back(event);
    }

    fn record_reported_pickup(&mut self, event: PendingBoostPickupEvent) {
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

    fn flush_stale_pickup_comparisons(&mut self, current_frame: usize) {
        while self
            .pending_inferred_pickups
            .front()
            .is_some_and(|event| event.frame + Self::PICKUP_MATCH_FRAME_WINDOW < current_frame)
        {
            self.pending_inferred_pickups.pop_front();
        }
    }

    pub fn finish_calculation(&mut self) -> SubtrActorResult<()> {
        self.pending_inferred_pickups.clear();
        Ok(())
    }

    fn inactive_pickup_stats(
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

    pub fn update_parts(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        vertical_state: &PlayerVerticalState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        let boost_levels_live = Self::boost_levels_live(live_play);
        let track_boost_levels = Self::tracks_boost_levels(boost_levels_live);
        let track_boost_pickups = Self::tracks_boost_pickups(gameplay, live_play);
        let boost_levels_resumed_this_sample =
            boost_levels_live && !self.previous_boost_levels_live.unwrap_or(false);
        let kickoff_phase_active = gameplay.game_state == Some(GAME_STATE_KICKOFF_COUNTDOWN)
            || gameplay.kickoff_countdown_time.is_some_and(|t| t > 0)
            || gameplay.ball_has_been_hit == Some(false);
        let kickoff_phase_started = kickoff_phase_active && !self.kickoff_phase_active_last_frame;
        if kickoff_phase_started {
            self.kickoff_respawn_awarded.clear();
        }
        for demo in &events.demo_events {
            self.pending_demo_respawns.insert(demo.victim.clone());
        }

        let mut current_boost_amounts = Vec::new();
        let mut pickup_counts_by_player = HashMap::<PlayerId, usize>::new();
        let mut respawn_amounts_by_player = HashMap::<PlayerId, f32>::new();

        for event in &events.boost_pad_events {
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

        for player in &players.players {
            let Some(boost_amount) = player.boost_amount else {
                continue;
            };
            let previous_sample_boost_amount =
                self.previous_boost_amounts.get(&player.player_id).copied();
            let previous_boost_amount = player
                .last_boost_amount
                .unwrap_or_else(|| previous_sample_boost_amount.unwrap_or(boost_amount));
            let previous_boost_amount = if boost_levels_resumed_this_sample {
                boost_amount
            } else {
                previous_boost_amount
            };
            let demo_respawn_supported = self.pending_demo_respawns.contains(&player.player_id)
                && player.rigid_body.is_some();
            if let Some(previous_sample_boost_amount) = previous_sample_boost_amount {
                let reasons = Self::classify_boost_increase_reasons(
                    previous_sample_boost_amount,
                    boost_amount,
                    kickoff_phase_active,
                    demo_respawn_supported,
                );
                for reason in reasons {
                    if let Ok(pad_type) = BoostPickupPadType::try_from(reason) {
                        self.record_inferred_pickup(PendingBoostPickupEvent {
                            frame: frame.frame_number,
                            time: frame.time,
                            player_id: player.player_id.clone(),
                            is_team_0: player.is_team_0,
                            pad_type,
                            field_half: Self::field_half_from_position(
                                player.is_team_0,
                                player.position(),
                            ),
                            activity: Self::activity_label(live_play),
                            boost_before: Some(previous_sample_boost_amount),
                            boost_after: Some(boost_amount),
                        });
                    }
                }
            }
            if track_boost_levels {
                let average_boost_amount = (previous_boost_amount + boost_amount) * 0.5;
                let time_zero_boost = frame.dt
                    * Self::interval_fraction_in_boost_range(
                        previous_boost_amount,
                        boost_amount,
                        0.0,
                        BOOST_ZERO_BAND_RAW,
                    );
                let time_hundred_boost = frame.dt
                    * Self::interval_fraction_in_boost_range(
                        previous_boost_amount,
                        boost_amount,
                        BOOST_FULL_BAND_MIN_RAW,
                        BOOST_MAX_AMOUNT + 1.0,
                    );
                let time_boost_0_25 = frame.dt
                    * Self::interval_fraction_in_boost_range(
                        previous_boost_amount,
                        boost_amount,
                        0.0,
                        boost_percent_to_amount(25.0),
                    );
                let time_boost_25_50 = frame.dt
                    * Self::interval_fraction_in_boost_range(
                        previous_boost_amount,
                        boost_amount,
                        boost_percent_to_amount(25.0),
                        boost_percent_to_amount(50.0),
                    );
                let time_boost_50_75 = frame.dt
                    * Self::interval_fraction_in_boost_range(
                        previous_boost_amount,
                        boost_amount,
                        boost_percent_to_amount(50.0),
                        boost_percent_to_amount(75.0),
                    );
                let time_boost_75_100 = frame.dt
                    * Self::interval_fraction_in_boost_range(
                        previous_boost_amount,
                        boost_amount,
                        boost_percent_to_amount(75.0),
                        BOOST_MAX_AMOUNT + 1.0,
                    );
                let stats = self
                    .player_stats
                    .entry(player.player_id.clone())
                    .or_default();
                let team_stats = if player.is_team_0 {
                    &mut self.team_zero_stats
                } else {
                    &mut self.team_one_stats
                };

                stats.tracked_time += frame.dt;
                stats.boost_integral += average_boost_amount * frame.dt;
                team_stats.tracked_time += frame.dt;
                team_stats.boost_integral += average_boost_amount * frame.dt;
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
            if demo_respawn_supported {
                respawn_amount += BOOST_KICKOFF_START_AMOUNT;
                self.pending_demo_respawns.remove(&player.player_id);
            }
            if respawn_amount > 0.0 {
                self.apply_respawn_amount(&player.player_id, player.is_team_0, respawn_amount);
            }
            respawn_amounts_by_player.insert(player.player_id.clone(), respawn_amount);

            current_boost_amounts.push((player.player_id.clone(), boost_amount));
        }

        for event in &events.boost_pad_events {
            match event.kind {
                BoostPadEventKind::PickedUp { sequence } => {
                    if !track_boost_pickups && !self.config.include_non_live_pickups {
                        let Some(player_id) = &event.player else {
                            continue;
                        };
                        let Some(player) = players
                            .players
                            .iter()
                            .find(|player| &player.player_id == player_id)
                        else {
                            continue;
                        };
                        let previous_boost_amount = self
                            .previous_boost_amounts
                            .get(player_id)
                            .copied()
                            .or(player.last_boost_amount)
                            .unwrap_or_else(|| player.boost_amount.unwrap_or(0.0));
                        let respawn_amount = respawn_amounts_by_player
                            .get(player_id)
                            .copied()
                            .unwrap_or(0.0);
                        let Some((collected_amount, pad_size)) = self.inactive_pickup_stats(
                            player,
                            &event.pad_id,
                            previous_boost_amount,
                            respawn_amount,
                        ) else {
                            continue;
                        };
                        if !self.inactive_pickup_frames.insert((
                            player_id.clone(),
                            event.frame,
                            pad_size,
                        )) {
                            continue;
                        }
                        self.apply_inactive_pickup(
                            player_id,
                            player.is_team_0,
                            collected_amount,
                            pad_size,
                        );
                        self.record_reported_pickup(PendingBoostPickupEvent {
                            frame: event.frame,
                            time: event.time,
                            player_id: player_id.clone(),
                            is_team_0: player.is_team_0,
                            pad_type: pad_size.into(),
                            field_half: Self::field_half_from_position(
                                player.is_team_0,
                                player.position(),
                            ),
                            activity: BoostPickupActivity::Inactive,
                            boost_before: None,
                            boost_after: None,
                        });
                        continue;
                    }
                    let Some(player_id) = &event.player else {
                        continue;
                    };
                    let Some(player) = players
                        .players
                        .iter()
                        .find(|player| &player.player_id == player_id)
                    else {
                        continue;
                    };
                    if self.unavailable_pad_is_recent(&event.pad_id, event.time, player.position())
                    {
                        continue;
                    }
                    let pickup_key = (event.pad_id.clone(), player_id.clone());
                    if self.pickup_frames.get(&pickup_key).copied() == Some(event.frame) {
                        continue;
                    }
                    self.pickup_frames.insert(pickup_key, event.frame);
                    if self.seen_pickup_sequence_is_recent(
                        &event.pad_id,
                        sequence,
                        event.time,
                        player.position(),
                    ) {
                        continue;
                    }
                    self.seen_pickup_sequence_times
                        .insert((event.pad_id.clone(), sequence), event.time);
                    self.unavailable_pads.insert(event.pad_id.clone());
                    self.last_pickup_times
                        .insert(event.pad_id.clone(), event.time);
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
                        let field_half =
                            self.resolve_pickup(&event.pad_id, pending_pickup, pad_size);
                        self.record_reported_pickup(PendingBoostPickupEvent {
                            frame: event.frame,
                            time: event.time,
                            player_id: player_id.clone(),
                            is_team_0: player.is_team_0,
                            pad_type: pad_size.into(),
                            field_half,
                            activity: Self::activity_label(track_boost_pickups),
                            boost_before: None,
                            boost_after: None,
                        });
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
        self.flush_stale_pickup_comparisons(frame.frame_number);

        let mut team_zero_used = self.team_zero_stats.amount_used;
        let mut team_one_used = self.team_one_stats.amount_used;
        for player in &players.players {
            let Some(boost_amount) = player.boost_amount else {
                continue;
            };
            let stats = self
                .player_stats
                .entry(player.player_id.clone())
                .or_default();
            let previous_amount_used = stats.amount_used;
            let amount_used_raw = (stats.amount_obtained() - boost_amount).max(0.0);
            let amount_used = amount_used_raw.max(stats.amount_used);
            if track_boost_levels {
                let split_amount = stats.amount_used_by_vertical_band();
                let amount_used_delta = (amount_used - split_amount).max(0.0);
                if amount_used_delta > 0.0 {
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
                    let used_while_supersonic = player.boost_active
                        && speed.unwrap_or(0.0) >= SUPERSONIC_SPEED_THRESHOLD
                        && previous_speed.unwrap_or(0.0) >= SUPERSONIC_SPEED_THRESHOLD;
                    let team_stats = if player.is_team_0 {
                        &mut self.team_zero_stats
                    } else {
                        &mut self.team_one_stats
                    };
                    if vertical_state.is_grounded(&player.player_id) {
                        stats.amount_used_while_grounded += amount_used_delta;
                        team_stats.amount_used_while_grounded += amount_used_delta;
                    } else {
                        stats.amount_used_while_airborne += amount_used_delta;
                        team_stats.amount_used_while_airborne += amount_used_delta;
                    }
                    if used_while_supersonic {
                        stats.amount_used_while_supersonic += amount_used_delta;
                        team_stats.amount_used_while_supersonic += amount_used_delta;
                    }
                }
            }
            stats.amount_used = amount_used;
            let amount_used_delta = amount_used - previous_amount_used;
            if amount_used_delta <= 0.0 {
                continue;
            }
            if player.is_team_0 {
                team_zero_used += amount_used_delta;
            } else {
                team_one_used += amount_used_delta;
            }
        }
        self.team_zero_stats.amount_used = team_zero_used;
        self.team_one_stats.amount_used = team_one_used;
        for (player_id, boost_amount) in current_boost_amounts {
            self.previous_boost_amounts.insert(player_id, boost_amount);
        }
        for player in &players.players {
            if let Some(speed) = player.speed() {
                self.previous_player_speeds
                    .insert(player.player_id.clone(), speed);
            }
        }
        self.warn_for_sample_boost_invariants(frame, players);
        self.kickoff_phase_active_last_frame = kickoff_phase_active;
        self.previous_boost_levels_live = Some(boost_levels_live);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_player(
        player_id: PlayerId,
        boost_amount: f32,
        last_boost_amount: f32,
        position: glam::Vec3,
    ) -> PlayerSample {
        PlayerSample {
            player_id,
            is_team_0: true,
            rigid_body: Some(boxcars::RigidBody {
                sleeping: false,
                location: glam_to_vec(&position),
                rotation: boxcars::Quaternion {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                },
                linear_velocity: None,
                angular_velocity: None,
            }),
            boost_amount: Some(boost_amount),
            last_boost_amount: Some(last_boost_amount),
            boost_active: false,
            dodge_active: false,
            powerslide_active: false,
            match_goals: None,
            match_assists: None,
            match_saves: None,
            match_shots: None,
            match_score: None,
        }
    }

    #[test]
    fn records_inactive_pickup_without_active_collection() {
        let mut calculator = BoostCalculator::new();
        let player_id = PlayerId::Steam(1);
        let (pad_position, _) = standard_soccar_boost_pad_layout()
            .iter()
            .find(|(_, size)| *size == BoostPadSize::Small)
            .copied()
            .expect("standard layout should include small pads");
        let player = test_player(
            player_id.clone(),
            BOOST_KICKOFF_START_AMOUNT + SMALL_PAD_AMOUNT_RAW,
            0.0,
            pad_position,
        );

        calculator
            .update_parts(
                &FrameInfo {
                    frame_number: 1,
                    time: 1.0,
                    dt: 1.0 / 30.0,
                    seconds_remaining: None,
                },
                &GameplayState {
                    game_state: Some(GAME_STATE_GOAL_SCORED_REPLAY),
                    ball_has_been_hit: Some(true),
                    ..GameplayState::default()
                },
                &PlayerFrameState {
                    players: vec![player],
                },
                &FrameEventsState {
                    boost_pad_events: vec![BoostPadEvent {
                        time: 1.0,
                        frame: 1,
                        pad_id: "inactive-small-pad".to_string(),
                        player: Some(player_id.clone()),
                        kind: BoostPadEventKind::PickedUp { sequence: 1 },
                    }],
                    ..FrameEventsState::default()
                },
                &PlayerVerticalState::default(),
                false,
            )
            .expect("inactive boost update should succeed");

        let player_stats = calculator
            .player_stats()
            .get(&player_id)
            .expect("player stats should be recorded");
        assert_eq!(player_stats.amount_collected, 0.0);
        assert_eq!(player_stats.small_pads_collected, 0);
        assert_eq!(player_stats.amount_collected_inactive, SMALL_PAD_AMOUNT_RAW);
        assert_eq!(player_stats.small_pads_collected_inactive, 1);
        assert_eq!(calculator.team_zero_stats().amount_collected, 0.0);
        assert_eq!(
            calculator.team_zero_stats().amount_collected_inactive,
            SMALL_PAD_AMOUNT_RAW
        );
        assert_eq!(
            calculator.team_zero_stats().small_pads_collected_inactive,
            1
        );
    }

    #[test]
    fn counts_reused_pickup_sequence_after_pad_respawn() {
        let mut calculator = BoostCalculator::new();
        let player_id = PlayerId::Steam(1);
        let (pad_position, _) = standard_soccar_boost_pad_layout()
            .iter()
            .find(|(_, size)| *size == BoostPadSize::Big)
            .copied()
            .expect("standard layout should include big pads");
        let pad_id = "reused-sequence-big-pad".to_string();
        let active_gameplay = GameplayState {
            ball_has_been_hit: Some(true),
            ..GameplayState::default()
        };

        calculator
            .update_parts(
                &FrameInfo {
                    frame_number: 1,
                    time: 1.0,
                    dt: 1.0 / 30.0,
                    seconds_remaining: None,
                },
                &active_gameplay,
                &PlayerFrameState {
                    players: vec![test_player(player_id.clone(), 100.0, 0.0, pad_position)],
                },
                &FrameEventsState {
                    boost_pad_events: vec![BoostPadEvent {
                        time: 1.0,
                        frame: 1,
                        pad_id: pad_id.clone(),
                        player: Some(player_id.clone()),
                        kind: BoostPadEventKind::PickedUp { sequence: 7 },
                    }],
                    ..FrameEventsState::default()
                },
                &PlayerVerticalState::default(),
                true,
            )
            .expect("first boost update should succeed");

        calculator
            .update_parts(
                &FrameInfo {
                    frame_number: 2,
                    time: 11.1,
                    dt: 1.0 / 30.0,
                    seconds_remaining: None,
                },
                &active_gameplay,
                &PlayerFrameState {
                    players: vec![test_player(player_id.clone(), 100.0, 100.0, pad_position)],
                },
                &FrameEventsState {
                    boost_pad_events: vec![BoostPadEvent {
                        time: 11.1,
                        frame: 2,
                        pad_id: pad_id.clone(),
                        player: None,
                        kind: BoostPadEventKind::Available,
                    }],
                    ..FrameEventsState::default()
                },
                &PlayerVerticalState::default(),
                true,
            )
            .expect("pad availability update should succeed");

        calculator
            .update_parts(
                &FrameInfo {
                    frame_number: 3,
                    time: 11.2,
                    dt: 1.0 / 30.0,
                    seconds_remaining: None,
                },
                &active_gameplay,
                &PlayerFrameState {
                    players: vec![test_player(player_id.clone(), 200.0, 100.0, pad_position)],
                },
                &FrameEventsState {
                    boost_pad_events: vec![BoostPadEvent {
                        time: 11.2,
                        frame: 3,
                        pad_id,
                        player: Some(player_id.clone()),
                        kind: BoostPadEventKind::PickedUp { sequence: 7 },
                    }],
                    ..FrameEventsState::default()
                },
                &PlayerVerticalState::default(),
                true,
            )
            .expect("second boost update should succeed");

        let player_stats = calculator
            .player_stats()
            .get(&player_id)
            .expect("player stats should be recorded");
        assert_eq!(player_stats.big_pads_collected, 2);
        assert_eq!(calculator.team_zero_stats().big_pads_collected, 2);
    }

    #[test]
    fn counts_pickup_after_respawn_without_available_event() {
        let mut calculator = BoostCalculator::new();
        let player_id = PlayerId::Steam(1);
        let (pad_position, _) = standard_soccar_boost_pad_layout()
            .iter()
            .find(|(_, size)| *size == BoostPadSize::Big)
            .copied()
            .expect("standard layout should include big pads");
        let pad_id = "missing-available-big-pad".to_string();
        let active_gameplay = GameplayState {
            ball_has_been_hit: Some(true),
            ..GameplayState::default()
        };

        for (frame_number, time, sequence, previous_boost, boost_amount) in
            [(1, 1.0, 7, 0.0, 100.0), (2, 11.2, 9, 100.0, 200.0)]
        {
            calculator
                .update_parts(
                    &FrameInfo {
                        frame_number,
                        time,
                        dt: 1.0 / 30.0,
                        seconds_remaining: None,
                    },
                    &active_gameplay,
                    &PlayerFrameState {
                        players: vec![test_player(
                            player_id.clone(),
                            boost_amount,
                            previous_boost,
                            pad_position,
                        )],
                    },
                    &FrameEventsState {
                        boost_pad_events: vec![BoostPadEvent {
                            time,
                            frame: frame_number,
                            pad_id: pad_id.clone(),
                            player: Some(player_id.clone()),
                            kind: BoostPadEventKind::PickedUp { sequence },
                        }],
                        ..FrameEventsState::default()
                    },
                    &PlayerVerticalState::default(),
                    true,
                )
                .expect("boost update should succeed");
        }

        let player_stats = calculator
            .player_stats()
            .get(&player_id)
            .expect("player stats should be recorded");
        assert_eq!(player_stats.big_pads_collected, 2);
        assert_eq!(calculator.team_zero_stats().big_pads_collected, 2);
    }

    #[test]
    fn skips_inactive_pickup_without_observed_boost_gain() {
        let mut calculator = BoostCalculator::new();
        let player_id = PlayerId::Steam(1);
        let (pad_position, _) = standard_soccar_boost_pad_layout()
            .iter()
            .find(|(_, size)| *size == BoostPadSize::Big)
            .copied()
            .expect("standard layout should include big pads");

        calculator
            .update_parts(
                &FrameInfo {
                    frame_number: 1,
                    time: 1.0,
                    dt: 1.0 / 30.0,
                    seconds_remaining: None,
                },
                &GameplayState::default(),
                &PlayerFrameState {
                    players: vec![test_player(player_id.clone(), 100.0, 100.0, pad_position)],
                },
                &FrameEventsState {
                    boost_pad_events: vec![BoostPadEvent {
                        time: 1.0,
                        frame: 1,
                        pad_id: "inactive-no-gain-big-pad".to_string(),
                        player: Some(player_id.clone()),
                        kind: BoostPadEventKind::PickedUp { sequence: 7 },
                    }],
                    ..FrameEventsState::default()
                },
                &PlayerVerticalState::default(),
                false,
            )
            .expect("boost update should succeed");

        let player_stats = calculator
            .player_stats()
            .get(&player_id)
            .expect("player stats should be recorded");
        assert_eq!(player_stats.big_pads_collected_inactive, 0);
        assert_eq!(player_stats.amount_collected_inactive, 0.0);
    }

    #[test]
    fn observed_boost_increases_do_not_emit_pickup_events_without_reported_pad_pickups() {
        let mut calculator = BoostCalculator::new();
        let small_player = PlayerId::Steam(1);
        let big_player = PlayerId::Steam(2);
        let ambiguous_player = PlayerId::Steam(3);
        let respawn_player = PlayerId::Steam(4);
        let two_small_player = PlayerId::Steam(5);
        let position = glam::Vec3::ZERO;
        let active_gameplay = GameplayState {
            ball_has_been_hit: Some(true),
            ..GameplayState::default()
        };

        calculator
            .update_parts(
                &FrameInfo {
                    frame_number: 1,
                    time: 1.0,
                    dt: 1.0 / 30.0,
                    seconds_remaining: None,
                },
                &active_gameplay,
                &PlayerFrameState {
                    players: vec![
                        test_player(small_player.clone(), 10.0, 10.0, position),
                        test_player(big_player.clone(), 10.0, 10.0, position),
                        test_player(ambiguous_player.clone(), 230.0, 230.0, position),
                        test_player(respawn_player.clone(), 0.0, 0.0, position),
                        test_player(
                            two_small_player.clone(),
                            BOOST_KICKOFF_START_AMOUNT,
                            BOOST_KICKOFF_START_AMOUNT,
                            position,
                        ),
                    ],
                },
                &FrameEventsState::default(),
                &PlayerVerticalState::default(),
                true,
            )
            .expect("first boost update should succeed");

        calculator
            .update_parts(
                &FrameInfo {
                    frame_number: 2,
                    time: 1.1,
                    dt: 1.0 / 30.0,
                    seconds_remaining: None,
                },
                &active_gameplay,
                &PlayerFrameState {
                    players: vec![
                        test_player(
                            small_player.clone(),
                            10.0 + SMALL_PAD_AMOUNT_RAW,
                            10.0,
                            position,
                        ),
                        test_player(big_player.clone(), BOOST_MAX_AMOUNT, 10.0, position),
                        test_player(ambiguous_player.clone(), BOOST_MAX_AMOUNT, 230.0, position),
                        test_player(
                            respawn_player.clone(),
                            BOOST_KICKOFF_START_AMOUNT,
                            0.0,
                            position,
                        ),
                        test_player(
                            two_small_player.clone(),
                            BOOST_KICKOFF_START_AMOUNT + 2.0 * SMALL_PAD_AMOUNT_RAW,
                            BOOST_KICKOFF_START_AMOUNT,
                            position,
                        ),
                    ],
                },
                &FrameEventsState::default(),
                &PlayerVerticalState::default(),
                true,
            )
            .expect("second boost update should succeed");

        calculator
            .finish_calculation()
            .expect("pending inferred pickups should be discarded");
        let events = calculator.pickup_comparison_events();
        assert!(events.is_empty());
        assert!(calculator.player_stats().get(&respawn_player).is_some());
    }

    #[test]
    fn reported_pickup_without_observed_boost_increase_is_emitted_as_counted_pickup() {
        let mut calculator = BoostCalculator::new();
        let player_id = PlayerId::Steam(1);
        let (pad_position, _) = standard_soccar_boost_pad_layout()
            .iter()
            .find(|(_, size)| *size == BoostPadSize::Small)
            .copied()
            .expect("standard layout should include small pads");
        let active_gameplay = GameplayState {
            ball_has_been_hit: Some(true),
            ..GameplayState::default()
        };

        calculator
            .update_parts(
                &FrameInfo {
                    frame_number: 1,
                    time: 1.0,
                    dt: 1.0 / 30.0,
                    seconds_remaining: None,
                },
                &active_gameplay,
                &PlayerFrameState {
                    players: vec![test_player(
                        player_id.clone(),
                        BOOST_MAX_AMOUNT,
                        BOOST_MAX_AMOUNT,
                        pad_position,
                    )],
                },
                &FrameEventsState::default(),
                &PlayerVerticalState::default(),
                true,
            )
            .expect("first boost update should succeed");

        calculator
            .update_parts(
                &FrameInfo {
                    frame_number: 2,
                    time: 1.1,
                    dt: 1.0 / 30.0,
                    seconds_remaining: None,
                },
                &active_gameplay,
                &PlayerFrameState {
                    players: vec![test_player(
                        player_id.clone(),
                        BOOST_MAX_AMOUNT,
                        BOOST_MAX_AMOUNT,
                        pad_position,
                    )],
                },
                &FrameEventsState {
                    boost_pad_events: vec![BoostPadEvent {
                        time: 1.1,
                        frame: 2,
                        pad_id: "full-boost-small-pad".to_string(),
                        player: Some(player_id.clone()),
                        kind: BoostPadEventKind::PickedUp { sequence: 1 },
                    }],
                    ..FrameEventsState::default()
                },
                &PlayerVerticalState::default(),
                true,
            )
            .expect("second boost update should succeed");

        calculator
            .finish_calculation()
            .expect("pickup comparisons should finish");

        let player_stats = calculator
            .player_stats()
            .get(&player_id)
            .expect("player stats should exist");
        assert_eq!(player_stats.small_pads_collected, 1);
        assert_eq!(player_stats.amount_collected_small, 0.0);
        assert_eq!(player_stats.overfill_total, SMALL_PAD_AMOUNT_RAW);

        let events = calculator.pickup_comparison_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].comparison, BoostPickupComparison::Both);
        assert_eq!(events[0].pad_type, BoostPickupPadType::Small);
        assert_eq!(events[0].reported_frame, Some(2));
        assert_eq!(events[0].inferred_frame, None);
    }

    #[test]
    fn matches_two_small_pickups_from_one_observed_boost_increase() {
        let mut calculator = BoostCalculator::new();
        let player_id = PlayerId::Steam(1);
        let (pad_position, _) = standard_soccar_boost_pad_layout()
            .iter()
            .find(|(_, size)| *size == BoostPadSize::Small)
            .copied()
            .expect("standard layout should include small pads");
        let active_gameplay = GameplayState {
            ball_has_been_hit: Some(true),
            ..GameplayState::default()
        };

        calculator
            .update_parts(
                &FrameInfo {
                    frame_number: 1,
                    time: 1.0,
                    dt: 1.0 / 30.0,
                    seconds_remaining: None,
                },
                &active_gameplay,
                &PlayerFrameState {
                    players: vec![test_player(
                        player_id.clone(),
                        BOOST_KICKOFF_START_AMOUNT,
                        BOOST_KICKOFF_START_AMOUNT,
                        pad_position,
                    )],
                },
                &FrameEventsState::default(),
                &PlayerVerticalState::default(),
                true,
            )
            .expect("first boost update should succeed");

        calculator
            .update_parts(
                &FrameInfo {
                    frame_number: 2,
                    time: 1.1,
                    dt: 1.0 / 30.0,
                    seconds_remaining: None,
                },
                &active_gameplay,
                &PlayerFrameState {
                    players: vec![test_player(
                        player_id.clone(),
                        BOOST_KICKOFF_START_AMOUNT + 2.0 * SMALL_PAD_AMOUNT_RAW,
                        BOOST_KICKOFF_START_AMOUNT,
                        pad_position,
                    )],
                },
                &FrameEventsState {
                    boost_pad_events: vec![
                        BoostPadEvent {
                            time: 1.1,
                            frame: 2,
                            pad_id: "small-pad-one".to_string(),
                            player: Some(player_id.clone()),
                            kind: BoostPadEventKind::PickedUp { sequence: 1 },
                        },
                        BoostPadEvent {
                            time: 1.1,
                            frame: 2,
                            pad_id: "small-pad-two".to_string(),
                            player: Some(player_id.clone()),
                            kind: BoostPadEventKind::PickedUp { sequence: 1 },
                        },
                    ],
                    ..FrameEventsState::default()
                },
                &PlayerVerticalState::default(),
                true,
            )
            .expect("second boost update should succeed");

        calculator
            .finish_calculation()
            .expect("pickup comparisons should flush");

        let events = calculator.pickup_comparison_events();
        assert_eq!(events.len(), 2);
        assert!(events.iter().all(|event| {
            event.comparison == BoostPickupComparison::Both
                && event.pad_type == BoostPickupPadType::Small
        }));
    }
}
