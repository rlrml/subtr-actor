use super::*;

impl RushCalculator {
    #[allow(clippy::too_many_arguments)]
    pub fn update_parts(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        possession_state: &PossessionState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        if !live_play_state.is_live_play || gameplay.kickoff_phase_active() {
            self.finalize_active_rush();
            return Ok(());
        }

        self.update_rush_state(
            frame,
            ball,
            players,
            events,
            possession_state.active_team_before_sample,
            possession_state.current_team_is_team_0,
        );

        Ok(())
    }
}
