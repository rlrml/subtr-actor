use super::*;

const MOVEMENT_SPEED_BAND_LABEL_VALUES: [&str; 3] = ["slow", "boost", "supersonic"];

fn movement_speed_band_label(value: &str) -> StatLabel {
    match value {
        "boost" => StatLabel::new("speed_band", "boost"),
        "supersonic" => StatLabel::new("speed_band", "supersonic"),
        _ => StatLabel::new("speed_band", "slow"),
    }
}

fn movement_height_band_label(value: &str) -> StatLabel {
    match value {
        "low_air" => StatLabel::new("height_band", "low_air"),
        "high_air" => StatLabel::new("height_band", "high_air"),
        _ => StatLabel::new("height_band", "ground"),
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct MovementStats {
    pub tracked_time: f32,
    pub total_distance: f32,
    pub speed_integral: f32,
    pub time_slow_speed: f32,
    pub time_boost_speed: f32,
    pub time_supersonic_speed: f32,
    pub time_on_ground: f32,
    pub time_low_air: f32,
    pub time_high_air: f32,
    #[serde(default, skip_serializing_if = "LabeledFloatSums::is_empty")]
    pub labeled_tracked_time: LabeledFloatSums,
}

impl MovementStats {
    pub fn average_speed(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.speed_integral / self.tracked_time
        }
    }

    pub fn average_speed_pct(&self) -> f32 {
        self.average_speed() * 100.0 / CAR_MAX_SPEED
    }

    pub fn slow_speed_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.time_slow_speed * 100.0 / self.tracked_time
        }
    }

    pub fn boost_speed_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.time_boost_speed * 100.0 / self.tracked_time
        }
    }

    pub fn supersonic_speed_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.time_supersonic_speed * 100.0 / self.tracked_time
        }
    }

    pub fn on_ground_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.time_on_ground * 100.0 / self.tracked_time
        }
    }

    pub fn low_air_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.time_low_air * 100.0 / self.tracked_time
        }
    }

    pub fn high_air_pct(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.time_high_air * 100.0 / self.tracked_time
        }
    }

    pub fn tracked_time_with_labels(&self, labels: &[StatLabel]) -> f32 {
        self.labeled_tracked_time.sum_matching(labels)
    }

    pub fn complete_labeled_tracked_time(&self) -> LabeledFloatSums {
        let mut entries: Vec<_> = ALL_PLAYER_VERTICAL_BANDS
            .into_iter()
            .flat_map(|height_band| {
                MOVEMENT_SPEED_BAND_LABEL_VALUES
                    .into_iter()
                    .map(move |speed_band| {
                        let mut labels = vec![
                            StatLabel::new("speed_band", speed_band),
                            height_band.as_label(),
                        ];
                        labels.sort();
                        LabeledFloatSumEntry {
                            value: self.labeled_tracked_time.sum_exact(&labels),
                            labels,
                        }
                    })
            })
            .collect();

        entries.sort_by(|left, right| left.labels.cmp(&right.labels));

        LabeledFloatSums { entries }
    }

    pub fn with_complete_labeled_tracked_time(mut self) -> Self {
        self.labeled_tracked_time = self.complete_labeled_tracked_time();
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MovementStatsAccumulator {
    player_stats: HashMap<PlayerId, MovementStats>,
    team_zero_stats: MovementStats,
    team_one_stats: MovementStats,
}

impl MovementStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, MovementStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &MovementStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &MovementStats {
        &self.team_one_stats
    }

    pub fn apply_event(&mut self, event: &MovementEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        Self::apply_to_stats(stats, event);
        let team_stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        Self::apply_to_stats(team_stats, event);
    }

    fn apply_to_stats(stats: &mut MovementStats, event: &MovementEvent) {
        stats.tracked_time += event.dt;
        stats.speed_integral += event.speed * event.dt;
        stats.total_distance += event.distance;

        match event.speed_band.as_str() {
            "boost" => stats.time_boost_speed += event.dt,
            "supersonic" => stats.time_supersonic_speed += event.dt,
            _ => stats.time_slow_speed += event.dt,
        }

        match event.height_band.as_str() {
            "low_air" => stats.time_low_air += event.dt,
            "high_air" => stats.time_high_air += event.dt,
            _ => stats.time_on_ground += event.dt,
        }

        stats.labeled_tracked_time.add(
            [
                movement_speed_band_label(&event.speed_band),
                movement_height_band_label(&event.height_band),
            ],
            event.dt,
        );
    }
}
