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
pub struct BoostStatsAccumulator {
    player_stats: HashMap<PlayerId, BoostStats>,
    team_zero_stats: BoostStats,
    team_one_stats: BoostStats,
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
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &BoostStats {
        &self.team_one_stats
    }

    pub fn apply_event(&mut self, event: &BoostStatsEvent) {
        let player_stats = self
            .player_stats
            .entry(event.player_id.clone())
            .or_default();
        apply_delta(player_stats, &event.delta);
        let team_stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        apply_delta(team_stats, &event.delta);
    }
}

fn apply_delta(stats: &mut BoostStats, delta: &BoostStats) {
    stats.tracked_time += delta.tracked_time;
    stats.boost_integral += delta.boost_integral;
    stats.time_zero_boost += delta.time_zero_boost;
    stats.time_hundred_boost += delta.time_hundred_boost;
    stats.time_boost_0_25 += delta.time_boost_0_25;
    stats.time_boost_25_50 += delta.time_boost_25_50;
    stats.time_boost_50_75 += delta.time_boost_50_75;
    stats.time_boost_75_100 += delta.time_boost_75_100;
    stats.amount_collected += delta.amount_collected;
    stats.amount_collected_inactive += delta.amount_collected_inactive;
    stats.big_pads_collected_inactive += delta.big_pads_collected_inactive;
    stats.small_pads_collected_inactive += delta.small_pads_collected_inactive;
    stats.amount_stolen += delta.amount_stolen;
    stats.big_pads_collected += delta.big_pads_collected;
    stats.small_pads_collected += delta.small_pads_collected;
    stats.big_pads_stolen += delta.big_pads_stolen;
    stats.small_pads_stolen += delta.small_pads_stolen;
    stats.amount_collected_big += delta.amount_collected_big;
    stats.amount_stolen_big += delta.amount_stolen_big;
    stats.amount_collected_small += delta.amount_collected_small;
    stats.amount_stolen_small += delta.amount_stolen_small;
    stats.amount_respawned += delta.amount_respawned;
    stats.overfill_total += delta.overfill_total;
    stats.overfill_from_stolen += delta.overfill_from_stolen;
    stats.amount_used += delta.amount_used;
    stats.amount_used_while_grounded += delta.amount_used_while_grounded;
    stats.amount_used_while_airborne += delta.amount_used_while_airborne;
    stats.amount_used_while_supersonic += delta.amount_used_while_supersonic;
    for entry in &delta.labeled_amounts.entries {
        stats.labeled_amounts.add(entry.labels.clone(), entry.value);
    }
    for entry in &delta.labeled_counts.entries {
        for _ in 0..entry.count {
            stats.labeled_counts.increment(entry.labels.clone());
        }
    }
}
