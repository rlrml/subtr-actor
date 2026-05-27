use super::*;

#[derive(Clone, Default)]
pub(crate) struct SaLiveEventGenerator {
    pub(crate) touch_state: TouchStateCalculator,
    pub(crate) live_play_tracker: subtr_actor::LivePlayTracker,
    pub(crate) dodge_refresh_counters: Vec<(RemoteId, i32)>,
    pub(crate) active_demos: Vec<SaActiveDemo>,
    pub(crate) known_demolishes: Vec<(DemoEventSample, usize)>,
    pub(crate) boost_pad_pickup_sequence_times: HashMap<(String, u8), f32>,
    pub(crate) last_goal_event: Option<GoalEvent>,
}

#[derive(Clone, Default)]
pub(crate) struct SaLiveEventHistory {
    pub(crate) demo_events: Vec<DemolishInfo>,
    pub(crate) boost_pad_events: Vec<BoostPadEvent>,
    pub(crate) touch_events: Vec<TouchEvent>,
    pub(crate) dodge_refreshed_events: Vec<DodgeRefreshedEvent>,
    pub(crate) player_stat_events: Vec<PlayerStatEvent>,
    pub(crate) goal_events: Vec<GoalEvent>,
}

impl SaLiveEventHistory {
    pub(crate) fn append_frame_events(&mut self, events: &FrameEventsState) {
        self.demo_events.extend(events.demo_events.iter().cloned());
        self.boost_pad_events
            .extend(events.boost_pad_events.iter().cloned());
        self.touch_events
            .extend(events.touch_events.iter().cloned());
        self.dodge_refreshed_events
            .extend(events.dodge_refreshed_events.iter().cloned());
        self.player_stat_events
            .extend(events.player_stat_events.iter().cloned());
        self.goal_events.extend(events.goal_events.iter().cloned());
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SaActiveDemo {
    pub(crate) sample: DemoEventSample,
    pub(crate) expires_at: f32,
}

#[path = "live_event_types.rs"]
mod live_event_types;
pub(crate) use live_event_types::*;
#[path = "live_demolish_attribute.rs"]
mod live_demolish_attribute;
pub(crate) use live_demolish_attribute::*;
