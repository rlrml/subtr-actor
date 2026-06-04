use super::*;
use std::collections::HashSet;

const BOOST_ZERO_BAND_RAW: f32 = 1.0;
const BOOST_FULL_BAND_MIN_RAW: f32 = BOOST_MAX_AMOUNT - 1.0;

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
struct BoostLedgerProjection {
    stats: BoostStats,
    counted_pickup_keys: HashSet<(usize, PlayerId, String, String, String)>,
    current_boost_amount: Option<f32>,
    current_boost_before: Option<f32>,
    current_boost_frame: Option<usize>,
    is_team_0: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BoostStatsAccumulator {
    players: HashMap<PlayerId, BoostLedgerProjection>,
    player_stats: HashMap<PlayerId, BoostStats>,
    team_zero: BoostLedgerProjection,
    team_one: BoostLedgerProjection,
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
        &self.team_zero.stats
    }

    pub fn team_one_stats(&self) -> &BoostStats {
        &self.team_one.stats
    }

    pub fn apply_ledger_event(&mut self, event: &BoostLedgerEvent) {
        let player = self.players.entry(event.player_id.clone()).or_default();
        player.is_team_0 = Some(event.is_team_0);
        Self::apply_ledger_event_to_projection(player, event);
        self.player_stats
            .insert(event.player_id.clone(), player.stats.clone());

        let team = if event.is_team_0 {
            &mut self.team_zero
        } else {
            &mut self.team_one
        };
        Self::apply_ledger_event_to_projection(team, event);
    }

    pub fn apply_state_event(&mut self, event: &BoostStateEvent) {
        let player = self.players.entry(event.player_id.clone()).or_default();
        player.is_team_0 = Some(event.is_team_0);
        player.current_boost_amount = Some(event.boost_amount);
        player.current_boost_before = event.boost_before;
        player.current_boost_frame = Some(event.frame);
        let previous_boost_amount = event.boost_before.unwrap_or(event.boost_amount);
        Self::add_continuous_boost_sample(
            &mut player.stats,
            previous_boost_amount,
            event.boost_amount,
            event.duration,
        );
        self.player_stats
            .insert(event.player_id.clone(), player.stats.clone());

        let team_stats = if event.is_team_0 {
            &mut self.team_zero.stats
        } else {
            &mut self.team_one.stats
        };
        Self::add_continuous_boost_sample(
            team_stats,
            previous_boost_amount,
            event.boost_amount,
            event.duration,
        );
    }

    pub fn apply_frame_sample(&mut self, frame: &FrameInfo) {
        let player_ids = self
            .players
            .iter()
            .filter(|(_, projection)| projection.current_boost_frame == Some(frame.frame_number))
            .map(|(player_id, _)| player_id.clone())
            .collect::<Vec<_>>();

        for player_id in player_ids {
            let Some(player) = self.players.get_mut(&player_id) else {
                continue;
            };
            let Some(boost_amount) = player.current_boost_amount else {
                continue;
            };
            let previous_boost_amount = player.current_boost_before.unwrap_or(boost_amount);
            Self::add_continuous_boost_sample(
                &mut player.stats,
                previous_boost_amount,
                boost_amount,
                frame.dt,
            );
            self.player_stats
                .insert(player_id.clone(), player.stats.clone());

            let Some(is_team_0) = player.is_team_0 else {
                continue;
            };
            let team_stats = if is_team_0 {
                &mut self.team_zero.stats
            } else {
                &mut self.team_one.stats
            };
            Self::add_continuous_boost_sample(
                team_stats,
                previous_boost_amount,
                boost_amount,
                frame.dt,
            );
        }
    }

    fn apply_ledger_event_to_projection(
        projection: &mut BoostLedgerProjection,
        event: &BoostLedgerEvent,
    ) {
        let amount = event.amount;
        if event.transaction != BoostLedgerTransactionKind::Used {
            projection
                .stats
                .add_labeled_amount(event.labels.clone(), amount);
        }
        if event.transaction == BoostLedgerTransactionKind::Collected {
            let count = event.count.max(1);
            for _ in 0..count {
                projection
                    .stats
                    .increment_labeled_count(event.labels.clone());
            }
        }

        let pad_size = Self::label_value(event, "pad_size");
        let activity = Self::label_value(event, "activity").unwrap_or("active");
        let field_half = Self::label_value(event, "field_half");
        match event.transaction {
            BoostLedgerTransactionKind::Collected => {
                Self::count_pickup_once(projection, event);
                if activity == "inactive" {
                    projection.stats.amount_collected_inactive += amount;
                    return;
                }
                projection.stats.amount_collected += amount;
                match pad_size {
                    Some("big") => projection.stats.amount_collected_big += amount,
                    Some("small") => projection.stats.amount_collected_small += amount,
                    _ => {}
                }
            }
            BoostLedgerTransactionKind::Stolen => {
                projection.stats.amount_stolen += amount;
                match pad_size {
                    Some("big") => {
                        projection.stats.big_pads_stolen += 1;
                        projection.stats.amount_stolen_big += amount;
                    }
                    Some("small") => {
                        projection.stats.small_pads_stolen += 1;
                        projection.stats.amount_stolen_small += amount;
                    }
                    _ => {}
                }
            }
            BoostLedgerTransactionKind::Overfill => {
                projection.stats.overfill_total += amount;
                if field_half == Some("opponent") {
                    projection.stats.overfill_from_stolen += amount;
                }
                Self::count_pickup_once(projection, event);
            }
            BoostLedgerTransactionKind::Respawn => {
                projection.stats.amount_respawned += amount;
            }
            BoostLedgerTransactionKind::Used => {
                projection.stats.amount_used += amount;
            }
            BoostLedgerTransactionKind::UsedAllocation => {
                if Self::label_value(event, "vertical_state") == Some("grounded") {
                    projection.stats.amount_used_while_grounded += amount;
                } else if Self::label_value(event, "vertical_state") == Some("aerial") {
                    projection.stats.amount_used_while_airborne += amount;
                }
                if Self::label_value(event, "supersonic") == Some("true") {
                    projection.stats.amount_used_while_supersonic += amount;
                }
            }
        }
    }

    fn count_pickup_once(projection: &mut BoostLedgerProjection, event: &BoostLedgerEvent) {
        if event.count == 0 {
            return;
        }
        let Some(pad_size @ ("big" | "small")) = Self::label_value(event, "pad_size") else {
            return;
        };
        let activity = Self::label_value(event, "activity").unwrap_or("unknown");
        let field_half = Self::label_value(event, "field_half").unwrap_or("unknown");
        let key = (
            event.frame,
            event.player_id.clone(),
            pad_size.to_owned(),
            activity.to_owned(),
            field_half.to_owned(),
        );
        if !projection.counted_pickup_keys.insert(key) {
            return;
        }

        if activity == "inactive" {
            if pad_size == "big" {
                projection.stats.big_pads_collected_inactive += 1;
            } else {
                projection.stats.small_pads_collected_inactive += 1;
            }
        } else if pad_size == "big" {
            projection.stats.big_pads_collected += 1;
        } else {
            projection.stats.small_pads_collected += 1;
        }
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

    fn label_value<'a>(event: &'a BoostLedgerEvent, key: &str) -> Option<&'a str> {
        event
            .labels
            .iter()
            .find(|label| label.key == key)
            .map(|label| label.value)
    }
}
