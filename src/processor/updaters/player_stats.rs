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
            for _ in 0..delta.max(0) {
                let event = PlayerStatEvent {
                    time: frame.time,
                    frame: frame_index,
                    player: player_id.clone(),
                    is_team_0,
                    kind,
                };
                self.current_frame_player_stat_events.push(event.clone());
                self.player_stat_events.push(event);
            }
        }

        Ok(())
    }
}
