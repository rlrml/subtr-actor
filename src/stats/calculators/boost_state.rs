use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BoostCalculatorConfig {
    pub include_non_live_pickups: bool,
}

#[derive(Debug, Clone, Default)]
pub struct BoostCalculator {
    pub(super) config: BoostCalculatorConfig,
    pub(super) player_stats: HashMap<PlayerId, BoostStats>,
    pub(super) team_zero_stats: BoostStats,
    pub(super) team_one_stats: BoostStats,
    pub(super) previous_boost_amounts: HashMap<PlayerId, f32>,
    pub(super) previous_player_speeds: HashMap<PlayerId, f32>,
    pub(super) observed_pad_positions: HashMap<String, PadPositionEstimate>,
    pub(super) known_pad_sizes: HashMap<String, BoostPadSize>,
    pub(super) known_pad_indices: HashMap<String, usize>,
    pub(super) unavailable_pads: HashSet<String>,
    pub(super) seen_pickup_sequence_times: HashMap<(String, u8), f32>,
    pub(super) pickup_frames: HashMap<(String, PlayerId), usize>,
    pub(super) inactive_pickup_frames: HashSet<(PlayerId, usize, BoostPadSize)>,
    pub(super) last_pickup_times: HashMap<String, f32>,
    pub(super) pending_inferred_pickups: VecDeque<PendingBoostPickupEvent>,
    pub(super) pickup_comparison_events: Vec<BoostPickupComparisonEvent>,
    pub(super) ledger_events: Vec<BoostLedgerEvent>,
    pub(super) state_events: Vec<BoostStateEvent>,
    pub(super) kickoff_phase_active_last_frame: bool,
    pub(super) kickoff_respawn_awarded: HashSet<PlayerId>,
    pub(super) initial_respawn_awarded: HashSet<PlayerId>,
    pub(super) pending_demo_respawns: HashMap<PlayerId, PendingDemoRespawn>,
    pub(super) demo_reset_boost_amounts: HashMap<PlayerId, f32>,
    pub(super) previous_boost_levels_live: Option<bool>,
    pub(super) active_invariant_warnings: HashSet<BoostInvariantWarningKey>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PendingDemoRespawn {
    pub(super) demo_time: f32,
    pub(super) pre_demo_boost_amount: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct BoostInvariantWarningKey {
    pub(super) scope: String,
    pub(super) kind: BoostInvariantKind,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct BoostLedgerContext {
    pub(super) frame: usize,
    pub(super) time: f32,
    pub(super) boost_before: Option<f32>,
    pub(super) boost_after: Option<f32>,
}

#[derive(Debug, Clone)]
pub(super) struct PendingBoostPickup {
    pub(super) frame: usize,
    pub(super) time: f32,
    pub(super) player_id: PlayerId,
    pub(super) is_team_0: bool,
    pub(super) previous_boost_amount: f32,
    pub(super) pre_applied_collected_amount: f32,
    pub(super) pre_applied_pad_size: Option<BoostPadSize>,
    pub(super) player_position: glam::Vec3,
    pub(super) boost_before: Option<f32>,
    pub(super) boost_after: Option<f32>,
}
