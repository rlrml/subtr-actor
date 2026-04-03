use serde::Serialize;

use crate::*;

/// Heuristic or otherwise derived replay enrichments attached to [`ReplayData`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ReplayDataHeuristicData {
    pub flip_reset_events: Vec<FlipResetEvent>,
    pub post_wall_dodge_events: Vec<PostWallDodgeEvent>,
    pub flip_reset_followup_dodge_events: Vec<FlipResetFollowupDodgeEvent>,
}

/// Optional collector outputs produced alongside [`ReplayDataCollector`] in the
/// same processor pass before being assembled into the final [`ReplayData`].
#[derive(Debug, Clone, Default)]
pub struct ReplayDataSupplementalData {
    pub boost_pads: Vec<ResolvedBoostPad>,
    pub heuristic_data: ReplayDataHeuristicData,
}

impl ReplayDataSupplementalData {
    pub fn from_flip_reset_tracker(tracker: FlipResetTracker) -> Self {
        let (flip_reset_events, post_wall_dodge_events, flip_reset_followup_dodge_events) =
            tracker.into_events();
        Self {
            boost_pads: Vec::new(),
            heuristic_data: ReplayDataHeuristicData {
                flip_reset_events,
                post_wall_dodge_events,
                flip_reset_followup_dodge_events,
            },
        }
    }

    pub fn with_boost_pads(mut self, boost_pads: Vec<ResolvedBoostPad>) -> Self {
        self.boost_pads = boost_pads;
        self
    }
}
