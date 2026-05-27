use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct BumpPlayerStats {
    pub bumps_inflicted: u32,
    pub bumps_taken: u32,
    pub team_bumps_inflicted: u32,
    pub team_bumps_taken: u32,
    pub last_bump_time: Option<f32>,
    pub last_bump_frame: Option<usize>,
    pub last_bump_strength: Option<f32>,
    pub max_bump_strength: f32,
    pub cumulative_bump_strength: f32,
}

impl BumpPlayerStats {
    pub fn average_bump_strength(&self) -> f32 {
        if self.bumps_inflicted == 0 {
            0.0
        } else {
            self.cumulative_bump_strength / self.bumps_inflicted as f32
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct BumpTeamStats {
    pub bumps_inflicted: u32,
    pub team_bumps_inflicted: u32,
}
