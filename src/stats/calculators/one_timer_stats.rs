use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct OneTimerPlayerStats {
    pub count: u32,
    pub total_ball_speed: f32,
    pub fastest_ball_speed: f32,
    pub total_pass_distance: f32,
    pub is_last_one_timer: bool,
    pub last_one_timer_time: Option<f32>,
    pub last_one_timer_frame: Option<usize>,
    pub time_since_last_one_timer: Option<f32>,
    pub frames_since_last_one_timer: Option<usize>,
}

impl OneTimerPlayerStats {
    pub fn average_ball_speed(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_ball_speed / self.count as f32
        }
    }

    pub fn average_pass_distance(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_pass_distance / self.count as f32
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct OneTimerTeamStats {
    pub count: u32,
    pub total_ball_speed: f32,
    pub fastest_ball_speed: f32,
}

impl OneTimerTeamStats {
    pub fn average_ball_speed(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.total_ball_speed / self.count as f32
        }
    }
}
