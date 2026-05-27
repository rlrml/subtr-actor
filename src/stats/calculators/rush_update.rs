use super::*;

impl RushCalculator {
    pub(super) fn maybe_start_rush(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        active_team_before_sample: Option<bool>,
        current_team_is_team_0: Option<bool>,
    ) {
        let Some(attacking_team_is_team_0) = current_team_is_team_0 else {
            return;
        };
        if active_team_before_sample == Some(attacking_team_is_team_0) {
            return;
        }

        if let Some((attackers, defenders)) =
            self.rush_numbers(ball, players, events, attacking_team_is_team_0)
        {
            self.active_rush = Some(ActiveRush {
                start_time: frame.time,
                start_frame: frame.frame_number,
                last_time: frame.time,
                last_frame: frame.frame_number,
                is_team_0: attacking_team_is_team_0,
                attackers,
                defenders,
                counted: false,
            });
        }
    }

    pub(super) fn update_rush_state(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        active_team_before_sample: Option<bool>,
        current_team_is_team_0: Option<bool>,
    ) {
        self.update_active_rush(frame, ball, players, events, current_team_is_team_0);
        if self.active_rush.is_none() {
            self.maybe_start_rush(
                frame,
                ball,
                players,
                events,
                active_team_before_sample,
                current_team_is_team_0,
            );
        }
    }

    pub fn finish_calculation(&mut self) -> SubtrActorResult<()> {
        self.finalize_active_rush();
        Ok(())
    }
}
