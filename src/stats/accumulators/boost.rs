use super::*;
use crate::stats::calculators::boost::{
    boost_activity_label, boost_field_half_label, boost_pad_size_label, boost_supersonic_label,
    boost_transaction_label,
};
use crate::stats::common::vertical_state_label;

const BOOST_ZERO_BAND_RAW: f32 = 1.0;
const BOOST_FULL_BAND_MIN_RAW: f32 = BOOST_MAX_AMOUNT - 1.0;

/// Accumulated boost stats: time in boost bands, boost integral, and pads collected/stolen/used.
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

    pub(crate) fn add_labeled_amount<I>(&mut self, labels: I, amount: f32)
    where
        I: IntoIterator<Item = StatLabel>,
    {
        if amount > 0.0 {
            self.labeled_amounts.add(labels, amount);
        }
    }

    pub(crate) fn increment_labeled_count<I>(&mut self, labels: I)
    where
        I: IntoIterator<Item = StatLabel>,
    {
        self.labeled_counts.increment(labels);
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct PlayerBoostProjection {
    stats: BoostStats,
    is_team_0: Option<bool>,
}

/// Accumulates [`BoostStats`] from typed boost transitions.
///
/// This replaces the former label-dispatched ledger projection: callers invoke the typed
/// `apply_*` methods directly (from the boost calculator) instead of constructing string-labeled
/// ledger events and re-parsing them here. The per-player and per-team arithmetic is identical;
/// only the representation changed. `labeled_amounts`/`labeled_counts` are still populated so the
/// exported labeled boost stats stay populated, now built from the clean pickup model.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BoostStatsAccumulator {
    players: HashMap<PlayerId, PlayerBoostProjection>,
    player_stats: HashMap<PlayerId, BoostStats>,
    team_zero: BoostStats,
    team_one: BoostStats,
}

