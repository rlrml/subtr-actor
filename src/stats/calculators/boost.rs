use super::*;
use crate::stats::accumulators::boost::{BoostStats, BoostStatsAccumulator};
use crate::stats::timeline::types::{AccumulationPoint, AccumulationQuantity, AccumulationTrack};

const DEMO_RESPAWN_WINDOW_SECONDS: f32 = 3.2;

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
pub enum BoostPickupDetection {
    /// Corroborated by both a reported pad event and an inferred boost-amount jump.
    Both,
    /// Inferred from a boost-amount jump with no matching reported pad event.
    InferredOnly,
    /// Reported by a pad event with no corroborating boost-amount jump.
    ReportedOnly,
}

#[derive(Clone, Debug)]
struct PendingBoostPickupEvent {
    frame: usize,
    time: f32,
    player_id: PlayerId,
    player_position: Option<[f32; 3]>,
    is_team_0: bool,
    pad_type: BoostPickupPadType,
    field_half: BoostPickupFieldHalf,
    activity: BoostPickupActivity,
    boost_before: Option<f32>,
    boost_after: Option<f32>,
}

/// A single boost pickup. Replaces the former pickup-comparison + ledger Collected/Stolen/
/// Overfill events: pad classification, theft, and amounts are all facets of one pickup.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BoostPickupEvent {
    pub frame: usize,
    pub time: f32,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player_id: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub pad_type: BoostPickupPadType,
    pub field_half: BoostPickupFieldHalf,
    pub activity: BoostPickupActivity,
    pub detection: BoostPickupDetection,
    /// A steal is a pickup collected on the opponent's half (mirrors the former `Stolen` ledger
    /// transaction condition).
    pub is_steal: bool,
    /// Boost actually gained from this pickup.
    pub collected_amount: f32,
    /// Boost lost to the cap because the player was already near full when collecting.
    pub overfill_amount: f32,
    pub boost_before: Option<f32>,
    pub boost_after: Option<f32>,
}

/// A player respawn (kickoff or post-demo). This is a general lifecycle event, not boost-specific;
/// the boost grant is just one of its effects.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RespawnEvent {
    pub frame: usize,
    pub time: f32,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player_id: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub kind: RespawnKind,
    /// Boost the player respawned with, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boost_granted: Option<f32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RespawnKind {
    Kickoff,
    Demo,
}

pub(crate) fn boost_transaction_label(kind: &'static str) -> StatLabel {
    StatLabel::new("transaction", kind)
}

pub(crate) fn boost_pad_size_label(pad_size: Option<BoostPadSize>) -> StatLabel {
    match pad_size {
        Some(BoostPadSize::Big) => StatLabel::new("pad_size", "big"),
        Some(BoostPadSize::Small) => StatLabel::new("pad_size", "small"),
        None => StatLabel::new("pad_size", "unknown"),
    }
}

pub(crate) fn boost_activity_label(activity: BoostPickupActivity) -> StatLabel {
    match activity {
        BoostPickupActivity::Active => StatLabel::new("activity", "active"),
        BoostPickupActivity::Inactive => StatLabel::new("activity", "inactive"),
        BoostPickupActivity::Unknown => StatLabel::new("activity", "unknown"),
    }
}

pub(crate) fn boost_field_half_label(field_half: BoostPickupFieldHalf) -> StatLabel {
    match field_half {
        BoostPickupFieldHalf::Own => StatLabel::new("field_half", "own"),
        BoostPickupFieldHalf::Opponent => StatLabel::new("field_half", "opponent"),
        BoostPickupFieldHalf::Unknown => StatLabel::new("field_half", "unknown"),
    }
}

