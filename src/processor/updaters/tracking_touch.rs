use crate::{
    touch_candidate_rank, PlayerId, ReplayProcessor, SubtrActorResult, TouchEvent,
    BALL_HIT_TEAM_NUM_KEY,
};

impl<'a> ReplayProcessor<'a> {
    fn estimate_touching_player(
        &self,
        touch_team_is_team_0: bool,
        target_time: f32,
    ) -> Option<(PlayerId, f32)> {
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
                            .map(|rank| (player_id.clone(), rank))
                    })
            })
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .and_then(|(player_id, (closest_distance, _current_distance))| {
                (closest_distance <= TOUCH_PLAYER_DISTANCE_THRESHOLD)
                    .then_some((player_id, closest_distance))
            })
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
            let dodge_contact = estimated_player
                .as_ref()
                .is_some_and(|(player, _)| self.get_dodge_active(player).unwrap_or(0) % 2 == 1);
            let event = TouchEvent {
                time: frame.time,
                frame: frame_index,
                team_is_team_0,
                player: estimated_player.as_ref().map(|(player, _)| player.clone()),
                closest_approach_distance: estimated_player.map(|(_, distance)| distance),
                dodge_contact,
            };
            self.current_frame_touch_events.push(event.clone());
            self.touch_events.push(event);
        }

        Ok(())
    }
}
