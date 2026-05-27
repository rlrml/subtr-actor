use std::sync::Arc;

use super::*;

pub(super) fn player_feature_adder_from_name<F>(
    name: &str,
) -> Option<Arc<dyn PlayerFeatureAdder<F> + Send + Sync + 'static>>
where
    F: TryFrom<f32> + Send + Sync + 'static,
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    match name {
        "PlayerRigidBody" => Some(PlayerRigidBody::<F>::arc_new()),
        "PlayerRigidBodyNoVelocities" => Some(PlayerRigidBodyNoVelocities::<F>::arc_new()),
        "PlayerRigidBodyQuaternions" => Some(PlayerRigidBodyQuaternions::<F>::arc_new()),
        "PlayerRigidBodyQuaternionVelocities" => {
            Some(PlayerRigidBodyQuaternionVelocities::<F>::arc_new())
        }
        "PlayerRigidBodyBasis" => Some(PlayerRigidBodyBasis::<F>::arc_new()),
        "PlayerRelativeBallPosition" => Some(PlayerRelativeBallPosition::<F>::arc_new()),
        "PlayerRelativeBallVelocity" => Some(PlayerRelativeBallVelocity::<F>::arc_new()),
        "PlayerLocalRelativeBallPosition" => Some(PlayerLocalRelativeBallPosition::<F>::arc_new()),
        "PlayerLocalRelativeBallVelocity" => Some(PlayerLocalRelativeBallVelocity::<F>::arc_new()),
        "VelocityAddedPlayerRigidBodyNoVelocities" => {
            Some(VelocityAddedPlayerRigidBodyNoVelocities::<F>::arc_new())
        }
        "InterpolatedPlayerRigidBodyNoVelocities" => {
            Some(InterpolatedPlayerRigidBodyNoVelocities::<F>::arc_new(0.003))
        }
        "PlayerBallDistance" | "PlayerDistanceToBall" => Some(PlayerBallDistance::<F>::arc_new()),
        "PlayerBoost" => Some(PlayerBoost::<F>::arc_new()),
        "PlayerJump" => Some(PlayerJump::<F>::arc_new()),
        "PlayerAnyJump" => Some(PlayerAnyJump::<F>::arc_new()),
        "PlayerDodgeRefreshed" => Some(PlayerDodgeRefreshed::<F>::arc_new()),
        "PlayerDemolishedBy" => Some(PlayerDemolishedBy::<F>::arc_new()),
        _ => None,
    }
}
