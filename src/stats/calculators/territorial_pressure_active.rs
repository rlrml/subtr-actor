use super::*;

impl TerritorialPressureCalculator {
    pub(super) fn update_active_session(
        &mut self,
        frame: &FrameInfo,
        ball_y: f32,
        possession_state: &PossessionState,
    ) {
        let Some(mut active) = self.active.take() else {
            return;
        };

        let normalized_ball_y = Self::normalized_ball_y(active.team_is_team_0, ball_y);
        update_active_sample(&mut active, frame.dt, normalized_ball_y);
        self.add_session_time(active.team_is_team_0, normalized_ball_y, frame.dt);
        self.update_longest_session_time(active.team_is_team_0, active.duration);
        self.update_relief_state(&mut active, frame.dt, normalized_ball_y, possession_state);

        let relieved = active.confirmed_relief_time >= self.config.confirmed_relief_grace_seconds
            || active.relief_time >= self.config.relief_grace_seconds;

        self.active = Some(active);
        if relieved {
            self.end_active_session(frame, TerritorialPressureEndReason::Relieved);
        }
    }

    fn update_relief_state(
        &self,
        active: &mut ActiveTerritorialPressureSession,
        dt: f32,
        normalized_ball_y: f32,
        possession_state: &PossessionState,
    ) {
        if normalized_ball_y > self.config.neutral_zone_half_width_y {
            active.relief_time = 0.0;
            active.confirmed_relief_time = 0.0;
        } else {
            active.relief_time += dt;
            if possession_state.active_team_before_sample == Some(!active.team_is_team_0) {
                active.confirmed_relief_time += dt;
            } else {
                active.confirmed_relief_time = 0.0;
            }
        }
    }
}

fn update_active_sample(
    active: &mut ActiveTerritorialPressureSession,
    dt: f32,
    normalized_ball_y: f32,
) {
    active.duration += dt;
    if normalized_ball_y > 0.0 {
        active.offensive_half_time += dt;
    }
    if normalized_ball_y > FIELD_ZONE_BOUNDARY_Y {
        active.offensive_third_time += dt;
    }
}
