#[path = "collector_config.rs"]
mod collector_config;
#[path = "collector_event.rs"]
mod collector_event;
#[path = "collector_event_process.rs"]
mod collector_event_process;
#[path = "collector_event_scaffold.rs"]
mod collector_event_scaffold;
#[path = "collector_graph.rs"]
mod collector_graph;
#[path = "collector_legacy.rs"]
mod collector_legacy;
#[path = "collector_legacy_methods.rs"]
mod collector_legacy_methods;
#[path = "collector_legacy_process.rs"]
mod collector_legacy_process;
#[path = "collector_legacy_snapshot.rs"]
mod collector_legacy_snapshot;

pub use collector_config::default_stats_timeline_config;
pub use collector_event::StatsTimelineEventCollector;
#[allow(deprecated)]
pub use collector_graph::build_timeline_graph;
pub use collector_graph::{build_legacy_timeline_graph, build_timeline_event_graph};
pub use collector_legacy::StatsTimelineCollector;

#[cfg(test)]
#[path = "collector_tests.rs"]
mod collector_tests;
