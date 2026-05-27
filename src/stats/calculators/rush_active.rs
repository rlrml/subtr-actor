use super::*;

impl RushCalculator {
    pub(super) fn update_active_rush(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        current_team_is_team_0: Option<bool>,
    ) {
        let Some(active_team_is_team_0) = self.active_rush.as_ref().map(|rush| rush.is_team_0)
        else {
            return;
        };

        let active_continues = current_team_is_team_0 == Some(active_team_is_team_0)
            && self
                .rush_numbers(ball, players, events, active_team_is_team_0)
                .is_some();
        if active_continues {
            self.extend_active_rush(frame);
            return;
        }

        self.finalize_active_rush();
    }

    fn extend_active_rush(&mut self, frame: &FrameInfo) {
        if let Some(active_rush) = self.active_rush.as_mut() {
            active_rush.last_time = frame.time;
            active_rush.last_frame = frame.frame_number;
        }
        if let Some(mut active_rush) = self.active_rush.take() {
            self.record_active_rush(&mut active_rush);
            self.active_rush = Some(active_rush);
        }
    }
}
