use super::*;

impl MatchStatsCalculator {
    pub(super) fn update_ball_ground_contact(&mut self, frame: &FrameInfo, ball: &BallFrameState) {
        if ball
            .position()
            .is_some_and(|position| position.z <= BALL_GROUND_CONTACT_MAX_Z)
        {
            self.last_ball_ground_contact_time = Some(frame.time);
        }
    }

    pub(super) fn ball_air_time_before_goal(&self, goal_time: f32) -> Option<f32> {
        self.last_ball_ground_contact_time
            .map(|ground_contact_time| (goal_time - ground_contact_time).max(0.0))
    }
}
