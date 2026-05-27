use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct TouchStats {
    pub touch_count: u32,
    pub control_touch_count: u32,
    pub medium_hit_count: u32,
    pub hard_hit_count: u32,
    pub aerial_touch_count: u32,
    pub high_aerial_touch_count: u32,
    #[serde(default)]
    pub wall_touch_count: u32,
    pub is_last_touch: bool,
    pub last_touch_time: Option<f32>,
    pub last_touch_frame: Option<usize>,
    pub time_since_last_touch: Option<f32>,
    pub frames_since_last_touch: Option<usize>,
    pub last_ball_speed_change: Option<f32>,
    pub max_ball_speed_change: f32,
    pub cumulative_ball_speed_change: f32,
    #[serde(default)]
    pub total_ball_travel_distance: f32,
    #[serde(default)]
    pub total_ball_advance_distance: f32,
    #[serde(default)]
    pub total_ball_retreat_distance: f32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_touch_counts: LabeledCounts,
}

impl TouchStats {
    pub fn average_ball_speed_change(&self) -> f32 {
        if self.touch_count == 0 {
            0.0
        } else {
            self.cumulative_ball_speed_change / self.touch_count as f32
        }
    }

    pub fn touch_count_with_labels(&self, labels: &[StatLabel]) -> u32 {
        self.labeled_touch_counts.count_matching(labels)
    }

    pub fn dodge_touch_count(&self) -> u32 {
        self.touch_count_with_labels(&[StatLabel::new("dodge_state", "dodge")])
    }

    pub fn dodge_hit_count(&self) -> u32 {
        self.touch_count_with_labels(&[
            StatLabel::new("dodge_state", "dodge"),
            StatLabel::new("kind", "medium_hit"),
        ]) + self.touch_count_with_labels(&[
            StatLabel::new("dodge_state", "dodge"),
            StatLabel::new("kind", "hard_hit"),
        ])
    }
}
