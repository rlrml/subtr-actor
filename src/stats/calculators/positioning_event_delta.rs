use super::*;

impl PositioningEvent {
    pub(crate) fn has_delta(&self) -> bool {
        [
            self.active_game_time,
            self.tracked_time,
            self.sum_distance_to_teammates,
            self.sum_distance_to_ball,
            self.sum_distance_to_ball_has_possession,
            self.time_has_possession,
            self.sum_distance_to_ball_no_possession,
            self.time_no_possession,
            self.time_demolished,
            self.time_no_teammates,
            self.time_most_back,
            self.time_most_forward,
            self.time_mid_role,
            self.time_other_role,
            self.time_defensive_zone,
            self.time_neutral_zone,
            self.time_offensive_zone,
            self.time_defensive_half,
            self.time_offensive_half,
            self.time_closest_to_ball,
            self.time_farthest_from_ball,
            self.time_behind_ball,
            self.time_level_with_ball,
            self.time_in_front_of_ball,
        ]
        .into_iter()
        .any(|value| value != 0.0)
            || self.times_caught_ahead_of_play_on_conceded_goals != 0
    }
}
