use super::*;

impl<K> ContinuousBallControlTracker<K>
where
    K: Copy + PartialEq,
{
    pub fn update<F, G>(
        &mut self,
        frame: &FrameInfo,
        candidate: Option<ContinuousBallControlCandidate<K>>,
        player_statuses: &[ContinuousBallControlPlayerStatus],
        touches: &[ContinuousBallControlTouch],
        min_duration_for_kind: F,
        requires_airborne_for_kind: G,
    ) -> Vec<CompletedBallControlSequence<K>>
    where
        F: Fn(K) -> f32 + Copy,
        G: Fn(K) -> bool + Copy,
    {
        let mut completed = Vec::new();
        self.track_touch_contacts(touches);

        if self.active_player_is_non_airborne(player_statuses, requires_airborne_for_kind) {
            if let Some(sequence) = self.finish_active_sequence(min_duration_for_kind) {
                completed.push(sequence);
            }
        }

        let Some(candidate) = candidate else {
            if let Some(sequence) = self.finish_active_sequence(min_duration_for_kind) {
                completed.push(sequence);
            }
            return completed;
        };

        let same_sequence = self
            .active_sequence
            .as_ref()
            .is_some_and(|active_sequence| {
                active_sequence.player_id == candidate.player_id
                    && active_sequence.kind == candidate.sample.kind
            });

        if same_sequence {
            if let Some(active_sequence) = self.active_sequence.as_mut() {
                Self::extend_sequence(
                    active_sequence,
                    frame,
                    candidate.sample,
                    candidate.touch_count,
                    candidate.air_touch_count,
                );
            }
        } else {
            if let Some(sequence) = self.finish_active_sequence(min_duration_for_kind) {
                completed.push(sequence);
            }
            let takeoff_touch_count = if requires_airborne_for_kind(candidate.sample.kind) {
                self.pending_takeoff_touches
                    .remove(&candidate.player_id)
                    .unwrap_or(0)
            } else {
                0
            };
            self.active_sequence =
                Some(Self::begin_sequence(frame, candidate, takeoff_touch_count));
        }

        completed
    }

    pub fn finish<F>(&mut self, min_duration_for_kind: F) -> Option<CompletedBallControlSequence<K>>
    where
        F: Fn(K) -> f32,
    {
        self.finish_active_sequence(min_duration_for_kind)
    }
}
