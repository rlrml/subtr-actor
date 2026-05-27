use super::*;

impl TouchCalculator {
    pub(crate) fn handle_fifty_fifty_movement(
        &mut self,
        delta: glam::Vec3,
        travel_distance: f32,
        fifty_fifty_state: &FiftyFiftyState,
    ) -> bool {
        if let Some(active_event) = fifty_fifty_state.active_event.as_ref() {
            self.buffer_fifty_fifty_movement(active_event.start_frame, delta, travel_distance);
            return true;
        }

        if let Some(event) = fifty_fifty_state.resolved_events.last() {
            self.buffer_fifty_fifty_movement(event.start_frame, delta, travel_distance);
            self.flush_fifty_fifty_movement(event);
            return true;
        }

        self.pending_fifty_fifty_movement = None;
        false
    }
}
