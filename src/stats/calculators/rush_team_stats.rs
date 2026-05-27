use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct RushTeamStats {
    pub count: u32,
    pub two_v_one_count: u32,
    pub two_v_two_count: u32,
    pub two_v_three_count: u32,
    pub three_v_one_count: u32,
    pub three_v_two_count: u32,
    pub three_v_three_count: u32,
}

impl RushStats {
    pub fn for_team(&self, is_team_zero: bool) -> RushTeamStats {
        if is_team_zero {
            RushTeamStats {
                count: self.team_zero_count,
                two_v_one_count: self.team_zero_two_v_one_count,
                two_v_two_count: self.team_zero_two_v_two_count,
                two_v_three_count: self.team_zero_two_v_three_count,
                three_v_one_count: self.team_zero_three_v_one_count,
                three_v_two_count: self.team_zero_three_v_two_count,
                three_v_three_count: self.team_zero_three_v_three_count,
            }
        } else {
            RushTeamStats {
                count: self.team_one_count,
                two_v_one_count: self.team_one_two_v_one_count,
                two_v_two_count: self.team_one_two_v_two_count,
                two_v_three_count: self.team_one_two_v_three_count,
                three_v_one_count: self.team_one_three_v_one_count,
                three_v_two_count: self.team_one_three_v_two_count,
                three_v_three_count: self.team_one_three_v_three_count,
            }
        }
    }
}
