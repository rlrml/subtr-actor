use super::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GroundContact {
    pub(super) time: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DodgeStart {
    pub(super) time: f32,
    pub(super) ground_contact: GroundContact,
}

impl HalfVolleyCalculator {
    pub(super) fn update_player_movement_state(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
    ) {
        for player in &players.players {
            self.update_ground_contact(frame, player);
            self.update_dodge_start(frame, player);
        }

        self.prune_movement_state(frame.time);
    }

    fn update_ground_contact(&mut self, frame: &FrameInfo, player: &PlayerSample) {
        if player
            .position()
            .is_some_and(|position| position.z <= PLAYER_GROUND_Z_THRESHOLD)
        {
            self.last_ground_contacts
                .insert(player.player_id.clone(), GroundContact { time: frame.time });
        }
    }

    fn update_dodge_start(&mut self, frame: &FrameInfo, player: &PlayerSample) {
        let was_dodge_active = self
            .previous_dodge_active
            .insert(player.player_id.clone(), player.dodge_active)
            .unwrap_or(false);
        if !player.dodge_active || was_dodge_active {
            return;
        }

        if let Some(ground_contact) = self.last_ground_contacts.get(&player.player_id) {
            self.recent_dodge_starts.insert(
                player.player_id.clone(),
                DodgeStart {
                    time: frame.time,
                    ground_contact: ground_contact.clone(),
                },
            );
        }
    }

    fn prune_movement_state(&mut self, current_time: f32) {
        self.recent_dodge_starts.retain(|_, dodge_start| {
            current_time - dodge_start.time <= HALF_VOLLEY_MAX_DODGE_TO_TOUCH_SECONDS
        });
        self.last_ground_contacts.retain(|_, ground_contact| {
            current_time - ground_contact.time
                <= HALF_VOLLEY_MAX_GROUND_TO_DODGE_SECONDS + HALF_VOLLEY_MAX_DODGE_TO_TOUCH_SECONDS
        });
    }
}
