use super::*;

impl SaLiveEventGenerator {
    pub(crate) fn explicit_demolish_events(
        &mut self,
        frame: &FrameInfo,
        events: &[SaDemolishEvent],
    ) -> Vec<SaDemolishEvent> {
        let mut accepted_events = Vec::new();
        for event in events {
            let (frame_number, _) = event_frame_and_time(frame, event.timing);
            self.known_demolishes.retain(|(_, known_frame)| {
                frame_number.abs_diff(*known_frame) < MAX_DEMOLISH_KNOWN_FRAMES_PASSED
            });
            let sample = DemoEventSample {
                attacker: player_id(event.attacker_index),
                victim: player_id(event.victim_index),
            };
            if demolish_is_known(&self.known_demolishes, &sample, frame_number) {
                continue;
            }
            self.known_demolishes.push((sample, frame_number));
            accepted_events.push(*event);
        }
        accepted_events
    }

    pub(crate) fn sync_active_demos(
        &mut self,
        frame: &FrameInfo,
        events: &[SaDemolishEvent],
    ) -> Vec<DemoEventSample> {
        self.active_demos
            .retain(|demo| demo.expires_at + f32::EPSILON >= frame.time);

        for event in events {
            let sample = DemoEventSample {
                attacker: player_id(event.attacker_index),
                victim: player_id(event.victim_index),
            };
            let active_duration_seconds = if event.active_duration_seconds.is_finite()
                && event.active_duration_seconds > 0.0
            {
                event.active_duration_seconds
            } else {
                0.0
            };
            let (_, event_time) = event_frame_and_time(frame, event.timing);
            let expires_at = event_time + active_duration_seconds;
            if expires_at + f32::EPSILON < frame.time {
                continue;
            }
            if let Some(active_demo) = self.active_demos.iter_mut().find(|active_demo| {
                active_demo.sample.attacker == sample.attacker
                    && active_demo.sample.victim == sample.victim
            }) {
                active_demo.expires_at = expires_at;
            } else {
                self.active_demos.push(SaActiveDemo { sample, expires_at });
            }
        }

        self.active_demos
            .iter()
            .map(|active_demo| active_demo.sample.clone())
            .collect()
    }
}
