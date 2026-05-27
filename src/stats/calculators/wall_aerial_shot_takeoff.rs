use super::*;

impl WallAerialShotCalculator {
    pub(super) fn update_wall_contacts_and_takeoffs(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
    ) {
        for player in &players.players {
            self.update_player_wall_state(frame, player);
        }
    }

    fn update_player_wall_state(&mut self, frame: &FrameInfo, player: &PlayerSample) {
        let Some(position) = player.position() else {
            return;
        };
        if position.z <= WALL_AERIAL_SHOT_GROUND_CONTACT_MAX_PLAYER_Z {
            self.recent_wall_contacts.remove(&player.player_id);
            self.armed_shots.remove(&player.player_id);
            return;
        }

        if let Some(wall) = wall_aerial_wall_for_position(position) {
            self.record_wall_contact(frame, player, wall, position);
            return;
        }

        self.maybe_arm_shot(frame, &player.player_id, position);
    }

    fn record_wall_contact(
        &mut self,
        frame: &FrameInfo,
        player: &PlayerSample,
        wall: WallAerialWall,
        position: glam::Vec3,
    ) {
        self.recent_wall_contacts.insert(
            player.player_id.clone(),
            RecentWallContact {
                player: player.player_id.clone(),
                is_team_0: player.is_team_0,
                wall,
                time: frame.time,
                frame: frame.frame_number,
                position,
            },
        );
    }

    fn maybe_arm_shot(&mut self, frame: &FrameInfo, player_id: &PlayerId, position: glam::Vec3) {
        if position.z < WALL_AERIAL_MIN_TOUCH_PLAYER_Z {
            self.armed_shots.remove(player_id);
            return;
        }
        if self.armed_shots.contains_key(player_id) {
            return;
        }

        let Some(contact) = self.recent_wall_contacts.remove(player_id) else {
            return;
        };
        if frame.time - contact.time <= WALL_AERIAL_SHOT_MAX_WALL_CONTACT_TO_TAKEOFF_SECONDS {
            self.armed_shots
                .insert(player_id.clone(), contact.armed(frame, position));
        }
    }

    pub(super) fn prune_armed_shots(&mut self, current_time: f32) {
        self.armed_shots.retain(|_, armed| {
            current_time - armed.takeoff_time <= WALL_AERIAL_SHOT_MAX_TAKEOFF_TO_SHOT_SECONDS
        });
    }
}
