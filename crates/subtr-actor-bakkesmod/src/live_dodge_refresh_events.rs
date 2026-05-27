use super::*;

impl SaLiveEventGenerator {
    pub(crate) fn explicit_dodge_refreshed_events(
        &mut self,
        frame: &FrameInfo,
        events: &[SaDodgeRefreshedEvent],
    ) -> Vec<DodgeRefreshedEvent> {
        let mut dodge_refreshed_events = Vec::new();
        for event in events {
            let player = player_id(event.player_index);
            if find_counter(&self.dodge_refresh_counters, &player)
                .is_some_and(|previous| event.counter_value <= previous)
            {
                continue;
            }
            set_counter(
                &mut self.dodge_refresh_counters,
                player.clone(),
                event.counter_value,
            );
            let (frame_number, time) = event_frame_and_time(frame, event.timing);
            dodge_refreshed_events.push(DodgeRefreshedEvent {
                time,
                frame: frame_number,
                player,
                is_team_0: event.is_team_0 != 0,
                counter_value: event.counter_value,
            });
        }
        dodge_refreshed_events
    }
}
