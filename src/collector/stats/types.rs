use std::sync::Arc;

use erased_serde::serialize_trait_object;
use serde::ser::{SerializeMap, SerializeStruct};
use serde::{Serialize, Serializer};
use serde_json::Value;

use crate::*;

pub trait StatsModule: StatsReducer + erased_serde::Serialize {
    fn name(&self) -> &'static str;

    fn playback_frame_json(&self, _replay_meta: &ReplayMeta) -> SubtrActorResult<Option<Value>> {
        Ok(None)
    }

    fn playback_config_json(&self) -> SubtrActorResult<Option<Value>> {
        Ok(None)
    }
}

serialize_trait_object!(StatsModule);

pub trait StatsModuleFactory: Send + Sync {
    fn key(&self) -> String {
        self.name().to_owned()
    }

    fn name(&self) -> &'static str;

    fn dependencies(&self) -> Vec<Arc<dyn StatsModuleFactory>> {
        Vec::new()
    }

    fn required_derived_signals(&self) -> Vec<DerivedSignalId> {
        self.build().required_derived_signals()
    }

    fn build(&self) -> Box<dyn StatsModule>;
}

struct DynStatsModule<'a>(&'a dyn StatsModule);

impl Serialize for DynStatsModule<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        erased_serde::serialize(self.0, serializer).map_err(serde::ser::Error::custom)
    }
}

pub(crate) fn stats_module_to_json_value(module: &dyn StatsModule) -> SubtrActorResult<Value> {
    serialize_to_json_value(&DynStatsModule(module))
}

pub(crate) fn serialize_to_json_value<T: Serialize + ?Sized>(value: &T) -> SubtrActorResult<Value> {
    serde_json::to_value(value).map_err(|error| {
        SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
            error.to_string(),
        ))
    })
}

pub(crate) struct RuntimeStatsModule {
    pub(crate) emit: bool,
    pub(crate) module: Box<dyn StatsModule>,
}

pub struct CollectedStats {
    pub replay_meta: ReplayMeta,
    pub(crate) modules: Vec<RuntimeStatsModule>,
}

struct CollectedStatsModules<'a>(&'a [RuntimeStatsModule]);

impl Serialize for CollectedStatsModules<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let emitted = self.0.iter().filter(|module| module.emit).count();
        let mut map = serializer.serialize_map(Some(emitted))?;
        for module in self.0.iter().filter(|module| module.emit) {
            map.serialize_entry(
                module.module.name(),
                &DynStatsModule(module.module.as_ref()),
            )?;
        }
        map.end()
    }
}

impl CollectedStats {
    pub fn module_names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.modules
            .iter()
            .filter(|module| module.emit)
            .map(|module| module.module.name())
    }
}

impl Serialize for CollectedStats {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("CollectedStats", 2)?;
        state.serialize_field("replay_meta", &self.replay_meta)?;
        state.serialize_field("modules", &CollectedStatsModules(&self.modules))?;
        state.end()
    }
}
