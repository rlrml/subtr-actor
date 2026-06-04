use super::*;

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
    player_position: Option<[f32; 3]>,
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
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player_id: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum BoostLedgerTransactionKind {
    Collected,
    Stolen,
    Overfill,
    Respawn,
    Used,
    UsedAllocation,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BoostLedgerEvent {
    pub frame: usize,
    pub time: f32,
    pub end_frame: usize,
    pub end_time: f32,
    pub duration: f32,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player_id: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub transaction: BoostLedgerTransactionKind,
    pub amount: f32,
    pub count: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<StatLabel>,
    pub boost_before: Option<f32>,
    pub boost_after: Option<f32>,
}

impl BoostLedgerEvent {
    fn absorb_delta(&mut self, event: Self) {
        self.end_frame = event.frame;
        self.end_time = event.time;
        self.duration += event.duration;
        self.player_position = event.player_position;
        self.amount += event.amount;
        self.count += event.count;
        self.boost_after = event.boost_after;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BoostStateEvent {
    pub frame: usize,
    pub time: f32,
    pub end_frame: usize,
    pub end_time: f32,
    pub duration: f32,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player_id: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub boost_amount: f32,
    pub boost_before: Option<f32>,
}

impl BoostStateEvent {
    fn absorb_sample(&mut self, event: Self) {
        self.end_frame = event.frame;
        self.end_time = event.time;
        self.duration += event.duration;
        self.player_position = event.player_position;
        self.boost_amount = event.boost_amount;
    }
}

fn boost_transaction_label(kind: &'static str) -> StatLabel {
    StatLabel::new("transaction", kind)
}

fn boost_pad_size_label(pad_size: Option<BoostPadSize>) -> StatLabel {
    match pad_size {
        Some(BoostPadSize::Big) => StatLabel::new("pad_size", "big"),
        Some(BoostPadSize::Small) => StatLabel::new("pad_size", "small"),
        None => StatLabel::new("pad_size", "unknown"),
    }
}

fn boost_activity_label(activity: BoostPickupActivity) -> StatLabel {
    match activity {
        BoostPickupActivity::Active => StatLabel::new("activity", "active"),
        BoostPickupActivity::Inactive => StatLabel::new("activity", "inactive"),
        BoostPickupActivity::Unknown => StatLabel::new("activity", "unknown"),
    }
}

fn boost_field_half_label(field_half: BoostPickupFieldHalf) -> StatLabel {
    match field_half {
        BoostPickupFieldHalf::Own => StatLabel::new("field_half", "own"),
        BoostPickupFieldHalf::Opponent => StatLabel::new("field_half", "opponent"),
        BoostPickupFieldHalf::Unknown => StatLabel::new("field_half", "unknown"),
    }
}

fn boost_supersonic_label(supersonic: bool) -> StatLabel {
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
    pickup_comparison_events: EventStream<BoostPickupComparisonEvent>,
    ledger_events: EventStream<BoostLedgerEvent>,
    state_events: EventStream<BoostStateEvent>,
    pending_ledger_events: HashMap<PendingBoostLedgerKey, PendingBoostLedgerEvent>,
    pending_state_events: HashMap<PlayerId, PendingBoostStateEvent>,
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

#[derive(Debug, Clone, Copy)]
struct BoostLedgerContext {
    frame: usize,
    time: f32,
    boost_before: Option<f32>,
    boost_after: Option<f32>,
}

#[derive(Debug, Clone, Copy, Default)]
struct BoostUsageState {
    amount_obtained: f32,
    amount_removed: f32,
    amount_used: f32,
    amount_used_while_grounded: f32,
    amount_used_while_airborne: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PendingBoostLedgerKey {
    player_id: PlayerId,
    is_team_0: bool,
    transaction: BoostLedgerTransactionKind,
    labels: Vec<StatLabel>,
}

#[derive(Debug, Clone)]
struct PendingBoostLedgerEvent {
    key: PendingBoostLedgerKey,
    event: BoostLedgerEvent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoostStateDirection {
    Increasing,
    Decreasing,
    Flat,
    Unknown,
}

impl BoostStateDirection {
    fn from_event(event: &BoostStateEvent) -> Self {
        let Some(boost_before) = event.boost_before else {
            return Self::Unknown;
        };
        if event.boost_amount > boost_before {
            Self::Increasing
        } else if event.boost_amount < boost_before {
            Self::Decreasing
        } else {
            Self::Flat
        }
    }
}

#[derive(Debug, Clone)]
struct PendingBoostStateEvent {
    is_team_0: bool,
    direction: BoostStateDirection,
    event: BoostStateEvent,
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

    pub fn pickup_comparison_events(&self) -> &[BoostPickupComparisonEvent] {
        self.pickup_comparison_events.all()
    }

    pub fn new_pickup_comparison_events(&self) -> &[BoostPickupComparisonEvent] {
        self.pickup_comparison_events.new_events()
    }

    pub fn ledger_events(&self) -> &[BoostLedgerEvent] {
        self.ledger_events.all()
    }

    pub fn new_ledger_events(&self) -> &[BoostLedgerEvent] {
        self.ledger_events.new_events()
    }

    pub fn projected_ledger_events(&self) -> Vec<BoostLedgerEvent> {
        let mut events = self.ledger_events.all().to_vec();
        let mut pending: Vec<_> = self.pending_ledger_events.values().cloned().collect();
        pending.sort_by(Self::compare_pending_ledger_events);
        events.extend(pending.into_iter().map(|pending| pending.event));
        events
    }

    pub fn flush_pending_ledger_events(&mut self) {
        let mut pending: Vec<_> = self
            .pending_ledger_events
            .drain()
            .map(|(_, pending)| pending)
            .collect();
        pending.sort_by(Self::compare_pending_ledger_events);
        self.ledger_events
            .extend(pending.into_iter().map(|pending| pending.event));
    }

    pub fn state_events(&self) -> &[BoostStateEvent] {
        self.state_events.all()
    }

    pub fn new_state_events(&self) -> &[BoostStateEvent] {
        self.state_events.new_events()
    }

    pub fn projected_state_events(&self) -> Vec<BoostStateEvent> {
        let mut events = self.state_events.all().to_vec();
        let mut pending: Vec<_> = self
            .pending_state_events
            .values()
            .map(|pending| pending.event.clone())
            .collect();
        pending.sort_by(|left, right| {
            left.frame.cmp(&right.frame).then_with(|| {
                format!("{:?}", left.player_id).cmp(&format!("{:?}", right.player_id))
            })
        });
        events.extend(pending);
        events
    }

    pub fn flush_pending_state_events(&mut self) {
        let mut pending: Vec<_> = self
            .pending_state_events
            .drain()
            .map(|(_, pending)| pending)
            .collect();
        pending.sort_by(|left, right| {
            left.event.frame.cmp(&right.event.frame).then_with(|| {
                format!("{:?}", left.event.player_id).cmp(&format!("{:?}", right.event.player_id))
            })
        });
        self.state_events
            .extend(pending.into_iter().map(|pending| pending.event));
    }

    fn player_usage_state(&self, player_id: &PlayerId) -> BoostUsageState {
        self.player_usage_state
            .get(player_id)
            .copied()
            .unwrap_or_default()
    }

    fn record_ledger_event(&mut self, event: BoostLedgerEvent) {
        if event.amount <= 0.0 && event.count == 0 {
            return;
        }

        if Self::is_continuous_ledger_event(&event) {
            self.record_continuous_ledger_event(event);
        } else {
            self.flush_pending_ledger_events_for_player(&event.player_id);
            self.ledger_events.push(event);
        }
    }

    fn record_state_event(&mut self, event: BoostStateEvent) {
        let player_id = event.player_id.clone();
        let direction = BoostStateDirection::from_event(&event);
        let Some(pending) = self.pending_state_events.get_mut(&player_id) else {
            self.pending_state_events.insert(
                player_id,
                PendingBoostStateEvent {
                    is_team_0: event.is_team_0,
                    direction,
                    event,
                },
            );
            return;
        };

        if pending.is_team_0 == event.is_team_0 && pending.direction == direction {
            pending.event.absorb_sample(event);
        } else {
            let previous = self.pending_state_events.insert(
                player_id,
                PendingBoostStateEvent {
                    is_team_0: event.is_team_0,
                    direction,
                    event,
                },
            );
            let Some(previous) = previous else {
                return;
            };
            self.state_events.push(previous.event);
        }
    }

    fn is_continuous_ledger_event(event: &BoostLedgerEvent) -> bool {
        matches!(
            event.transaction,
            BoostLedgerTransactionKind::Used | BoostLedgerTransactionKind::UsedAllocation
        )
    }

    fn pending_ledger_key(event: &BoostLedgerEvent) -> PendingBoostLedgerKey {
        PendingBoostLedgerKey {
            player_id: event.player_id.clone(),
            is_team_0: event.is_team_0,
            transaction: event.transaction,
            labels: event.labels.clone(),
        }
    }

    fn record_continuous_ledger_event(&mut self, event: BoostLedgerEvent) {
        let key = Self::pending_ledger_key(&event);
        let Some(pending) = self.pending_ledger_events.get_mut(&key) else {
            self.pending_ledger_events
                .insert(key.clone(), PendingBoostLedgerEvent { key, event });
            return;
        };
        pending.event.absorb_delta(event);
    }

    fn flush_pending_ledger_events_for_player(&mut self, player_id: &PlayerId) {
        let mut pending: Vec<_> = self
            .pending_ledger_events
            .extract_if(|_, pending| pending.key.player_id == *player_id)
            .map(|(_, pending)| pending)
            .collect();
        pending.sort_by(Self::compare_pending_ledger_events);
        self.ledger_events
            .extend(pending.into_iter().map(|pending| pending.event));
    }

    fn compare_pending_ledger_events(
        left: &PendingBoostLedgerEvent,
        right: &PendingBoostLedgerEvent,
    ) -> std::cmp::Ordering {
        left.event
            .frame
            .cmp(&right.event.frame)
            .then_with(|| {
                format!("{:?}", left.event.player_id).cmp(&format!("{:?}", right.event.player_id))
            })
            .then_with(|| left.event.is_team_0.cmp(&right.event.is_team_0))
            .then_with(|| {
                format!("{:?}", left.event.transaction)
                    .cmp(&format!("{:?}", right.event.transaction))
            })
            .then_with(|| {
                format!("{:?}", left.event.labels).cmp(&format!("{:?}", right.event.labels))
            })
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

        let collected_labels = [
            boost_transaction_label("collected"),
            boost_pad_size_label(Some(pad_size)),
            boost_activity_label(BoostPickupActivity::Active),
            boost_field_half_label(field_half),
        ];
        self.player_usage_state
            .entry(pending_pickup.player_id.clone())
            .or_default()
            .apply_obtained_amount(collected_amount_delta);

        let overfill_labels = [
            boost_transaction_label("overfill"),
            boost_pad_size_label(Some(pad_size)),
            boost_activity_label(BoostPickupActivity::Active),
            boost_field_half_label(field_half),
        ];

        self.record_ledger_event(BoostLedgerEvent {
            frame: pending_pickup.frame,
            time: pending_pickup.time,
            end_frame: pending_pickup.frame,
            end_time: pending_pickup.time,
            duration: 0.0,
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
                end_frame: pending_pickup.frame,
                end_time: pending_pickup.time,
                duration: 0.0,
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
            end_frame: pending_pickup.frame,
            end_time: pending_pickup.time,
            duration: 0.0,
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

    fn apply_pickup_collected_amount(
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

        let collected_labels = [
            boost_transaction_label("collected"),
            boost_pad_size_label(pad_size),
            boost_activity_label(BoostPickupActivity::Active),
            boost_field_half_label(BoostPickupFieldHalf::Unknown),
        ];
        if pad_size.is_some() {
            self.player_usage_state
                .entry(player_id.clone())
                .or_default()
                .apply_obtained_amount(amount);
        }
        self.record_ledger_event(BoostLedgerEvent {
            frame: ledger_context.frame,
            time: ledger_context.time,
            end_frame: ledger_context.frame,
            end_time: ledger_context.time,
            duration: 0.0,
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

    fn apply_inactive_pickup(
        &mut self,
        ledger_context: BoostLedgerContext,
        player_id: &PlayerId,
        player_position: Option<[f32; 3]>,
        is_team_0: bool,
        amount: f32,
        pad_size: BoostPadSize,
    ) {
        let collected_labels = [
            boost_transaction_label("collected"),
            boost_pad_size_label(Some(pad_size)),
            boost_activity_label(BoostPickupActivity::Inactive),
            boost_field_half_label(BoostPickupFieldHalf::Unknown),
        ];
        self.record_ledger_event(BoostLedgerEvent {
            frame: ledger_context.frame,
            time: ledger_context.time,
            end_frame: ledger_context.frame,
            end_time: ledger_context.time,
            duration: 0.0,
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

    fn apply_respawn_amount(
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

        let respawn_labels = [boost_transaction_label("respawn")];
        self.player_usage_state
            .entry(player_id.clone())
            .or_default()
            .apply_obtained_amount(amount);
        self.record_ledger_event(BoostLedgerEvent {
            frame: ledger_context.frame,
            time: ledger_context.time,
            end_frame: ledger_context.frame,
            end_time: ledger_context.time,
            duration: 0.0,
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
            if let Some(index) = self
                .pending_inferred_pickups
                .iter()
                .position(|pending_event| {
                    pending_event.player_id == event.player_id
                        && pending_event.frame.abs_diff(event.frame)
                            <= Self::PICKUP_MATCH_FRAME_WINDOW
                })
            {
                self.pending_inferred_pickups.remove(index);
            }
            self.emit_pickup_comparison_event(BoostPickupComparison::Both, None, Some(event));
        }
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

            let field_half =
                self.resolve_pickup(&deferred.pad_id, deferred.pending_pickup, deferred.pad_size);
            deferred.reported_event.field_half = field_half;
            self.record_reported_pickup(deferred.reported_event);
        }
        self.pending_reported_pickups = remaining_pickups;
    }

    fn flush_deferred_reported_pickups(&mut self) {
        while let Some(mut deferred) = self.pending_reported_pickups.pop_front() {
            let field_half =
                self.resolve_pickup(&deferred.pad_id, deferred.pending_pickup, deferred.pad_size);
            deferred.reported_event.field_half = field_half;
            self.record_reported_pickup(deferred.reported_event);
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
            self.emit_pickup_comparison_event(BoostPickupComparison::Ghost, Some(event), None);
            return;
        };

        let pad_size = self.inferred_pickup_pad_size(&event);
        let player_position = event
            .player_position
            .map(glam::Vec3::from_array)
            .unwrap_or(glam::Vec3::ZERO);
        let pad_id = format!("inferred:{:?}:{}", event.player_id, event.frame);
        let field_half = self.resolve_pickup(
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

        let mut comparison_event = event;
        comparison_event.pad_type = pad_size.into();
        comparison_event.field_half = field_half;
        self.emit_pickup_comparison_event(
            BoostPickupComparison::Ghost,
            Some(comparison_event),
            None,
        );
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
        self.flush_pending_ledger_events();
        self.flush_pending_state_events();
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
        self.pickup_comparison_events.begin_update();
        self.ledger_events.begin_update();
        self.state_events.begin_update();
        let boost_levels_live = Self::boost_levels_live(live_play);
        let track_boost_levels = Self::tracks_boost_levels(boost_levels_live);
        let track_boost_pickups = Self::tracks_boost_pickups(gameplay, live_play);
        if !track_boost_levels {
            self.flush_pending_ledger_events();
            self.flush_pending_state_events();
        }
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
                let boost_before = if boost_levels_resumed_this_sample {
                    None
                } else {
                    Some(previous_boost_amount)
                };
                self.record_state_event(BoostStateEvent {
                    frame: frame.frame_number,
                    time: frame.time,
                    end_frame: frame.frame_number,
                    end_time: frame.time,
                    duration: frame.dt,
                    player_id: player.player_id.clone(),
                    player_position: player.position().map(|position| position.to_array()),
                    is_team_0: player.is_team_0,
                    boost_amount,
                    boost_before,
                });
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
            if demo_respawn_supported || generic_respawn_supported {
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
                self.apply_respawn_amount(
                    BoostLedgerContext {
                        frame: frame.frame_number,
                        time: frame.time,
                        boost_before: Some(previous_boost_amount),
                        boost_after: Some(boost_amount),
                    },
                    &player.player_id,
                    player.position().map(|position| position.to_array()),
                    player.is_team_0,
                    respawn_amount,
                );
            }
            respawn_amounts_by_player.insert(player.player_id.clone(), respawn_amount);

            current_boost_amounts.push((player.player_id.clone(), boost_amount));
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
                        self.apply_inactive_pickup(
                            BoostLedgerContext {
                                frame: event.frame,
                                time: event.time,
                                boost_before: Some(previous_boost_amount),
                                boost_after: player.boost_amount,
                            },
                            player_id,
                            player.position().map(|position| position.to_array()),
                            player.is_team_0,
                            collected_amount,
                            pad_size,
                        );
                        self.record_reported_pickup(PendingBoostPickupEvent {
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
                        BoostLedgerContext {
                            frame: event.frame,
                            time: event.time,
                            boost_before: Some(previous_boost_amount),
                            boost_after: player.boost_amount,
                        },
                        player_id,
                        player.position().map(|position| position.to_array()),
                        player.is_team_0,
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
                        let mut reported_event = PendingBoostPickupEvent {
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

                        let field_half =
                            self.resolve_pickup(&event.pad_id, pending_pickup, pad_size);
                        reported_event.field_half = field_half;
                        self.record_reported_pickup(reported_event);
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
                let boost_before = self
                    .previous_boost_amounts
                    .get(&player.player_id)
                    .copied()
                    .or(player.last_boost_amount);
                let mut used_ledger_event = None;
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
                    let vertical_label = if vertical_state.is_grounded(&player.player_id) {
                        vertical_state_label(false)
                    } else {
                        vertical_state_label(true)
                    };
                    let used_labels = [
                        boost_transaction_label("used"),
                        vertical_label,
                        boost_supersonic_label(used_while_supersonic),
                    ];
                    used_ledger_event = Some(BoostLedgerEvent {
                        frame: frame.frame_number,
                        time: frame.time,
                        end_frame: frame.frame_number,
                        end_time: frame.time,
                        duration: frame.dt,
                        player_id: player.player_id.clone(),
                        player_position: player.position().map(|position| position.to_array()),
                        is_team_0: player.is_team_0,
                        transaction: BoostLedgerTransactionKind::UsedAllocation,
                        amount: amount_used_allocation_delta,
                        count: 0,
                        labels: used_labels.into_iter().collect(),
                        boost_before,
                        boost_after: Some(boost_amount),
                    });
                    self.player_usage_state
                        .entry(player.player_id.clone())
                        .or_default()
                        .apply_used_allocation(
                            amount_used_allocation_delta,
                            vertical_state.is_grounded(&player.player_id),
                        );
                }
                if let Some(event) = used_ledger_event {
                    self.record_ledger_event(event);
                }
                if amount_used_delta <= 0.0 {
                    continue;
                }
                self.player_usage_state
                    .entry(player.player_id.clone())
                    .or_default()
                    .apply_used_amount(amount_used_delta);
                self.record_ledger_event(BoostLedgerEvent {
                    frame: frame.frame_number,
                    time: frame.time,
                    end_frame: frame.frame_number,
                    end_time: frame.time,
                    duration: frame.dt,
                    player_id: player.player_id.clone(),
                    player_position: player.position().map(|position| position.to_array()),
                    is_team_0: player.is_team_0,
                    transaction: BoostLedgerTransactionKind::Used,
                    amount: amount_used_delta,
                    count: 0,
                    labels: [boost_transaction_label("used")].into_iter().collect(),
                    boost_before,
                    boost_after: Some(boost_amount),
                });
            }
        }
        for (player_id, boost_amount) in current_boost_amounts {
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
