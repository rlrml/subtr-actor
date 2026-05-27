use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct CenterPlayerStats {
    pub count: u32,
    pub total_ball_travel_distance: f32,
    pub total_ball_advance_distance: f32,
    pub total_lateral_centering_distance: f32,
    pub longest_center_distance: f32,
    pub is_last_center: bool,
    pub last_center_time: Option<f32>,
    pub last_center_frame: Option<usize>,
    pub time_since_last_center: Option<f32>,
    pub frames_since_last_center: Option<usize>,
}

impl CenterPlayerStats {
    pub fn average_ball_travel_distance(&self) -> f32 {
        average(self.total_ball_travel_distance, self.count)
    }

    pub fn average_ball_advance_distance(&self) -> f32 {
        average(self.total_ball_advance_distance, self.count)
    }

    pub fn average_lateral_centering_distance(&self) -> f32 {
        average(self.total_lateral_centering_distance, self.count)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct CenterTeamStats {
    pub count: u32,
    pub total_ball_travel_distance: f32,
    pub total_ball_advance_distance: f32,
    pub total_lateral_centering_distance: f32,
    pub longest_center_distance: f32,
}

impl CenterTeamStats {
    pub fn average_ball_travel_distance(&self) -> f32 {
        average(self.total_ball_travel_distance, self.count)
    }

    pub fn average_ball_advance_distance(&self) -> f32 {
        average(self.total_ball_advance_distance, self.count)
    }

    pub fn average_lateral_centering_distance(&self) -> f32 {
        average(self.total_lateral_centering_distance, self.count)
    }
}

fn average(total: f32, count: u32) -> f32 {
    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}
