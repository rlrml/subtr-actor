use super::*;

#[path = "boost_helpers.rs"]
mod boost_helpers;
use boost_helpers::*;

const DEMO_RESPAWN_WINDOW_SECONDS: f32 = 3.2;

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
    #[serde(default, skip_serializing_if = "LabeledFloatSums::is_empty")]
    pub labeled_amounts: LabeledFloatSums,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_counts: LabeledCounts,
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
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, ts_rs::TS)]
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
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
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

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BoostStateEvent {
    pub frame: usize,
    pub time: f32,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player_id: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub boost_amount: f32,
    pub boost_before: Option<f32>,
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

    fn add_labeled_amount<I>(&mut self, labels: I, amount: f32)
    where
        I: IntoIterator<Item = StatLabel>,
    {
        if amount > 0.0 {
            self.labeled_amounts.add(labels, amount);
        }
    }

    fn increment_labeled_count<I>(&mut self, labels: I)
    where
        I: IntoIterator<Item = StatLabel>,
    {
        self.labeled_counts.increment(labels);
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
    ledger_events: Vec<BoostLedgerEvent>,
    state_events: Vec<BoostStateEvent>,
    kickoff_phase_active_last_frame: bool,
    kickoff_respawn_awarded: HashSet<PlayerId>,
    initial_respawn_awarded: HashSet<PlayerId>,
    pending_demo_respawns: HashMap<PlayerId, PendingDemoRespawn>,
    demo_reset_boost_amounts: HashMap<PlayerId, f32>,
    previous_boost_levels_live: Option<bool>,
    active_invariant_warnings: HashSet<BoostInvariantWarningKey>,
}

#[derive(Debug, Clone, Copy)]
struct PendingDemoRespawn {
    demo_time: f32,
    pre_demo_boost_amount: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BoostInvariantWarningKey {
    scope: String,
    kind: BoostInvariantKind,
}

#[derive(Debug, Clone, Copy)]
struct BoostLedgerContext {
    frame: usize,
    time: f32,
    boost_before: Option<f32>,
    boost_after: Option<f32>,
}

#[derive(Debug, Clone)]
struct PendingBoostPickup {
    frame: usize,
    time: f32,
    player_id: PlayerId,
    is_team_0: bool,
    previous_boost_amount: f32,
    pre_applied_collected_amount: f32,
    pre_applied_pad_size: Option<BoostPadSize>,
    player_position: glam::Vec3,
    boost_before: Option<f32>,
    boost_after: Option<f32>,
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

    pub fn ledger_events(&self) -> &[BoostLedgerEvent] {
        &self.ledger_events
    }

    pub fn state_events(&self) -> &[BoostStateEvent] {
        &self.state_events
    }

    fn record_ledger_event(&mut self, event: BoostLedgerEvent) {
        if event.amount <= 0.0 && event.count == 0 {
            return;
        }

        self.ledger_events.push(event);
    }

    fn record_state_event(&mut self, event: BoostStateEvent) {
        self.state_events.push(event);
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
            let previous_boost_amount = player
                .last_boost_amount
                .unwrap_or_else(|| previous_sample_boost_amount.unwrap_or(boost_amount));
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
                            player_position: player.position().map(|position| position.to_array()),
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
                let boost_before = if boost_levels_resumed_this_sample {
                    None
                } else {
                    Some(previous_boost_amount)
                };
                self.record_state_event(BoostStateEvent {
                    frame: frame.frame_number,
                    time: frame.time,
                    player_id: player.player_id.clone(),
                    player_position: player.position().map(|position| position.to_array()),
                    is_team_0: player.is_team_0,
                    boost_amount,
                    boost_before,
                });

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
            // already ended on the first frame.
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
                if let Some(pending) = self.pending_demo_respawns.get(&player.player_id) {
                    let demo_reset_amount = pending
                        .pre_demo_boost_amount
                        .unwrap_or(previous_boost_amount)
                        .max(0.0);
                    *self
                        .demo_reset_boost_amounts
                        .entry(player.player_id.clone())
                        .or_default() += demo_reset_amount;
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
                        pre_applied_pad_size,
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
                            player_position: player.position().map(|position| position.to_array()),
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
            let stats = self
                .player_stats
                .entry(player.player_id.clone())
                .or_default();
            let previous_amount_used = stats.amount_used;
            let demo_reset_boost_amount = self
                .demo_reset_boost_amounts
                .get(&player.player_id)
                .copied()
                .unwrap_or(0.0);
            let amount_used_raw =
                (stats.amount_obtained() - demo_reset_boost_amount - boost_amount).max(0.0);
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
                    stats.add_labeled_amount(used_labels.clone(), amount_used_delta);
                    team_stats.add_labeled_amount(used_labels.clone(), amount_used_delta);
                    used_ledger_event = Some(BoostLedgerEvent {
                        frame: frame.frame_number,
                        time: frame.time,
                        player_id: player.player_id.clone(),
                        player_position: player.position().map(|position| position.to_array()),
                        is_team_0: player.is_team_0,
                        transaction: BoostLedgerTransactionKind::UsedAllocation,
                        amount: amount_used_delta,
                        count: 0,
                        labels: used_labels.into_iter().collect(),
                        boost_before,
                        boost_after: Some(boost_amount),
                    });
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
            if let Some(event) = used_ledger_event {
                self.record_ledger_event(event);
            }
            if amount_used_delta <= 0.0 {
                continue;
            }
            self.record_ledger_event(BoostLedgerEvent {
                frame: frame.frame_number,
                time: frame.time,
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
#[path = "boost_tests.rs"]
mod tests;
