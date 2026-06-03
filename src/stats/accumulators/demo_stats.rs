use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct DemoPlayerStats {
    pub demos_inflicted: u32,
    pub demos_taken: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct DemoTeamStats {
    pub demos_inflicted: u32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DemoStatsAccumulator {
    player_stats: HashMap<PlayerId, DemoPlayerStats>,
    team_zero_stats: DemoTeamStats,
    team_one_stats: DemoTeamStats,
}

impl DemoStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, DemoPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &DemoTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &DemoTeamStats {
        &self.team_one_stats
    }

    pub fn apply_timeline_event(&mut self, event: &TimelineEvent) {
        let Some(player_id) = event.player_id.as_ref() else {
            return;
        };

        match event.kind {
            TimelineEventKind::Kill => {
                self.player_stats
                    .entry(player_id.clone())
                    .or_default()
                    .demos_inflicted += 1;
                match event.is_team_0 {
                    Some(true) => self.team_zero_stats.demos_inflicted += 1,
                    Some(false) => self.team_one_stats.demos_inflicted += 1,
                    None => {}
                }
            }
            TimelineEventKind::Death => {
                self.player_stats
                    .entry(player_id.clone())
                    .or_default()
                    .demos_taken += 1;
            }
            _ => {}
        }
    }
}
