use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct TerritorialPressureStats {
    pub tracked_time: f32,
    pub team_zero_session_count: u32,
    pub team_one_session_count: u32,
    pub team_zero_session_time: f32,
    pub team_one_session_time: f32,
    pub team_zero_offensive_half_time: f32,
    pub team_one_offensive_half_time: f32,
    pub team_zero_offensive_third_time: f32,
    pub team_one_offensive_third_time: f32,
    pub team_zero_longest_session_time: f32,
    pub team_one_longest_session_time: f32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_session_counts: LabeledCounts,
    #[serde(default, skip_serializing_if = "LabeledFloatSums::is_empty")]
    pub labeled_time: LabeledFloatSums,
}

impl TerritorialPressureStats {
    pub fn for_team(&self, is_team_zero: bool) -> TerritorialPressureTeamStats {
        let (
            session_count,
            opponent_session_count,
            session_time,
            opponent_session_time,
            offensive_half_time,
            offensive_third_time,
            longest_session_time,
            opponent_longest_session_time,
        ) = if is_team_zero {
            (
                self.team_zero_session_count,
                self.team_one_session_count,
                self.team_zero_session_time,
                self.team_one_session_time,
                self.team_zero_offensive_half_time,
                self.team_zero_offensive_third_time,
                self.team_zero_longest_session_time,
                self.team_one_longest_session_time,
            )
        } else {
            (
                self.team_one_session_count,
                self.team_zero_session_count,
                self.team_one_session_time,
                self.team_zero_session_time,
                self.team_one_offensive_half_time,
                self.team_one_offensive_third_time,
                self.team_one_longest_session_time,
                self.team_zero_longest_session_time,
            )
        };

        let average_session_time = if session_count == 0 {
            0.0
        } else {
            session_time / session_count as f32
        };

        TerritorialPressureTeamStats {
            tracked_time: self.tracked_time,
            session_count,
            opponent_session_count,
            session_time,
            opponent_session_time,
            offensive_half_time,
            offensive_third_time,
            longest_session_time,
            opponent_longest_session_time,
            average_session_time,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct TerritorialPressureTeamStats {
    pub tracked_time: f32,
    pub session_count: u32,
    pub opponent_session_count: u32,
    pub session_time: f32,
    pub opponent_session_time: f32,
    pub offensive_half_time: f32,
    pub offensive_third_time: f32,
    pub longest_session_time: f32,
    pub opponent_longest_session_time: f32,
    pub average_session_time: f32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TerritorialPressureStatsAccumulator {
    stats: TerritorialPressureStats,
}

impl TerritorialPressureStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stats(&self) -> &TerritorialPressureStats {
        &self.stats
    }

    pub fn apply_event(&mut self, event: &TerritorialPressureStatsEvent) {
        self.apply_delta(&event.delta);
    }

    fn apply_delta(&mut self, delta: &TerritorialPressureStats) {
        self.stats.tracked_time += delta.tracked_time;
        self.stats.team_zero_session_count += delta.team_zero_session_count;
        self.stats.team_one_session_count += delta.team_one_session_count;
        self.stats.team_zero_session_time += delta.team_zero_session_time;
        self.stats.team_one_session_time += delta.team_one_session_time;
        self.stats.team_zero_offensive_half_time += delta.team_zero_offensive_half_time;
        self.stats.team_one_offensive_half_time += delta.team_one_offensive_half_time;
        self.stats.team_zero_offensive_third_time += delta.team_zero_offensive_third_time;
        self.stats.team_one_offensive_third_time += delta.team_one_offensive_third_time;
        self.stats.team_zero_longest_session_time = self
            .stats
            .team_zero_longest_session_time
            .max(delta.team_zero_longest_session_time);
        self.stats.team_one_longest_session_time = self
            .stats
            .team_one_longest_session_time
            .max(delta.team_one_longest_session_time);
        for entry in &delta.labeled_session_counts.entries {
            for _ in 0..entry.count {
                self.stats
                    .labeled_session_counts
                    .increment(entry.labels.clone());
            }
        }
        for entry in &delta.labeled_time.entries {
            self.stats
                .labeled_time
                .add(entry.labels.clone(), entry.value);
        }
    }
}
