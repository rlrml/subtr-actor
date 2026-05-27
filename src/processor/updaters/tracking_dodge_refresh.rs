use crate::{DodgeRefreshedEvent, ReplayProcessor, SubtrActorResult};

impl<'a> ReplayProcessor<'a> {
    /// Detects dodge-refresh counter increments for players in the current frame.
    pub(crate) fn update_dodge_refreshed_events(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        self.current_frame_dodge_refreshed_events.clear();
        let Some(dodges_refreshed_counter_key) = self.cached_object_ids.dodges_refreshed_counter
        else {
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
