mod builtins;
mod collector;
mod resolver;
#[cfg(test)]
mod tests;
mod types;

pub use builtins::{
    builtin_stats_module_factories, builtin_stats_module_factory_by_name,
    builtin_stats_module_names,
};
pub use collector::StatsCollector;
pub use types::{CollectedStats, StatsModule, StatsModuleFactory};

#[cfg(test)]
pub(crate) use resolver::resolve_stats_module_factories;
#[cfg(test)]
pub(crate) use types::RuntimeStatsModule;
