use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PassPlayerStats {
    pub completed_pass_count: u32,
    pub received_pass_count: u32,
    pub total_pass_distance: f32,
    pub total_pass_advance: f32,
    pub longest_pass_distance: f32,
    pub is_last_completed_pass: bool,
    pub last_completed_pass_time: Option<f32>,
    pub last_completed_pass_frame: Option<usize>,
    pub time_since_last_completed_pass: Option<f32>,
    pub frames_since_last_completed_pass: Option<usize>,
}

impl PassPlayerStats {
    pub fn average_pass_distance(&self) -> f32 {
        if self.completed_pass_count == 0 {
            0.0
        } else {
            self.total_pass_distance / self.completed_pass_count as f32
        }
    }

    pub fn average_pass_advance(&self) -> f32 {
        if self.completed_pass_count == 0 {
            0.0
        } else {
            self.total_pass_advance / self.completed_pass_count as f32
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PassTeamStats {
    pub completed_pass_count: u32,
    pub total_pass_distance: f32,
    pub total_pass_advance: f32,
    pub longest_pass_distance: f32,
}

impl PassTeamStats {
    pub fn average_pass_distance(&self) -> f32 {
        if self.completed_pass_count == 0 {
            0.0
        } else {
            self.total_pass_distance / self.completed_pass_count as f32
        }
    }

    pub fn average_pass_advance(&self) -> f32 {
        if self.completed_pass_count == 0 {
            0.0
        } else {
            self.total_pass_advance / self.completed_pass_count as f32
        }
    }
}
