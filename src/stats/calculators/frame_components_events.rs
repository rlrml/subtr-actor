use super::super::DemoEventSample;
use crate::{
    BoostPadEvent, DemolishInfo, DodgeRefreshedEvent, GoalEvent, PlayerStatEvent, TouchEvent,
};

#[derive(Debug, Clone, Default)]
pub struct FrameEventsState {
    pub active_demos: Vec<DemoEventSample>,
    pub demo_events: Vec<DemolishInfo>,
    pub boost_pad_events: Vec<BoostPadEvent>,
    pub touch_events: Vec<TouchEvent>,
    pub dodge_refreshed_events: Vec<DodgeRefreshedEvent>,
    pub player_stat_events: Vec<PlayerStatEvent>,
    pub goal_events: Vec<GoalEvent>,
}
