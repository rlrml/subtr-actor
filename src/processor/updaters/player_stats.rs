use super::*;

impl<'a> ReplayProcessor<'a> {
    /// Detects shot, save, and assist counter increments for players.
    pub(crate) fn update_player_stat_events(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        self.current_frame_player_stat_events.clear();
        let match_shots_key = self.get_object_id_for_key(MATCH_SHOTS_KEY).ok().copied();
        let match_saves_key = self.get_object_id_for_key(MATCH_SAVES_KEY).ok().copied();
        let match_assists_key = self.get_object_id_for_key(MATCH_ASSISTS_KEY).ok().copied();

        for update in &frame.updated_actors {
            let (kind, new_value) = match update.attribute {
                boxcars::Attribute::Int(value) if Some(update.object_id) == match_shots_key => {
                    (PlayerStatEventKind::Shot, value)
                }
                boxcars::Attribute::Int(value) if Some(update.object_id) == match_saves_key => {
                    (PlayerStatEventKind::Save, value)
                }
                boxcars::Attribute::Int(value) if Some(update.object_id) == match_assists_key => {
                    (PlayerStatEventKind::Assist, value)
                }
                _ => continue,
            };
            let Some(player_id) = self.get_player_id_from_actor_id(&update.actor_id).ok() else {
                continue;
            };
            let Ok(is_team_0) = self.get_player_is_team_0(&player_id) else {
                continue;
            };
            let previous_value = self
                .player_stat_counters
                .get(&(player_id.clone(), kind))
                .copied()
                .unwrap_or(0);
            let delta = new_value - previous_value;
            self.player_stat_counters
                .insert((player_id.clone(), kind), new_value);
            let shot = (kind == PlayerStatEventKind::Shot)
                .then(|| self.shot_event_metadata(frame.time, &player_id, is_team_0))
                .flatten();
            for _ in 0..delta.max(0) {
                let event = PlayerStatEvent {
                    time: frame.time,
                    frame: frame_index,
                    player: player_id.clone(),
                    player_position: self.get_normalized_player_position(&player_id),
                    is_team_0,
                    kind,
                    shot: shot.clone(),
                };
                self.current_frame_player_stat_events.push(event.clone());
                self.player_stat_events.push(event);
            }
        }

        Ok(())
    }

    fn shot_event_metadata(
        &self,
        time: f32,
        player_id: &PlayerId,
        is_team_0: bool,
    ) -> Option<ShotEventMetadata> {
        let ball_body = self.get_interpolated_ball_rigid_body(time, 0.0).ok()?;
        let player_body = self
            .get_interpolated_player_rigid_body(player_id, time, 0.0)
            .ok();
        Some(ShotEventMetadata::from_rigid_bodies(
            is_team_0,
            &ball_body,
            player_body.as_ref(),
        ))
    }
}
