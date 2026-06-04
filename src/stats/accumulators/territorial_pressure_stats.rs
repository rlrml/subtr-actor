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

    pub fn set_tracked_time(&mut self, tracked_time: f32) {
        self.stats.tracked_time = tracked_time;
    }

    pub fn add_tracked_time(&mut self, dt: f32) {
        self.stats.tracked_time += dt;
    }

    pub fn apply_event(&mut self, event: &TerritorialPressureEvent) {
        let pressure_team_label = Self::pressure_team_label(event.team_is_team_0);
        self.stats
            .labeled_session_counts
            .increment([pressure_team_label.clone()]);

        let offensive_half_only_time =
            (event.offensive_half_time - event.offensive_third_time).max(0.0);
        let relief_time = (event.duration - event.offensive_half_time).max(0.0);
        self.stats.labeled_time.add(
            [
                pressure_team_label.clone(),
                StatLabel::new("territory", "offensive_half"),
            ],
            offensive_half_only_time,
        );
        self.stats.labeled_time.add(
            [
                pressure_team_label.clone(),
                StatLabel::new("territory", "offensive_third"),
            ],
            event.offensive_third_time,
        );
        self.stats.labeled_time.add(
            [pressure_team_label, StatLabel::new("territory", "relief")],
            relief_time,
        );

        if event.team_is_team_0 {
            self.stats.team_zero_session_count += 1;
            self.stats.team_zero_session_time += event.duration;
            self.stats.team_zero_offensive_half_time += event.offensive_half_time;
            self.stats.team_zero_offensive_third_time += event.offensive_third_time;
            self.stats.team_zero_longest_session_time = self
                .stats
                .team_zero_longest_session_time
                .max(event.duration);
        } else {
            self.stats.team_one_session_count += 1;
            self.stats.team_one_session_time += event.duration;
            self.stats.team_one_offensive_half_time += event.offensive_half_time;
            self.stats.team_one_offensive_third_time += event.offensive_third_time;
            self.stats.team_one_longest_session_time =
                self.stats.team_one_longest_session_time.max(event.duration);
        }
    }

    fn pressure_team_label(team_is_team_0: bool) -> StatLabel {
        StatLabel::new(
            "pressure_team",
            if team_is_team_0 {
                "team_zero"
            } else {
                "team_one"
            },
        )
    }
}
