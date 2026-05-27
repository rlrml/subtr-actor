use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RecentCeilingContact {
    pub(super) time: f32,
    pub(super) frame: usize,
    pub(super) position: [f32; 3],
    pub(super) roof_alignment: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct CeilingContactObservation {
    pub(super) position: glam::Vec3,
    pub(super) roof_alignment: f32,
}

impl CeilingShotCalculator {
    pub(super) fn ceiling_contact_observation(
        player: &PlayerSample,
    ) -> Option<CeilingContactObservation> {
        let rigid_body = player.rigid_body.as_ref()?;
        let position = player.position()?;
        let gap_to_ceiling = SOCCAR_CEILING_Z - position.z;
        if !(0.0..=CEILING_CONTACT_MAX_GAP).contains(&gap_to_ceiling) {
            return None;
        }

        let up = quat_to_glam(&rigid_body.rotation) * glam::Vec3::Z;
        let roof_alignment = (-up).dot(glam::Vec3::Z);
        if roof_alignment < CEILING_CONTACT_MIN_ROOF_ALIGNMENT {
            return None;
        }

        Some(CeilingContactObservation {
            position,
            roof_alignment,
        })
    }

    pub(super) fn update_recent_ceiling_contacts(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
    ) {
        for player in &players.players {
            let observation = Self::ceiling_contact_observation(player);
            let Some(observation) = observation else {
                continue;
            };

            self.recent_ceiling_contacts.insert(
                player.player_id.clone(),
                RecentCeilingContact {
                    time: frame.time,
                    frame: frame.frame_number,
                    position: observation.position.to_array(),
                    roof_alignment: observation.roof_alignment,
                },
            );
        }
    }

    pub(super) fn prune_recent_ceiling_contacts(&mut self, current_time: f32) {
        self.recent_ceiling_contacts.retain(|_, contact| {
            current_time - contact.time <= CEILING_SHOT_MAX_TOUCH_AFTER_CONTACT_SECONDS
        });
    }
}
