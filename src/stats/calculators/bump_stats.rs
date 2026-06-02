use super::bump::BumpEvent;
use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct BumpPlayerStats {
    pub bumps_inflicted: u32,
    pub bumps_taken: u32,
    pub team_bumps_inflicted: u32,
    pub team_bumps_taken: u32,
    pub last_bump_time: Option<f32>,
    pub last_bump_frame: Option<usize>,
    pub last_bump_strength: Option<f32>,
    pub max_bump_strength: f32,
    pub cumulative_bump_strength: f32,
}

impl BumpPlayerStats {
    pub fn average_bump_strength(&self) -> f32 {
        if self.bumps_inflicted == 0 {
            0.0
        } else {
            self.cumulative_bump_strength / self.bumps_inflicted as f32
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct BumpTeamStats {
    pub bumps_inflicted: u32,
    pub team_bumps_inflicted: u32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BumpStatsAccumulator {
    player_stats: HashMap<PlayerId, BumpPlayerStats>,
    team_zero_stats: BumpTeamStats,
    team_one_stats: BumpTeamStats,
}

impl BumpStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_events(events: &[BumpEvent]) -> Self {
        let mut accumulator = Self::new();
        accumulator.extend(events);
        accumulator
    }

    pub fn extend<'a>(&mut self, events: impl IntoIterator<Item = &'a BumpEvent>) {
        for event in events {
            self.apply_event(event);
        }
    }

    pub fn apply_event(&mut self, event: &BumpEvent) {
        record_bump_inflicted(
            self.player_stats
                .entry(event.initiator.clone())
                .or_default(),
            event,
        );
        record_bump_taken(
            self.player_stats.entry(event.victim.clone()).or_default(),
            event,
        );
        record_bump_team_stats(
            if event.initiator_is_team_0 {
                &mut self.team_zero_stats
            } else {
                &mut self.team_one_stats
            },
            event,
        );
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, BumpPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &BumpTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &BumpTeamStats {
        &self.team_one_stats
    }
}

fn record_bump_inflicted(stats: &mut BumpPlayerStats, event: &BumpEvent) {
    stats.bumps_inflicted += 1;
    if event.is_team_bump {
        stats.team_bumps_inflicted += 1;
    }
    stats.last_bump_time = Some(event.time);
    stats.last_bump_frame = Some(event.frame);
    stats.last_bump_strength = Some(event.strength);
    stats.max_bump_strength = stats.max_bump_strength.max(event.strength);
    stats.cumulative_bump_strength += event.strength;
}

fn record_bump_taken(stats: &mut BumpPlayerStats, event: &BumpEvent) {
    stats.bumps_taken += 1;
    if event.is_team_bump {
        stats.team_bumps_taken += 1;
    }
}

fn record_bump_team_stats(stats: &mut BumpTeamStats, event: &BumpEvent) {
    stats.bumps_inflicted += 1;
    if event.is_team_bump {
        stats.team_bumps_inflicted += 1;
    }
}
