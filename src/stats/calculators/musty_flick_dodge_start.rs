use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RecentDodgeStart {
    pub(super) time: f32,
    pub(super) frame: usize,
    pub(super) forward_z: f32,
}

impl MustyFlickCalculator {
    pub(super) fn track_dodge_starts(&mut self, frame: &FrameInfo, players: &PlayerFrameState) {
        for player in &players.players {
            let was_dodge_active = self
                .previous_dodge_active
                .insert(player.player_id.clone(), player.dodge_active)
                .unwrap_or(false);
            if !player.dodge_active || was_dodge_active {
                continue;
            }

            let Some(rigid_body) = player.rigid_body.as_ref() else {
                continue;
            };
            let forward = quat_to_glam(&rigid_body.rotation) * glam::Vec3::X;
            self.recent_dodge_starts.insert(
                player.player_id.clone(),
                RecentDodgeStart {
                    time: frame.time,
                    frame: frame.frame_number,
                    forward_z: forward.z,
                },
            );
        }
    }

    pub(super) fn prune_recent_dodge_starts(&mut self, current_time: f32) {
        self.recent_dodge_starts
            .retain(|_, dodge| current_time - dodge.time <= MUSTY_MAX_DODGE_TO_TOUCH_SECONDS);
    }
}
