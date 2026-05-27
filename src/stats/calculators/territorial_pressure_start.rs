use super::*;

impl TerritorialPressureCalculator {
    pub(super) fn candidate_should_start(&self) -> bool {
        self.candidate.as_ref().is_some_and(|candidate| {
            candidate.duration >= self.config.min_establish_seconds
                || candidate.offensive_third_time >= self.config.min_establish_third_seconds
        })
    }

    pub(super) fn start_session(&mut self, candidate: CandidateTerritorialPressureSession) {
        self.add_session_count(candidate.team_is_team_0);
        self.add_session_time(
            candidate.team_is_team_0,
            1.0,
            candidate.offensive_half_time - candidate.offensive_third_time,
        );
        self.add_session_time(
            candidate.team_is_team_0,
            FIELD_ZONE_BOUNDARY_Y + 1.0,
            candidate.offensive_third_time,
        );
        self.update_longest_session_time(candidate.team_is_team_0, candidate.duration);
        self.active = Some(ActiveTerritorialPressureSession {
            team_is_team_0: candidate.team_is_team_0,
            start_time: candidate.start_time,
            start_frame: candidate.start_frame,
            duration: candidate.duration,
            offensive_half_time: candidate.offensive_half_time,
            offensive_third_time: candidate.offensive_third_time,
            relief_time: 0.0,
            confirmed_relief_time: 0.0,
        });
    }
}
