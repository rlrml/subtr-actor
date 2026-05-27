use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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
