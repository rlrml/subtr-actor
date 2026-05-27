use std::sync::Arc;

use super::*;

#[path = "collector_registry_global.rs"]
mod collector_registry_global;
#[path = "collector_registry_player.rs"]
mod collector_registry_player;

use collector_registry_global::global_feature_adder_from_name;
use collector_registry_player::player_feature_adder_from_name;

impl<F> NDArrayCollector<F>
where
    F: TryFrom<f32> + Send + Sync + 'static,
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    /// Builds a collector from the registered string names of feature adders.
    pub fn from_strings_typed(fa_names: &[&str], pfa_names: &[&str]) -> SubtrActorResult<Self> {
        let feature_adders: Vec<Arc<dyn FeatureAdder<F> + Send + Sync>> = fa_names
            .iter()
            .map(|name| {
                global_feature_adder_from_name(name).ok_or_else(|| {
                    SubtrActorError::new(SubtrActorErrorVariant::UnknownFeatureAdderName(
                        name.to_string(),
                    ))
                })
            })
            .collect::<SubtrActorResult<Vec<_>>>()?;
        let player_feature_adders: Vec<Arc<dyn PlayerFeatureAdder<F> + Send + Sync>> = pfa_names
            .iter()
            .map(|name| {
                player_feature_adder_from_name(name).ok_or_else(|| {
                    SubtrActorError::new(SubtrActorErrorVariant::UnknownFeatureAdderName(
                        name.to_string(),
                    ))
                })
            })
            .collect::<SubtrActorResult<Vec<_>>>()?;
        Ok(Self::new(feature_adders, player_feature_adders))
    }
}
