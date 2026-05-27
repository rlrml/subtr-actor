use super::*;

impl FlipResetTracker {
    pub(crate) fn update_post_wall_dodge_events(
        &mut self,
        processor: &dyn ProcessorView,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        self.current_frame_post_wall_dodge_events.clear();
        let current_time = frame.time;
        let player_ids: Vec<_> = processor.iter_player_ids_in_order().cloned().collect();
        self.update_wall_contact_times(processor, current_time, &player_ids);

        let dodge_edges = self.current_frame_dodge_rising_edges.clone();
        for player_id in dodge_edges {
            self.maybe_emit_post_wall_dodge(processor, current_time, frame_index, &player_id);
        }

        Ok(())
    }

    fn maybe_emit_post_wall_dodge(
        &mut self,
        processor: &dyn ProcessorView,
        current_time: f32,
        frame_index: usize,
        player_id: &PlayerId,
    ) {
        let Ok(player_rigid_body) = processor.get_normalized_player_rigid_body(player_id) else {
            return;
        };
        if Self::player_is_grounded_for_wall_sequence(&player_rigid_body) {
            return;
        }

        let Some(wall_contact_time) = self.recent_wall_contact_time.get(player_id).copied() else {
            return;
        };
        let time_since_wall_contact = current_time - wall_contact_time;
        if !(0.20..=1.10).contains(&time_since_wall_contact)
            || Self::player_is_touching_wall(&player_rigid_body)
        {
            return;
        }

        let event = PostWallDodgeEvent {
            time: current_time,
            frame: frame_index,
            player: player_id.clone(),
            is_team_0: processor.get_player_is_team_0(player_id).unwrap_or(false),
            wall_contact_time,
            time_since_wall_contact,
        };
        self.current_frame_post_wall_dodge_events
            .push(event.clone());
        self.post_wall_dodge_events.push(event);
    }
}