pub(crate) fn boost_supersonic_label(supersonic: bool) -> StatLabel {
    if supersonic {
        StatLabel::new("supersonic", "true")
    } else {
        StatLabel::new("supersonic", "false")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BoostCalculatorConfig {
    pub include_non_live_pickups: bool,
}

#[derive(Debug, Clone, Default)]
pub struct BoostCalculator {
    config: BoostCalculatorConfig,
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
    pickup_events: EventStream<BoostPickupEvent>,
    respawn_events: EventStream<RespawnEvent>,
    stats: BoostStatsAccumulator,
    track_builders: HashMap<(PlayerId, AccumulationQuantity), BoostTrackBuilder>,
    player_usage_state: HashMap<PlayerId, BoostUsageState>,
    kickoff_phase_active_last_frame: bool,
    kickoff_respawn_awarded: HashSet<PlayerId>,
    initial_respawn_awarded: HashSet<PlayerId>,
    pending_demo_respawns: HashMap<PlayerId, PendingDemoRespawn>,
    pending_reported_pickups: VecDeque<DeferredReportedBoostPickup>,
    previous_boost_levels_live: Option<bool>,
}

#[derive(Debug, Clone, Copy)]
struct PendingDemoRespawn {
    demo_time: f32,
    pre_demo_boost_amount: Option<f32>,
}

/// Builder for one player's [`AccumulationTrack`]: accumulates change-points, skipping samples
/// whose value matches the previous point.
#[derive(Debug, Clone, Default)]
struct BoostTrackBuilder {
    is_team_0: bool,
    points: Vec<AccumulationPoint>,
}

#[derive(Debug, Clone, Copy, Default)]
struct BoostUsageState {
    amount_obtained: f32,
    amount_removed: f32,
    amount_used: f32,
    amount_used_while_grounded: f32,
    amount_used_while_airborne: f32,
}

impl BoostUsageState {
    fn amount_used_by_vertical_band(self) -> f32 {
        self.amount_used_while_grounded + self.amount_used_while_airborne
    }

    fn apply_obtained_amount(&mut self, amount: f32) {
        self.amount_obtained += amount;
    }

    fn apply_used_amount(&mut self, amount: f32) {
        self.amount_used += amount;
    }

    fn apply_used_allocation(&mut self, amount: f32, grounded: bool) {
        if grounded {
            self.amount_used_while_grounded += amount;
        } else {
            self.amount_used_while_airborne += amount;
        }
    }

    fn apply_removed_amount(&mut self, amount: f32) {
        self.amount_removed += amount;
    }

    fn inferred_amount_used(self, current_boost_amount: f32) -> f32 {
        // Boost balance:
        //   current = obtained - used - removed
        //
        // `obtained` contains pickups plus respawn grants. `removed` covers
        // non-usage losses such as demoing a player with boost. Rearranging the
        // balance lets us infer cumulative use from the observed current boost:
        //   used = obtained - removed - current
        (self.amount_obtained - self.amount_removed - current_boost_amount).max(0.0)
    }
}

#[derive(Debug, Clone)]
struct PendingBoostPickup {
    frame: usize,
    time: f32,
    player_id: PlayerId,
    is_team_0: bool,
    previous_boost_amount: f32,
    pre_applied_collected_amount: f32,
    player_position: glam::Vec3,
    boost_before: Option<f32>,
    boost_after: Option<f32>,
}

#[derive(Debug, Clone)]
struct DeferredReportedBoostPickup {
    pad_id: String,
    pending_pickup: PendingBoostPickup,
    pad_size: BoostPadSize,
    reported_event: PendingBoostPickupEvent,
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

    pub fn pickup_events(&self) -> &[BoostPickupEvent] {
        self.pickup_events.all()
    }

    pub fn new_pickup_events(&self) -> &[BoostPickupEvent] {
        self.pickup_events.new_events()
    }

    pub fn respawn_events(&self) -> &[RespawnEvent] {
        self.respawn_events.all()
    }

    pub fn new_respawn_events(&self) -> &[RespawnEvent] {
        self.respawn_events.new_events()
    }

    pub fn player_boost_stats(&self) -> &HashMap<PlayerId, BoostStats> {
        self.stats.player_stats()
    }

    pub fn player_boost_stats_for(&self, player_id: &PlayerId) -> BoostStats {
        self.stats.player_stats_for(player_id)
    }

    pub fn team_zero_boost_stats(&self) -> &BoostStats {
        self.stats.team_zero_stats()
    }

    pub fn team_one_boost_stats(&self) -> &BoostStats {
        self.stats.team_one_stats()
    }

    /// The boost-stats accumulator, populated directly as frames are processed (no event replay).
    pub fn boost_stats(&self) -> &BoostStatsAccumulator {
        &self.stats
    }

    /// Build the compressed per-player accumulation tracks (boost amount + cumulative used /
    /// collected / stolen / overfill). Sorted by player id then quantity for deterministic output.
    pub fn accumulation_tracks(&self) -> Vec<AccumulationTrack> {
        let mut tracks: Vec<AccumulationTrack> = self
            .track_builders
            .iter()
            .filter(|(_, builder)| !builder.points.is_empty())
            .map(|((player_id, quantity), builder)| AccumulationTrack {
                player_id: player_id.clone(),
                is_team_0: builder.is_team_0,
                quantity: *quantity,
                points: builder.points.clone(),
            })
            .collect();
        tracks.sort_by(|left, right| {
            format!("{:?}", left.player_id)
                .cmp(&format!("{:?}", right.player_id))
                .then_with(|| format!("{:?}", left.quantity).cmp(&format!("{:?}", right.quantity)))
        });
        tracks
    }

    fn player_usage_state(&self, player_id: &PlayerId) -> BoostUsageState {
        self.player_usage_state
            .get(player_id)
            .copied()
            .unwrap_or_default()
    }

    fn record_track_point(
        &mut self,
        player_id: &PlayerId,
        is_team_0: bool,
        frame: usize,
        quantity: AccumulationQuantity,
        value: f32,
    ) {
        let builder = self
            .track_builders
            .entry((player_id.clone(), quantity))
            .or_default();
        builder.is_team_0 = is_team_0;
        if builder
            .points
            .last()
            .is_some_and(|point| (point.value - value).abs() <= f32::EPSILON)
        {
            return;
        }
        builder.points.push(AccumulationPoint { frame, value });
    }

    /// Sample this player's instantaneous boost amount and their cumulative boost stats into the
    /// per-frame accumulation tracks.
    fn record_boost_tracks(
        &mut self,
        player_id: &PlayerId,
        is_team_0: bool,
        frame: usize,
        boost_amount: f32,
    ) {
        let stats = self.stats.player_stats_for(player_id);
        self.record_track_point(
            player_id,
            is_team_0,
            frame,
            AccumulationQuantity::BoostAmount,
            boost_amount,
        );
        self.record_track_point(
            player_id,
            is_team_0,
            frame,
            AccumulationQuantity::BoostUsed,
            stats.amount_used,
        );
        self.record_track_point(
            player_id,
            is_team_0,
            frame,
            AccumulationQuantity::BoostUsedGrounded,
            stats.amount_used_while_grounded,
        );
        self.record_track_point(
            player_id,
            is_team_0,
            frame,
            AccumulationQuantity::BoostUsedAirborne,
            stats.amount_used_while_airborne,
        );
        self.record_track_point(
            player_id,
            is_team_0,
            frame,
            AccumulationQuantity::BoostUsedSupersonic,
            stats.amount_used_while_supersonic,
        );
        self.record_track_point(
            player_id,
            is_team_0,
            frame,
            AccumulationQuantity::BoostCollected,
            stats.amount_collected,
        );
        self.record_track_point(
            player_id,
            is_team_0,
            frame,
            AccumulationQuantity::BoostStolen,
            stats.amount_stolen,
        );
        self.record_track_point(
            player_id,
            is_team_0,
            frame,
            AccumulationQuantity::BoostOverfill,
            stats.overfill_total,
        );
    }

    /// Push a reported pickup, resolving its detection against any pending inferred pickup that
    /// corroborates it (and consuming that inferred pickup so it does not later surface as an
    /// inferred-only pickup).
    fn push_reported_pickup(
        &mut self,
        reported: PendingBoostPickupEvent,
        mut pickup: BoostPickupEvent,
    ) {
        let matched = if let Some(index) =
            Self::matching_pending_pickup_index(&self.pending_inferred_pickups, &reported, true)
        {
            self.pending_inferred_pickups.remove(index);
            true
        } else if let Some(index) = self.pending_inferred_pickups.iter().position(|pending| {
            pending.player_id == reported.player_id
                && pending.frame.abs_diff(reported.frame) <= Self::PICKUP_MATCH_FRAME_WINDOW
        }) {
            self.pending_inferred_pickups.remove(index);
            true
        } else {
            false
        };
        pickup.detection = if matched {
            BoostPickupDetection::Both
        } else {
            BoostPickupDetection::ReportedOnly
        };
        self.pickup_events.push(pickup);
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
    ) -> BoostPickupEvent {
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
        let nominal_gain = match pad_size {
            BoostPadSize::Big => BOOST_MAX_AMOUNT,
            BoostPadSize::Small => SMALL_PAD_AMOUNT_RAW,
        };
        let collected_amount = (BOOST_MAX_AMOUNT - pending_pickup.previous_boost_amount)
            .min(nominal_gain)
            .max(0.0);
        let collected_amount_delta = collected_amount - pending_pickup.pre_applied_collected_amount;
        let overfill = (nominal_gain - collected_amount).max(0.0);
        let field_half = if stolen {
            BoostPickupFieldHalf::Opponent
        } else {
            BoostPickupFieldHalf::Own
        };

        // The obtained balance is applied incrementally (pre-applied + this delta) so usage
        // inference stays correctly timed; the pickup stats below carry the full collected amount
        // once, which sums identically to the former split Collected ledger entries.
        self.player_usage_state
            .entry(pending_pickup.player_id.clone())
            .or_default()
            .apply_obtained_amount(collected_amount_delta);

        self.stats.apply_pickup(
            &pending_pickup.player_id,
            pending_pickup.is_team_0,
            Some(pad_size),
            BoostPickupActivity::Active,
            field_half,
            stolen,
            collected_amount,
            overfill,
        );

        BoostPickupEvent {
            frame: pending_pickup.frame,
            time: pending_pickup.time,
            player_id: pending_pickup.player_id.clone(),
            player_position: Some(pending_pickup.player_position.to_array()),
            is_team_0: pending_pickup.is_team_0,
            pad_type: pad_size.into(),
            field_half,
            activity: BoostPickupActivity::Active,
            // Overwritten by `push_reported_pickup` for reported pickups; inferred-only callers
            // set it explicitly.
            detection: BoostPickupDetection::Both,
            is_steal: stolen,
            collected_amount,
            overfill_amount: overfill,
            boost_before: pending_pickup.boost_before,
            boost_after: pending_pickup.boost_after,
        }
    }

    /// Pre-apply a partial collected amount to the usage-inference balance only (no stats / event).
    /// The full pickup is recorded later when [`Self::resolve_pickup`] runs.
    fn apply_pickup_collected_amount(
        &mut self,
        player_id: &PlayerId,
        amount: f32,
        pad_size: Option<BoostPadSize>,
    ) {
        if amount <= 0.0 {
            return;
        }
        if pad_size.is_some() {
            self.player_usage_state
                .entry(player_id.clone())
                .or_default()
                .apply_obtained_amount(amount);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_inactive_pickup(
        &mut self,
        player_id: &PlayerId,
        player_position: Option<[f32; 3]>,
        is_team_0: bool,
        frame: usize,
        time: f32,
        amount: f32,
        pad_size: BoostPadSize,
        boost_before: Option<f32>,
        boost_after: Option<f32>,
    ) -> BoostPickupEvent {
        self.stats.apply_pickup(
            player_id,
            is_team_0,
            Some(pad_size),
            BoostPickupActivity::Inactive,
            BoostPickupFieldHalf::Unknown,
            false,
            amount,
            0.0,
        );
        BoostPickupEvent {
            frame,
            time,
            player_id: player_id.clone(),
            player_position,
            is_team_0,
            pad_type: pad_size.into(),
            field_half: BoostPickupFieldHalf::Unknown,
            activity: BoostPickupActivity::Inactive,
            detection: BoostPickupDetection::ReportedOnly,
            is_steal: false,
            collected_amount: amount,
            overfill_amount: 0.0,
            boost_before,
            boost_after,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_respawn_amount(
        &mut self,
        player_id: &PlayerId,
        player_position: Option<[f32; 3]>,
        is_team_0: bool,
        frame: usize,
        time: f32,
        amount: f32,
        kind: RespawnKind,
    ) {
        if amount <= 0.0 {
            return;
        }
        self.player_usage_state
            .entry(player_id.clone())
            .or_default()
            .apply_obtained_amount(amount);
        self.stats.apply_respawn(player_id, is_team_0, amount);
        self.respawn_events.push(RespawnEvent {
            frame,
            time,
            player_id: player_id.clone(),
            player_position,
            is_team_0,
            kind,
            boost_granted: Some(amount),
        });
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
            || (gameplay.ball_has_been_hit == Some(false) && !gameplay.kickoff_countdown_active())
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
        let big_pad_floor = SMALL_PAD_AMOUNT_RAW + TOLERANCE;
        if delta > big_pad_floor {
            return vec![BoostIncreaseReason::BigPad];
        }
        if boost < BOOST_FULL_BAND_MIN_RAW && delta >= small_pad_floor {
            const SMALL_PICKUP_COUNT_TOLERANCE: f32 = 3.0;
            let inferred_small_pickups = ((delta - SMALL_PICKUP_COUNT_TOLERANCE)
                / SMALL_PAD_AMOUNT_RAW)
                .ceil()
                .max(1.0) as usize;
            return vec![BoostIncreaseReason::SmallPad; inferred_small_pickups];
        }

        if boost >= BOOST_MAX_AMOUNT - TOLERANCE {
            return vec![BoostIncreaseReason::AmbiguousPad];
        }
        if delta >= small_pad_floor {
            return vec![BoostIncreaseReason::SmallPad];
        }
        vec![BoostIncreaseReason::Unknown]
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

    fn resolve_deferred_reported_pickups(&mut self, frame: &FrameInfo, players: &PlayerFrameState) {
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

            let pickup =
                self.resolve_pickup(&deferred.pad_id, deferred.pending_pickup, deferred.pad_size);
            self.push_reported_pickup(deferred.reported_event, pickup);
        }
        self.pending_reported_pickups = remaining_pickups;
    }

    fn flush_deferred_reported_pickups(&mut self) {
        while let Some(deferred) = self.pending_reported_pickups.pop_front() {
            let pickup =
                self.resolve_pickup(&deferred.pad_id, deferred.pending_pickup, deferred.pad_size);
            self.push_reported_pickup(deferred.reported_event, pickup);
        }
    }

    fn inferred_pickup_pad_size(&self, event: &PendingBoostPickupEvent) -> BoostPadSize {
        match event.pad_type {
            BoostPickupPadType::Big => BoostPadSize::Big,
            BoostPickupPadType::Small => BoostPadSize::Small,
            BoostPickupPadType::Ambiguous => event
                .player_position
                .and_then(|position| {
                    self.guess_pad_size_from_position(
                        &format!("inferred:{:?}:{}", event.player_id, event.frame),
                        glam::Vec3::from_array(position),
                    )
                })
                .unwrap_or(BoostPadSize::Big),
        }
    }

    fn resolve_inferred_pickup(
        &mut self,
        event: PendingBoostPickupEvent,
        application_frame: &FrameInfo,
    ) {
        let Some(boost_before) = event.boost_before else {
            // No boost-before sample to compute amounts from; emit a detection-only inferred
            // pickup with no stats contribution (matches the former no-resolve Ghost path).
            self.pickup_events.push(BoostPickupEvent {
                frame: event.frame,
                time: event.time,
                player_id: event.player_id.clone(),
                player_position: event.player_position,
                is_team_0: event.is_team_0,
                pad_type: event.pad_type,
                field_half: event.field_half,
                activity: event.activity,
                detection: BoostPickupDetection::InferredOnly,
                is_steal: matches!(event.field_half, BoostPickupFieldHalf::Opponent),
                collected_amount: 0.0,
                overfill_amount: 0.0,
                boost_before: event.boost_before,
                boost_after: event.boost_after,
            });
            return;
        };

        let pad_size = self.inferred_pickup_pad_size(&event);
        let player_position = event
            .player_position
            .map(glam::Vec3::from_array)
            .unwrap_or(glam::Vec3::ZERO);
        let pad_id = format!("inferred:{:?}:{}", event.player_id, event.frame);
        let mut pickup = self.resolve_pickup(
            &pad_id,
            PendingBoostPickup {
                frame: application_frame.frame_number,
                time: application_frame.time,
                player_id: event.player_id.clone(),
                is_team_0: event.is_team_0,
                previous_boost_amount: boost_before,
                pre_applied_collected_amount: 0.0,
                player_position,
                boost_before: event.boost_before,
                boost_after: event.boost_after,
            },
            pad_size,
        );
        // The pickup happened when the boost-amount jump was observed, not at the flush frame.
        pickup.frame = event.frame;
        pickup.time = event.time;
        pickup.detection = BoostPickupDetection::InferredOnly;
        self.pickup_events.push(pickup);
    }

    fn flush_stale_pickup_comparisons(&mut self, frame: &FrameInfo) {
        while self
            .pending_inferred_pickups
            .front()
            .is_some_and(|event| event.frame + Self::PICKUP_MATCH_FRAME_WINDOW < frame.frame_number)
        {
            if let Some(event) = self.pending_inferred_pickups.pop_front() {
                self.resolve_inferred_pickup(event, frame);
            }
        }
    }

    pub fn finish_calculation(&mut self) -> SubtrActorResult<()> {
        self.flush_deferred_reported_pickups();
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
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.pickup_events.begin_update();
        self.respawn_events.begin_update();
        let live_play = live_play_state.counts_toward_player_motion();
        let boost_levels_live = Self::boost_levels_live(live_play);
        let track_boost_levels = Self::tracks_boost_levels(boost_levels_live);
        let track_boost_pickups = Self::tracks_boost_pickups(gameplay, live_play);
        let boost_levels_resumed_this_sample =
            boost_levels_live && !self.previous_boost_levels_live.unwrap_or(false);
        let kickoff_phase_active = gameplay.kickoff_phase_active();
        let kickoff_phase_started = kickoff_phase_active && !self.kickoff_phase_active_last_frame;
        if kickoff_phase_started {
            self.kickoff_respawn_awarded.clear();
        }
        for demo in &events.demo_events {
            let pre_demo_boost_amount = self.previous_boost_amounts.get(&demo.victim).copied();
            self.pending_demo_respawns
                .entry(demo.victim.clone())
                .or_insert(PendingDemoRespawn {
                    demo_time: demo.time,
                    pre_demo_boost_amount,
                });
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
            let previous_boost_amount = previous_sample_boost_amount
                .or(player.last_boost_amount)
                .unwrap_or(boost_amount);
            let previous_boost_amount = if boost_levels_resumed_this_sample {
                boost_amount
            } else {
                previous_boost_amount
            };
            let pending_demo_respawn = self.pending_demo_respawns.get(&player.player_id);
            let demo_respawn_ready = pending_demo_respawn.is_some_and(|pending| {
                player.rigid_body.is_some()
                    && frame.time - pending.demo_time >= DEMO_RESPAWN_WINDOW_SECONDS
            });
            let demo_respawn_pending = pending_demo_respawn.is_some() && !demo_respawn_ready;
            let demo_respawn_supported = demo_respawn_ready;

            if demo_respawn_pending {
                if let Some(pending) = self.pending_demo_respawns.get_mut(&player.player_id) {
                    pending.pre_demo_boost_amount = pending
                        .pre_demo_boost_amount
                        .or(previous_sample_boost_amount);
                }
                continue;
            }

            let mut boost_increase_reasons = Vec::new();
            if let Some(previous_sample_boost_amount) = previous_sample_boost_amount {
                boost_increase_reasons = Self::classify_boost_increase_reasons(
                    previous_sample_boost_amount,
                    boost_amount,
                    kickoff_phase_active,
                    demo_respawn_supported,
                );
                if track_boost_levels {
                    for reason in &boost_increase_reasons {
                        if let Ok(pad_type) = BoostPickupPadType::try_from(*reason) {
                            self.record_inferred_pickup(PendingBoostPickupEvent {
                                frame: frame.frame_number,
                                time: frame.time,
                                player_id: player.player_id.clone(),
                                player_position: player
                                    .position()
                                    .map(|position| position.to_array()),
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
            }
            let generic_respawn_supported = track_boost_levels
                && boost_increase_reasons.contains(&BoostIncreaseReason::Respawn);
            if track_boost_levels {
                let previous = if boost_levels_resumed_this_sample {
                    boost_amount
                } else {
                    previous_boost_amount
                };
                self.stats.apply_boost_sample(
                    &player.player_id,
                    player.is_team_0,
                    previous,
                    boost_amount,
                    frame.dt,
                );
            }

            let mut respawn_amount = 0.0;
            // Grant initial kickoff respawn the first time we see each player.
            // This handles replays that start after the kickoff countdown has
            // already ended on the first frame.
            let first_seen_player = self
                .initial_respawn_awarded
                .insert(player.player_id.clone());
            let kickoff_respawn_due = kickoff_phase_active
                && !self.kickoff_respawn_awarded.contains(&player.player_id)
                && track_boost_levels;
            if first_seen_player || kickoff_respawn_due {
                respawn_amount += BOOST_KICKOFF_START_AMOUNT;
                self.kickoff_respawn_awarded
                    .insert(player.player_id.clone());
            }
            let demo_respawn = demo_respawn_supported || generic_respawn_supported;
            if demo_respawn {
                if let Some(pending) = self.pending_demo_respawns.get(&player.player_id) {
                    // A demo removes the victim's pre-demo boost; the respawn
                    // grant below is a separate obtained amount.
                    let boost_removed_by_demo_amount = pending
                        .pre_demo_boost_amount
                        .unwrap_or(previous_boost_amount)
                        .max(0.0);
                    self.player_usage_state
                        .entry(player.player_id.clone())
                        .or_default()
                        .apply_removed_amount(boost_removed_by_demo_amount);
                }
                respawn_amount += BOOST_KICKOFF_START_AMOUNT;
                self.pending_demo_respawns.remove(&player.player_id);
            }
            if respawn_amount > 0.0 {
                let respawn_kind = if demo_respawn {
                    RespawnKind::Demo
                } else {
                    RespawnKind::Kickoff
                };
                self.apply_respawn_amount(
                    &player.player_id,
                    player.position().map(|position| position.to_array()),
                    player.is_team_0,
                    frame.frame_number,
                    frame.time,
                    respawn_amount,
                    respawn_kind,
                );
            }
            respawn_amounts_by_player.insert(player.player_id.clone(), respawn_amount);

            current_boost_amounts.push((player.player_id.clone(), player.is_team_0, boost_amount));
        }

        self.resolve_deferred_reported_pickups(frame, players);

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
                        let pickup = self.apply_inactive_pickup(
                            player_id,
                            player.position().map(|position| position.to_array()),
                            player.is_team_0,
                            event.frame,
                            event.time,
                            collected_amount,
                            pad_size,
                            Some(previous_boost_amount),
                            player.boost_amount,
                        );
                        let reported_event = PendingBoostPickupEvent {
                            frame: event.frame,
                            time: event.time,
                            player_id: player_id.clone(),
                            player_position: player.position().map(|position| position.to_array()),
                            is_team_0: player.is_team_0,
                            pad_type: pad_size.into(),
                            field_half: Self::field_half_from_position(
                                player.is_team_0,
                                player.position(),
                            ),
                            activity: BoostPickupActivity::Inactive,
                            boost_before: None,
                            boost_after: None,
                        };
                        self.push_reported_pickup(reported_event, pickup);
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
                    let previous_boost_amount = self
                        .previous_boost_amounts
                        .get(player_id)
                        .copied()
                        .or(player.last_boost_amount)
                        .unwrap_or_else(|| player.boost_amount.unwrap_or(0.0));
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
                        pre_applied_collected_amount,
                        pre_applied_pad_size,
                    );
                    let pending_pickup = PendingBoostPickup {
                        frame: event.frame,
                        time: event.time,
                        player_id: player_id.clone(),
                        is_team_0: player.is_team_0,
                        previous_boost_amount,
                        pre_applied_collected_amount,
                        player_position: player.position().unwrap_or(glam::Vec3::ZERO),
                        boost_before: Some(previous_boost_amount),
                        boost_after: player.boost_amount,
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
                                && pre_applied_collected_amount > SMALL_PAD_AMOUNT_RAW + 1.0
                            {
                                size = BoostPadSize::Big;
                            }
                            self.known_pad_sizes.insert(event.pad_id.clone(), size);
                            Some(size)
                        });
                    if let Some(pad_size) = pad_size {
                        let reported_event = PendingBoostPickupEvent {
                            frame: event.frame,
                            time: event.time,
                            player_id: player_id.clone(),
                            player_position: player.position().map(|position| position.to_array()),
                            is_team_0: player.is_team_0,
                            pad_type: pad_size.into(),
                            field_half: Self::field_half_from_position(
                                player.is_team_0,
                                player.position(),
                            ),
                            activity: Self::activity_label(track_boost_pickups),
                            boost_before: None,
                            boost_after: None,
                        };
                        let pickup_gain_is_visible = pre_applied_collected_amount > 1.0
                            || player.boost_amount.is_some_and(|boost_amount| {
                                boost_amount > previous_boost_amount + 1.0
                            });
                        if !pickup_gain_is_visible {
                            self.pending_reported_pickups
                                .push_back(DeferredReportedBoostPickup {
                                    pad_id: event.pad_id.clone(),
                                    pending_pickup,
                                    pad_size,
                                    reported_event,
                                });
                            continue;
                        }

                        let pickup = self.resolve_pickup(&event.pad_id, pending_pickup, pad_size);
                        self.push_reported_pickup(reported_event, pickup);
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
        self.flush_stale_pickup_comparisons(frame);

        if track_boost_levels {
            for player in &players.players {
                if self.pending_demo_respawns.contains_key(&player.player_id) {
                    continue;
                }
                let Some(boost_amount) = player.boost_amount else {
                    continue;
                };
                let usage_state = self.player_usage_state(&player.player_id);
                let previous_amount_used = usage_state.amount_used;
                let amount_used_raw = usage_state.inferred_amount_used(boost_amount);
                let amount_used = amount_used_raw.max(usage_state.amount_used);
                let amount_used_delta = amount_used - previous_amount_used;
                let split_amount = usage_state.amount_used_by_vertical_band();
                let amount_used_allocation_delta = (amount_used - split_amount).max(0.0);
                if amount_used_allocation_delta > 0.0 {
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
                    let grounded = vertical_state.is_grounded(&player.player_id);
                    self.stats.apply_used_allocation(
                        &player.player_id,
                        player.is_team_0,
                        amount_used_allocation_delta,
                        grounded,
                        used_while_supersonic,
                    );
                    self.player_usage_state
                        .entry(player.player_id.clone())
                        .or_default()
                        .apply_used_allocation(amount_used_allocation_delta, grounded);
                }
                if amount_used_delta <= 0.0 {
                    continue;
                }
                self.player_usage_state
                    .entry(player.player_id.clone())
                    .or_default()
                    .apply_used_amount(amount_used_delta);
                self.stats
                    .apply_used(&player.player_id, player.is_team_0, amount_used_delta);
            }
        }
        for (player_id, is_team_0, boost_amount) in current_boost_amounts {
            if track_boost_levels {
                self.record_boost_tracks(&player_id, is_team_0, frame.frame_number, boost_amount);
            }
            self.previous_boost_amounts.insert(player_id, boost_amount);
        }
        for player in &players.players {
            if let Some(speed) = player.speed() {
                self.previous_player_speeds
                    .insert(player.player_id.clone(), speed);
            }
        }
        self.kickoff_phase_active_last_frame = kickoff_phase_active;
        self.previous_boost_levels_live = Some(boost_levels_live);

        Ok(())
    }
}

#[cfg(test)]
#[path = "boost_tests.rs"]
mod tests;
