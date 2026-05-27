use super::*;
use crate::stats::calculators::*;
use crate::*;

#[path = "stats_timeline_events_capture.rs"]
mod stats_timeline_events_capture;
#[path = "stats_timeline_events_dependencies.rs"]
mod stats_timeline_events_dependencies;
#[path = "stats_timeline_events_goal_sources.rs"]
mod stats_timeline_events_goal_sources;
#[path = "stats_timeline_events_mechanic_build.rs"]
mod stats_timeline_events_mechanic_build;
#[path = "stats_timeline_events_mechanic_build_moments.rs"]
mod stats_timeline_events_mechanic_build_moments;
#[path = "stats_timeline_events_mechanic_build_spans.rs"]
mod stats_timeline_events_mechanic_build_spans;
#[path = "stats_timeline_events_mechanic_build_spans_extra.rs"]
mod stats_timeline_events_mechanic_build_spans_extra;
#[path = "stats_timeline_events_mechanic_build_spans_more.rs"]
mod stats_timeline_events_mechanic_build_spans_more;
#[path = "stats_timeline_events_mechanic_build_spans_tail.rs"]
mod stats_timeline_events_mechanic_build_spans_tail;
#[path = "stats_timeline_events_mechanic_helpers.rs"]
mod stats_timeline_events_mechanic_helpers;
#[path = "stats_timeline_events_mechanic_sources.rs"]
mod stats_timeline_events_mechanic_sources;
#[path = "stats_timeline_events_mechanic_types.rs"]
mod stats_timeline_events_mechanic_types;
#[path = "stats_timeline_events_node.rs"]
mod stats_timeline_events_node;
#[path = "stats_timeline_events_sources.rs"]
mod stats_timeline_events_sources;

use stats_timeline_events_goal_sources::GoalTagEventSources;
use stats_timeline_events_mechanic_build::build_mechanic_events;
use stats_timeline_events_mechanic_build_spans_extra::*;
use stats_timeline_events_mechanic_build_spans_more::*;
use stats_timeline_events_mechanic_build_spans_tail::*;
use stats_timeline_events_mechanic_helpers::{
    mechanic_event_start_time, mechanic_event_text_property, mechanic_event_unsigned_property,
    moment_mechanic_event, span_mechanic_event,
};
use stats_timeline_events_mechanic_sources::MechanicEventSources;
pub use stats_timeline_events_mechanic_types::STATS_TIMELINE_MECHANIC_KINDS;
use stats_timeline_events_mechanic_types::*;
pub use stats_timeline_events_node::{StatsTimelineEventsNode, StatsTimelineEventsState};
use stats_timeline_events_sources::StatsTimelineEventSources;

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(StatsTimelineEventsNode::new())
}
