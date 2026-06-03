use super::*;

#[derive(Debug, Clone)]
struct TouchingPlayerEstimate {
    player_id: PlayerId,
    closest_approach_distance: f32,
    player_position: boxcars::Vector3f,
}

impl<'a> ReplayProcessor<'a> {
    fn estimate_touching_player(
        &self,
        touch_team_is_team_0: bool,
        target_time: f32,
    ) -> Option<TouchingPlayerEstimate> {
        const TOUCH_PLAYER_DISTANCE_THRESHOLD: f32 = 700.0;

        let ball_rigid_body = self
            .get_velocity_applied_ball_rigid_body(target_time)
            .ok()?;
        self.iter_player_ids_in_order()
            .filter(|player_id| {
                self.get_player_is_team_0(player_id).ok() == Some(touch_team_is_team_0)
            })
            .filter_map(|player_id| {
                self.get_velocity_applied_player_rigid_body(player_id, target_time)
                    .ok()
                    .and_then(|rigid_body| {
                        touch_candidate_rank(&ball_rigid_body, &rigid_body)
                            .map(|rank| (player_id.clone(), rigid_body.location, rank))
                    })
            })
            .min_by(|(_, _, a), (_, _, b)| a.partial_cmp(b).unwrap())
            .and_then(
                |(player_id, player_position, (closest_distance, _current_distance))| {
                    (closest_distance <= TOUCH_PLAYER_DISTANCE_THRESHOLD).then_some(
                        TouchingPlayerEstimate {
                            player_id,
                            closest_approach_distance: closest_distance,
                            player_position,
                        },
                    )
                },
            )
    }

    /// Detects ball touch events and estimates the responsible player when possible.
    pub(crate) fn update_touch_events(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        self.current_frame_touch_events.clear();
        let hit_team_num_key = self.required_cached_object_id(
            self.cached_object_ids.ball_hit_team_num,
            BALL_HIT_TEAM_NUM_KEY,
        )?;

        for update in &frame.updated_actors {
            if update.object_id != hit_team_num_key {
                continue;
            }

            let boxcars::Attribute::Byte(team_num) = update.attribute else {
                continue;
            };
            let team_is_team_0 = match team_num {
                0 => true,
                1 => false,
                _ => continue,
            };
            let estimated_player = self.estimate_touching_player(team_is_team_0, frame.time);
            let dodge_contact = estimated_player.as_ref().is_some_and(|estimate| {
                self.get_dodge_active(&estimate.player_id).unwrap_or(0) % 2 == 1
            });
            let event = TouchEvent {
                time: frame.time,
                frame: frame_index,
                team_is_team_0,
                player: estimated_player
                    .as_ref()
                    .map(|estimate| estimate.player_id.clone()),
                player_position: estimated_player
                    .as_ref()
                    .map(|estimate| estimate.player_position),
                closest_approach_distance: estimated_player
                    .map(|estimate| estimate.closest_approach_distance),
                dodge_contact,
            };
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
