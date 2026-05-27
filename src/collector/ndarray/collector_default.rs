use super::*;

impl NDArrayCollector<f32> {
    /// Builds an `f32` collector from the registered string names of feature adders.
    pub fn from_strings(fa_names: &[&str], pfa_names: &[&str]) -> SubtrActorResult<Self> {
        Self::from_strings_typed(fa_names, pfa_names)
    }
}

impl<F: TryFrom<f32> + Send + Sync + 'static> Default for NDArrayCollector<F>
where
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    fn default() -> Self {
        NDArrayCollector::new(
            vec![BallRigidBody::arc_new()],
            vec![
                PlayerRigidBody::arc_new(),
                PlayerBoost::arc_new(),
                PlayerAnyJump::arc_new(),
            ],
        )
    }
}
