use super::*;

impl BumpCalculator {
    pub fn update(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        self.update_with_fifty_fifty_state(
            frame,
            players,
            events,
            &FiftyFiftyState::default(),
            live_play,
        )
    }

    pub fn update_with_fifty_fifty_state(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        events: &FrameEventsState,
        fifty_fifty_state: &FiftyFiftyState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        for player in &players.players {
            self.player_teams
                .insert(player.player_id.clone(), player.is_team_0);
        }

        if !live_play {
            self.previous_players.clear();
            return Ok(());
        }

        if frame.dt > 0.0 && frame.dt <= BUMP_MAX_SAMPLE_DT {
            self.detect_bumps(frame, players, events, fifty_fifty_state);
        }

        self.previous_players = players
            .players
            .iter()
            .filter_map(|player| {
                Some((
                    player.player_id.clone(),
                    PreviousPlayerSample {
                        rigid_body: player.rigid_body?,
                    },
                ))
            })
            .collect();

        Ok(())
    }
}
