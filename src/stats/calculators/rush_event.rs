use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct RushEvent {
    pub start_time: f32,
    pub start_frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub is_team_0: bool,
    pub attackers: usize,
    pub defenders: usize,
}

impl RushEvent {
    pub(super) fn labels(&self) -> [StatLabel; 3] {
        [
            rush_team_label(self.is_team_0),
            rush_attackers_label(self.attackers),
            rush_defenders_label(self.defenders),
        ]
    }
}
