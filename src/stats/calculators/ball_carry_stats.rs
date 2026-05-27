use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct BallCarryStats {
    pub carry_count: u32,
    pub total_carry_time: f32,
    pub total_straight_line_distance: f32,
    pub total_path_distance: f32,
    pub longest_carry_time: f32,
    pub furthest_carry_distance: f32,
    pub fastest_carry_speed: f32,
    pub carry_speed_sum: f32,
    pub average_horizontal_gap_sum: f32,
    pub average_vertical_gap_sum: f32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_event_counts: LabeledCounts,
}

impl BallCarryStats {
    fn pct_count_average(&self, value: f32) -> f32 {
        if self.carry_count == 0 {
            0.0
        } else {
            value / self.carry_count as f32
        }
    }

    pub fn average_carry_time(&self) -> f32 {
        self.pct_count_average(self.total_carry_time)
    }

    pub fn average_straight_line_distance(&self) -> f32 {
        self.pct_count_average(self.total_straight_line_distance)
    }

    pub fn average_path_distance(&self) -> f32 {
        self.pct_count_average(self.total_path_distance)
    }

    pub fn average_carry_speed(&self) -> f32 {
        self.pct_count_average(self.carry_speed_sum)
    }

    pub fn average_horizontal_gap(&self) -> f32 {
        self.pct_count_average(self.average_horizontal_gap_sum)
    }

    pub fn average_vertical_gap(&self) -> f32 {
        self.pct_count_average(self.average_vertical_gap_sum)
    }

    pub(super) fn record_event(&mut self, event: &BallCarryEvent) {
        self.labeled_event_counts
            .increment([ball_carry_kind_label(event.kind)]);
        self.carry_count = self.labeled_event_counts.total();
    }

    pub fn event_count_with_labels(&self, labels: &[StatLabel]) -> u32 {
        self.labeled_event_counts.count_matching(labels)
    }

    pub fn complete_labeled_event_counts(&self) -> LabeledCounts {
        LabeledCounts::complete_from_label_sets(
            &[&BALL_CARRY_KIND_LABELS],
            &self.labeled_event_counts,
        )
    }
}
