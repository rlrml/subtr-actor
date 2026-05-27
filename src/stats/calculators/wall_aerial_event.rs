use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct WallAerialEvent {
    pub time: f32,
    pub frame: usize,
    pub sample_time: f32,
    pub sample_frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub wall: WallAerialWall,
    pub wall_contact_time: f32,
    pub wall_contact_frame: usize,
    pub takeoff_time: f32,
    pub takeoff_frame: usize,
    pub time_since_takeoff: f32,
    pub wall_contact_position: [f32; 3],
    pub takeoff_position: [f32; 3],
    pub player_position: [f32; 3],
    pub ball_position: [f32; 3],
    pub setup_start_time: f32,
    pub setup_start_frame: usize,
    pub setup_duration: f32,
    pub ball_speed: f32,
    pub ball_speed_change: f32,
    pub goal_alignment: f32,
    pub confidence: f32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct WallAerialStats {
    pub count: u32,
    pub high_confidence_count: u32,
    pub is_last_wall_aerial: bool,
    pub last_wall_aerial_time: Option<f32>,
    pub last_wall_aerial_frame: Option<usize>,
    pub time_since_last_wall_aerial: Option<f32>,
    pub frames_since_last_wall_aerial: Option<usize>,
    pub last_confidence: Option<f32>,
    pub best_confidence: f32,
    pub cumulative_confidence: f32,
    pub cumulative_setup_duration: f32,
    pub cumulative_takeoff_to_touch_time: f32,
    pub cumulative_touch_height: f32,
}

impl WallAerialStats {
    fn average(&self, value: f32) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            value / self.count as f32
        }
    }

    pub fn average_confidence(&self) -> f32 {
        self.average(self.cumulative_confidence)
    }

    pub fn average_setup_duration(&self) -> f32 {
        self.average(self.cumulative_setup_duration)
    }

    pub fn average_takeoff_to_touch_time(&self) -> f32 {
        self.average(self.cumulative_takeoff_to_touch_time)
    }

    pub fn average_touch_height(&self) -> f32 {
        self.average(self.cumulative_touch_height)
    }
}
