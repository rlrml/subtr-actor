use super::*;

impl PositioningCalculator {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn process_sample(
        &mut self,
        frame: &FrameInfo,
        gameplay: &GameplayState,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        live_play: bool,
        possession_player_before_sample: Option<&PlayerId>,
    ) -> SubtrActorResult<()> {
        let Some(ball) = ball.sample() else {
            return Ok(());
        };
        let ball_position = ball.position();
        if frame.dt == 0.0 {
            self.remember_player_positions(ball_position, players);
            return Ok(());
        }

        let mut event_deltas = HashMap::new();
        if !events.goal_events.is_empty() {
            self.record_goal_positioning_events(
                frame,
                players,
                events,
                ball_position,
                &mut event_deltas,
            );
        }
        let demoed_players: HashSet<_> = events
            .active_demos
            .iter()
            .map(|demo| demo.victim.clone())
            .collect();

        self.process_player_samples(
            frame,
            players,
            ball_position,
            live_play,
            possession_player_before_sample,
            &demoed_players,
            &mut event_deltas,
        );
        if live_play {
            self.process_team_samples(
                frame,
                gameplay,
                players,
                ball_position,
                &demoed_players,
                &mut event_deltas,
            );
        }

        let mut frame_events: Vec<_> = event_deltas
            .into_values()
            .filter(PositioningEvent::has_delta)
            .collect();
        frame_events.sort_by(|left, right| {
            format!("{:?}", left.player).cmp(&format!("{:?}", right.player))
        });
        self.events.extend(frame_events);
        self.remember_player_positions(ball_position, players);

        Ok(())
    }
}
