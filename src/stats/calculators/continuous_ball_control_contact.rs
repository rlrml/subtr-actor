use super::*;

impl<K> ContinuousBallControlTracker<K>
where
    K: Copy + PartialEq,
{
    pub(crate) fn track_touch_contacts(&mut self, touches: &[ContinuousBallControlTouch]) {
        for touch in touches {
            self.pending_takeoff_touches
                .retain(|player_id, _| player_id == &touch.player_id);

            if !touch.is_airborne {
                *self
                    .pending_takeoff_touches
                    .entry(touch.player_id.clone())
                    .or_default() += 1;
            }
        }
    }

    pub(crate) fn active_player_is_non_airborne<G>(
        &self,
        player_statuses: &[ContinuousBallControlPlayerStatus],
        requires_airborne_for_kind: G,
    ) -> bool
    where
        G: Fn(K) -> bool,
    {
        self.active_sequence
            .as_ref()
            .is_some_and(|active_sequence| {
                requires_airborne_for_kind(active_sequence.kind)
                    && player_statuses
                        .iter()
                        .find(|status| status.player_id == active_sequence.player_id)
                        .is_some_and(|status| !status.is_airborne)
            })
    }

    pub(crate) fn finish_active_sequence<F>(
        &mut self,
        min_duration_for_kind: F,
    ) -> Option<CompletedBallControlSequence<K>>
    where
        F: Fn(K) -> f32,
    {
        let active_sequence = self.active_sequence.take()?;
        if active_sequence.duration < min_duration_for_kind(active_sequence.kind) {
            return None;
        }
        Some(Self::complete_sequence(active_sequence))
    }
}
