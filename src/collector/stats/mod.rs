mod builtins;
mod collector;
mod playback;
mod resolver;
#[cfg(test)]
mod tests;
mod types;

pub use builtins::{
    builtin_stats_module_factories, builtin_stats_module_factory_by_name,
    builtin_stats_module_names,
};
pub use collector::{FrameTransform, IdentityFrameTransform, ModuleFrameTransform, StatsCollector};
pub use playback::{CapturedStatsData, CapturedStatsFrame, StatsPlaybackData, StatsPlaybackFrame};
pub use types::{CollectedStats, StatsModule, StatsModuleFactory};

pub(crate) use builtins::{builtin_module_json, builtin_playback_frame_json};
#[cfg(test)]
pub(crate) use resolver::resolve_stats_module_factories;
