use super::*;

impl<'a> ReplayProcessor<'a> {
    fn build_touch_event(
        &self,
        frame: &boxcars::Frame,
        frame_index: usize,
        team_is_team_0: bool,
    ) -> TouchEvent {
        TouchEvent {
            touch_id: None,
            time: frame.time,
            frame: frame_index,
            team_is_team_0,
            player: None,
            player_position: None,
            closest_approach_distance: None,
            contact_local_ball_position: None,
            contact_local_hitbox_point: None,
            contact_world_hitbox_point: None,
            dodge_contact: false,
        }
    }

    /// Records replay-replicated ball touch events.
    pub(crate) fn update_touch_events(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        self.current_frame_touch_events.clear();
        let hit_team_num_key = self.cached_object_ids.ball_hit_team_num;

        for update in frame
            .updated_actors
            .iter()
            .filter(|update| hit_team_num_key == Some(update.object_id))
        {
            let boxcars::Attribute::Byte(team_num) = update.attribute else {
                continue;
            };
            let team_is_team_0 = match team_num {
                0 => true,
                1 => false,
                _ => continue,
            };
            let event = self.build_touch_event(frame, frame_index, team_is_team_0);
            self.current_frame_touch_events.push(event.clone());
            self.touch_events.push(event);
        }

        Ok(())
    }

    /// Detects dodge-refresh counter increments for players in the current frame.
    pub(crate) fn update_dodge_refreshed_events(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        self.current_frame_dodge_refreshed_events.clear();
        let dodges_refreshed_counter_key = self.cached_object_ids.dodges_refreshed_counter;

        let Some(dodges_refreshed_counter_key) = dodges_refreshed_counter_key else {
            return Ok(());
        };

        for update in &frame.updated_actors {
            if update.object_id != dodges_refreshed_counter_key {
                continue;
            }
            let boxcars::Attribute::Int(counter_value) = update.attribute else {
                continue;
            };
            let Some(player_id) = self.get_player_id_from_car_id(&update.actor_id).ok() else {
                continue;
            };
            let previous_value = self
                .dodge_refreshed_counters
                .get(&player_id)
                .copied()
                .unwrap_or(counter_value);
            self.dodge_refreshed_counters
                .insert(player_id.clone(), counter_value);
            let delta = counter_value - previous_value;
            if delta <= 0 {
                continue;
            }

            let is_team_0 = self.get_player_is_team_0(&player_id).unwrap_or(false);
            for offset in 0..delta {
                let event = DodgeRefreshedEvent {
                    time: frame.time,
                    frame: frame_index,
                    player: player_id.clone(),
                    player_position: self
                        .get_normalized_player_position(&player_id)
                        .map(|position| vec_to_glam(&position).to_array()),
                    is_team_0,
                    counter_value: previous_value + offset + 1,
                };
                self.current_frame_dodge_refreshed_events
                    .push(event.clone());
                self.dodge_refreshed_events.push(event);
            }
        }

        Ok(())
    }
}
