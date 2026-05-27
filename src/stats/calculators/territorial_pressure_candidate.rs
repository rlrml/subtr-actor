use super::*;

impl TerritorialPressureCalculator {
    pub(super) fn candidate_sample(
        team_is_team_0: bool,
        frame: &FrameInfo,
        normalized_ball_y: f32,
    ) -> CandidateTerritorialPressureSession {
        CandidateTerritorialPressureSession {
            team_is_team_0,
            start_time: frame.time,
            start_frame: frame.frame_number,
            duration: frame.dt,
            offensive_half_time: if normalized_ball_y > 0.0 {
                frame.dt
            } else {
                0.0
            },
            offensive_third_time: if normalized_ball_y > FIELD_ZONE_BOUNDARY_Y {
                frame.dt
            } else {
                0.0
            },
        }
    }

    pub(super) fn update_candidate(&mut self, frame: &FrameInfo, ball_y: f32) {
        let Some(team_is_team_0) = self.pressure_team_for_ball_y(ball_y) else {
            self.candidate = None;
            return;
        };
        let normalized_ball_y = Self::normalized_ball_y(team_is_team_0, ball_y);

        if self
            .candidate
            .as_ref()
            .is_none_or(|candidate| candidate.team_is_team_0 != team_is_team_0)
        {
            self.candidate = Some(Self::candidate_sample(
                team_is_team_0,
                frame,
                normalized_ball_y,
            ));
        } else if let Some(candidate) = &mut self.candidate {
            update_candidate_sample(candidate, frame.dt, normalized_ball_y);
        }

        if self.candidate_should_start() {
            let candidate = self.candidate.take().expect("candidate exists");
            self.start_session(candidate);
        }
    }
}

fn update_candidate_sample(
    candidate: &mut CandidateTerritorialPressureSession,
    dt: f32,
    normalized_ball_y: f32,
) {
    candidate.duration += dt;
    if normalized_ball_y > 0.0 {
        candidate.offensive_half_time += dt;
    }
    if normalized_ball_y > FIELD_ZONE_BOUNDARY_Y {
        candidate.offensive_third_time += dt;
    }
}
