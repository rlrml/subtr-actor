use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PositioningEvent {
    pub time: f32,
    pub frame: usize,
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    pub is_team_0: bool,
    pub active_game_time: f32,
    pub tracked_time: f32,
    pub sum_distance_to_teammates: f32,
    pub sum_distance_to_ball: f32,
    pub sum_distance_to_ball_has_possession: f32,
    pub time_has_possession: f32,
    pub sum_distance_to_ball_no_possession: f32,
    pub time_no_possession: f32,
    pub time_demolished: f32,
    pub time_no_teammates: f32,
    pub time_most_back: f32,
    pub time_most_forward: f32,
    pub time_mid_role: f32,
    pub time_other_role: f32,
    #[serde(rename = "time_defensive_third")]
    pub time_defensive_zone: f32,
    #[serde(rename = "time_neutral_third")]
    pub time_neutral_zone: f32,
    #[serde(rename = "time_offensive_third")]
    pub time_offensive_zone: f32,
    pub time_defensive_half: f32,
    pub time_offensive_half: f32,
    pub time_closest_to_ball: f32,
    pub time_farthest_from_ball: f32,
    pub time_behind_ball: f32,
    pub time_level_with_ball: f32,
    pub time_in_front_of_ball: f32,
    pub times_caught_ahead_of_play_on_conceded_goals: u32,
}

impl PositioningEvent {
    pub(crate) fn new(frame: &FrameInfo, player: PlayerId, is_team_0: bool) -> Self {
        Self {
            time: frame.time,
            frame: frame.frame_number,
            player,
            is_team_0,
            active_game_time: 0.0,
            tracked_time: 0.0,
            sum_distance_to_teammates: 0.0,
            sum_distance_to_ball: 0.0,
            sum_distance_to_ball_has_possession: 0.0,
            time_has_possession: 0.0,
            sum_distance_to_ball_no_possession: 0.0,
            time_no_possession: 0.0,
            time_demolished: 0.0,
            time_no_teammates: 0.0,
            time_most_back: 0.0,
            time_most_forward: 0.0,
            time_mid_role: 0.0,
            time_other_role: 0.0,
            time_defensive_zone: 0.0,
            time_neutral_zone: 0.0,
            time_offensive_zone: 0.0,
            time_defensive_half: 0.0,
            time_offensive_half: 0.0,
            time_closest_to_ball: 0.0,
            time_farthest_from_ball: 0.0,
            time_behind_ball: 0.0,
            time_level_with_ball: 0.0,
            time_in_front_of_ball: 0.0,
            times_caught_ahead_of_play_on_conceded_goals: 0,
        }
    }
}
