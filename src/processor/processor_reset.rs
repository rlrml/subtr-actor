use super::*;

impl<'a> ReplayProcessor<'a> {
    pub fn reset(&mut self) {
        self.ball_actor_id = None;
        self.actor_state = ActorStateModeler::new();
        self.boost_pad_events = Vec::new();
        self.current_frame_boost_pad_events = Vec::new();
        self.boost_pad_pickup_sequence_times = HashMap::new();
        self.touch_events = Vec::new();
        self.current_frame_touch_events = Vec::new();
        self.dodge_refreshed_events = Vec::new();
        self.current_frame_dodge_refreshed_events = Vec::new();
        self.dodge_refreshed_counters = HashMap::new();
        self.goal_events = Vec::new();
        self.current_frame_goal_events = Vec::new();
        self.player_stat_events = Vec::new();
        self.current_frame_player_stat_events = Vec::new();
        self.player_stat_counters = HashMap::new();
        self.demolishes = Vec::new();
        self.known_demolishes = Vec::new();
        self.demolish_format = None;
        self.kickoff_phase_active_last_frame = false;
    }
}
