use super::*;

impl MovementCalculator {
    pub fn update(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        vertical_state: &PlayerVerticalState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        if frame.dt == 0.0 {
            self.capture_positions(players);
            return Ok(());
        }

        for player in &players.players {
            self.update_player(frame, player, vertical_state, live_play);
        }

        Ok(())
    }

    fn capture_positions(&mut self, players: &PlayerFrameState) {
        for player in &players.players {
            if let Some(position) = player.position() {
                self.previous_positions
                    .insert(player.player_id.clone(), position);
            }
        }
    }

    fn update_player(
        &mut self,
        frame: &FrameInfo,
        player: &PlayerSample,
        vertical_state: &PlayerVerticalState,
        live_play: bool,
    ) {
        self.player_teams
            .insert(player.player_id.clone(), player.is_team_0);
        let Some(position) = player.position() else {
            return;
        };

        if live_play {
            self.update_player_live(frame, player, position, vertical_state);
        }
        self.previous_positions
            .insert(player.player_id.clone(), position);
    }

    fn update_player_live(
        &mut self,
        frame: &FrameInfo,
        player: &PlayerSample,
        position: glam::Vec3,
        vertical_state: &PlayerVerticalState,
    ) {
        let speed = player.speed().unwrap_or(0.0);
        let distance = self.player_distance(&player.player_id, position);
        let height_band = vertical_state
            .band_for_player(&player.player_id)
            .unwrap_or_else(|| PlayerVerticalBand::from_height(position.z));
        let classification = Self::classify_movement(speed, height_band);

        let stats = self
            .player_stats
            .entry(player.player_id.clone())
            .or_default();
        let team_stats = if player.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        apply_movement_stats(stats, frame.dt, speed, distance, classification);
        apply_movement_stats(team_stats, frame.dt, speed, distance, classification);
        self.events.push(movement_event(
            frame,
            player,
            speed,
            distance,
            classification,
        ));
    }

    fn player_distance(&self, player_id: &PlayerId, position: glam::Vec3) -> f32 {
        self.previous_positions
            .get(player_id)
            .map(|previous_position| position.distance(*previous_position))
            .unwrap_or(0.0)
    }
}
