use super::*;

pub(crate) struct MustyFlickCandidateSpin {
    pub(super) pitch_rate: f32,
    pub(super) other_spin: f32,
}

impl MustyFlickCandidateSpin {
    pub(super) fn new(
        rigid_body: &boxcars::RigidBody,
        player_rotation: glam::Quat,
    ) -> Option<Self> {
        let local_angular_velocity = local_angular_velocity(rigid_body, player_rotation);
        let pitch_rate = local_angular_velocity.y.abs();
        let other_spin = local_angular_velocity
            .x
            .abs()
            .max(local_angular_velocity.z.abs());
        if pitch_rate < MUSTY_MIN_PITCH_RATE
            || pitch_rate < other_spin * MUSTY_MIN_PITCH_DOMINANCE_RATIO
        {
            return None;
        }

        Some(Self {
            pitch_rate,
            other_spin,
        })
    }
}

fn local_angular_velocity(
    rigid_body: &boxcars::RigidBody,
    player_rotation: glam::Quat,
) -> glam::Vec3 {
    let angular_velocity = rigid_body
        .angular_velocity
        .as_ref()
        .map(vec_to_glam)
        .unwrap_or(glam::Vec3::ZERO);
    player_rotation.inverse() * angular_velocity
}
