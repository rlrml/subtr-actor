use super::*;

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
        pct(self.time_slow_speed, self.tracked_time)
    }

    pub fn boost_speed_pct(&self) -> f32 {
        pct(self.time_boost_speed, self.tracked_time)
    }

    pub fn supersonic_speed_pct(&self) -> f32 {
        pct(self.time_supersonic_speed, self.tracked_time)
    }

    pub fn on_ground_pct(&self) -> f32 {
        pct(self.time_on_ground, self.tracked_time)
    }

    pub fn low_air_pct(&self) -> f32 {
        pct(self.time_low_air, self.tracked_time)
    }

    pub fn high_air_pct(&self) -> f32 {
        pct(self.time_high_air, self.tracked_time)
    }

    pub fn tracked_time_with_labels(&self, labels: &[StatLabel]) -> f32 {
        self.labeled_tracked_time.sum_matching(labels)
    }
}

fn pct(value: f32, tracked_time: f32) -> f32 {
    if tracked_time == 0.0 {
        0.0
    } else {
        value * 100.0 / tracked_time
    }
}
