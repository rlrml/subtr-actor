use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct AirDribbleStats {
    pub count: u32,
    #[serde(default)]
    pub ground_to_air_count: u32,
    #[serde(default)]
    pub wall_to_air_count: u32,
    #[serde(default)]
    pub total_touch_count: u32,
    #[serde(default)]
    pub max_touch_count: u32,
    pub total_time: f32,
    pub total_straight_line_distance: f32,
    pub total_path_distance: f32,
    pub longest_time: f32,
    pub furthest_distance: f32,
    pub fastest_speed: f32,
    pub speed_sum: f32,
    pub average_horizontal_gap_sum: f32,
    pub average_vertical_gap_sum: f32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_event_counts: LabeledCounts,
}

impl AirDribbleStats {
    fn count_average(&self, value: f32) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            value / self.count as f32
        }
    }

    pub fn average_time(&self) -> f32 {
        self.count_average(self.total_time)
    }

    pub fn average_straight_line_distance(&self) -> f32 {
        self.count_average(self.total_straight_line_distance)
    }

    pub fn average_path_distance(&self) -> f32 {
        self.count_average(self.total_path_distance)
    }

    pub fn average_speed(&self) -> f32 {
        self.count_average(self.speed_sum)
    }

    pub fn average_touch_count(&self) -> f32 {
        self.count_average(self.total_touch_count as f32)
    }

    pub fn average_horizontal_gap(&self) -> f32 {
        self.count_average(self.average_horizontal_gap_sum)
    }

    pub fn average_vertical_gap(&self) -> f32 {
        self.count_average(self.average_vertical_gap_sum)
    }

    pub(super) fn record_event(&mut self, event: &BallCarryEvent) {
        if let Some(origin) = event.air_dribble_origin {
            self.labeled_event_counts
                .increment([air_dribble_origin_label(origin)]);
        }
        self.sync_legacy_counts();
    }

    pub fn event_count_with_labels(&self, labels: &[StatLabel]) -> u32 {
        self.labeled_event_counts.count_matching(labels)
    }

    pub fn complete_labeled_event_counts(&self) -> LabeledCounts {
        LabeledCounts::complete_from_label_sets(
            &[&AIR_DRIBBLE_ORIGIN_LABELS],
            &self.labeled_event_counts,
        )
    }

    fn sync_legacy_counts(&mut self) {
        self.count = self.labeled_event_counts.total();
        self.ground_to_air_count = self
            .event_count_with_labels(&[air_dribble_origin_label(AirDribbleOrigin::GroundToAir)]);
        self.wall_to_air_count =
            self.event_count_with_labels(&[air_dribble_origin_label(AirDribbleOrigin::WallToAir)]);
    }
}
