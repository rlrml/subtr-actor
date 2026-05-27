use super::*;

impl FlipResetTracker {
    pub(crate) fn maybe_emit_flip_reset_followup_dodge(
        &mut self,
        processor: &dyn ProcessorView,
        current_time: f32,
        frame_index: usize,
        player_id: &PlayerId,
    ) {
        let Ok(player_rigid_body) = processor.get_normalized_player_rigid_body(player_id) else {
            return;
        };
        if Self::player_is_grounded_for_wall_sequence(&player_rigid_body) {
            return;
        }

        let Some(candidate_event) = self.recent_flip_reset_candidates.get(player_id).cloned()
        else {
            return;
        };
        let time_since_candidate_touch = current_time - candidate_event.time;
        if !(0.05..=1.75).contains(&time_since_candidate_touch) {
            return;
        }

        let event = FlipResetFollowupDodgeEvent {
            time: current_time,
            frame: frame_index,
            player: player_id.clone(),
            is_team_0: processor.get_player_is_team_0(player_id).unwrap_or(false),
            candidate_touch_time: candidate_event.time,
            time_since_candidate_touch,
            candidate_touch_confidence: candidate_event.confidence,
        };
        self.current_frame_flip_reset_followup_dodge_events
            .push(event.clone());
        self.flip_reset_followup_dodge_events.push(event);
        self.recent_flip_reset_candidates.remove(player_id);
    }
}
