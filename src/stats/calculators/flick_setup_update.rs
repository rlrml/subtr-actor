use super::*;

impl FlickCalculator {
    pub(super) fn update_control_setups(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_events: &[TouchEvent],
        controlling_player: Option<&PlayerId>,
    ) {
        let Some(ball) = ball.sample() else {
            let player_ids: Vec<_> = self.active_setups.keys().cloned().collect();
            for player_id in player_ids {
                self.finish_setup(&player_id);
            }
            return;
        };

        let mut observed_players = HashSet::new();
        for player in &players.players {
            let Some(observation) = Self::control_observation(ball, player, controlling_player)
            else {
                continue;
            };
            observed_players.insert(player.player_id.clone());
            let setup = self
                .active_setups
                .entry(player.player_id.clone())
                .or_insert_with(|| ActiveFlickSetup {
                    is_team_0: player.is_team_0,
                    start_time: (frame.time - frame.dt).max(0.0),
                    start_frame: frame.frame_number.saturating_sub(1),
                    last_time: frame.time,
                    last_frame: frame.frame_number,
                    duration: frame.dt.max(0.0),
                    horizontal_gap_integral: observation.horizontal_gap * frame.dt.max(0.0),
                    vertical_gap_integral: observation.vertical_gap * frame.dt.max(0.0),
                    touch_count: 0,
                });

            if setup.last_frame != frame.frame_number {
                setup.last_time = frame.time;
                setup.last_frame = frame.frame_number;
                setup.duration += frame.dt.max(0.0);
                setup.horizontal_gap_integral += observation.horizontal_gap * frame.dt.max(0.0);
                setup.vertical_gap_integral += observation.vertical_gap * frame.dt.max(0.0);
            }
        }

        for touch_event in touch_events {
            let Some(player_id) = touch_event.player.as_ref() else {
                continue;
            };
            if let Some(setup) = self.active_setups.get_mut(player_id) {
                setup.touch_count += 1;
            }
        }

        let active_ids: Vec<_> = self.active_setups.keys().cloned().collect();
        for player_id in active_ids {
            if !observed_players.contains(&player_id) {
                self.finish_setup(&player_id);
            }
        }
    }
}
