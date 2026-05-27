use super::*;

impl PositioningCalculator {
    pub(crate) fn remember_player_positions(
        &mut self,
        ball_position: glam::Vec3,
        players: &PlayerFrameState,
    ) {
        self.previous_ball_position = Some(ball_position);
        for player in &players.players {
            if let Some(position) = player.position() {
                self.previous_player_positions
                    .insert(player.player_id.clone(), position);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn process_player_samples(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        ball_position: glam::Vec3,
        live_play: bool,
        possession_player_before_sample: Option<&PlayerId>,
        demoed_players: &HashSet<PlayerId>,
        event_deltas: &mut HashMap<PlayerId, PositioningEvent>,
    ) {
        for player in &players.players {
            if live_play && demoed_players.contains(&player.player_id) {
                self.record_demoed_player_sample(frame, player, event_deltas);
                continue;
            }

            let Some(position) = player.position() else {
                continue;
            };
            if live_play {
                self.record_live_player_sample(
                    frame,
                    player,
                    position,
                    ball_position,
                    possession_player_before_sample,
                    event_deltas,
                );
            }
        }
    }

    fn record_demoed_player_sample(
        &mut self,
        frame: &FrameInfo,
        player: &PlayerSample,
        event_deltas: &mut HashMap<PlayerId, PositioningEvent>,
    ) {
        let stats = self
            .player_stats
            .entry(player.player_id.clone())
            .or_default();
        stats.active_game_time += frame.dt;
        stats.time_demolished += frame.dt;

        let delta = Self::event_delta(event_deltas, frame, &player.player_id, player.is_team_0);
        delta.active_game_time += frame.dt;
        delta.time_demolished += frame.dt;
    }
}
