use super::*;

impl FlipResetTracker {
    pub(crate) fn wall_sequence_scale_factor(player_rigid_body: &boxcars::RigidBody) -> f32 {
        if player_rigid_body
            .location
            .x
            .abs()
            .max(player_rigid_body.location.y.abs())
            < 200.0
        {
            100.0
        } else {
            1.0
        }
    }

    pub(crate) fn player_is_grounded_for_wall_sequence(
        player_rigid_body: &boxcars::RigidBody,
    ) -> bool {
        player_rigid_body.location.z * Self::wall_sequence_scale_factor(player_rigid_body) <= 80.0
    }

    pub(crate) fn player_is_touching_wall(player_rigid_body: &boxcars::RigidBody) -> bool {
        let scale_factor = Self::wall_sequence_scale_factor(player_rigid_body);
        let location = &player_rigid_body.location;
        let x = location.x.abs() * scale_factor;
        let y = location.y.abs() * scale_factor;
        let z = location.z * scale_factor;
        z >= 120.0 && (x >= 3600.0 || y >= 5000.0)
    }
}
