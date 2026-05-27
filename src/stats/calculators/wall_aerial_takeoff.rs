use super::*;
use wall_aerial_wall::wall_aerial_setup_wall_for_position;

impl WallAerialCalculator {
    pub(super) fn update_wall_contacts_and_takeoffs(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
    ) {
        for player in &players.players {
            let Some(position) = player.position() else {
                continue;
            };
            self.update_wall_contact(frame, player, position);
            self.update_wall_takeoff(frame, player, position);
        }
    }

    fn update_wall_contact(
        &mut self,
        frame: &FrameInfo,
        player: &PlayerSample,
        position: glam::Vec3,
    ) {
        let Some(wall) = wall_aerial_setup_wall_for_position(position) else {
            return;
        };
        let controlled_setup = self
            .active_wall_controls
            .get(&player.player_id)
            .and_then(Self::completed_setup)
            .or_else(|| {
                self.recent_wall_contacts
                    .get(&player.player_id)
                    .and_then(|contact| contact.controlled_setup.clone())
            });
        self.recent_wall_contacts.insert(
            player.player_id.clone(),
            RecentWallContact {
                player: player.player_id.clone(),
                is_team_0: player.is_team_0,
                wall,
                time: frame.time,
                frame: frame.frame_number,
                position,
                controlled_setup,
            },
        );
    }

    fn update_wall_takeoff(
        &mut self,
        frame: &FrameInfo,
        player: &PlayerSample,
        position: glam::Vec3,
    ) {
        if wall_aerial_setup_wall_for_position(position).is_some() && player_is_on_wall(position) {
            return;
        }
        if position.z < WALL_AERIAL_MIN_TOUCH_PLAYER_Z {
            self.armed_aerials.remove(&player.player_id);
            return;
        }
        let Some(contact) = self.recent_wall_contacts.remove(&player.player_id) else {
            return;
        };
        if !self.wall_takeoff_is_valid(frame, player, &contact) {
            return;
        }
        self.armed_aerials.insert(
            player.player_id.clone(),
            ArmedWallAerial::new(contact, frame, position),
        );
    }

    fn wall_takeoff_is_valid(
        &self,
        frame: &FrameInfo,
        player: &PlayerSample,
        contact: &RecentWallContact,
    ) -> bool {
        frame.time - contact.time <= WALL_AERIAL_MAX_WALL_CONTACT_TO_TAKEOFF_SECONDS
            && contact.controlled_setup.is_some()
            && !self.armed_aerials.contains_key(&player.player_id)
            && !self
                .recent_event_times
                .get(&player.player_id)
                .is_some_and(|time| frame.time - time < WALL_AERIAL_MIN_SECONDS_BETWEEN_ATTEMPTS)
    }

    pub(super) fn prune_armed_aerials(&mut self, current_time: f32) {
        self.armed_aerials.retain(|_, armed| {
            current_time - armed.takeoff_time <= WALL_AERIAL_MAX_TAKEOFF_TO_TOUCH_SECONDS
        });
    }
}