impl BoostStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, BoostStats> {
        &self.player_stats
    }

    pub fn player_stats_for(&self, player_id: &PlayerId) -> BoostStats {
        self.player_stats
            .get(player_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn team_zero_stats(&self) -> &BoostStats {
        &self.team_zero
    }

    pub fn team_one_stats(&self) -> &BoostStats {
        &self.team_one
    }

    /// Apply an additive `update` to both the player's stats and their team's stats, refreshing
    /// the cached per-player snapshot. `update` runs once per stats object.
    fn project(&mut self, player_id: &PlayerId, is_team_0: bool, update: impl Fn(&mut BoostStats)) {
        let player = self.players.entry(player_id.clone()).or_default();
        player.is_team_0 = Some(is_team_0);
        update(&mut player.stats);
        self.player_stats
            .insert(player_id.clone(), player.stats.clone());
        let team = if is_team_0 {
            &mut self.team_zero
        } else {
            &mut self.team_one
        };
        update(team);
    }

    /// Record a single boost pickup, folding the former Collected/Stolen/Overfill ledger
    /// transactions into one transition. `collected_amount` is the full boost gained; the
    /// per-pickup totals here sum identically to the old split ledger entries.
    #[allow(clippy::too_many_arguments)]
    pub fn apply_pickup(
        &mut self,
        player_id: &PlayerId,
        is_team_0: bool,
        pad_size: Option<BoostPadSize>,
        activity: BoostPickupActivity,
        field_half: BoostPickupFieldHalf,
        is_steal: bool,
        collected_amount: f32,
        overfill_amount: f32,
    ) {
        self.project(player_id, is_team_0, |stats| {
            let labels = |transaction: &'static str| -> Vec<StatLabel> {
                vec![
                    boost_transaction_label(transaction),
                    boost_pad_size_label(pad_size),
                    boost_activity_label(activity),
                    boost_field_half_label(field_half),
                ]
            };
            stats.add_labeled_amount(labels("collected"), collected_amount);
            stats.increment_labeled_count(labels("collected"));
            if matches!(activity, BoostPickupActivity::Inactive) {
                stats.amount_collected_inactive += collected_amount;
                match pad_size {
                    Some(BoostPadSize::Big) => stats.big_pads_collected_inactive += 1,
                    Some(BoostPadSize::Small) => stats.small_pads_collected_inactive += 1,
                    None => {}
                }
            } else {
                stats.amount_collected += collected_amount;
                match pad_size {
                    Some(BoostPadSize::Big) => {
                        stats.amount_collected_big += collected_amount;
                        stats.big_pads_collected += 1;
                    }
                    Some(BoostPadSize::Small) => {
                        stats.amount_collected_small += collected_amount;
                        stats.small_pads_collected += 1;
                    }
                    None => {}
                }
            }
            if is_steal {
                stats.add_labeled_amount(labels("stolen"), collected_amount);
                stats.amount_stolen += collected_amount;
                match pad_size {
                    Some(BoostPadSize::Big) => {
                        stats.big_pads_stolen += 1;
                        stats.amount_stolen_big += collected_amount;
                    }
                    Some(BoostPadSize::Small) => {
                        stats.small_pads_stolen += 1;
                        stats.amount_stolen_small += collected_amount;
                    }
                    None => {}
                }
            }
            if overfill_amount > 0.0 {
                stats.add_labeled_amount(labels("overfill"), overfill_amount);
                stats.overfill_total += overfill_amount;
                if matches!(field_half, BoostPickupFieldHalf::Opponent) {
                    stats.overfill_from_stolen += overfill_amount;
                }
            }
        });
    }

    /// Record a respawn boost grant (kickoff or demo respawn).
    pub fn apply_respawn(&mut self, player_id: &PlayerId, is_team_0: bool, amount: f32) {
        if amount <= 0.0 {
            return;
        }
        self.project(player_id, is_team_0, |stats| {
            stats.add_labeled_amount(vec![boost_transaction_label("respawn")], amount);
            stats.amount_respawned += amount;
        });
    }

    /// Record cumulative boost usage for a frame (total drained).
    pub fn apply_used(&mut self, player_id: &PlayerId, is_team_0: bool, amount: f32) {
        if amount <= 0.0 {
            return;
        }
        self.project(player_id, is_team_0, |stats| {
            stats.amount_used += amount;
        });
    }

    /// Record the vertical-band / supersonic breakdown of boost usage for a frame.
    pub fn apply_used_allocation(
        &mut self,
        player_id: &PlayerId,
        is_team_0: bool,
        amount: f32,
        grounded: bool,
        supersonic: bool,
    ) {
        if amount <= 0.0 {
            return;
        }
        self.project(player_id, is_team_0, |stats| {
            stats.add_labeled_amount(
                vec![
                    boost_transaction_label("used"),
                    vertical_state_label(!grounded),
                    boost_supersonic_label(supersonic),
                ],
                amount,
            );
            if grounded {
                stats.amount_used_while_grounded += amount;
            } else {
                stats.amount_used_while_airborne += amount;
            }
            if supersonic {
                stats.amount_used_while_supersonic += amount;
            }
        });
    }

    /// Record a continuous per-frame boost-amount sample (drives the boost integral and the
    /// time-in-band totals).
    pub fn apply_boost_sample(
        &mut self,
        player_id: &PlayerId,
        is_team_0: bool,
        previous_boost_amount: f32,
        boost_amount: f32,
        dt: f32,
    ) {
        self.project(player_id, is_team_0, |stats| {
            Self::add_continuous_boost_sample(stats, previous_boost_amount, boost_amount, dt);
        });
    }

    fn add_continuous_boost_sample(
        stats: &mut BoostStats,
        previous_boost_amount: f32,
        boost_amount: f32,
        dt: f32,
    ) {
        let average_boost_amount = (previous_boost_amount + boost_amount) * 0.5;
        stats.tracked_time += dt;
        stats.boost_integral += average_boost_amount * dt;
        stats.time_zero_boost += dt
            * Self::interval_fraction_in_boost_range(
                previous_boost_amount,
                boost_amount,
                0.0,
                BOOST_ZERO_BAND_RAW,
            );
        stats.time_hundred_boost += dt
            * Self::interval_fraction_in_boost_range(
                previous_boost_amount,
                boost_amount,
                BOOST_FULL_BAND_MIN_RAW,
                BOOST_MAX_AMOUNT + 1.0,
            );
        stats.time_boost_0_25 += dt
            * Self::interval_fraction_in_boost_range(
                previous_boost_amount,
                boost_amount,
                0.0,
                boost_percent_to_amount(25.0),
            );
        stats.time_boost_25_50 += dt
            * Self::interval_fraction_in_boost_range(
                previous_boost_amount,
                boost_amount,
                boost_percent_to_amount(25.0),
                boost_percent_to_amount(50.0),
            );
        stats.time_boost_50_75 += dt
            * Self::interval_fraction_in_boost_range(
                previous_boost_amount,
                boost_amount,
                boost_percent_to_amount(50.0),
                boost_percent_to_amount(75.0),
            );
        stats.time_boost_75_100 += dt
            * Self::interval_fraction_in_boost_range(
                previous_boost_amount,
                boost_amount,
                boost_percent_to_amount(75.0),
                BOOST_MAX_AMOUNT + 1.0,
            );
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
}
