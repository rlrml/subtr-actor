use super::*;

impl TerritorialPressureCalculator {
    pub(super) fn pressure_team_label(team_is_team_0: bool) -> StatLabel {
        StatLabel::new(
            "pressure_team",
            if team_is_team_0 {
                "team_zero"
            } else {
                "team_one"
            },
        )
    }

    pub(super) fn territory_label(normalized_ball_y: f32) -> StatLabel {
        if normalized_ball_y > FIELD_ZONE_BOUNDARY_Y {
            StatLabel::new("territory", "offensive_third")
        } else if normalized_ball_y > 0.0 {
            StatLabel::new("territory", "offensive_half")
        } else {
            StatLabel::new("territory", "relief")
        }
    }

    pub(super) fn add_session_count(&mut self, team_is_team_0: bool) {
        if team_is_team_0 {
            self.stats.team_zero_session_count += 1;
        } else {
            self.stats.team_one_session_count += 1;
        }
        self.stats
            .labeled_session_counts
            .increment([Self::pressure_team_label(team_is_team_0)]);
    }

    pub(super) fn add_session_time(
        &mut self,
        team_is_team_0: bool,
        normalized_ball_y: f32,
        dt: f32,
    ) {
        self.add_session_time_fields(team_is_team_0, normalized_ball_y, dt);
        self.stats.labeled_time.add(
            [
                Self::pressure_team_label(team_is_team_0),
                Self::territory_label(normalized_ball_y),
            ],
            dt,
        );
    }
}
