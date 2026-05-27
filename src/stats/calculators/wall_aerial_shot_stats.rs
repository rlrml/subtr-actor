use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct WallAerialShotStats {
    pub count: u32,
    pub high_confidence_count: u32,
    pub is_last_wall_aerial_shot: bool,
    pub last_wall_aerial_shot_time: Option<f32>,
    pub last_wall_aerial_shot_frame: Option<usize>,
    pub time_since_last_wall_aerial_shot: Option<f32>,
    pub frames_since_last_wall_aerial_shot: Option<usize>,
    pub last_confidence: Option<f32>,
    pub best_confidence: f32,
    pub cumulative_confidence: f32,
    pub cumulative_takeoff_to_shot_time: f32,
    pub cumulative_shot_height: f32,
}

impl WallAerialShotStats {
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

    pub fn average_takeoff_to_shot_time(&self) -> f32 {
        self.average(self.cumulative_takeoff_to_shot_time)
    }

    pub fn average_shot_height(&self) -> f32 {
        self.average(self.cumulative_shot_height)
    }
}
