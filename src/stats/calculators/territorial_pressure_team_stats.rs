use super::territorial_pressure_team_counts::territorial_pressure_team_counts;
use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct TerritorialPressureTeamStats {
    pub tracked_time: f32,
    pub session_count: u32,
    pub opponent_session_count: u32,
    pub session_time: f32,
    pub opponent_session_time: f32,
    pub offensive_half_time: f32,
    pub offensive_third_time: f32,
    pub longest_session_time: f32,
    pub opponent_longest_session_time: f32,
    pub average_session_time: f32,
}

impl TerritorialPressureStats {
    pub fn for_team(&self, is_team_zero: bool) -> TerritorialPressureTeamStats {
        let stats = territorial_pressure_team_counts(self, is_team_zero);
        TerritorialPressureTeamStats {
            tracked_time: self.tracked_time,
            session_count: stats.session_count,
            opponent_session_count: stats.opponent_session_count,
            session_time: stats.session_time,
            opponent_session_time: stats.opponent_session_time,
            offensive_half_time: stats.offensive_half_time,
            offensive_third_time: stats.offensive_third_time,
            longest_session_time: stats.longest_session_time,
            opponent_longest_session_time: stats.opponent_longest_session_time,
            average_session_time: average_session_time(stats.session_count, stats.session_time),
        }
    }
}

fn average_session_time(session_count: u32, session_time: f32) -> f32 {
    if session_count == 0 {
        0.0
    } else {
        session_time / session_count as f32
    }
}
