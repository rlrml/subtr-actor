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

pub(crate) fn serialize_to_json_value<T: Serialize + ?Sized>(value: &T) -> SubtrActorResult<Value> {
    serde_json::to_value(value).map_err(|error| {
        SubtrActorError::new(SubtrActorErrorVariant::StatsSerializationError(
            error.to_string(),
        ))
    })
}

pub(crate) struct CollectedStatsModule {
    pub(crate) name: &'static str,
    pub(crate) value: Value,
}

pub struct CollectedStats {
    pub replay_meta: ReplayMeta,
    pub(crate) modules: Vec<CollectedStatsModule>,
}

struct CollectedStatsModules<'a>(&'a [CollectedStatsModule]);

impl Serialize for CollectedStatsModules<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for module in self.0 {
            map.serialize_entry(module.name, &module.value)?;
        }
        map.end()
    }
}

impl CollectedStats {
    pub fn module_names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.modules.iter().map(|module| module.name)
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
