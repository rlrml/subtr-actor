use super::*;

impl RotationCalculator {
    pub fn update(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        if frame.dt == 0.0 {
            return Ok(());
        }

        let Some(ball) = ball.sample() else {
            self.reset_trackers();
            self.emit_inactive_player_events(frame, players);
            return Ok(());
        };

        if !live_play || !events.goal_events.is_empty() {
            self.reset_trackers();
            self.emit_inactive_player_events(frame, players);
            return Ok(());
        }

        let demoed_players: HashSet<_> = events
            .active_demos
            .iter()
            .map(|demo| demo.victim.clone())
            .collect();
        let ball_position = ball.position();
        self.update_team(
            true,
            frame,
            gameplay,
            ball_position,
            players,
            &demoed_players,
        );
        self.update_team(
            false,
            frame,
            gameplay,
            ball_position,
            players,
            &demoed_players,
        );
        Ok(())
    }
}
