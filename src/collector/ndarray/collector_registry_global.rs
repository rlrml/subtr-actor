use std::sync::Arc;

use super::*;

pub(super) fn global_feature_adder_from_name<F>(
    name: &str,
) -> Option<Arc<dyn FeatureAdder<F> + Send + Sync + 'static>>
where
    F: TryFrom<f32> + Send + Sync + 'static,
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    match name {
        "BallRigidBody" => Some(BallRigidBody::<F>::arc_new()),
        "BallRigidBodyNoVelocities" => Some(BallRigidBodyNoVelocities::<F>::arc_new()),
        "BallRigidBodyQuaternions" => Some(BallRigidBodyQuaternions::<F>::arc_new()),
        "BallRigidBodyQuaternionVelocities" => {
            Some(BallRigidBodyQuaternionVelocities::<F>::arc_new())
        }
        "BallRigidBodyBasis" => Some(BallRigidBodyBasis::<F>::arc_new()),
        "VelocityAddedBallRigidBodyNoVelocities" => {
            Some(VelocityAddedBallRigidBodyNoVelocities::<F>::arc_new())
        }
        "InterpolatedBallRigidBodyNoVelocities" => {
            Some(InterpolatedBallRigidBodyNoVelocities::<F>::arc_new(0.0))
        }
        "SecondsRemaining" => Some(SecondsRemaining::<F>::arc_new()),
        "CurrentTime" => Some(CurrentTime::<F>::arc_new()),
        "FrameTime" => Some(FrameTime::<F>::arc_new()),
        "ReplicatedStateName" => Some(ReplicatedStateName::<F>::arc_new()),
        "ReplicatedGameStateTimeRemaining" => {
            Some(ReplicatedGameStateTimeRemaining::<F>::arc_new())
        }
        "BallHasBeenHit" => Some(BallHasBeenHit::<F>::arc_new()),
        _ => None,
    }
}
