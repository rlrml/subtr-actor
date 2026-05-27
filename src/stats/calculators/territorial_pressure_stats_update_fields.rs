use super::*;

impl TerritorialPressureCalculator {
    pub(super) fn add_session_time_fields(
        &mut self,
        team_is_team_0: bool,
        normalized_ball_y: f32,
        dt: f32,
    ) {
        if team_is_team_0 {
            self.stats.team_zero_session_time += dt;
            if normalized_ball_y > 0.0 {
                self.stats.team_zero_offensive_half_time += dt;
            }
            if normalized_ball_y > FIELD_ZONE_BOUNDARY_Y {
                self.stats.team_zero_offensive_third_time += dt;
            }
        } else {
            self.stats.team_one_session_time += dt;
            if normalized_ball_y > 0.0 {
                self.stats.team_one_offensive_half_time += dt;
            }
            if normalized_ball_y > FIELD_ZONE_BOUNDARY_Y {
                self.stats.team_one_offensive_third_time += dt;
            }
        }
    }

    pub(super) fn update_longest_session_time(&mut self, team_is_team_0: bool, duration: f32) {
        if team_is_team_0 {
            self.stats.team_zero_longest_session_time =
                self.stats.team_zero_longest_session_time.max(duration);
        } else {
            self.stats.team_one_longest_session_time =
                self.stats.team_one_longest_session_time.max(duration);
        }
    }
}
