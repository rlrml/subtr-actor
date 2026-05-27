use super::*;

impl SaLiveEventGenerator {
    pub(crate) fn explicit_boost_pad_events(
        &mut self,
        frame: &FrameInfo,
        events: &[SaBoostPadEvent],
    ) -> Vec<BoostPadEvent> {
        let mut boost_pad_events = Vec::new();
        for event in events {
            let (frame_number, time) = event_frame_and_time(frame, event.timing);
            let pad_id = event.pad_id.to_string();
            let kind = match event.kind {
                SaBoostPadEventKind::PickedUp => {
                    if boost_pad_pickup_sequence_is_recent(
                        &self.boost_pad_pickup_sequence_times,
                        &pad_id,
                        event.sequence,
                        time,
                    ) {
                        continue;
                    }
                    self.boost_pad_pickup_sequence_times
                        .insert((pad_id.clone(), event.sequence), time);
                    BoostPadEventKind::PickedUp {
                        sequence: event.sequence,
                    }
                }
                SaBoostPadEventKind::Available => BoostPadEventKind::Available,
            };
            boost_pad_events.push(BoostPadEvent {
                time,
                frame: frame_number,
                pad_id,
                player: (event.has_player != 0).then_some(player_id(event.player_index)),
                kind,
            });
        }
        boost_pad_events
    }
}
